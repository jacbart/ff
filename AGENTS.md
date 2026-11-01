# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## What this is

`ff` is a terminal fuzzy finder written in Rust. It ships as **both** a CLI
binary (`ff`) and a library that other Rust programs embed. The product
constraint that shapes most decisions: **the compiled binary must stay under
1 MiB**.

Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) before making structural
changes.

## Toolchain

- The Rust version is pinned **once**, in `rust-toolchain.toml`. `flake.nix`
  reads it from there. Never hard-code a compiler version in `flake.nix`.
- Current pin: Rust 1.99.0, edition 2024.
- Supported systems: `x86_64-linux`, `aarch64-linux`, `aarch64-darwin`
  (`x86_64-darwin` was dropped by nixpkgs 26.11).
- To bump Rust: edit `rust-toolchain.toml`, run `nix flake update`, then verify.

## Commands

Prefer the Nix path — it uses the pinned compiler and cross-checks the build.

```bash
nix develop                 # dev shell (rust + rust-analyzer, bacon, lldb, git-cliff)
nix build                   # builds ./result/bin/ff
nix flake check             # build + 1 MiB binary-size check
nix fmt                     # nixfmt-tree

cargo test                  # 224 tests (unit + integration + doc-tests)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release && ./scripts/check-size.sh target/release/ff
```

All four gates (fmt, clippy, test, size) must pass before a change is
considered done.

## Layout at a glance

```
src/bin/ff.rs      binary entry
src/cli/           argv planning (planner), execution (main), tty detection
src/input.rs       file / dir / unix / http / stdin sources
src/fuzzy/         scoring + finder state + item stream  (no TUI dependencies)
src/tui/           loop (ui), keys (events), drawing (buffer), preview, layout
docs/              architecture notes
scripts/           release.sh, check-size.sh
```

Dependency direction is one-way: `cli → tui → fuzzy`. Do not make `fuzzy`
depend on `tui` or `crossterm`.

## Hard rules

1. **Results on stdout, UI on stderr.** The TUI must never write to stdout;
   `ff | next` has to stay clean.
2. **Respect the 1 MiB budget.** Check with `./scripts/check-size.sh` after any
   dependency or codegen change. Add a dependency only if you can justify its
   bytes; measure before and after.
3. **Selection is keyed by original index.** `filtered_indices` maps display
   order back to source order. Never key selection on display position.
4. **`filtered_items`, `filtered_indices`, `match_positions` stay in lockstep.**
   They are parallel vectors; update all three together.
5. **Restore terminal state on every exit path** (alternate screen, inline rows,
   cursor, raw mode).
6. **Keep `unsafe` out of new code.** The existing `libc::dup2` block in
   `input.rs` is the only permitted one; it exists to reopen `/dev/tty` after
   piped stdin is consumed.
7. **Edition 2024 match ergonomics.** Do not write `ref`/`&` inside a pattern
   that is already implicitly borrowing; bind and then borrow (`as_deref()` is
   usually cleanest).

## Testing expectations

- New behaviour needs a test. Unit tests live next to the code; integration
  tests go in `tests/`.
- Fuzzy ranking changes must keep `tests/fuzzy_integration_tests.rs` and the
  tier tests in `src/fuzzy/scoring.rs` green, and should add a case that pins
  the expected ordering.
- CLI parsing changes belong in `src/cli/planner.rs` (pure function, easy to
  test) rather than in `cli_main`.
- Terminal interaction is not covered by automated tests. Manually verify with:
  `ff file.txt`, `ls | ff`, `ff file.txt --height 10`, and `-p 'cat' --preview-auto`.

## Known debt (do not make it worse)

- `src/tui/ui.rs` contains two near-identical interactive loops
  (`run_interactive_tui`, `run_interactive_tui_with_indicators`). Fixes usually
  need to be applied to both until they are unified.
- `src/tui/controls.rs::handle_key_event` is legacy; the live path is
  `src/tui/events.rs::handle_async_key_event`.
- The filter hot path clones and lowercases every item on each keystroke — see
  `docs/ARCHITECTURE.md` §6 before adding work to `update_filter`.
- The library has no feature flags yet; any consumer links the full TUI stack.

## Commits and releases

- Commit messages follow Conventional Commits (`feat:`, `fix:`, `refactor:`,
  `chore:`).
- Releases are cut with `./scripts/release.sh <version>`, which bumps
  `Cargo.toml`, verifies the Nix build, regenerates `CHANGELOG.md` with
  `git-cliff`, and tags `v<version>`.
