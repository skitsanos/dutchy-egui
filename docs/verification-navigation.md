# Breadcrumbs and pagination verification — 2026-11-06

Package: **dutchy-egui 1.0.8**, developed locally on
`ticket/breadcrumbs-pagination`. Version 1.0.7 was committed and pushed to main as
`198ab48715e3b4e08c46a01b76da3af30bcecae4`; the remote reference was verified.
This record was captured before the 1.0.8 release commit; subsequent release
history is in Git. Registry publication was not performed. No dependencies were added or upgraded; egui 0.37.0 is the sole direct
runtime crate.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 133 integration tests, two unit tests, one doctest.
- `cargo test --no-default-features`: 96 integration tests, two unit tests, one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo build --example showcase`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`, including extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.8/Cargo.toml --example showcase --locked`: independent packaged-example check.
- `git diff --check`; 134 Rust/Python source files remain under 400 lines
  (largest: 344).

Host: macOS aarch64, Rust/Cargo 1.99.1. Feature checks use the bundled fonts and
host-font fallback configurations. Packaged sources retain the font licenses and
third-party notices.

## Interaction evidence

Breadcrumb tests cover native ancestor Enter/Space and pointer actions, current
page metadata and noninteraction, disabled destinations, collapsed menu keyboard
navigation/disabled skipping/Escape/focus return, stable identities after sibling
insertion/renaming/reflow, narrow bounds, empty/single paths, and compact icon
geometry independent of the accessible label length.

Pagination tests cover explicit host-owned requests, no current-page action,
current metadata, focus retention when a requested button becomes current,
previous/next actions, endpoint/empty/invalid inputs, maximum counts, stable IDs,
narrow wrapping, disabled controls, compact Tab order, and centered summaries.
Two model tests assert bounded, ordered page windows with endpoints/current page
and useful handling of single-page gaps. The showcase workflow applies ancestor
requests with a surviving focus target and replaces its sample record range.

The longer component navigation list now requires scrolling at the existing
1200x940 test viewport. Its theme/navigation test scrolls Badges into view before
clicking, exercising the existing sidebar rather than assuming every item fits.

## Rendered evidence

Reviewed WGPU baselines:
`tests/snapshots/navigation-{light-1x,dark-1_5x,purple-2x}-{focus,menu}.png`.
They cover both separator styles, a muted bar, collapsed ancestors, short and
ellipsis ranges, compact summaries, current/disabled pages, empty navigation,
keyboard focus, and open ancestor menus. Baselines pass without update mode.

Desktop and 480-point showcase previews are saved as
`target/visual-proof/{breadcrumbs,pagination}-{desktop,narrow}.png`.
`navigation-narrow.png` also exercises a 260-point component matrix. Rendering
review corrected allocation before wrapping scopes, unneeded hidden-label layout
for icon-only breadcrumbs, and compact/empty summary vertical alignment. Focus
rings, sharp borders, wrapping and menu positioning were reviewed across scales.

## Native evidence

The rebuilt Glow showcase exposed named breadcrumb navigation regions and Path
lists, with native ancestor buttons and a noninteractive current label. The icon
root retained the Home name. The collapsed menu opened; End focused Library,
Enter applied it in the host, changed the current path to Library, and focused
Restore Deep Path. Restoring and reopening then pressing Escape returned focus
to the ellipsis trigger.

On Pagination, choosing Page 2 changed Records 1–5 to Records 6–10 while retaining
Page 2 focus. Tab/Space activated Page 3, displaying Records 11–15. The large-range
Page 20 request reduced its visible window and disabled Next at the endpoint.
The compact Next request changed Page 3 of 12 to Page 4 of 12 independently.
Light and dark surfaces, focused controls, and centered compact summaries were
inspected. Native Glow and automated WGPU evidence are separate.

## Limits

Native GUI execution is macOS-only, without an active screen reader. Windows/
Linux GUI and browser WASM execution, plus an actual Rust 1.95 minimum-toolchain
run, remain outstanding. AccessKit metadata checks and native tree inspection
provide narrower evidence than screen-reader acceptance.

Routing, URL construction, data loading, page-size/jump inputs, responsive collapse
policy, and focus after external removal remain host responsibilities. The native
current-page pagination button remains focusable to preserve focus after page
application, unlike the web component's active span. Collapse menus use the
existing dropdown contract. No whole-system parity or blanket contrast claim is
made; source palette limits remain in [DS-003](issues/DS-003.md).
