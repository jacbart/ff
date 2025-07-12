# Installation

## From Source

### Prerequisites

- Rust toolchain (1.70.0 or later)

### Installation Steps

```bash
# Clone the repository
git clone https://github.com/jacbart/ff.git
cd ff

# Build and install
cargo install --path .
```

## With Cargo

```bash
# Install directly from git
cargo install --git https://github.com/jacbart/ff.git
```

## As a Library Dependency

Add FF to your `Cargo.toml` to use it as a library in your Rust project:

```toml
[dependencies]
ff = { git = "https://github.com/jacbart/ff.git" }
```

## Verification

After installation, verify that FF is working correctly:

```bash
# Check version
ff --version

# Test basic functionality
echo "apple\nbanana\ncherry" | ff
``` 