# Verification — IconButton, ToggleButton and Toolbar

This is the pre-release record of the component work; it shipped in
[release 1.0.24](verification-release-1.0.24.md).

Date: 2026-11-08. Local Unreleased work after repository release 1.0.23
(`cd2688fb8f5d8b42ad9d23ea73970491a4f6f79e`). No new dependencies.
At this original DS-062 qualification the README was 42 lines. It was later
expanded into a developer landing page. This report is historical; subsequent
fixes and Markdown integration are qualified in
[the combined record](verification-fixes-markdown.md).

## Interaction and rendering

- IconButton: stable explicit IDs, descriptive names including unpainted Unicode,
  square token targets at all sizes, pointer/Enter/Space and no activation while
  busy, disabled or inside disabled parents.
- ToggleButton: host bool, user-only change notifications, pressed metadata,
  stable names and identity under inserted siblings/external host changes.
- Toolbar: named/oriented group, one remembered Tab entry/exit, arrow/Home/End
  focus without activation, disabled/busy skipping, wraparound, typed requests
  that the host can reject, stable keys through reorder/renaming, removal repair
  and explicit empty fallback. Overflow reveals arriving focus once; manual wheel
  scrolling keeps focus without snapping back. Wrapping stays within the frame.
- Original and custom vector icons, inverse-card ink, hover feedback, disabled
  pressed distinction and enforced custom-painter clipping. Reviewed three new
  baselines: Orange/light 1x, Orange/dark 1.5x and Blue/light 2x. Existing tracked
  baselines are unchanged. No rounded or blurred widget rectangles.
- Three showcase pages exercise host action counts and shared formatting values;
  desktop and 480-point layouts pass accessibility/render checks. The sidebar
  keeps foundation pages first and all 38 pages appear once in sorted groups.

A regression exposed egui's default 48-point horizontal row reservation around
Small (32-point) toolbar controls and a ScrollArea viewport larger than its
painted row. Native review also caught a vertical toolbar inheriting its
horizontal parent's layout. Toolbar now reserves its own token height and
explicit content margin; returned viewports cover the actual content area. Vertical columns
set their own layout and reserve natural height; an explicit `.max_height(...)` enables
bounded vertical scrolling, including inside horizontal parents.

## Native macOS

Initial native review verified pointer/Enter icon activation, Space toggle
reversal, pressed values shared across pages, named Toolbar groups, arrows
without activation, Enter formatting, End/Space save, Tab exit and remembered
Shift+Tab reentry. Rebuilt native acceptance repeated formatting/save,
Tab exit/reentry, compact End/Space reveal/activation, vertical disabled/busy skipping, and reviewed all
three vertical targets together. Light/orange and dark/blue layouts passed.
The task-owned preview was closed; unrelated applications were preserved.

## Qualification and limits

All final checks passed: `cargo fmt --all --check`; `cargo test --all-features`
(452 passed) and `cargo test --no-default-features` (335 passed); all three
required Clippy configurations with `-D warnings`; strict all-feature rustdoc;
all-feature and no-default WASM library checks; and the native showcase build.
No failures or ignored tests. The formatting preview's Center/Right commands
also have headless rendered-placement assertions.

Package qualification uses an isolated Cargo verification build and an actual
archive-content comparison with the checkout. New sources/guides, snapshots,
font licenses and notices must be present; media, target outputs and generated
snapshot differences must be absent. Private maintenance receipts preserve
command results and archive hashes; no registry publication is performed.

No Windows/Linux native,
browser-runtime or screen-reader acceptance run is claimed. WASM compilation
alone does not prove browser behavior. Toolbar accepts action/toggle items;
arbitrary child inputs, menus, command shortcuts, persistence and rich-text
editing remain host concerns. Toggles are two-state.

The authorized 1.0.23 push preceded this implementation. Its cleanup reclaimed
32.55 GiB, preserved media/storyboard work and left only main locally/remotely.
This new component work remains local and Unreleased; no registry publication
or additional push is performed.
