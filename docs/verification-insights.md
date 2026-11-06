# Stats and empty states verification — 2026-11-06

Package: **dutchy-egui 1.0.4**, prepared on `ticket/stats-empty-states`.
This record was captured before the release commit; subsequent history is in Git.
The 1.0.3 progress/skeletons release was committed and pushed to main as
`d750373ae2875d8f203765bc86b0f2e4b13b5590`; the remote main reference was checked.
Host: macOS aarch64, Rust/Cargo 1.99.1. No dependencies were added or upgraded;
egui 0.37.0 remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 65 integration tests and one doctest.
- `cargo test --no-default-features`: 47 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive creation and extracted-library build.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.4/Cargo.toml --example showcase --locked`: independent packaged-example build.
- All 74 Rust/Python source files remain below 400 lines (largest: 303).

New tests cover preserved host formatting and missing-value strings, independent
trend direction/sentiment/context, non-focusable metrics, stable identities after
grid reflow/sibling insertion, long values, narrow bounds, and style isolation.
Empty-state tests cover heading metadata, opt-in announcements, separate pointer/
Enter/Space actions, disabled/parent-disabled controls, action focus after guidance
and width changes, and stacked narrow controls. The example workflow verifies
metric refresh, project/template actions, search clearing, entry/widget creation,
host-directed focus after replacement, and reset.

## Rendered evidence

Reviewed WGPU baselines under `tests/snapshots/`:

- `insights-light-1x.png`: Orange/light.
- `insights-dark-1_5x.png`: Orange/dark.
- `insights-purple-2x.png`: Purple/light.

They cover four stat surfaces, positive/negative/neutral trend context, compact
rows, accented detail, search guidance, and dashboard setup/actions. The complete
suite passed again without snapshot-update mode; prior baselines remain passing.
Rendering review caught a trend-arrow baseline mismatch; a fixed icon column
beside wrapping trend text resolved it before acceptance.

Generated and reviewed application previews in `target/visual-proof/`:
`stats-desktop.png`, `stats-narrow.png`, `empty-states-desktop.png`, and
`empty-states-narrow.png`. Narrow stats reflow and empty actions stack; the
application content scrolls vertically as needed. Generic/table patterns are
included in the application previews.

## Native evidence

The rebuilt Glow showcase ran through the temporary macOS app bundle. Native
accessibility exposed named stat groups with formatted values and explicit trend
context. Refresh changed Revenue from $48,290 to $52,460. Empty-state headings,
guidance, and independent action buttons were named; decorative icons did not
appear as content. The template action changed only its request count. Create
replaced the empty region with project content and focused Reset Projects;
Clear Search replaced search guidance with results and focused Apply Sample
Filters. Navigation retained host state. Light/dark statistic surfaces were
visually inspected with reduced motion enabled.

## Limits and release state

- Native execution was macOS only, without a running screen reader. AccessKit
  tests and native accessibility inspection provide narrower evidence.
- WASM evidence is library compilation, not a browser showcase. Windows/Linux
  GUI execution and an actual Rust 1.95 minimum-toolchain run remain outstanding.
- Values/analytics, navigation, real content, and action execution remain host-owned.
  Table empty states supply a body, without table headers or data behavior.
- Source-palette contrast limitations remain in [DS-003](issues/DS-003.md), including
  small semantic trend text and some on-primary text.
- Registry publication was not performed. Remaining application essentials are
  tracked in [coverage](coverage.md).
