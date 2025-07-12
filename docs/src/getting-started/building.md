# Building

## Basic Build

```bash
cargo build
```

## Release Build

```bash
cargo build --release
```

## With Real Timestamps

```bash
SOURCE_DATE_EPOCH=$(date +%s) cargo build
```

## Nix Build

```bash
nix build
```

## Build Information

The binary includes embedded build information:

```bash
ff --version
# Shows: package version, build timestamp, Rust compiler version
```

## Cross-Platform

FF supports Linux, macOS, and Windows. Use standard Rust cross-compilation:

```bash
# For Linux
cargo build --target x86_64-unknown-linux-gnu

# For macOS
cargo build --target x86_64-apple-darwin

# For Windows
cargo build --target x86_64-pc-windows-gnu
``` 