# FF - Fast Fuzzy Finder

A simple, fast fuzzy finder for Rust applications with TUI support.

## What is FF?

FF is a lightweight fuzzy finder that can be used both as a library in Rust applications and as a standalone CLI tool. It provides an intuitive terminal interface for searching through lists of items.

## Features

- **Fast fuzzy matching** with substring and character sequence matching
- **Case-insensitive search** by default
- **Multi-select support** for selecting multiple items
- **TUI interface** with keyboard navigation
- **Cross-platform** support (Linux, macOS, Windows)

## Quick Example

```bash
# Select a file from current directory
ls | ff

# Multi-select from a list
ff apple banana cherry --multi-select

# Show version information
ff --version
```

## Library Usage

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

- [Installation](getting-started/installation.md) - Learn how to install FF
- [Quick Start](getting-started/quick-start.md) - Get up and running quickly
- [CLI Usage](user-guide/cli-usage.md) - Learn the command line interface 