# FF - Fast Fuzzy Finder

A high-performance fuzzy finder. **Fast, not precise** - designed for quick filtering rather than exact matching.

## Features

- **Fast fuzzy matching** - Quick filtering, not precise matching
- **Case-insensitive search** by default
- **Multi-select support** for selecting multiple items
- **TUI interface** with keyboard navigation
- **Flexible input** - files, stdin, or direct items

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

### Basic Examples

```bash
# Single select from file
ff items.txt

# Multi-select from file
ff items.txt --multi-select

# Direct items
ff apple banana cherry

# Height options (non-fullscreen)
ff items.txt --height 10
ff items.txt --height-percentage 50
```

### Library Usage

```rust
use ff::FuzzyFinderSession;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Start the session
    let (session, tui_future) = FuzzyFinderSession::new(true);

    // Spawn the TUI runner
    let runner = tokio::spawn(tui_future);

    // Push items asynchronously
    session.add("apple").await?;
    session.add("banana").await?;
    session.add_batch(vec!["cherry", "date"]).await?;

    // Wait for the user to make a selection
    // Note: The runner returns a Result<Result<Vec<String>, Error>, JoinError>
    // The first ? handles the JoinError, the second ? handles the TUI Error
    let result = runner.await??;
    
    println!("Selected: {:?}", result);
    Ok(())
}
```

## TUI Controls

- **Type to search** - Filter items in real-time
- **↑/↓ arrows** - Navigate through results
- **Enter** - Select item (single mode) or confirm selection (multi mode)
- **Tab/Space** - Toggle selection (multi-select mode only)
- **Esc** - Exit without selection
- **Ctrl+Q** - Exit without selection

## Performance

Optimized for speed over precision:
- **Substring matching** for immediate results
- **Character sequence matching** for flexible searches
- **Query caching** for repeated searches
- **Case-insensitive** by default

## License

MIT License - see LICENSE file for details. 