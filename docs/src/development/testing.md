# Testing & Coverage

## Run Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# Integration tests
cargo test --test cli_integration_tests
```

## Code Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html

# Coverage with output
cargo tarpaulin --out Html --output-dir coverage/
```

## Benchmark Tests

```bash
# Run benchmarks
cargo bench

# Specific benchmark
cargo bench --bench bench
```

## Test Structure

- **Unit tests** - Individual function testing
- **Integration tests** - CLI and TUI functionality
- **Benchmark tests** - Performance testing
- **Doc tests** - Documentation examples

## Coverage Targets

- Aim for >90% code coverage
- Focus on critical paths (fuzzy matching, TUI)
- Include error handling scenarios 