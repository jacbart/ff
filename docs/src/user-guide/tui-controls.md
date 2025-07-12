# TUI Controls

FF provides an intuitive terminal user interface (TUI) for interactive fuzzy finding.

## Interface Overview

The TUI consists of:

- **Search bar**: Shows your current query
- **Item list**: Shows filtered items with cursor
- **Status bar**: Shows position and available actions

## Keyboard Controls

### Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` | Move cursor up/down |
| `k` / `j` | Move cursor up/down |

### Selection

| Key | Action |
|-----|--------|
| `Enter` | Select item (single mode) or confirm selection (multi mode) |
| `Tab` / `Space` | Toggle selection (multi-select mode only) |

### Search

| Key | Action |
|-----|--------|
| Any printable character | Add to search query |
| `Backspace` | Remove last character from query |

### Exit

| Key | Action |
|-----|--------|
| `Esc` | Exit without selection |
| `Ctrl+Q` | Exit without selection |

## Multi-Select Mode

In multi-select mode, the interface shows checkboxes:

```
> [x] apple    ← Selected
  [ ] banana   ← Not selected
  [x] cherry   ← Selected
```

- `[x]` - Item is selected
- `[ ]` - Item is not selected
- `>` - Current cursor position

## Search Behavior

FF uses fuzzy matching:

- **Substring matching**: Direct substring search
- **Character sequence matching**: Fuzzy matching for flexible searches
- **Case-insensitive**: All searches ignore case

### Search Examples

```
Query: "app"
Matches: "apple", "application", "snap"

Query: "bn"
Matches: "banana", "brown", "cabin"
``` 