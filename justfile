# List the available recipes.
default:
    @just --list

# Run lint and tests, stopping at the first failure.
check: lint test

# Check Rust and Markdown formatting, Clippy diagnostics, and the rule catalog.
lint:
    cargo +nightly fmt --all --check
    dprint check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo run --locked -- validate

# Run the test suite.
test:
    cargo test --locked

# Format Rust with nightly rustfmt and Markdown with dprint.
fmt:
    cargo +nightly fmt --all
    dprint fmt

# Apply formatting and machine-applicable Clippy fixes.
fix:
    cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged --locked
    cargo +nightly fmt --all
    dprint fmt

# Install the CLI from this checkout into Cargo's user binary directory.
install:
    cargo install --path . --locked

# Rebuild and reinstall every catalog rule skill into both harnesses.
reeject:
    cargo run --locked -- install --all --target all --prune
