# Release qualification — 1.0.24

Date: 2026-11-08. Release of DS-062–071: IconButton, ToggleButton, Toolbar,
Markdown and CodeBlock, plus the reviewed layout, state and documentation fixes.
Cargo.toml, Cargo.lock and CHANGELOG.md advance together from main's 1.0.23.
The implementation and native rendering evidence are recorded in the
[combined integration report](verification-fixes-markdown.md).

## Source integration

The release incorporates remote Markdown commit
`684a8400547f504fae1b35860f6426bc8ff28384` and records it as a parent without
replacing the combined, qualified content. The branch's duplicated DS-058 is
renumbered DS-071; main's original package ticket and root-anchored archive
exclusions remain. Markdown is default-enabled and optional. CodeBlock stays
unconditional, and the fonts-only showcase works without the parser.

## Required gates

The release version is checked before committing/pushing with:

- `cargo fmt --all --check`.
- `cargo test --all-features` and `cargo test --no-default-features`.
- `cargo clippy --all-targets -- -D warnings`, plus the all-features and
  no-default-features configurations.
- Strict rustdoc; all-feature/minimal WASM library checks; the declared Rust
  1.95 minimum; fonts-only and parser-only runtime checks; native showcase build.
- `cargo package --allow-dirty --locked --all-features` in an isolated target
  directory, followed by inspection of the actual archive against approved
  paths, current bytes, font licenses, dependency metadata and VCS information.

The native acceptance already covers light/dark modes, toggle click/Space,
Markdown report-only links, anchors, heading demotion, live editing and Copy
feedback. Headless image and interaction checks cover multiple display scales.
This does not establish browser execution or screen-reader acceptance.

All release gates passed on macOS 27.2 with Rust 1.99.1: 490 all-feature tests,
342 minimal-feature tests, 12 fonts-only checks and 48 parser-only checks,
without failures or ignored tests. Formatting, all three Clippy configurations,
strict docs, both WASM configurations, the Rust 1.95 minimum and the native
showcase build pass. The archive contains 613 files; its 609 ordinary files
match the checkout, and generated Cargo/VCS metadata is checked separately.
No media/storyboard outputs, IDE/tool settings or snapshot differences enter
the package. Relative documentation links pass.

## Delivery and cleanup

Commit and push are separate from qualification; failed gates stop delivery.
After push, verify live origin/main equals the release commit before removing
generated artifacts, stale worktree metadata or unused merged branches. Keep
the Markdown history reachable from main before deleting its topic ref. Preserve
the active checkout, dirty/unmerged/in-use work, tracked assets and snapshots,
media/storyboard outputs, local IDE/tool settings and private check receipts.

Exact check results, archive hash, pushed SHA and cleanup/preservation inventory
are retained under `.git/codex-maintenance/` and reported with the release.
This repository release does not publish the crate to a registry.
