# CLI Usage

## Basic Commands

```bash
# Single select from file
ff items.txt

# Multi-select from file
ff items.txt --multi-select

# Direct items
ff apple banana cherry

# From stdin
echo "apple\nbanana" | ff
```

## Height Options

```bash
# Fixed height (lines)
ff items.txt --height 10

# Percentage height
ff items.txt --height-percentage 50
```

## Input Sources

### File Input
```bash
ff file.txt
ff path/to/file.txt
```

### Direct Items
```bash
ff "item 1" "item 2" "item 3"
```

### Stdin
```bash
ls | ff
history | ff
cat file.txt | ff
```

## Options

- `--multi-select`, `-m` - Enable multi-select mode
- `--height <lines>` - Set fixed height in lines
- `--height-percentage <percent>` - Set height as percentage
- `--version`, `-V` - Show version information
- `--help`, `-h` - Show help

## Examples

```bash
# Select files
ls | ff

# Multi-select from history
history | ff --multi-select

# Custom height
ff items.txt --height 15

# Percentage height
ff items.txt --height-percentage 75
``` 