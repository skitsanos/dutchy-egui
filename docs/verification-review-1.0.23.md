# Post-release review — 1.0.20 to 1.0.23 and the Unreleased actions work

This is the pre-release review record; the fixes it describes shipped in
[release 1.0.24](verification-release-1.0.24.md).

Date: 2026-11-08. Reviewed releases `0fb3af6` (1.0.20) through `cd2688f`
(1.0.23) and the local Unreleased IconButton/ToggleButton/Toolbar tree against
`facb01c` (1.0.19), reading DS-049 through DS-062, the component guides, the
verification reports and the refreshed baselines. The initial read-only review added three ledger entries and this record.
Later sections document the subsequent fixes.

## Result

Every automated claim in the ledger and verification reports reproduced on the
current tree. Native inspection of the new pages found three defects, registered
as [DS-063](issues/DS-063.md) (grid cells justify wrapped text),
[DS-064](issues/DS-064.md) (hovered unpressed Outline toggles paint the pressed
surface) and [DS-065](issues/DS-065.md) (showcase scroll offset persists across
pages; mixed kicker names). These initial findings were subsequently resolved
as documented in the fix section below. Follow-up DS-067–069 and Markdown
integration are covered by [the combined record](verification-fixes-markdown.md).

## Automated evidence

macOS 27.2 / arm64, 2026-11-08, on 1.0.23 plus the Unreleased work:

- `cargo fmt --all --check` passed.
- `cargo test --all-features --no-fail-fast`: 452 passed, 0 failed, 0 ignored.
  `cargo test --no-default-features`: 335 passed, 0 failed, 0 ignored. Both
  match DS-062 and [the actions report](verification-actions-toolbars.md). No
  `*.diff.png`, `*.new.png` or `*.old.png` artifacts were produced.
- `cargo clippy --all-targets` with default, all and no default features, each
  with `-D warnings`: no warnings.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` and both
  `wasm32-unknown-unknown` library checks passed.
- Checked against the tree: 38 pages in two sorted groups (test and sidebar),
  22 built-in icons in `Icon::ALL`, README at 42 lines, a root-anchored Cargo
  include list, `cargo package --list` with 564 entries and no `media/`,
  `target/` or generated snapshot differences, rfd 0.18.0 in `Cargo.lock`,
  Renovate automerge disabled, every relative Markdown link in README,
  AGENTS.md, CHANGELOG.md and `docs/` resolving, and the new guide indexed in
  `docs/index.md`.
- Not repeated: earlier sessions' native acceptance runs, archive extraction and
  the isolated package build (the file list was checked instead), and registry
  publication, which has not occurred.

## Native and rendered evidence

Built the all-features showcase into a temporary app bundle and inspected it at
1x in a 1200 × 972-point window: Color picker (trigger, 16-point chevron inset,
popup anchored below the trigger with palette, hex and channel rows, Escape),
Media cards (search, sort, filters, three cards with independent save and
preview targets), Search input (Small/Medium/Large with magnifier and hint
chip), Icon buttons (22 icons plus a custom painter, five variants at three
sizes, busy, disabled and inverse card), Toggle buttons (text, icon, icon-only,
disabled pressed/unpressed, busy) and Toolbars (full width, 220-point overflow,
wrapped, vertical with disabled and busy items) in Orange/light, plus Toolbars
in Blue/dark. Reviewed the tracked actions, color-picker and collection
baselines at 1x, 1.5x and 2x in light, dark and Blue.

Findings are recorded in DS-063 to DS-065. One observation has no ticket: the
220-point overflow toolbar shows no affordance for hidden tools until the pointer
hovers, because egui's floating scrollbar is hidden at rest; the guide already
lists an overflow menu as out of scope.

## Fixes — same day

The user asked to fix all three findings. Each fix started from a failing
regression: `grid_cells_and_dutchy_text_keep_wrapped_lines_ragged`
(`tests/layout.rs`) and `title_and_description_stay_ragged_inside_justified_host_layouts`
(`tests/media_card.rs`) for DS-063, `toggle_hover_keeps_pressed_and_unpressed_surfaces_distinct`
(`tests/actions_rendering.rs`) for DS-064 and `switching_pages_opens_the_new_page_at_the_top`
(`tests/rendering.rs`) for DS-065. All four failed on the reviewed tree and pass
after the changes in `src/text.rs` (`add_ragged`), `src/layout/grid.rs`,
`src/widgets/media/card/{render,paint}.rs`, `src/widgets/actions/core.rs` and the
showcase (`gallery/mod.rs`, `pages.rs`, kickers in `colors.rs`, `primitives.rs`
and `actions/mod.rs`).

A first version of `add_ragged` wrapped the label in a non-justified child
scope; it kept text ragged but broke `add_sized` label columns in the color
picker (centering and reserved width), caught by the `color-picker-*`
baselines. The shipped version lays the job out with justification off and
hands the finished galley to egui's `Label`, which leaves allocation and
placement untouched; the color-picker baselines are byte-identical again.

Final matrix on the fixed tree: `cargo fmt --all --check`; `cargo test
--all-features --no-fail-fast` 456 passed, 0 failed, 0 ignored;
`cargo test --no-default-features` 337 passed, 0 failed, 0 ignored; the three
`cargo clippy --all-targets` configurations with `-D warnings`; strict rustdoc;
both `wasm32-unknown-unknown` library checks. Only the three `collection-*`
baselines changed (description word spacing), reviewed at 1x, 1.5x and 2x; no
`*.diff.png`, `*.new.png` or `*.old.png` artifacts remain. A rebuilt native
showcase confirmed ragged copy on the Color picker page, the "Native additions"
kicker, and a page opened after scrolling another now starting at the top.

While fixing DS-063 the narrow-card proof exposed MediaCard titles running
under the direction arrow, registered as [DS-066](issues/DS-066.md). On the
user's request it was fixed the same way: the narrow-card test now asserts the
title ends left of the arrow column (it failed at 172 against 158 points), and
`text::add_measured` lays the title out at the reserved width. No baseline
changed. The current combined matrix, including the later follow-up fixes, is
recorded in [verification-fixes-markdown.md](verification-fixes-markdown.md).

## README landing page

At the user's request the README was rewritten as a developer landing page
(118 lines): value proposition, light and dark showcase screenshots rendered
from the kittest harness into `docs/images/`, a grouped component overview
with names only, the Cargo feature table and the unchanged documentation and
license links. Component details, verification and development guidance stay
in `docs/`. The quick-start snippet was compiled in a temporary test before the
test was removed, and every relative link and image path resolves. With the
user's authorization the GitHub repository description was updated and fifteen
topics were added (rust, egui, eframe, gui, rust-gui, design-system,
ui-components, ui-kit, widgets, theme, dark-mode, accessibility, wasm,
desktop-app, immediate-mode-gui); both were read back from the repository.

## Cleanup

The task-owned showcase app was closed and its temporary bundle removed; check
logs live outside the repository. Nothing was committed or pushed. The
Unreleased changelog entry still needs a version advance before any push.
