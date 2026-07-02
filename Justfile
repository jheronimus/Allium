set shell := ["bash", "-uc"]

# Show available commands
default:
    @just --list

# Lint play and libretro code using Clippy
lint:
    cargo clippy -p play -p libretro --all-targets -- -D warnings

# Check code formatting using rustfmt across the entire workspace
fmt-check:
    cargo fmt --all -- --check

# Check workspace compilation
check:
    cargo check --workspace

# Run tests across the entire workspace (with and without the minime feature)
test:
    cargo test --workspace --features minime
    cargo test --workspace

# Run all quality checks in sequence
verify: fmt-check lint test
    @echo "All quality checks passed successfully!"

# Setup local hooks
setup:
    @mkdir -p .git/hooks
    @cp scripts/pre_push_hook.sh .git/hooks/pre-push
    @chmod +x .git/hooks/pre-push
    @echo "Git pre-push hook installed successfully!"
