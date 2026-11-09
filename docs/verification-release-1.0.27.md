# Release qualification — 1.0.27

Date: 2026-11-09. Follow-up to 1.0.26 (`2feb3bf`) with the DutchyTodo
import-safety fixes [DS-085](issues/DS-085.md) and [DS-086](issues/DS-086.md).
The [fix verification](verification-todo-import-safety.md) records the
regressions, optimized arithmetic probe and native macOS import/Undo flow.
Cargo.toml, Cargo.lock and CHANGELOG.md advance together from 1.0.26.
No dependency or public library API changes.

## Gates

All 14 gates pass on the final Rust source and updated baselines:

- `cargo fmt --all --check`: clean.
- `cargo test --all-features`: 563 passed, including 33 DutchyTodo tests;
  `cargo test --no-default-features`: 379 passed. No failures or ignores.
- `cargo clippy --all-targets -- -D warnings`, plus the all-features and
  no-default-features configurations: clean.
- Strict rustdoc (`RUSTDOCFLAGS="-D warnings"`), all features, no dependencies:
  clean.
- WASM library checks with all features and with no default features: pass.
- Bundled fonts only: all targets compile; 12 rendering/CodeBlock tests pass.
  Markdown parser only: 48 library/Markdown/CodeBlock tests pass.
- Rust 1.95.0 MSRV: locked all-features library check passes.
- `cargo build --examples --all-features`: pass.

No Rust source changed after these checks. The Todo light Orange 1x and dark
Blue 2x snapshots were regenerated for the header's version number; all pixel
differences are inside the final digit (61 pixels at 1x, 202 at 2x). No other
baseline changed. Changed Rust files remain below 400 lines.

## Crate archive

`cargo package --allow-dirty --locked --all-features` builds and verifies the
actual archive in an isolated target directory: 692 files. All 688 ordinary
files match the checkout; Cargo's four generated metadata files are inspected
separately. The root-anchored package allowlist excludes media, storyboard
work, IDE/tool settings and generated snapshot comparisons. All font licences
and third-party notices are included. Relative documentation links pass.
Cargo.toml and Cargo.lock differ from the baseline only in the package version.

## Native scope

The fix qualification exercised real macOS file dialogs, import errors and
recovery, Undo-import, and exhausted creation with preserved drafts in both
themes. That native run predates the release bump; the version header and its
two baselines are the only subsequent rendering changes. Windows, Linux and
screen-reader sessions remain outside this qualification.

## Delivery and cleanup

Commit and push are separate from qualification. After pushing, cleanup
inspects generated build outputs, stale worktree directories/metadata and
merged task branches; it preserves the active checkout, tracked assets and
snapshots, media/IDE work, unrelated processes and unmerged or in-use work.
Private receipts and logs live under `.git/codex-maintenance/release-1.0.27/`.
