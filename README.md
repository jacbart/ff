# FF - Fast Fuzzy Finder

A high-performance fuzzy finder. **Fast, not precise** - designed for quick filtering rather than exact matching.

## Features

- **Fast fuzzy matching** - Quick filtering, not precise matching
- **Case-insensitive search** by default
- **Multi-select support** for selecting multiple items
- **TUI interface** with keyboard navigation
- **Flexible input** - files, stdin, or direct items
- **Loading indicator** - Animated spinner while items are being loaded
- **Per-item indicators** - Dynamic status indicators for each item (library only)

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

#### Basic Session

```rust
use ff::FuzzyFinderSession;

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
    let result = runner.await??;
    
    println!("Selected: {:?}", result);
    Ok(())
}
```

#### With Per-Item Indicators

For tasks that need status indicators on each item (e.g., showing progress):

```rust
use ff::{FuzzyFinderWithIndicators, ItemIndicator, GlobalStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (session, tui_future) = FuzzyFinderWithIndicators::new(true);
    let runner = tokio::spawn(tui_future);

    // Add items with spinner indicators
    session.add_with_indicator("task1", ItemIndicator::Spinner).await?;
    session.add_with_indicator("task2", ItemIndicator::Spinner).await?;
    session.add("task3").await?;  // No indicator
    
    // Update indicators as tasks complete
    session.set_indicator("task1", ItemIndicator::Success).await?;
    session.set_indicator("task2", ItemIndicator::Error).await?;
    
    // Optionally set global status
    session.set_global_status(GlobalStatus::Ready(Some("All done!".into()))).await?;

    let result = runner.await??;
    println!("Selected: {:?}", result);
    Ok(())
}
```

**Available indicators:**
- `ItemIndicator::Spinner` - Animated spinner
- `ItemIndicator::Success` - Green checkmark
- `ItemIndicator::Error` - Red X
- `ItemIndicator::Warning` - Yellow warning
- `ItemIndicator::Text(String)` - Custom text
- `ItemIndicator::ColoredText(String, Color)` - Custom colored text
- `ItemIndicator::None` - No indicator

## TUI Controls

- **Type to search** - Filter items in real-time
- **↑/↓ arrows** - Navigate through results
- **Enter** - Select item (single mode) or confirm selection (multi mode)
- **Tab** - Toggle selection and move to next item (multi-select mode)
- **Space** - Toggle selection (multi-select mode)
- **Esc** - Exit without selection
- **Ctrl+Q/Ctrl+C** - Exit without selection

## Configuration

The `TuiConfig` struct allows customization:

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

## Performance

Optimized for speed over precision:
- **Substring matching** for immediate results
- **Character sequence matching** for flexible searches
- **Query caching** for repeated searches
- **Case-insensitive** by default

## License

MIT License - see LICENSE file for details. 