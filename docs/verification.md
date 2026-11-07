# Visual proof verification — 2026-11-05

Package: **dutchy-egui 1.0.0**. Host: macOS aarch64, Rust/Cargo 1.99.1.
Direct crate versions were verified through crates.io: egui, eframe, and
egui_kittest 0.37.0. `cargo update --dry-run` proposes no dependency updates for
the declared Rust compatibility. The direct runtime dependency is egui only.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 16 integration tests and 1 doctest, passing without
  snapshot-update mode after baseline review.
- `cargo test --no-default-features`: 10 integration tests and 1 doctest. The
  test host supplies fonts; bundled-font rendering/face checks are not enabled.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`: archive creation and extracted library build.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.0/Cargo.toml --example showcase --locked`: the packaged example builds independently of the working source tree.
- Code-size inspection: every Rust/Python source file is below 400 lines.

## Rendered evidence

Reviewed WGPU image baselines in `tests/snapshots/`:

- Orange/light at 1x; Purple/light at 1.5x; Crimson/light at 2x.
- Orange/dark at 1x; Blue/light at 1.5x; Green/dark at 2x.
- Keyboard-focus and hover states.

The images cover all five button variants in normal/disabled/loading states,
all eight badge variants, typography, and the square spinner at a fixed phase.
Baselines use reduced motion and a predictable renderer. Desktop (1200×940)
and narrow (480×900) showcase previews are generated in `target/visual-proof/`.
The narrow-column regression checks wrapping, natural button width, and bounds.

The native `cargo run --example showcase` application also launched successfully
with Glow. A temporary macOS app bundle under `target/visual-proof/` enabled
native accessibility inspection. Verified action count changes, Purple/dark
switching, the named badge-close button, removal/restoration, and return to
Orange/light mode. The review window is left open with motion paused.

## Limits

- Native runtime verification is macOS only. Windows/Linux GUI execution has not
  been performed; WASM evidence is library compilation, not a browser showcase.
- The manifest declares Rust 1.95, matching egui's minimum; an actual Rust 1.95
  toolchain run has not been performed.
- Bundled fonts cover their upstream glyph ranges. The host supplies additional
  language/system fallbacks. Screen-reader interoperability has not been tested
  with a running screen reader; AccessKit names and actions were checked.
- Contrast limitations are tracked in [DS-003](issues/DS-003.md).
- Registry publication, commits, and pushes have not been performed. Later
  component milestones remain planned in [coverage](coverage.md).
