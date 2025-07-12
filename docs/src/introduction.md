# FF - Fast Fuzzy Finder

A high-performance fuzzy finder. **Fast, not precise** - designed for quick filtering rather than exact matching.

## What is FF?

FF is a lightweight fuzzy finder that can be used both as a library in Rust applications and as a standalone CLI tool. It provides fast filtering through lists of items with an intuitive terminal interface.

## Features

- **Fast fuzzy matching** - Quick filtering, not precise matching
- **Case-insensitive search** by default
- **Multi-select support** for selecting multiple items
- **TUI interface** with keyboard navigation
- **Flexible input** - files, stdin, or direct items

## Quick Example

```bash
# Select a file from current directory
ls | ff

# Multi-select from a list
ff apple banana cherry --multi-select

# Height options
ff items.txt --height 10
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