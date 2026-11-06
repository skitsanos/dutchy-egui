# Cards and alerts verification — 2026-11-06

Package: **dutchy-egui 1.0.2**, prepared on `ticket/cards-alerts`.
This record was captured before the release commit; subsequent history is in Git.
The 1.0.1 form release was committed and pushed to main as
`cefe635b2e0991e94ad316c8ffe3970df91d0a28`; the remote main reference was checked.
Host: macOS aarch64, Rust/Cargo 1.99.1. No dependencies were added or upgraded;
egui 0.37.0 remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 44 integration tests and one doctest.
- `cargo test --no-default-features`: 32 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive creation and extracted-library build.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.2/Cargo.toml --example showcase --locked`: independent packaged-example build.
- All 50 Rust/Python source files are below 400 lines (largest: 303).

New behavior tests cover pointer and Enter/Space card activation, child-button
precedence, disabled/parent-disabled cards, stable identities after sibling
insertion, scoped/nested text colors, separate alert action/dismiss events,
host-owned removal, explicit dismissal focus restoration, disabled notification
controls, severity/live metadata, and suppression of static announcements.
The showcase workflow verifies card pin/open and alert retry/dismiss/restore.

## Rendered evidence

Reviewed new WGPU baselines under `tests/snapshots/`:

- `surfaces-light-1x.png`: Orange/light.
- `surfaces-dark-1_5x.png`: Orange/dark.
- `surfaces-purple-2x.png`: Purple/light.

These cover all six card surfaces, native/Dutchy text, all four alert severities,
a solid banner with action/dismiss controls, a focused card, and a hovered card.
The complete suite then passed without snapshot-update mode. Existing form,
typography, and component baselines remain passing.

Generated and reviewed application previews in `target/visual-proof/`:
`cards-desktop.png`, `cards-narrow.png`, `alerts-desktop.png`, and
`alerts-narrow.png`. Narrow cards stack; messages wrap and actions remain below
their messages. The content area scrolls vertically when needed.

## Native evidence

The rebuilt Glow showcase ran through its temporary macOS app bundle. Native
accessibility exposed card headings, named parent/child buttons, disabled cards,
named notification containers, and separate action/dismiss buttons. Pin changed
only the pin counter; opening changed the open counter. Retry changed only its
counter. Dismiss removed the example alert and focused its restore button;
restoring brought it back. State persisted across Cards/Alerts navigation.
Light/dark surfaces and inverse text were visually inspected.

## Limits and release state

- Native execution was macOS only; no running screen reader was tested. Native
  accessibility inspection and AccessKit tests provide narrower evidence.
- WASM evidence is library compilation, not a browser showcase. Windows/Linux
  GUI execution and an actual minimum Rust 1.95 toolchain run remain outstanding.
- Cards/alerts are inline composition primitives. Navigation, message lifetime,
  focus targets, and action execution remain host-owned. Interactive cards with
  complex child controls warrant application-level assistive-technology testing.
- Source-palette contrast limitations remain in [DS-003](issues/DS-003.md).
- Registry publication was not performed. Remaining application essentials are
  tracked in [coverage](coverage.md).
