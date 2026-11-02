## [Unreleased]

### 🐛 Fixed

- Freeze on more than 1000 input items: `ItemStream` mirrored every item into an
  undrained channel and blocked forever once it filled
- `ff --filter … | head` panicked with "failed printing to stdout" when the
  reader closed early; a broken pipe now exits quietly
  (`ff --version | head -1` included)

### 🚀 Features

- `-f/--filter <QUERY>` prints matches and exits without a TUI, exiting 1 when
  nothing matches
- Flags may appear before, after, or between inputs; `--` ends flag parsing and
  a lone `-` reads stdin
- Unknown flags are rejected instead of being treated as the input source
- Combined short flags (`-mn`) and inline values (`--height=10`, `-pbat`) work

### ⚡ Performance

- Filtering no longer strips ANSI sequences or lowercases every item on every
  keystroke: `ItemStream` precomputes `clean`/`lower` keys once per item
- Items are matched from a borrowed slice, so `update_filter` no longer clones
  the whole corpus before scoring
- Non-matching items are rejected by an allocation-free subsequence check before
  any character vector is built
- The query's characters are collected once per keystroke, not once per item
- The query cache stores indices and match data, bounded to 16 queries, instead
  of three cloned vectors per query
- Measured on 100k items: per-keystroke mean 34 ms → 8.5 ms, worst 52 ms → 26 ms

### 🚜 Refactor

- Collapse the two near-identical interactive loops into one `run_event_loop`
  driven by an `ItemFeed` adapter (plain channel or command channel); `ui.rs`
  drops ~390 lines and one duplicate row renderer
- Both entry points now share the inline-mode cursor/size fallbacks, so a
  terminal that cannot report its cursor position falls back to fullscreen
  instead of failing
- A producer-set `Ready`/`Custom`/`Hidden` status is no longer overwritten with
  the configured ready message when the channel closes
- Split argument handling into `parse_args` (pure) and `resolve_input`
  (positionals + `stdin_piped`), replacing the duplicated TUI action variants
  with `InputSpec::{Stdin,Source,Items}`
- Explicit inputs now win over piped stdin, so `ff file.txt` behaves the same in
  a pipeline

### 🗑️ Removed

- Unused `ItemStream` stream helpers (`stream`, `filtered_stream`, `process_async`,
  `receive`) and `process_vector_as_stream`
- `futures` dependency, which was only used by those helpers
- Dead `process_items_async` and `run_async_tui_with_height_validation`

## [1.1.0] - 2026-11-01

### 🐛 Bug Fixes

- Stop the TUI freezing once more than 1000 items are ingested

### ⚙️ Miscellaneous Tasks

- *(toolchain)* Bump to Rust 1.99.0, edition 2024, docs and 1 MiB size gate
## [1.0.14] - 2026-05-09

### 🚀 Features

- Preview commands per extension and defaults
- Improvements to preview panel and flags, auto/smart mode for preview panel added

### 🚜 Refactor

- Update locks and reorganized some of the larger files, removed repeated func and formatted code, version bump
## [1.0.13] - 2026-02-23

### 🚜 Refactor

- Slim the help menu and update the readme
## [1.0.12] - 2026-02-21

### 🚀 Features

- Enable pipeline with use-dev-tty
## [1.0.11] - 2026-02-17

### 🚀 Features

- Add line number flag and corresponding handling in cli args and tui
## [1.0.10] - 2026-02-07

### 🐛 Bug Fixes

- Esc key with text in input does not cloes immeditaly, added a release script to use git-cliff, added git cliff to nix shell
## [1.0.9] - 2026-02-07

### 🐛 Bug Fixes

- Tui flickering, added in a double buffer

### ⚡ Performance

- More accurate and visually easier to see the matches, worst performance though

### ⚙️ Miscellaneous Tasks

- *(1.0.9)* Updating version in cargo
## [1.0.7] - 2026-02-01

### 🚀 Features

- *(indicator)* Also fix for multiselect auto scrolling, indicator while there is input
## [1.0.6] - 2026-01-31

### 🐛 Bug Fixes

- *(multi-select)* Now can multi-select while filtering
- *(tui)* Sped up file input and selector is followed, removed unused deps and code
## [1.0.5] - 2025-10-25

### 🚀 Features

- *(1.0.5)* Move where tui prints out items
## [1.0.4] - 2025-10-25

### 🚀 Features

- *(1.0.4)* Streaming items

### 🐛 Bug Fixes

- Trimming and checking for empty lines before displaying in tui
## [1.0.3] - 2025-07-19

### 🚀 Features

- *(async)* Updated the search to us quick sort and binary trees plus lsh to group similar items

### 📚 Documentation

- Removed stdin option from readme
## [1.0.2] - 2025-07-12

### 🐛 Bug Fixes

- Selection highlight, removed docs, lock update and version bump
## [1.0.0] - 2025-07-12

### 🚀 Features

- *(v1)* Updates to tui to handle end of terminal and dir as an arg

### 🐛 Bug Fixes

- Fuzzy highlight case insensitive

### 📚 Documentation

- Reorg on the docs again and moved tests out of lib
