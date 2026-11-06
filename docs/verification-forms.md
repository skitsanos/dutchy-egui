# Form controls verification — 2026-11-06

Package: **dutchy-egui 1.0.1**, branch `ticket/form-controls-validation`.
The 1.0.0 baseline was committed and pushed to main as
`f94408969939c322680273c9933bb9f1e373825a`. This verification record was captured
before the form release commit; subsequent release history is recorded in Git.
Host: macOS aarch64, Rust/Cargo 1.99.1. Current stable egui, eframe, and
egui_kittest versions were rechecked through crates.io: **0.37.0**.
egui remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 35 integration tests and one doctest.
- `cargo test --no-default-features`: 26 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`: archive creation and extracted-library build.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.1/Cargo.toml --example showcase --locked`: independent packaged-example build.
- All Rust and Python source files remain below 400 lines.

Tests exercise label/pointer focus, required/help/error/busy metadata, change/
blur/submit timing, validation only on events, Unicode scalar limits, read-only
and disabled editors, password protection, keyboard checkbox/radio/select use,
focus restoration, stable IDs, shared heights, and stale/cross-field/single-use
async tickets. The application example is exercised through validate and reset.
The Unicode paste test permits a missing-glyph tofu for its fox emoji; scalar
count behavior does not depend on glyph availability. Other rendering tests
retain strict missing-glyph checks.

## Rendered and native evidence

Reviewed `tests/snapshots/forms-light-1x.png` and `forms-dark-1_5x.png`, then ran
the complete suite without snapshot-update mode. These include normal, invalid,
read-only, selected, counted, and pending field states. Existing theme/scale
baselines also continue to pass.

Reviewed generated application previews in `target/visual-proof/`:
`forms-desktop.png`, `forms-desktop-errors.png`, and `forms-narrow.png`.
The form switches from two columns to a stacked layout at narrow widths.

The native Glow showcase was rebuilt and launched through the temporary macOS
app bundle under `target/visual-proof/`. Inspected named text/password/read-only
fields and native accessibility metadata; exercised invalid-to-valid email blur
feedback, role selection, checkbox activation, and light/dark switching.
Dropdown options retain egui's native selectable-button roles because custom
option-role overrides lost their names in the macOS accessibility bridge.

## Limits

- Native execution was macOS only. WASM evidence is library compilation, not a
  running browser application; Windows/Linux GUI execution was not performed.
- Rust 1.95 is the declared minimum, but this run used Rust 1.99.1.
- No running screen reader was tested. AccessKit tests and macOS accessibility
  inspection provide narrower evidence.
- Interactive textarea resizing remains planned. RadioGroup does not aggregate
  child blur; Select blur refers to its trigger. Use change/submit validation for
  these choice controls, as documented in the [API guide](forms.md).
- The availability check is a deterministic local example, with no remote
  service. The host supplies real async execution, repaint, and server validation.
- Source-palette contrast limitations remain in [DS-003](issues/DS-003.md).
- Registry publication has not been performed.
