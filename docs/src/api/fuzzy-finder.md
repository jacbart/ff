# FuzzyFinder

The main struct for fuzzy finding functionality in FF.

## Overview

`FuzzyFinder` provides fast fuzzy matching with support for both single-select and multi-select modes.

## Struct Definition

```rust
pub struct FuzzyFinder {
    pub items: Vec<String>,           // Original items
    pub filtered_items: Vec<String>,   // Current filtered results
    pub selected_indices: Vec<usize>,  // Selected item indices
    pub query: String,                 // Current search query
    pub cursor_position: usize,        // Current cursor position
    pub multi_select: bool,            // Multi-select mode flag
}
```

## Constructor

### `new`

Creates a new `FuzzyFinder` instance.

```rust
pub fn new(items: Vec<String>, multi_select: bool) -> Self
```

**Example:**

```rust
use ff::FuzzyFinder;

let items = vec![
    "apple".to_string(),
    "banana".to_string(),
    "cherry".to_string(),
];

let mut finder = FuzzyFinder::new(items, false);
```

## Core Methods

### `update_filter`

Updates the filtered items based on the current query.

```rust
pub fn update_filter(&mut self) -> ()
```

**Example:**

```rust
let mut finder = FuzzyFinder::new(vec!["apple".to_string(), "banana".to_string()], false);
finder.query = "app".to_string();
finder.update_filter();
assert_eq!(finder.filtered_items, vec!["apple".to_string()]);
```

### `move_cursor`

Moves the cursor up or down in the filtered items list.

```rust
pub fn move_cursor(&mut self, direction: i32) -> ()
```

**Example:**

```rust
let mut finder = FuzzyFinder::new(vec!["a".to_string(), "b".to_string(), "c".to_string()], false);
finder.update_filter();

// Move cursor down
finder.move_cursor(1);
assert_eq!(finder.cursor_position, 1);

// Move cursor up
finder.move_cursor(-1);
assert_eq!(finder.cursor_position, 0);
```

### `toggle_selection`

Toggles the selection of the current item (multi-select mode only).

```rust
pub fn toggle_selection(&mut self) -> ()
```

**Example:**

```rust
let mut finder = FuzzyFinder::new(vec!["a".to_string(), "b".to_string()], true);
finder.update_filter();

// Toggle selection of first item
finder.toggle_selection();
assert_eq!(finder.get_selected_items(), vec!["a".to_string()]);
```

### `get_selected_items`

Returns the currently selected items.

```rust
pub fn get_selected_items(&self) -> Vec<String>
```

**Example:**

```rust
let mut finder = FuzzyFinder::new(vec!["a".to_string(), "b".to_string()], true);
finder.update_filter();
finder.toggle_selection();
assert_eq!(finder.get_selected_items(), vec!["a".to_string()]);
```

## Basic Usage

```rust
use ff::FuzzyFinder;

// Create a fuzzy finder
let mut finder = FuzzyFinder::new(vec![
    "apple".to_string(),
    "banana".to_string(),
    "cherry".to_string(),
], false);

// Set a query and filter
finder.query = "app".to_string();
finder.update_filter();

// Get filtered results
println!("Filtered items: {:?}", finder.filtered_items);
``` 