# Release qualification — 1.0.25

Date: 2026-11-08. Documentation release recording [DS-072](issues/DS-072.md),
the post-release status drift found while verifying 1.0.24. The library is
unchanged; the only Rust change is the Markdown showcase kicker. Cargo.toml,
Cargo.lock and CHANGELOG.md advance together from 1.0.24.

## Gates

Run on macOS 27.2 with Rust 1.99.1 on the exact tree being committed:

- `cargo fmt --all --check`: clean.
- `cargo test --all-features`: 490 passed, 0 failed, 0 ignored.
- `cargo test --no-default-features`: 342 passed, 0 failed, 0 ignored.
- `cargo clippy --all-targets -- -D warnings`, plus the all-features and
  no-default-features configurations: clean.
- `cargo doc --no-deps --all-features` with `RUSTDOCFLAGS="-D warnings"`: clean.
- `cargo check --lib` for `wasm32-unknown-unknown` with all features and with
  no default features: pass.
- `cargo check --all-targets --no-default-features --features bundled-fonts`
  (fonts-only showcase without the parser): pass.
- `cargo +1.95.0 check --lib --all-features --locked` (declared minimum): pass.
- `cargo build --example showcase --all-features`: pass.
- Relative documentation links: 134 files, none broken.

## Crate archive

`cargo package --allow-dirty --locked --all-features` built the archive. A
script compared every ordinary file in it with the checkout and with the
root-anchored `include` list: all files match, nothing is missing or
unexpected, and no IDE, media, tool-settings or snapshot-difference file is
present. The file count and archive hash are kept in the private receipt
under `.git/codex-maintenance/`.

## Delivery and cleanup

Commit and push follow only after the gates above, and the push is verified
against `origin/main` and the GitHub API. Afterwards the generated outputs
under `target/` (build, documentation, WebAssembly and package directories)
are removed, preserving `target/storyboard/`, `media/`, IDE settings and the
private receipts. No branch or worktree other than `main` exists.
