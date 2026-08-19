# List the available recipes.
default:
    @just --list

# Run lint and tests, stopping at the first failure.
check: lint test

# Check formatting, Clippy diagnostics, and the rule catalog.
lint:
    cargo +nightly fmt --all --check
    cargo clippy --all-targets --all-features --locked -- -D warnings
    cargo run --locked -- validate

# Run the test suite.
test:
    cargo test --locked

# Format the workspace with nightly rustfmt.
fmt:
    cargo +nightly fmt --all

# Apply formatting and machine-applicable Clippy fixes.
fix:
    cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged --locked
    cargo +nightly fmt --all

# Install the CLI from this checkout into Cargo's user binary directory.
install:
    cargo install --path . --locked

# Rebuild and reinstall every catalog rule skill into both harnesses.
reeject:
    cargo run --locked -- install --all --target all --prune
