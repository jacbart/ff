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

### Shell Integration

To enable piped input support, source the shell integration script:

```bash
# For Zsh
source <(ff-bin --zsh)

# For Bash
source <(ff-bin --bash)

# For Fish
source <(ff-bin --fish)
```

After sourcing, you can use the `ff` function for piped input:
```bash
echo "item1\nitem2\nitem3" | ff -m --height 10
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

# Custom prompt
ff items.txt --prompt "Search: "

# Piped input (requires shell integration)
echo "apple\nbanana\ncherry" | ff -m --height 10
cat file.txt | ff --prompt "Select: "
```

### Library Usage

```rust
use ff::FuzzyFinder;

let items = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
let mut finder = FuzzyFinder::new(items, false);
finder.query = "app".to_string();
finder.update_filter();
assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
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