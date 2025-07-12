# Quick Start

Get up and running with FF in minutes.

## Basic Usage

### Single Select Mode

Select one item from a list:

```bash
# From a file
ff items.txt

# From stdin
echo "apple\nbanana\ncherry" | ff

# Direct items
ff apple banana cherry
```

### Multi-Select Mode

Select multiple items:

```bash
# From a file with multi-select
ff items.txt --multi-select

# Direct items with multi-select
ff apple banana cherry --multi-select
```

## TUI Controls

Once FF starts, you'll see an interactive interface:

- **Type to search**: Filter items in real-time
- **↑/↓ arrows**: Navigate through results
- **Enter**: Select item (single mode) or confirm selection (multi mode)
- **Tab/Space**: Toggle selection (multi-select mode only)
- **Esc**: Exit without selection
- **Ctrl+Q**: Exit without selection

## Common Examples

### File Selection

```bash
# Select a file from current directory
ls | ff

# Select multiple files
ls | ff --multi-select
```

### Command History

```bash
# Search command history
history | ff
```

## Library Usage

Use FF in your Rust applications:

```rust
use ff::FuzzyFinder;

let items = vec![
    "apple".to_string(),
    "banana".to_string(),
    "cherry".to_string(),
];

let mut finder = FuzzyFinder::new(items, false);
finder.query = "app".to_string();
finder.update_filter();

assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
```

## Next Steps

- [CLI Usage](user-guide/cli-usage.md) - Learn all command line options
- [TUI Controls](user-guide/tui-controls.md) - Master the interface 