# Installation

## From Source

```bash
git clone https://github.com/jacbart/ff.git
cd ff
cargo install --path .
```

## With Nix

```bash
nix build
./result/bin/ff --version
```

## Verify Installation

```bash
ff --version
```

## Dependencies

- Rust 1.70+ (for building from source)
- Nix (optional, for reproducible builds) 