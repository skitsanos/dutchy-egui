# Development

Follow [AGENTS.md](../AGENTS.md), the [versioning policy](versioning.md) and the
[Issues Ledger](issues/). Use shared tokens, focused functions and composition;
keep code files generally below 400 lines. New or upgraded dependencies use
current stable releases. Keep dependencies minimal and retain applicable
third-party notices.

## Architecture

`tokens` owns the semantic palette, spacing, borders, motion, control sizes and
text. `theme` adapts tokens to egui defaults. `fonts` installs named static
faces without weight or fallback ambiguity. `text` builds layout jobs for
family, size, weight, tracking, leading and color. `interaction` centralizes
focus outlines, `widgets` holds the components and `layout` composes native
layouts with shared tokens.

Leaf widgets implement `egui::Widget` and return a `Response`; containers use
closure-based `show` methods. Values and durable state stay with the host, and
stateful controls take stable IDs. Theme updates don't reinstall fonts.
Sorting, filtering, upload transport and async validation belong to the host.
Repaint only while a visible animation needs it.

egui is the only required runtime dependency. The default `markdown` feature
adds pulldown-cmark without default features; `url-images` adds ehttp and
image; `png-export` adds image alone. eframe and egui_kittest are development
dependencies. Implement small helpers locally, and adapt compatible MIT code
only with provenance and notices.

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
Review changes at multiple themes and scales. Update only intentional
baselines, using `UPDATE_SNAPSHOTS=1` for the focused test, then rerun
normally. Compilation does not prove native or browser parity. Color and
contrast decisions and host palette overrides are recorded in
[DS-003](issues/DS-003.md) and [DS-027](issues/DS-027.md).

## Documentation

`docs/` holds guides, the [reference decisions](reference.md), the
[versioning policy](versioning.md) and the Issues Ledger. Each guide states what
a component supports and its remaining limits. Don't add per-change plans,
verification reports or receipts: a ledger entry's Resolution states its
evidence in a sentence or two, a release commit message summarizes its gate
results, and CHANGELOG.md holds the history.

## Release and cleanup

Every main push advances the version and updates Cargo.lock and CHANGELOG.md.
The DutchyTodo header shows the crate version, so its two baselines
(`dutchy-todo-light-1x`, `dutchy-todo-dark-2x`) change with every bump;
regenerate them for the example's rendering test and confirm only the version
digits differ.
Never push without authorization. Before delivery, run checks, generate and
inspect the actual crate, verify its sources and bundled license notices, and
ensure tests, image baselines, examples, video work and storyboard outputs are
excluded: the package carries the library, fonts with their licence notices and
docs. Explicit includes override gitignore, so keep package patterns
root-anchored. A failed gate stops delivery; commit and push are separate steps.

After pushing, inspect and clean generated repository outputs, obsolete clean
worktrees and metadata, and unused merged branches. Preserve the active
checkout, dirty, unmerged or in-use work, tracked assets and snapshots,
media and storyboard work, and unrelated processes. Registry publication is
separate from a main push.

## Automated qualification

Run `bash scripts/check.sh` after installing stable rustfmt and Clippy, the
`wasm32-unknown-unknown` target and Rust 1.95.0. It runs the required gates,
partial-feature profiles, the native example build and the actual crate
inspection. `python3 scripts/check_api.py` fails when a public enum, or a
struct with only public fields, is neither `#[non_exhaustive]` nor kept
exhaustive by the [versioning policy](versioning.md).
`python3 scripts/check_package.py [archive.crate]` also audits an existing
archive for approved paths, matching source bytes, licences, packaged links
and the crates.io upload limit. Tests and examples run from a repository
checkout; `cargo package` warns once for each unpackaged target, as expected.

The repository's `.github/workflows/ci.yml` runs the gate on Ubuntu with
software Vulkan and separately checks the library on macOS and Windows. It
uses read-only repository permissions and a pinned official checkout action;
no registry publication is performed. Native interaction remains a separate
check. Rendering tests that include platform key labels explicitly select macOS
formatting to keep their image baselines deterministic across hosts.
Baselines are rendered with macOS Metal and compared strictly there; refresh
them on macOS only. `kittest.toml` lets Linux software Vulkan differ in a
bounded number of pixels per snapshot, because it rasterizes some edges one
column differently. The macOS gate remains the authoritative visual check.

The quality job uses Ubuntu 26.04. Stable actionlint 1.7.12 predates that
hosted label; `.github/actionlint.yaml` adds only the verified `ubuntu-26.04`
label through its custom-label configuration. No lint rule is suppressed.
Remove the entry when a stable actionlint release includes the runner.
