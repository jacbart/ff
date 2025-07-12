# Library Usage

## Basic Usage

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

## Multi-Select Mode

```rust
let mut finder = FuzzyFinder::new(items, true); // multi_select = true
finder.query = "a".to_string();
finder.update_filter();

// Toggle selection
finder.toggle_selection(0);
assert!(finder.selected_indices.contains(&0));
```

## TUI Integration

```rust
use ff::{run_tui, run_tui_with_config, TuiConfig};

// Fullscreen TUI
let items = vec!["item1".to_string(), "item2".to_string()];
match run_tui(items, false) {
    Ok(selected) => println!("Selected: {:?}", selected),
    Err(e) => eprintln!("Error: {}", e),
}

// Height-constrained TUI
let config = TuiConfig::with_height(10);
match run_tui_with_config(items, false, config) {
    Ok(selected) => println!("Selected: {:?}", selected),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Key Features

- **Case-insensitive matching** by default
- **Substring matching** for immediate results
- **Character sequence matching** for flexible searches
- **Query caching** for repeated searches
- **Multi-select support** for selecting multiple items 