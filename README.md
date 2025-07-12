# FF - Fast Fuzzy Finder

A high-performance fuzzy finder library with TUI support for Rust applications.

## Features

- **Fast fuzzy matching** with substring and character sequence matching
- **Case-insensitive search** by default
- **Multi-select support** for selecting multiple items
- **TUI interface** with keyboard navigation
- **Cross-platform** support (Linux, macOS, Windows)
- **Build information** with version tracking

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/jacbart/ff.git
cd ff
cargo install --path .

# With Nix (includes real build timestamps)
nix build
./result/bin/ff --version
```

### Basic Usage

```bash
# Single select from file
ff items.txt

# Multi-select from file
ff items.txt --multi-select

# Direct items
ff apple banana cherry --multi-select

# From stdin
echo "apple\nbanana" | ff
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

## Documentation

📚 **Complete documentation is available in the [docs/](docs/) directory:**

- **[Introduction](docs/src/introduction.md)** - Overview and features
- **[Installation](docs/src/getting-started/installation.md)** - Detailed installation instructions
- **[Quick Start](docs/src/getting-started/quick-start.md)** - Get up and running quickly
- **[CLI Usage](docs/src/user-guide/cli-usage.md)** - Complete command-line reference
- **[TUI Controls](docs/src/user-guide/tui-controls.md)** - Interactive interface guide
- **[API Reference](docs/src/api/fuzzy-finder.md)** - Library API documentation

## Build Information

The binary includes embedded build information:

```bash
ff --version
# Shows: package version, build timestamp, Rust compiler version
```

### Build Timestamps

When building with Nix, the flake automatically sets `SOURCE_DATE_EPOCH` to ensure real build timestamps instead of the reproducible 1980-01-01 date.

For manual builds with real timestamps:

```bash
SOURCE_DATE_EPOCH=$(date +%s) cargo build
```

## Examples

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

### Custom Items
```bash
# Select from custom list
ff "option 1" "option 2" "option 3"
```

## Performance

The fuzzy finder uses several optimizations:

- **Substring matching**: Direct substring search for immediate results
- **Character sequence matching**: Fuzzy matching for more flexible searches
- **Query caching**: Repeated queries are cached for better performance
- **Case-insensitive matching**: All searches are case-insensitive by default

## Development

### Building with Real Timestamps

```bash
SOURCE_DATE_EPOCH=$(date +%s) cargo build
```

### Running Tests

```bash
cargo test
```

### Code Coverage

```bash
cargo tarpaulin --out Html
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 