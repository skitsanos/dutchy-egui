# Verification — DutchyTodo example application

Shipped in release 1.0.26; see the [release record](verification-release-1.0.26.md).

Date: 2026-11-08. Local work after repository release 1.0.25 (`5d68a93`),
tracked as [DS-077](issues/DS-077.md). No dependency was added: the example
uses the existing `rfd`, `eframe` and `egui_kittest` dev-dependencies and a
plain-text exchange format instead of serde.

## Tests inside the example

`cargo test --all-features` builds `examples/dutchy_todo` as a test target
(`[[example]]` with `test = true`) and runs its 24 tests:

- Store: stable ids and undo restoration, blank-title fallbacks, counts
  (open, done, overdue, due today), filter/search ordering and project
  removal with in-place restoration; the seeded data is deterministic and
  relative to the supplied date.
- Format: readable export, round trip, id continuation after import,
  comments/blank lines/CRLF, and line-numbered errors for every malformed
  record.
- Keyboard flows with `egui_kittest`: row order and Up/Down/Home/End; Space
  completes with an undo toast whose action returns focus to the row; the
  Checkbox toggles by pointer without opening the editor; Enter opens the
  Drawer with the title focused and Escape returns focus to the row; Enter in
  the title saves; a blank title is rejected with the Field alert; `N`
  creates a task unless a text field has focus; Delete and Backspace remove
  the row, focus its neighbor and undo restores it; the editor's Delete
  button; filters, search and the empty states; project creation from the
  sidebar; project deletion through the Modal with undo; the welcome state
  without projects. `check_accessibility` runs on the list, the open Drawer
  and the empty application.
- Rendering: two baselines, `dutchy-todo-light-1x` (Orange) and
  `dutchy-todo-dark-2x` (Blue), with a keyboard-focused row.

## Gates

All five required checks passed on the final tree on 2026-11-08:
`cargo fmt --all --check`; `cargo test --all-features` with 514 tests passing
(the 490 existing tests plus the example's 24) and no failures or ignores;
and `cargo clippy --all-targets -- -D warnings` in the default, `--all-features`
and `--no-default-features` configurations. Without `bundled-fonts` the example
is skipped by its `required-features`, as the showcase is. Image baselines:
`tests/snapshots/dutchy-todo-light-1x.png` and `dutchy-todo-dark-2x.png`,
both reviewed. No existing baseline changed.

## Native macOS

Background access to the example window (bundle `design.dutchy.todo`) was
denied in this session, so the native run is a rendering run driven from the
shell through the support capture hooks: `DUTCHY_EXAMPLE_ZOOM`,
`DUTCHY_EXAMPLE_MODE` and `DUTCHY_EXAMPLE_CAPTURE` start the real eframe/Glow
window on the Retina display, save the settled frame and close. Six frames
were reviewed: light and dark at zoom 1, 1.5 and 2 with the 1120×760-point
window scaled by the zoom (at 2 the window clamps to the display, leaving
870×548 logical points, a small-window case).

The first pass found four small-window defects, all fixed the same day: the
status strip wrapped hint labels letter by letter and grew until the sidebar's
pinned actions overlapped its content; the project heading collided with the
New task controls; the Segmented filter was squeezed until its labels
overlapped (now measured by the host and stacked under the search field); and
clipped scroll content showed egui 0.37's fade gradient, zeroed by the support
theme. The final frames show one-line status strips that drop trailing hints,
scrolling sidebar content above the pinned actions, a wrapping heading, the
Stats grid reflowing to fewer columns at 2x, and flat clipped lists.
Pointer and keyboard interaction, the `rfd` dialogs and the Drawer were not
exercised natively; the kittest flows above cover them.

## Limits

Import and export open the host's `rfd` dialog, which the tests do not drive;
the format and the store replacement are covered by unit tests and the dialog
by the native run. No Windows or Linux run, no screen-reader session, and no
browser build: `rfd` and `eframe` are desktop dev-dependencies. Each task row
has two tab stops (row and Checkbox); see the gaps in DS-077.
