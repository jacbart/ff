# Architecture

`ff` is a terminal fuzzy finder with two entry points: a CLI binary (`ff`) and an
embeddable Rust library (`ff` crate). Both share the same engine and TUI.

- **Goal:** a fast, small (`< 1 MiB` compiled) `fzf` alternative.
- **Scale:** ~7.8k lines across 24 Rust files.
- **Runtime:** Tokio multi-thread runtime, `crossterm` for terminal control.

---

## 1. Crate layout

| Path | Responsibility |
|---|---|
| `src/bin/ff.rs` | Binary entry point; delegates to `ff::cli_main()`. |
| `src/lib.rs` | Public API surface, session handles, build info. |
| `src/cli/planner.rs` | Pure argv → `CliAction` planning (no I/O). |
| `src/cli/main.rs` | Executes a `CliAction`: reads input, runs the TUI, prints results. |
| `src/cli/tty.rs` | TTY detection (`stdin`/`stdout`/`stderr`). |
| `src/help.rs` | `--help` text. |
| `src/input.rs` | Input sources: file, directory, `unix://`, `http(s)://`, inline, piped stdin. |
| `src/fuzzy/finder.rs` | Finder state machine: query, results, selection, cursor, cache. |
| `src/fuzzy/scoring.rs` | Single-pass subsequence matcher, scoring, tiers, batch sort. |
| `src/fuzzy/stream.rs` | `ItemStream`: ordered item buffer, keyed by original index. |
| `src/tui/ui.rs` | Interactive loop, layout, channels, all drawing orchestration. |
| `src/tui/events.rs` | Key event → `Action` translation (async, used by the loop). |
| `src/tui/controls.rs` | Legacy synchronous key handler and the `Action` enum. |
| `src/tui/buffer.rs` | `Cell`/`Style`/`ScreenBuffer`; double-buffered ANSI rendering. |
| `src/tui/preview.rs` | Preview rules, command building/escaping, ANSI parsing, pane state. |
| `src/tui/layout.rs` | Cursor/size fallbacks via `stderr` queries and `stty`. |

Dependency direction is strictly one-way: `bin → cli → tui → fuzzy`, with `input`
used by `cli` and `tui`-adjacent helpers. `fuzzy` never depends on `tui`.

---

## 2. Execution flow

### 2.1 Startup

```
argv
 └─ ff::cli_main()                       src/cli/main.rs
     ├─ planner::plan_cli_action(&args) → CliAction::{ShowVersion,ShowHelp,
     │                                     RunAsyncTui, RunAsyncTuiFromStdin, Error}
     ├─ validate_tty_requirements()      // stderr must be a TTY
     ├─ input source resolution
     │    piped stdin → read_piped_stdin() + reopen_stdin_from_tty() (dup2 /dev/tty → fd 0)
     │    file/dir/url/inline → producer task
     └─ tui::run_tui_with_config(rx, multi_select, TuiConfig)
          └─ Result<Vec<(original_index, String)>>
               └─ println! to stdout (optionally with line numbers)
```

`plan_cli_action` is pure and unit-tested; it decides between stdin mode and
argument mode via `tty::is_stdin_piped()`.

### 2.2 Item ingestion

`input::send_input_to_channel(source, sender)` dispatches on the source prefix:

| Source | Handling |
|---|---|
| `unix://path` | `tokio::net::UnixStream` |
| `http://` / `https://` | TCP connect + manual HTTP request |
| `dir:path` | Directory listing, sorted |
| existing path | File lines, or directory listing |
| anything else | Whitespace-split inline items |

Items cross `mpsc::channel(1000)` into the TUI; the TUI drains up to 1000 per
tick and appends them to the finder's buffer, which never blocks. The producer
dropping the sender marks the stream as exhausted, which switches the status line
from spinner to ready.

### 2.3 Interactive loop (`run_interactive_tui`)

Each tick:

1. Drain up to `MAX_BATCH_SIZE = 1000` items from the receiver (`try_recv`) and
   feed them to `FuzzyFinder::add_items`.
2. Drain at most one preview result from the preview channel.
3. Recompute terminal size and effective `tui_height`.
4. If dirty: clear `ScreenBuffer`, draw prompt/status, visible rows, preview
   pane, instructions; render to one ANSI string and write+flush **stderr**.
5. `event::poll(50ms)`; on a key, `events::handle_async_key_event` returns
   `Action::{Continue,Exit,Select}`.
6. Advance the spinner every 80 ms while the input is still open.

Terminal restoration (leave alternate screen / clear inline rows / show cursor /
`disable_raw_mode`) runs on every exit path.

### 2.4 Preview pane

`maybe_update_preview` runs when the preview is visible and the cursor moved to a
different item:

```
item + rules → build_preview_command()   // {ext} filters, "auto" sentinel, shell escaping
             → inject_color_flag()       // bat/eza/exa get --color=always
             → spawn_blocking(sh -c …)
             → parse_ansi_output()       // → StyledLine (text, fg, bg, bold, underline)
             → std::sync::mpsc → PreviewState::apply_result / cache
```

The previous preview task is aborted when the cursor moves on.

---

## 3. Data model and invariants

`FuzzyFinder` holds:

```rust
stream: ItemStream,
query: String,
filtered_items: Vec<String>,      // display order
filtered_indices: Vec<usize>,     // display position → original index
match_positions: Vec<MatchPositions>, // highlight spans, parallel to filtered_items
selected_items: HashSet<usize>,   // keyed by ORIGINAL index
cursor_position: usize,           // index into filtered_items
query_cache: HashMap<String, (Vec<String>, Vec<usize>, Vec<MatchPositions>)>,
```

Invariants to preserve:

1. **Selection is keyed by original index.** `filtered_indices` maps a visual row
   back to its source row; re-filtering must never invalidate a selection.
2. **Results go to stdout, UI goes to stderr.** The TUI writes only to stderr so
   `ff | next-command` stays clean.
3. **Esc is two-stage:** with a non-empty query it clears the query; with an
   empty query it exits.
4. **The query cache is invalidated** on every `add_items`; it is keyed by the
   exact query string.
5. **`filtered_items`, `filtered_indices`, and `match_positions` are parallel and
   equal-length.** Any code path that changes one must change all three.
6. **Terminal state is restored** regardless of how the loop ends.

Scoring ranks by tier first, then score, then original index:

```
MatchTier: Exact > Prefix > Substring > Fuzzy
```

---

## 4. Terminal and I/O model

- Raw mode is enabled on the process terminal; rendering goes to `stderr`.
- Fullscreen mode uses the alternate screen; inline mode (`--height`,
  `--height-percentage`) draws under the cursor and clears those rows on exit.
- When `stdout`/`stderr` are redirected, cursor and size queries fall back to
  `layout::get_cursor_position_from_stderr` and `stty size -F /dev/stderr`.
- In stdin-pipeline mode, fd 0 is replaced with `/dev/tty` via `libc::dup2`
  so `crossterm` can read keys after the pipe is consumed. This is the only
  `unsafe` block in the codebase.

---

## 5. Library surface

`src/lib.rs` re-exports:

- Engine: `FuzzyFinder`, `fuzzy::{score_match, score_batch, …}`.
- TUI: `run_tui`, `run_tui_with_config`, `run_tui_with_indicators`,
  `create_items_channel`, `create_command_channel`, `TuiConfig`,
  `ItemIndicator`, `GlobalStatus`, `TuiCommand`, `PreviewState`, `PreviewResult`,
  `PreviewRule`.
- Sessions: `FuzzyFinderSession`, `FuzzyFinderWithIndicators`, `SessionResult`.

All of these are unconditional: a library consumer always links `crossterm` and
`tokio`. Feature-gating the TUI is on the roadmap (see §8).

---

## 6. Performance characteristics

Known hot path, executed on every keystroke:

```
FuzzyFinder::update_filter
  └─ stream.get_all_items()        // clones every item into a new Vec
      └─ scoring::score_batch
           ├─ strip_ansi_sequences(item)   // allocates per item, always
           ├─ item.to_lowercase()          // allocates per item
           └─ score_match_with_original
      └─ clone results into filtered_items + match_positions
      └─ clone everything again into query_cache
```

This is the main obstacle to `fzf`-class throughput on 100k+ line inputs.
Improvements, in order of expected impact:

1. Store items once (`Arc<str>` or `Vec<Box<str>>`) and filter by index.
2. Precompute lowercase / ANSI-stripped keys once per item, not per query.
3. Return indices from `score_batch` and avoid cloning display strings.
4. Cap `query_cache` (LRU or clear on threshold).

Item ingestion itself is O(1) per item and never blocks: `ItemStream` is a
plain `Vec<String>`. (It used to mirror every item into an `mpsc` channel that
nothing drained, which froze the TUI once 1000 items had accumulated.)

---

## 7. Size budget

The compiled binary must stay below **1 MiB** (`1,048,576` bytes).

- Enforced by `scripts/check-size.sh` and by `checks.<system>.binary-size`
  (`nix flake check`).
- Release profile: `opt-level = 3`, `lto = true`, `codegen-units = 1`,
  `panic = "abort"`, `strip = true`.
- Reference sizes (aarch64-darwin, Rust 1.99.0): ~704 KiB via cargo,
  ~820 KiB via `nix build`.
- If the budget is exceeded, prefer removing a dependency or trimming `tokio`
  features before changing `opt-level`.

---

## 8. Known debt

| Item | Impact |
|---|---|
| `run_interactive_tui` and `run_interactive_tui_with_indicators` duplicate ~450 lines | Fixes must be made twice; `ui.rs` is ~1.9k lines. |
| `controls::handle_key_event` is superseded by `events::handle_async_key_event` | Dead production code kept alive by its own tests. |
| Dead public helpers: `process_items_async`, `run_async_tui_with_height_validation`, `create_stream_from_items` | Misleading API surface. |
| Per-keystroke cloning and lowercasing (§6) | Latency on large inputs. |
| No feature flags | Library users pay for the terminal stack. |
| `README`/`lib.rs` used to claim LSH and quicksort | Corrected; scoring uses a tiered stable sort. |

---

## 9. Testing

- `cargo test` — 227 tests: 163 unit, 56 integration across
  `tests/{cli,fuzzy,input}_integration_tests.rs`, 8 doc-tests.
- `cargo clippy --all-targets -- -D warnings` — must stay clean.
- `cargo fmt --check` — edition 2024 formatting.
- `nix build` / `nix flake check` — build plus size budget.
- Interactive behaviours (raw mode, preview, inline height) are not covered by
  automated tests; verify manually with `ff file.txt`, `ls | ff`, `--height 10`,
  and `-p 'cat'`.
