# Progress and skeleton verification — 2026-11-06

Package: **dutchy-egui 1.0.3**, prepared on `ticket/progress-skeletons`.
This record was captured before the release commit; subsequent history is in Git.
The 1.0.2 cards/alerts release was committed and pushed to main as
`3c3b5ae614e453aab5be92410cfbc65371ac85f7`; the remote main reference was checked.
Host: macOS aarch64, Rust/Cargo 1.99.1. No dependencies were added or upgraded;
egui 0.37.0 remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 55 integration tests and one doctest.
- `cargo test --no-default-features`: 40 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive creation and extracted-library build.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.3/Cargo.toml --example showcase --locked`: independent packaged-example build.
- All 62 Rust/Python files are below 400 lines (largest: 303).

New tests cover clamping and NaN/infinity handling, finite fallback maxima,
determinate-to-unknown metadata clearing, stable IDs, focus preservation, narrow
geometry, hidden decorative blocks, named busy/live groups, zero count/size,
and all skeleton patterns. Motion tests check actual shape changes and repaint
scheduling: determinate, clipped, empty, paused, and reduced-motion indicators
do not run a continuous repaint loop. The application example checks progress
updates/completion/reset and content replacement while the next section keeps
the same position.

## Rendered evidence

Reviewed WGPU baselines under `tests/snapshots/`:

- `loading-light-1x.png`: Orange/light.
- `loading-dark-1_5x.png`: Orange/dark.
- `loading-crimson-2x.png`: Crimson/light.

The baselines cover progress fills/sizes, empty/full/custom-range and unknown
progress, inverse-surface labels/values, all five skeleton shapes, and four
compositions. Motion is reduced for deterministic snapshots. The full suite
passed again without snapshot-update mode; previous component baselines pass.
Animation tests separately verify time-dependent paint.

Generated desktop/narrow previews under `target/visual-proof/`:
`progress-desktop.png`, `progress-narrow.png`, `skeletons-desktop.png`, and
`skeletons-narrow.png`. Narrow card/list previews stack and table cells shrink.
Content uses vertical scrolling when needed.

## Native evidence

The rebuilt Glow showcase ran through the temporary macOS app bundle. Native
accessibility exposed named progress indicators and percentage values, including
35% for 420/1,200 records. Add 10 updated the example from 64% to 74%.
Unknown completion exposed "In progress". Skeleton groups appeared as named
loading regions without per-block accessibility content. Switching to loaded
content removed its busy group; the loaded action updated its own counter.
Restoring placeholders and light/dark rendering were checked. Reduced motion
was enabled before the final static review.

## Limits and release state

- Native execution was macOS only, without a running screen reader. AccessKit
  tests and native accessibility inspection provide narrower evidence.
- WASM evidence is library compilation, not a browser showcase. Windows/Linux
  GUI execution and an actual Rust 1.95 minimum-toolchain run remain outstanding.
- Compositions are inline placeholders. Real work, repaint after background
  updates, layout matching, and content replacement remain host-owned.
- Pulse easing approximates the live web CSS; the two-second period follows its
  implementation rather than the prose document's 1.5-second example.
- Source-palette contrast limitations remain in [DS-003](issues/DS-003.md).
- Registry publication was not performed. Remaining application essentials are
  tracked in [coverage](coverage.md).
