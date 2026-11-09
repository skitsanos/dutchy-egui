# Development and verification

Follow [AGENTS.md](../AGENTS.md), the [plan](plan.md) and the [Issues Ledger](issues/).
Use shared tokens, focused functions and composition; keep code files generally
below 400 lines. New/upgraded dependencies use current stable releases. Keep
dependencies minimal and retain applicable third-party notices.

## Required Rust checks

```sh
cargo fmt --all --check
cargo test --all-features
cargo test --no-default-features
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features
cargo check --lib --all-features --target wasm32-unknown-unknown
cargo check --lib --no-default-features --target wasm32-unknown-unknown
```

Rendering tests need a WGPU-capable adapter; software Vulkan works on Linux.
Review changes at multiple themes/scales. Update only intentional baselines,
using `UPDATE_SNAPSHOTS=1` for the focused test, then rerun normally. Generated
proofs live under target. Compilation does not prove native/browser parity;
the component-specific verification reports state actual platform evidence.
Color/contrast decisions and host palette overrides are recorded in
[DS-003](issues/DS-003.md) and [DS-027](issues/DS-027.md).

## Release and cleanup

Every main push advances the version and updates Cargo.lock and CHANGELOG.md.
The DutchyTodo header shows the crate version, so its two baselines
(`dutchy-todo-light-1x`, `dutchy-todo-dark-2x`) change with every bump;
regenerate them for the example's rendering test and confirm only the version
digits differ.
Never push without user authorization. Before delivery, run checks, generate
and inspect the actual crate, verify its sources and bundled license notices,
and ensure tests, image baselines, examples, video work and storyboard outputs
are excluded: the package carries the library, fonts with their licence
notices and docs. Explicit includes override gitignore, so keep package patterns
root-anchored. A failed gate stops delivery; commit and push are separate steps.

After pushing, inspect and clean generated repository outputs, obsolete clean
worktrees/metadata and unused merged branches. Preserve the active checkout,
dirty/unmerged/in-use work, tracked assets/snapshots, media/storyboard work and
unrelated processes. Record cleanup and retained work in private maintenance
receipts. Registry publication is separate from a main push.

## Automated qualification

Run `bash scripts/check.sh` after installing stable rustfmt/Clippy, the
`wasm32-unknown-unknown` target and Rust 1.95.0. It runs the required gates,
partial-feature profiles, native example build and actual crate inspection.
`python3 scripts/check_package.py [archive.crate]` also audits an existing
archive for approved paths, matching source bytes, licences, packaged links
and the crates.io upload limit. Tests and examples run from a repository
checkout; `cargo package` warns once for each unpackaged target, as expected.

The repository's `.github/workflows/ci.yml` runs the gate on Ubuntu with
software Vulkan and separately checks the library on macOS and Windows. It
uses read-only repository permissions and a pinned official checkout action;
no registry publication is performed. Hosted execution begins after an
approved push or pull request. Native interaction remains a separate check.
Rendering tests that include platform key labels explicitly select macOS
formatting to keep their image baselines deterministic across hosts.
Baselines are rendered with macOS Metal and compared strictly there; refresh
them on macOS only. `kittest.toml` lets Linux software Vulkan differ in a
bounded number of pixels per snapshot, because it rasterizes some edges one
column differently. The macOS gate remains the authoritative visual check.

The quality job uses Ubuntu 26.04. Stable actionlint 1.7.12 predates that
hosted label; `.github/actionlint.yaml` adds only the verified `ubuntu-26.04`
label through its custom-label configuration. No lint rule is suppressed.
Remove the entry when a stable actionlint release includes the runner.
