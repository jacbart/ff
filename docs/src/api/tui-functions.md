# TUI Functions

FF provides TUI (Terminal User Interface) functions for interactive fuzzy finding.

## Main TUI Function

### `run_tui`

Runs an interactive TUI for fuzzy finding through a list of items.

```rust
pub fn run_tui(items: Vec<String>, multi_select: bool) -> Result<Vec<String>, Box<dyn std::error::Error>>
```

**Parameters:**
- `items`: Vector of strings to search through
- `multi_select`: If `true`, enables multi-select mode

**Returns:**
- `Ok(selected_items)`: Vector of selected items (empty if none selected)
- `Err(e)`: Error that occurred during TUI operation

**Example:**

```rust
use ff::run_tui;

let items = vec![
    "apple".to_string(),
    "banana".to_string(),
    "cherry".to_string(),
];

match run_tui(items, false) {
    Ok(selected) => {
        if !selected.is_empty() {
            println!("Selected: {}", selected[0]);
        }
    }
    Err(e) => eprintln!("TUI error: {}", e),
}
```

## TUI Behavior

### Single-Select Mode

In single-select mode (`multi_select = false`):

- User can navigate through items with arrow keys
- Pressing Enter selects the current item and exits
- Only one item can be selected
- Returns a vector with 0 or 1 items

### Multi-Select Mode

In multi-select mode (`multi_select = true`):

- User can navigate through items with arrow keys
- Pressing Tab or Space toggles selection of current item
- Pressing Enter confirms all selections and exits
- Multiple items can be selected
- Returns a vector with all selected items

## Basic Usage

```rust
use ff::run_tui;

fn main() {
    let items = vec![
        "option1".to_string(),
        "option2".to_string(),
        "option3".to_string(),
    ];

    match run_tui(items, false) {
        Ok(selected) => {
            for item in selected {
                println!("{}", item);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

## Multi-Select Usage

```rust
use ff::run_tui;

fn main() {
    let items = vec![
        "file1.txt".to_string(),
        "file2.txt".to_string(),
        "file3.txt".to_string(),
    ];

    match run_tui(items, true) {
        Ok(selected) => {
            if !selected.is_empty() {
                println!("Selected files:");
                for file in selected {
                    println!("  {}", file);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
} 