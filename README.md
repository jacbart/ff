# ff - Fast Fuzzy Finder

A fast fuzzy finder for the terminal. Prioritizes speed over precision -- designed for quick interactive filtering.

## Installation

```bash
# From source
git clone https://github.com/jacbart/ff.git
cd ff
cargo install --path .

# With Nix
nix build
./result/bin/ff --version
```

## Usage

```
ff [OPTIONS] [INPUT]
<command> | ff [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-m`, `--multi-select` | Enable multi-select mode |
| `-n`, `--line-number` | Output line numbers (`file:line` for file input) |
| `--height <N>` | Set TUI height in lines (non-fullscreen) |
| `--height-percentage <N>` | Set TUI height as % of terminal (non-fullscreen) |
| `-h`, `--help` | Show help message |
| `-V`, `--version` | Show version information |

### Examples

```bash
# Select from a file
ff items.txt

# Multi-select from a file
ff items.txt -m

# Select from a directory listing
ff ./src/

# Inline items
ff apple banana cherry

# Piped input
ls | ff
cat items.txt | ff -m

# Non-fullscreen mode
ff items.txt --height 10
ff items.txt --height-percentage 50
```

### Input Sources

ff accepts input from multiple sources:

- **Files** -- read lines from a file (`ff items.txt`)
- **Directories** -- list entries in a directory (`ff ./src/`)
- **Stdin** -- pipe output from another command (`ls | ff`)
- **Inline items** -- pass items directly as arguments (`ff a b c`)
- **URLs** -- read from HTTP/HTTPS endpoints or Unix sockets

## Controls

| Key | Action |
|-----|--------|
| Type | Filter items in real-time |
| Up/Down | Navigate results |
| Enter | Select (single) or confirm selection (multi) |
| Tab/Space | Toggle selection (multi-select mode) |
| Esc, Ctrl+C, Ctrl+Q | Exit without selection |

## Library Usage

ff can also be used as a Rust library for embedding fuzzy selection in your own tools.

### Basic Session

```rust
use ff::FuzzyFinderSession;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (session, tui_future) = FuzzyFinderSession::new(true);
    let runner = tokio::spawn(tui_future);

    session.add("apple").await?;
    session.add("banana").await?;
    session.add_batch(vec!["cherry", "date"]).await?;

    let result = runner.await??;
    println!("Selected: {:?}", result);
    Ok(())
}
```

### With Per-Item Indicators

For tasks that need status indicators on each item (e.g., showing progress):

```rust
use ff::{FuzzyFinderWithIndicators, ItemIndicator, GlobalStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (session, tui_future) = FuzzyFinderWithIndicators::new(true);
    let runner = tokio::spawn(tui_future);

    session.add_with_indicator("task1", ItemIndicator::Spinner).await?;
    session.add_with_indicator("task2", ItemIndicator::Spinner).await?;
    session.add("task3").await?;

    session.set_indicator("task1", ItemIndicator::Success).await?;
    session.set_indicator("task2", ItemIndicator::Error).await?;
    session.set_global_status(GlobalStatus::Ready(Some("All done!".into()))).await?;

    let result = runner.await??;
    println!("Selected: {:?}", result);
    Ok(())
}
```

**Available indicators:**
- `ItemIndicator::Spinner` -- animated spinner
- `ItemIndicator::Success` -- green checkmark
- `ItemIndicator::Error` -- red X
- `ItemIndicator::Warning` -- yellow warning
- `ItemIndicator::Text(String)` -- custom text
- `ItemIndicator::ColoredText(String, Color)` -- custom colored text
- `ItemIndicator::None` -- no indicator

### Configuration

```rust
use ff::{TuiConfig, FuzzyFinderSession};

let config = TuiConfig {
    fullscreen: false,
    height: Some(10),
    height_percentage: None,
    show_help_text: true,
    show_loading_indicator: true,
    loading_message: Some("Loading...".into()),
    ready_message: Some("Ready".into()),
};

let (session, tui_future) = FuzzyFinderSession::with_config(true, config);
```

## License

MIT License - see LICENSE file for details.
