# Close-control feedback and spacing — 2026-11-06

Local follow-up for **dutchy-egui 1.0.7**, on `ticket/menus-dropdowns`.
This record was captured before the release commit alongside the menu/dropdown
step; subsequent release history is in Git.
No dependencies were added or upgraded. The sole direct runtime dependency
remains egui 0.37.0.

## Geometry and behavior

Before correction, the alert's empty frameless button produced a response with
zero width and 32-point height. The custom-painted glyph extended beyond it.
The new response is explicitly 32x32 logical points. Both top and right target
insets are 16 points; a centered 16-point glyph gives 24-point visual insets on
both axes. These measurements are asserted for tinted alerts and solid banners
at 280- and 760-point viewport widths, with wrapping content and an action.
A pointer press near the close target's corner activates on release.

`tests/close_controls.rs` covers alert/banner, tab, and tag close controls. Pixel
comparisons verify that hover changes occur within the target, pressed feedback
differs from hover, disabled hover leaves the render unchanged, and a pointing
cursor is emitted. Hover/press do not activate early or switch the selected tab;
release returns one host action. Existing tests retain Enter/Space activation,
independent badge/tab actions, disabled behavior, stable IDs, and dismissal focus
restoration.

## Rendering and native evidence

Reviewed twelve WGPU baselines:
`tests/snapshots/close-{light-1x,dark-1_5x,purple-2x}-{alert,banner,tab,tag}-hover.png`.
They include all eight removable-tag variants, selected/unselected/disabled tabs,
tinted alerts, and inverse banners. Pressed/disabled feedback is checked by image
comparison. Three `surfaces-*` baselines were updated for the corrected alert
geometry and resting glyph emphasis. All baselines pass without update mode.

The rebuilt macOS Glow showcase was inspected with the native accessibility tree
and rendered UI. Alert close hover showed a flat square and stronger glyph, with
equal visible top/right spacing. Clicking near the square's corner dismissed the
alert and focused Restore Connection Alert; the alert was then restored. The tag
close showed its hover surface without removal. Hovering Document 2's close
control showed its own square/glyph while Document 1 remained selected.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 119 integration tests and one doctest.
- `cargo test --no-default-features`: 86 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`, plus an independent packaged showcase check.
- `git diff --check`; Rust/Python source files remain below the 400-line soft cap.

Host: macOS aarch64, Rust/Cargo 1.99.1. Native execution remains macOS-only,
without an active screen reader. WASM is compile-checked, not browser-executed.
Source-palette contrast limitations remain in [DS-003](issues/DS-003.md).
