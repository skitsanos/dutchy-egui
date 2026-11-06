# Toast verification — 2026-11-06

Package: **dutchy-egui 1.0.11**, developed locally on `ticket/toasts`.
Version 1.0.10 was committed and pushed to main as
`8132facc31aa12f88db05fdb507140c6cc617daf`; the remote reference was verified.
This record was captured before the 1.0.11 release commit; subsequent release
history is in Git. Registry publication was not performed. No dependencies were added/upgraded. The crates.io API confirmed
stable egui, eframe, and egui_kittest at 0.37.0; egui is still the sole direct
runtime crate.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 181 integration tests, six unit tests, one doctest.
- `cargo test --no-default-features`: 130 integration tests, six unit tests, one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo build --example showcase`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`, including extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.11/Cargo.toml --example showcase --locked`: independent packaged-example check.
- `git diff --check`; all 166 Rust/Python source files remain below 400 lines
  (largest: 359). Packaged source/docs/font licenses and notices match the working tree.

Host: macOS aarch64, Rust/Cargo 1.99.1. Both bundled fonts and host-font fallback
were checked. Logs: `target/visual-proof/{checks,package}-1.0.11.log`.

## Interaction and timing evidence

Eight interaction tests cover nonblocking host actions, no focus stealing on
arrival, native close Space/action Enter, single-use payloads without Clone,
FIFO pending slots, slot-count clamping, assertive errors/polite warnings,
disabled controls, and non-live history metadata. They assert child controls
remain inside the named toast region. Stable focus survives replacement/corner
changes; dismissal/host clear restore another toast or a surviving host target.
All four corners assert the exact 24-point gutter. Narrow overflow reveals a
focused close control.

Five raw-context timing tests cover one expiry event per entry; waiting entries
receiving a full lifetime; explicit pause, hover, focused controls, inactivity,
modal suspension and resume; clipped-content pause; persistent/maximum durations;
entry fading, reduced motion, and settled repaint scheduling. Two model unit
tests cover finite clock boundaries, ID wraparound among live keys, visible-only
ticking, replacement, and payload consumption.

## Rendered evidence

Reviewed WGPU baselines:
`tests/snapshots/toasts-{light-1x,dark-1_5x,purple-2x}.png` and
`toasts-close-hover.png`. Each matrix compares four solid severities against four
outlined treatments, with wrapped title/body/action text, close/action controls,
keyboard focus, and flat hover feedback. Baselines pass without update mode.

Desktop and 480-point showcase previews are
`target/visual-proof/toasts-{desktop,narrow}.png`. Four rendering/example tests
also verify host Undo/Retry execution once, retained host counts, and accessible
narrow composition. Eight interaction tests independently cover 280-point stack
scrolling and reveal of an off-screen keyboard control.

Rendering review corrected a doubled viewport gutter caused by combining an
inset anchor rectangle with another anchor offset. Outlined padding now matches
the documented 16 points; solid padding uses 24x16. Accessibility review moved
live metadata from a frame leaf to its actual parent, preserving controls in the
notification subtree. Shared status/close implementations are reused; no new
widget crate or third-party implementation was copied.

## Native evidence

The rebuilt macOS Glow showcase displayed 1.0.11. An action notification queued
while the native window was inactive stayed retained/hidden; activating the
window revealed it. Undo returned the host archive count from one to zero.
Retry Sync incremented the host retry counter once and removed its notification.

Queue Five Notices rendered three notifications and retained two waiting entries.
Tab from the host controls reached Dismiss Queued 1. Space removed it, changed
the counts to four total/one waiting, revealed Queued 4, and moved focus to Dismiss
Queued 2. Cmd+K opened the existing command palette, hid the toast stack and
retained all four entries. Escape restored the stack and its focused dismiss
control.

Dark outlined notifications were inspected at the top-left corner; light solid
notifications were inspected at bottom-right. The 24-point client-area gutter,
wrapped content, close placement and stack gaps were visible. With persistence
off, a new information toast expired after a 6.1-second observation interval:
the queue returned to zero, the expiry counter increased once, and focus stayed
on the host checkbox. Native Glow and automated WGPU evidence are separate.

## Limits

Native GUI execution is macOS-only, without an active screen reader. Windows/
Linux GUI, browser WASM execution, and an actual Rust 1.95 minimum-toolchain run
remain outstanding. AccessKit tests/native tree inspection do not establish
screen-reader acceptance, whole-system/browser parity, or blanket contrast
compliance; inherited palette limits remain in [DS-003](issues/DS-003.md).

The host owns pending capacity, deduplication, durable history, persistence, and
business actions. Keep calling the stack globally; omitting it leaves its clock
unsampled. Native modal state is reported from the preceding pass; toast areas
remain below modal/popup layers during the transition. Entry uses a fade rather
than the web slide; removal is immediate. There are no OS notifications, exit
transitions, or background services. Very tall entries pause expiry while clipped
and require scrolling or host removal.
