# Tooltips and layout verification — 2026-11-06

Package: **dutchy-egui 1.0.5**, prepared on `ticket/tooltips-layout`.
The 1.0.4 stats/empty-states release was committed and pushed to main as
`c8c8747f6f2f96c9346761ecdb21f937aea4c0ed`; the remote main reference was checked.
This record was captured before the release commit; subsequent history is in Git.
Registry publication was not performed.
Host: macOS aarch64, Rust/Cargo 1.99.1.

No dependencies were added or upgraded. The crates.io API was checked on
2026-11-06: egui, eframe, and egui_kittest each report 0.37.0 as their latest stable
version. egui remains the sole direct runtime dependency. No third-party widget
code was copied; bundled font notices remain included.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 76 integration tests and one doctest.
- `cargo test --no-default-features`: 55 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive and extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.5/Cargo.toml --example showcase --locked`: independent packaged-example build.
- `git diff --check`; all 87 Rust/Python source files are below 400 lines
  (largest: 303).

Tooltip tests cover closed accessible descriptions with existing help retained,
unchanged names, native focus/activation, keyboard presentation, disabled pointer
help, Escape reaching the host, dismissal persisting under hover, re-entry,
clipped controls, preferred sides, edge flipping, and bounded wrapping. Native
egui clears button focus on Escape; the wrapper preserves that policy.

Layout tests cover token gaps, row wrapping/reverse order, centered maximum
width/gutters, empty grids, zero-column normalization, and stable editable
controls through column changes, reordering, and inserted siblings. The example
test preserves field focus through reflow, saves through Enter, and exercises
tooltip navigation/actions. The existing form validation/reset workflow passes
after adopting the grid. All snapshot checks pass without update mode.

## Rendered evidence

Reviewed WGPU baselines in `tests/snapshots/`:

- `layout-light-1x.png`: Orange/light.
- `layout-dark-1_5x.png`: Orange/dark.
- `layout-purple-2x.png`: Purple/light.

These cover wrapping toolbars, centered settings fields, keyed card columns,
focus outlines, and a visible inverted tooltip with its sharp arrow. Desktop
and 480-point application previews are in `target/visual-proof/`:
`tooltips-desktop.png`, `tooltips-narrow.png`, `layout-desktop.png`, and
`layout-narrow.png`. Reviewed centered/reflowed settings/dashboard content,
wrapping controls, reverse direction, and visible guidance. The navigation
sidebar now scrolls as its component inventory grows.

## Native evidence

The rebuilt Glow showcase ran through the temporary macOS app bundle. Native
accessibility exposed names/help for all four tooltip controls, disabled publish
help, and both field guidance and supplemental tooltip description. Tab reached
Above and displayed its inverted popup and arrow; Escape removed the popup.
The control's pointer action incremented the example count.

The Layout page exposed labeled settings, checkbox values, and named statistics.
Changing the name to "Native studio" updated the overview. Narrow preview
centered the constrained container and stacked the columns while retaining the
value. Save Settings incremented the displayed save count and exposed the
success message. Light/dark surfaces and the narrow composition were visually
inspected. Native execution and automated rendering are separate evidence.

## Limits

- Native execution is macOS only, without a running screen reader. AccessKit
  tests and native accessibility inspection provide narrower evidence.
- WASM evidence is library compilation, not browser execution. Windows/Linux
  GUI execution and an actual Rust 1.95 minimum-toolchain run remain outstanding.
- Layout uses egui sizing; it is not a CSS layout engine. Baseline/distribution,
  column wrapping, equal-height painted tiles, virtualization, and an HTML shell
  are outside this scope. Children must respect their available width.
- Tooltips are supplementary, noninteractive text. Native hover timing and
  focus policy apply. The host owns essential guidance, panel layout, values,
  saving, validation, navigation, and content replacement.
- Source-palette contrast limitations remain tracked in [DS-003](issues/DS-003.md).
  Core application essentials are complete; [coverage](coverage.md) still tracks
  partial controls and broader component parity. No registry publication occurred.
