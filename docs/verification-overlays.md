# Modal and drawer verification — 2026-11-06

Package: **dutchy-egui 1.0.10**, developed locally on `ticket/modals-drawers`.
Version 1.0.9 was committed and pushed to main as
`82e625c29a061b6c1fc220da9157e97257832bc0`; its remote reference was verified.
This record was captured before the 1.0.10 release commit; subsequent release
history is in Git. Registry publication was not performed. No dependencies were added/upgraded. The crates.io API confirmed
stable egui, eframe, and egui_kittest at 0.37.0; egui remains the sole direct
runtime crate.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 164 integration tests, four unit tests, one doctest.
- `cargo test --no-default-features`: 117 integration tests, four unit tests, one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo build --example showcase`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`, including extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.10/Cargo.toml --example showcase --locked`: independent packaged-example check.
- `git diff --check`; all 153 Rust/Python source files stay below 400 lines
  (largest: 351). Packaged source/docs/licenses match the working tree.

Host: macOS aarch64, Rust/Cargo 1.99.1. Bundled and host-font configurations were
checked. Logs: `target/visual-proof/{checks,package}-1.0.10.log`.

## Interaction and motion evidence

Ten overlay tests cover named modal/heading metadata, native Tab/Shift+Tab
confinement, default and explicit initial focus, close-button Space activation,
Escape/backdrop/content dismissal, disabled dismissal policies, no-close-button
heading fallback, reverse Tab wrapping to the footer rather than an extra title
stop, and pointer input blocked from underlying actions. Child popup
Escape is handled before the dialog. A native stacked-modal regression closes
the top layer first and returns focus to the parent, then the original opener.

Host-driven saving/closing, retained values, stable fields after sibling
insertion, external closure, and a replacement return target after opener removal
are covered separately. Drawer geometry is full-height and bounded on both edges
at 280 points. A header regression asserts a 32-point close target with the token
24-point horizontal/16-point vertical inset, plus the modal border.

Two raw-context motion tests exercise entrance movement toward both edges, final
placement, immediate reduced-motion placement, settled scheduling, and stopped
rendering/scheduling after closure. Native egui sizing and focus passes are allowed
to settle before repaint-delay assertions.

## Rendered evidence

Reviewed WGPU baselines:
`tests/snapshots/overlay-{light-1x,dark-1_5x,purple-2x}-{modal,left,right}.png`
and `overlay-close-hover.png`. Baselines pass without update mode. Coverage includes
centered/four-edge modal borders, full-height drawer edges, header title/close,
body/footer dividers, form controls, focus and hover, and blocking backdrops.

The 280x360 long-content previews are
`target/visual-proof/overlay-{modal,left,right}-narrow.png`. Tests use Tab to focus and reveal the footer, then exercise assistive scroll-into-
view and activation, verifying dismissal while the header remains reachable.
Rendering review corrected the shrinking title column that initially pulled
Close beside the title. Regressions also corrected drawer max-height allocation
and replacement focus targets being updated only on entry. A final keyboard
regression caught focus below the viewport; the owning scroll area now reveals
newly focused content controls without animation. No blurred shadows,
gradients, or rounded corners were introduced.

Five rendering/example tests additionally verify confirmation cancellation versus
host action execution, profile validation/focus, and retained drawer drafts with
host application and focus return. Native rendering is separate evidence below.

## Native evidence

The rebuilt macOS Glow showcase displayed 1.0.10. Opening confirmation exposed a
named dialog, level-two heading, and named Close button. Tab moved to Cancel;
Enter dismissed without changing the archive counter and returned focus to Open
Confirmation. Reopening and confirming incremented the local counter once and
returned focus to the same opener.

In Edit Profile, Tab reached the field; clearing it and saving kept the dialog
open, displayed the required-name error, and focused the invalid input. Entering
Grace Hopper and saving updated the host's visible profile and focused Edit
Profile. The right drawer filled the viewport height; editing its name then
Escape returned focus to Open Right Drawer. Opening the left drawer under dark
mode with reduced motion retained the edited value. Applying preferences
incremented the host counter once and focused Open Left Drawer. Light/dark
surfaces, both edges, headers, close placement, body/footer separation, and
backdrops were inspected.

## Limits

Native GUI execution is macOS-only, without an active screen reader. Windows/
Linux GUI, browser WASM execution, and an actual Rust 1.95 minimum-toolchain run
remain outstanding. AccessKit tests/native tree inspection do not establish
screen-reader acceptance or whole-system/browser parity. Source-palette contrast
limits remain in [DS-003](issues/DS-003.md).

Footers scroll with body content; they are not sticky. Modal opening and all
closing are immediate; drawers animate entry only. Docked, top/bottom, resizable,
or draggable overlays are outside this step. Host widgets must respect available
width; initial/return targets must be real, surviving enabled widget IDs. Calls
must continue while closed for deferred focus restoration. Form data, validation,
actions, and draft/persistence policy stay with the application.

## Confirmation width refinement

[DS-017](issues/DS-017.md) changes the archive confirmation showcase to the
512-point Medium token. Native dark-theme inspection at 2x confirmed both actions
on the same row. The footer retains wrapping and viewport constraints. Required
checks were rerun for this refinement; their log is
`target/visual-proof/confirmation-width-checks.log`.
