#!/usr/bin/env bash
# Install stable rustfmt/clippy, wasm32 and Rust 1.95.0 before running this gate.
set -euo pipefail
cargo fmt --all --check
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo clippy --locked --all-targets --no-default-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps --all-features
cargo check --locked --lib --all-features --target wasm32-unknown-unknown
cargo check --locked --lib --no-default-features --target wasm32-unknown-unknown
cargo check --locked --all-targets --no-default-features --features bundled-fonts
cargo test --locked --no-default-features --features bundled-fonts --test rendering --test code_block
cargo test --locked --no-default-features --features markdown --lib --test markdown --test markdown_links --test markdown_composition --test code_block
CARGO_TARGET_DIR=target/msrv cargo +1.95.0 check --locked --lib --all-features
cargo build --locked --examples --all-features
cargo package --allow-dirty --locked --all-features
python3 scripts/check_package.py
