# CLI Usage

FF provides a command-line interface for fuzzy finding through various input sources.

## Basic Syntax

```bash
ff [OPTIONS] [INPUT_SOURCE]
```

## Input Sources

### File Input

Read items from a file:

```bash
ff items.txt
ff /path/to/file.txt
```

### Stdin Input

Read items from standard input:

```bash
echo "apple\nbanana\ncherry" | ff
cat items.txt | ff
ls | ff
```

### Direct Items

Provide items directly as arguments:

```bash
ff apple banana cherry
ff "item with spaces" "another item"
```

## Command Line Options

| Option | Short | Description |
|--------|-------|-------------|
| `--multi-select` | `-m` | Enable multi-select mode |
| `--version` | `-V` | Show version information |
| `--help` | `-h` | Show help message |

## Usage Examples

### File Operations

```bash
# Select a file from current directory
ls | ff

# Select multiple files
ls | ff --multi-select

# Select from specific directory
ls /path/to/directory | ff
```

### Command History

```bash
# Search command history
history | ff
```

### Configuration Selection

```bash
# Select from configuration options
ff "option 1" "option 2" "option 3"

# Select multiple configuration options
ff "option 1" "option 2" "option 3" --multi-select
```

## Output Format

### Single Select Mode

In single select mode, FF outputs the selected item:

```bash
echo "apple\nbanana\ncherry" | ff
# Output: apple (if apple is selected)
```

### Multi-Select Mode

In multi-select mode, FF outputs each selected item on a separate line:

```bash
echo "apple\nbanana\ncherry" | ff --multi-select
# Output:
# apple
# cherry
# (if apple and cherry are selected)
```

## Error Handling

### No Input

If no items are provided, FF exits with an error:

```bash
ff
# Error: No items to search through
```

### File Not Found

If a file doesn't exist, FF exits with an error:

```bash
ff nonexistent.txt
# Error: Failed to read file: No such file or directory
``` 