# Release Preparation

## Pre-Release Checklist

### Code Quality
```bash
# Run all tests
cargo test

# Check code coverage
cargo tarpaulin --out Html

# Run clippy
cargo clippy --all-targets --all-features

# Check formatting
cargo fmt --check
```

### Security Scans
```bash
# Install cargo-audit
cargo install cargo-audit

# Check for vulnerabilities
cargo audit

# Update dependencies
cargo update
```

### Documentation
```bash
# Build docs
cargo doc --no-deps

# Check doc tests
cargo test --doc

# Build book
mdbook build docs/
```

## Release Build

```bash
# Clean build
cargo clean
cargo build --release

# With real timestamps
SOURCE_DATE_EPOCH=$(date +%s) cargo build --release

# Cross-platform builds
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target x86_64-pc-windows-gnu
```

## Version Management

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md
# Create git tag
git tag v0.1.0
git push origin v0.1.0
```

## Distribution

- **Crates.io**: `cargo publish`
- **GitHub Releases**: Upload binaries
- **Nix**: Update flake.nix if needed 