# File upload verification — 2026-11-06

**Release:** 1.0.13 was committed and pushed to `main` as
`c4a728d4a40f4b1f708daf62b2fc659092c94a5f`. A fresh remote-head query matched that
commit. Before release, the successful required check log and packaged-source
verification were inspected, and 374 distributed source/assets/docs files matched
the working tree. No registry publication or tag was requested.

**Development:** 1.0.14 remains local on `ticket/file-uploads-drag-drop`.
The crates.io API confirmed egui 0.37.0 and rfd 0.17.2 as current stable releases.
rfd is development-only; `cargo tree -e normal --depth 1` still shows only egui
in the default runtime graph. No widget/helper source was copied.

## Post-push cleanup

At the user's request, a cleanup subagent removed the 81 GiB `target/` artifact
tree and 13 unused merged local ticket branches. Only the active worktree existed;
worktree metadata was pruned. `main`, the active branch/current edits and unmerged
`origin/renovate/configure` were retained. No tracked source, snapshots, bundled
assets or unrelated directories were removed. The obsolete native showcase was
closed; development recreated build outputs needed for this milestone.

The rule is now in `AGENTS.md` for every future push. Generated target evidence
is disposable; tracked baselines and this record retain the conclusions. Snapshot
backup images are ignored and were removed before package qualification.

## Policy, drops, actions and host workflows

Four policy tests cover extension case/compound suffixes, exact/wildcard MIME
alternatives/parameters, unsupported types, missing names, unknown byte sizes,
zero/exact/over-limit sizes, retained-count capacity, rejection ordering,
non-consuming invalid candidates, overflow-safe counts, no-I/O basename extraction
and decimal size formatting.

Five drop-surface tests cover native click/Enter/Space browse requests, lazy
handle delivery without reads, spatial routing to one of two targets, outside
rejection, disabled/parent-disabled/clipped/modal-obscured targets, external drag
highlight without selection, and single handling across discarded layout passes.

Four file-list tests cover independent remove/cancel/retry requests, native
keyboard actions, host-controlled retention, known/unknown progress metadata,
stable keys/focus through reorder/rename, next-row/final-picker focus after actual
host removal, and disabled list/parent behavior.

Four showcase tests cover local simulated failure/retry/completion, cancellation
of only one host file with focus return, native path drops through background
metadata inspection with type/size rejection and no byte reads, and desktop/narrow
previews. No network transfer or real upload is performed by these tests/demo.

## Rendering and native evidence

Three rendering tests cover all transfer states, disabled rows, long unbroken
filenames, action hover geometry/feedback, and List/ListItem/ProgressIndicator
metadata. Reviewed baselines are `file-uploads-{light-1x,dark-1_5x,purple-2x}.png`
and `file-remove-hover.png` under `tests/snapshots/`. Narrow outputs at 280 and
160 points verify contained action/progress geometry. Very narrow rows omit the
decorative icon and stack actions to retain room for text/retry. Showcase outputs
are under `target/visual-proof/file-showcase-{desktop,narrow}.png`.

Native Glow verification on macOS opened the rfd picker, selected the local
44-byte `upload-sample.txt` fixture, added its row after metadata inspection, and
returned focus to the opening drop target. A simulated interruption showed Retry;
retry reached a 100% Complete track in dark mode. The temporary Finder window was
closed without altering the user's prior Finder window.

A physical cross-window OS drag was not qualified with the available UI geometry;
only deterministic egui hovered/dropped-input routing and native picker delivery
are claimed. Windows/Linux GUI, browser runtime/picker, active-screen-reader and
minimum Rust 1.95 toolchain acceptance remain separate. The library's WASM build
does not establish browser OS/file integration. Host I/O, content verification,
folder traversal and actual transfer cancellation remain outside widget scope.

See [file decisions](file-uploads.md) and [DS-024](issues/DS-024.md).

## Executed checks

Passed on macOS aarch64 with Rust/Cargo 1.99.1:

- `cargo fmt --all --check`
- `cargo test --all-features`: 261 integration tests, ten unit tests, one doctest.
- `cargo test --no-default-features`: 180 integration tests, ten unit tests,
  one doctest, including host-font fallback rendering.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo check --lib --all-features --target wasm32-unknown-unknown`
- `cargo build --example showcase --features url-images`

Log: `target/visual-proof/checks-1.0.14.log`. All 229 Rust source files are below
400 lines (largest: 344), and cognitive-complexity threshold 15 is enforced by
Clippy. Package/extracted-example commands and results are captured in
`target/visual-proof/package-1.0.14.log`. Tracked fonts/licenses/notices and
source/docs are included; local picker fixtures and generated artifacts remain
outside the distributed archive.
