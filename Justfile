# Meowchain Justfile - all builds auto-fetch latest dependencies

# Default: update all deps (including latest reth from main) then build release
default: build

# Update all crates to latest versions and build release
build:
    cargo update
    cargo build --release

# Quick build without updating deps
build-fast:
    cargo build --release

# Debug build with latest deps
build-debug:
    cargo update
    cargo build

# Update all dependencies (fetches latest reth commit from main)
update:
    cargo update

# Run all tests with latest deps
test:
    cargo update
    cargo test

# Quick test without updating
test-fast:
    cargo test

# Dev mode: update + build + run
dev:
    cargo update
    RUST_LOG=info cargo run --release

# Run in production mode
run-production:
    cargo update
    cargo run --release -- --production --block-time 12

# Run with custom args
run-custom *ARGS:
    cargo update
    cargo run --release -- {{ARGS}}

# Build Docker image
docker:
    cargo update
    cargo build --release
    docker build -t meowchain .

# Clean build artifacts
clean:
    cargo clean

# Check compilation without building
check:
    cargo update
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy lints
lint:
    cargo update
    cargo clippy -- -D warnings

# Regenerate sample-genesis.json
genesis:
    cargo test test_regenerate_sample_genesis
