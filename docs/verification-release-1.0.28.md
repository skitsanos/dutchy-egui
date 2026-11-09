# Release qualification — 1.0.28

Date: 2026-11-09. Follow-up to 1.0.27 (`0fd402e`) with shared commands and
shortcuts, [DS-087](issues/DS-087.md) through [DS-092](issues/DS-092.md), and
the review fixes [DS-093](issues/DS-093.md) through [DS-097](issues/DS-097.md).
The [implementation record](verification-command-workflow.md) and the
[independent review](verification-review-1.0.28.md) hold the interaction,
rendering and native evidence. Cargo.toml, Cargo.lock and CHANGELOG.md advance
together from 1.0.27. No dependency changed. The public library API gains
CommandRegistry, SplitButton and KeyRecorder; existing APIs are unchanged.

## Gates

All 21 gates pass on the final Rust source and baselines:

- `cargo fmt --all --check`: clean.
- `cargo test --all-features`: 596 passed, including 36 DutchyTodo tests;
  `cargo test --no-default-features`: 405 passed. No failures or ignores.
- `cargo clippy --all-targets -- -D warnings`, plus the all-features and
  no-default-features configurations: clean.
- Strict rustdoc (`RUSTDOCFLAGS="-D warnings"`), all features, no dependencies:
  clean.
- WASM library checks with all features and with no default features: pass.
- Bundled fonts only: all targets compile; 12 rendering/CodeBlock tests pass.
  Markdown parser only: 48 library/Markdown/CodeBlock tests pass.
- Rust 1.95.0 MSRV: locked all-features library check passes.
- `cargo build --examples --all-features`: pass.
- Crate archive build, archive inspector and audit, actionlint, shellcheck,
  relative documentation links and whitespace: pass.

A Docker rehearsal of the CI test step passes 596 tests with no
failures. It used Ubuntu 24.04 on arm64, the workflow's packages and software
Vulkan. No Rust source changed after these checks. The Todo light Orange 1x
and dark Blue 2x snapshots were regenerated for the header's version number.
All pixel differences are inside the final digit: 55 pixels at 1x and 172 at
2x. No other baseline changed with the bump. Changed Rust files stay below 400
lines.

## Crate archive

`cargo package --allow-dirty --locked --all-features` builds and verifies the
actual archive: 731 files. Every ordinary file matches the checkout;
Cargo's generated metadata files are inspected separately. The allowlist now
also approves `kittest.toml`. It still excludes media, storyboard work,
IDE/tool settings, repository scripts other than font preparation, and
generated snapshot comparisons. All font licences and third-party notices are
included, and relative documentation links pass. Cargo.toml differs from
1.0.27 in the version and the `kittest.toml` include entry; Cargo.lock differs
only in the package version.

## Native and hosted scope

Native macOS checks of Todo and Canal Studio in both themes, and the review's
showcase checks, predate the review fixes and the version bump. A native
relaunch after the fixes was covered by another window, so DS-093 to DS-095
rest on interaction and pixel tests. The hosted workflow first runs after the
approved push and is not verified here. Windows, Linux desktops and screen
readers remain outside this qualification.

## Delivery and cleanup

Commit and push are separate from qualification. After pushing, cleanup
inspects generated build outputs, stale worktree metadata and merged task
branches. It preserves the active checkout, tracked assets and snapshots,
media and IDE work, unrelated processes and unmerged or in-use work.
