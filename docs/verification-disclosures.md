# Accordions and tabs verification — 2026-11-06

Package: **dutchy-egui 1.0.6**, prepared on `ticket/accordions-tabs`.
The 1.0.5 tooltips/layout release was committed and pushed to main as
`954ca2318a6946f6bee95a13594fc66cdd873a12`; the remote reference was checked.
This record was captured before the release commit; subsequent history is in Git.
Registry publication was not performed.
Host: macOS aarch64, Rust/Cargo 1.99.1. No dependencies were added or upgraded;
egui 0.37.0 remains the sole direct runtime dependency.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 96 integration tests and one doctest.
- `cargo test --no-default-features`: 69 integration tests and one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo build --example showcase`
- `cargo package --allow-dirty`: archive and extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.6/Cargo.toml --example showcase --locked`: independent packaged-example build.
- `git diff --check`; all 103 Rust/Python source files remain below 400 lines
  (largest: 324).

Accordion checks cover single/multiple host state, closed/unknown keys, body
callback visibility, expanded/controls/region relationships, independent child
actions, native Enter/Space, Up/Down/Home/End navigation, disabled items/parents,
stable editor focus after reordering/resizing/sibling insertion, narrow bounds,
and retained host spacing. Chevron tests verify animation scheduling stops for
reduced motion, clipped headers, and settled transitions.

Tab checks cover manual activation, arrow wrapping/disabled skipping, roving Tab
order, panel relationships, close requests without implicit selection/removal,
Delete, surviving/empty focus destinations, disabled editing/actions, missing
selection normalization, empty/all-disabled lists, stable editor IDs after
reordering/renaming/resizing, and keyboard reveal of overflow. A nested vertical
scroll regression checks the tab strip's actual height. Example workflows verify
saved settings, mode switching, retained document notes, adjacent selection after
close, and adding after the last document is closed.

## Rendered evidence

Reviewed WGPU baselines under `tests/snapshots/`:

- `disclosure-light-1x.png`: Orange/light.
- `disclosure-dark-1_5x.png`: Orange/dark.
- `disclosure-purple-2x.png`: Purple/light.
- `tab-roving-focus.png`: an unselected keyboard target retains its outline.
- `accordion-keyboard-focus.png`: complete inset focus edges above the next row.

They include open/closed/disabled accordion items, both accordion surfaces,
interactive bodies, underline/segmented tabs, close affordances, and sharp focus.
All baselines pass again without update mode. Desktop/480-point application
previews are under `target/visual-proof/`: `accordions-desktop.png`,
`accordions-narrow.png`, `tabs-desktop.png`, and `tabs-narrow.png`.

Rendering review corrected a baseline placed at ScrollArea's pre-layout height,
focus clipping at the first tab, and panel spacing inheritance. Native review
then exposed a zero-height pre-layout rect below a vertical scroll area's initial
bounds; using the horizontal content height fixed the baseline while scrolling.
The regression test covers that nesting.
User review removed the extra outline on selected tabs; active underlines/surfaces
remain, with focus cues on unselected keyboard targets, close controls, and panels.
The shared focus-visible policy distinguishes pointer presses from keyboard and
assistive actions without removing focus. Accordion rings are inset, preventing
neighboring fills from covering the bottom edge. Input-mode persistence is tested.

## Native evidence

The rebuilt Glow showcase ran through the temporary macOS app bundle. Accordion
buttons and visible regions were named. A nested name field was changed to
"Native workspace"; Apply incremented its count without collapsing the section.
Up/Down moved header focus and Enter toggled content. Reopening retained the
name and action count.

The tab strip exposed named tab groups, selected/disabled tabs, and active panels.
Right moved focus to Workspace Settings while it remained unselected; Enter
selected it and exposed the field. Editing and saving showed a success message.
The rebuilt document strip retained its baseline during vertical scrolling.
Closing Document 1 selected/focused Document 2. Delete advanced focus through
surviving documents, the last close focused Add Document, and Enter created
Document 6. Yearly changed the sample interval and dark surfaces were inspected.
Native execution and WGPU rendering are separate evidence.
Follow-up native review verified complete inset accordion rings after keyboard
navigation, no extra ring after a coordinate-based pointer click, and the selected
Workspace Settings tab showing its active underline without a surrounding box.

## Limits

- Native execution is macOS only, without a running screen reader. Native
  accessibility inspection and AccessKit tests provide narrower evidence.
- WASM evidence is library compilation, not browser execution. Windows/Linux
  GUI execution and an actual Rust 1.95 minimum-toolchain run remain outstanding.
- The host owns durable state, business actions, unsaved-change prompts, data
  retention, and focus when externally removing a focused panel child. Inactive
  panel callbacks do not run. A single-open layout may settle on the next repaint.
- Overflow uses native horizontal scrolling and keyboard reveal. Overflow menus,
  explicit arrow buttons, icon slots, docking, reorder dragging, vertical tabs,
  and panel-height animation remain outside this step.
- Source-palette contrast limits remain in [DS-003](issues/DS-003.md). This is the
  first interaction-components step; [coverage](coverage.md) tracks remaining work.
