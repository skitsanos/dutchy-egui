# Stepper verification — 2026-11-06

Package: **dutchy-egui 1.0.9**, developed locally on `ticket/stepper`.
Version 1.0.8 was committed and pushed to main as
`59a80b4832be86e75ae751f8379b86448ef97c18`; the remote reference was verified.
This record was captured before the 1.0.9 release commit; subsequent release
history is in Git. Registry publication was not performed. No dependencies were added or upgraded; egui 0.37.0 remains the sole
direct runtime crate.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 147 integration tests, four unit tests, one doctest.
- `cargo test --no-default-features`: 105 integration tests, four unit tests, one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo build --example showcase`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`, including extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.9/Cargo.toml --example showcase --locked`: independent packaged-example check.
- `git diff --check`; all 143 Rust/Python source files remain below 400 lines
  (largest: 344). Package source/docs/font licenses and third-party notices were
  checked against the working tree.

Host: macOS aarch64, Rust/Cargo 1.99.1. Feature checks exercise bundled fonts
and host-font fallback. The check log is `target/visual-proof/checks-1.0.9.log`.

## Interaction evidence

Nine component tests cover read-only/default behavior, optional pointer/Enter/
Space requests, current-step no-op, disabled navigation, native Tab traversal,
list/current/status metadata, host state preservation, keyed focus through
renaming/sibling insertion/adaptation, narrow bounds, and compact focus revealing
overflow. Empty, single, unknown-key, not-started, and complete states are
explicit and finite. Model unit tests check status and percentage boundaries.
Repeated-frame/focus geometry and fractional-width horizontal alignment have
separate regressions.

Five rendering/example tests cover variants and keyboard focus across themes/
scales, visible hover feedback, narrow adaptation/reflow, desktop/narrow showcase
rendering, and a complete host workflow. The latter gates advancement with
existing form validation, preserves values across Back/revisit, explicitly marks
completion, counts submission once, and restores meaningful focus after changes.

## Rendered evidence

Reviewed WGPU baselines: `tests/snapshots/stepper-light-1x.png`,
`stepper-dark-1_5x.png`, `stepper-purple-2x.png`, and
`stepper-navigation-hover.png`. Baselines pass without update mode. They cover
all four variants, current/completed/upcoming squares and connectors, descriptions,
optional navigation feedback, position percentages, and explicit completion.

Generated previews are `target/visual-proof/stepper-{desktop,narrow}.png` and
`stepper-matrix-narrow.png`, including 480-point showcase and 260-point component
widths. Progress labels reflow at a token minimum to avoid cramped trailing
characters. Native inspection exposed a fractional-width wrapping error that
put the final horizontal marker on a second row. Explicit native column rows
replaced automatic wrapping; regression widths and the rebuilt app confirm a
straight connector and aligned indicators.

## Native evidence

The rebuilt macOS Glow showcase displayed 1.0.9 and named workflow/list/status
metadata. Continue with an empty email stayed at step one and presented validation.
Typing the sample email advanced to the workspace field and focused it. After
entering a workspace name, Back and Continue retained both values. Clicking the
completed Account step revisited it; Shift+Tab from the workspace field then
Enter activated that same native step button and restored email-field focus.

The review stage focused Create Workspace. Space completed the host workflow,
showed all three steps completed, incremented submissions to one, and focused
Start Setup Again. Restart retained the sample values and returned to account.
Horizontal alignment and light surfaces were inspected, then dark vertical,
compact, position-progress, and completed-progress treatments were reviewed.
Native Glow and automated WGPU evidence are separate.

## Limits

Native GUI execution is macOS-only, without an active screen reader. Windows/
Linux GUI and browser WASM execution, plus an actual Rust 1.95 minimum-toolchain
run, remain outstanding. AccessKit metadata tests/tree inspection do not establish
screen-reader acceptance. This milestone does not claim whole-system parity or
blanket contrast compliance; source-palette limits remain in
[DS-003](issues/DS-003.md).

This is linear workflow presentation. Branching, skipped/failed steps, data,
validation, allowed navigation, asynchronous work, and focus after external panel
replacement stay with the host. Position can reach 100% while the final step is
still current; only explicit `Complete` marks the workflow finished.
