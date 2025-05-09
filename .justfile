_default:
    @just --list --unsorted

# Run rustfmt
fmt *args='':
    cargo +nightly fmt {{ args }}

check *args='--all-features':
    cargo clippy --all-targets {{ args }}

# Profile using cargo-flamegraph
profile *args='':
    cargo flamegraph --deterministic --palette rust

# Install/update all dev tools from crates.io
install:
    cargo install --locked flamegraph
