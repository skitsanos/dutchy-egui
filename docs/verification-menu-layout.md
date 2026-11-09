# Verification — menu overlay layout

Date: 2026-11-09. [DS-076](issues/DS-076.md), following the
[focus and cascade review](verification-menu-polish.md). Work remains local on
`ticket/ds-073-richer-menus`, based on `5d68a93`, version 1.0.25. No dependency,
version, commit or push change.

## Cause and remediation

Two new regressions failed before the fix. Opening File increased MenuBar's
layout response from 68 to 76 points high, moving following content by 8 points.
Attaching Dropdown to an existing button also advanced the parent's cursor
by 8 points, even with the popup disabled or closed.

The attachment reused a scope intended to draw a new trigger. Returning an
already drawn Response from that empty scope still advances egui's layout
cursor and adds item spacing. MenuBar invokes the attachment only while open,
so the otherwise constant spacing becomes an open-state layout shift.

Only `Dropdown::show` now allocates a scoped trigger. `show_for` and
`show_context` reuse their existing response directly. Stable popup IDs,
pre-trigger Tab routing and host-owned enabled state are preserved. Popup
positioning remains an overlay; no spacer compensation or negative margin is
used.

## Regression coverage

- MenuBar, following Toolbar and subsequent text retain identical rectangles
  through opening File, entering a child, switching to View, root dismissal,
  opening a Toolbar menu, entering its child and closing both levels.
- The geometry matrix covers Orange/light 1x, Orange/dark 1.5x and Blue/light
  2x, including a 240-point-wide host inside a vertical ScrollArea.
- Existing-response attachments leave the cursor unchanged for ordinary and
  context menus, open/closed and enabled/disabled. Original trigger identity
  and host-enabled state remain intact.
- Prior nested Tab, Escape, pointer ownership, dynamic menu/Toolbar and
  accessibility regressions continue to pass.

## Qualification

All 14 automated gates pass on macOS/arm64 with Rust 1.99.1, with no failed or
ignored tests. Changed Rust files remain below 400 lines; no lint suppression
or dependency was added.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | Pass |
| `cargo test --all-features` | 517 passed |
| `cargo test --no-default-features` | 366 passed |
| `cargo clippy --all-targets -- -D warnings` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo clippy --all-targets --no-default-features -- -D warnings` | Pass |
| Strict all-feature rustdoc (`RUSTDOCFLAGS='-D warnings'`) | Pass |
| All-feature/minimal WASM library checks | Both pass |
| Fonts-only all-target check and rendering/CodeBlock tests | Pass; 12 tests |
| Parser-only library/Markdown/CodeBlock tests | 48 passed |
| `cargo +1.95.0 check --lib --all-features --locked` | Pass |
| `cargo build --example showcase --all-features` | Pass |
| `cargo package --allow-dirty --locked --all-features` | Isolated archive build and audit pass |

The 648-file archive matches current sources and approved paths, includes
fonts/licenses and intended docs/baselines, and excludes media/storyboard,
IDE settings, temporary previews and generated snapshot differences. Generated
manifest, lock subset and VCS metadata checks pass, as do relative docs links
and `git diff --check`.

## Native and rendering evidence

The rebuilt macOS Glow showcase was checked in Orange/light and Blue/dark.
Closed/open captures retain the menu bar, toolbar and canvas positions;
switching File to View and opening the Toolbar popup also leave content fixed.
A 730-pixel vertical sample outside the overlays is identical between closed
and File-open images in both themes, and between closed and Toolbar-open in
light mode. Native nested Tab/Shift+Tab still reach the remembered Export tool
and preceding Delete action respectively. The preview was quit and confirmed
no longer running before its temporary bundle was removed.

Nine rich-menu baselines are intentionally refreshed to remove the redundant
spacing. Representative root/nested/checked renders were reviewed at light 1x,
dark 1.5x and Blue 2x. All 119 earlier baselines and 779 protected files remain
byte-identical. Evidence covers these macOS observations, headless interactions
and renderer checks; it does not establish other-platform or screen-reader
acceptance. Private maintenance receipts retain the failing reproduction,
commands, source hashes and native captures.
