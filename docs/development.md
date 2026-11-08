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
Never push without user authorization. Before delivery, run checks, generate
and inspect the actual crate, verify its sources and bundled license notices,
and ensure video work, storyboard outputs and generated snapshot differences
are excluded. Explicit includes override gitignore, so keep package patterns
root-anchored. A failed gate stops delivery; commit and push are separate steps.

After pushing, inspect and clean generated repository outputs, obsolete clean
worktrees/metadata and unused merged branches. Preserve the active checkout,
dirty/unmerged/in-use work, tracked assets/snapshots, media/storyboard work and
unrelated processes. Record cleanup and retained work in private maintenance
receipts. Registry publication is separate from a main push.
