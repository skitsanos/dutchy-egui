# Post-review verification — 1.0.17

Date: 2026-11-07. Reviewed `17fc52d` (1.0.16) and `9d841db` (1.0.17)
against `e5d60a0` (1.0.15), reading DS-027 through DS-044 and the affected
implementations, regression tests, guides and refreshed baselines. The existing
uncommitted `docs/plan.md` change was preserved.

## Result

The recorded defect fixes passed their automated checks and the native flows
below. This does not close the entries' explicitly retained design differences.
Native verification found one additional sidebar defect, fixed locally as
[DS-045](issues/DS-045.md). The earlier sidebar test explicitly requested
scrolling, so it did not prove keyboard-focus reveal. The new test failed on
the released code and passes after the fix, at 600- and 940-point heights;
retained focus also permits wheel scrolling away without moving the footer.
The rebuilt native app passed the same Shift+Tab path: Tooltips and all four
focus-outline edges were visible, and wheeling the sidebar back to Overview
kept Tooltips focused without snapping back.
One stale table-guide sentence was corrected under [DS-041](issues/DS-041.md).

## Automated evidence

macOS 27.2 / arm64, Rust and Cargo 1.99.1:

- `cargo fmt --all --check`.
- `cargo test --all-features`: 348 tests, including unit, integration and
  doctests; no ignored tests. Rendering baselines compared without update mode.
- All three required `cargo clippy --all-targets` configurations, with default,
  all and no default features, each using `-- -D warnings`.
- `cargo test --no-default-features`: 250 tests; no ignored tests.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`.
- WASM library checks with `--all-features` and `--no-default-features`.
- Native all-features showcase build.

Required checks were repeated after the DS-045 Rust change. The additional
no-default test, rustdoc and WASM runs cover the reviewed library; DS-045 only
changes the showcase navigation and its bundled-fonts test. Existing rendering
tests cover 1x/1.5x/2x, light/dark, the five accents and narrow layouts.
No tracked snapshot was changed. Logs and disposable fixtures are under
`target/review-1.0.17/`; this record remains after artifact cleanup.

## Ticket coverage

| Entries | Evidence examined |
| --- | --- |
| DS-027, DS-028, DS-041 | Contrast decisions, corrected historical claims, current release/guide scope. The earlier live-web measurements were not repeated. |
| DS-029 | Tooltip width, padding and tracking assertions; native keyboard display and Escape dismissal. |
| DS-030, DS-034 | Stat/empty-state and field/card gap assertions and baselines; native compact stats, empty action and card footer. |
| DS-031, DS-042 | Layer/repaint and guarded-clock regressions; native command selection, modal/drawer entry, Escape and opener focus return. |
| DS-032 | Original-case accessibility assertions; native names remain mixed-case while visible labels are uppercase. |
| DS-033 | Painted alert color/icon assertions; native muted descriptions, circle glyphs, retry, dismissal and focus return. |
| DS-035 | Pagination/breadcrumb paint assertions; native foreground page numbers, bold summary numbers and host page changes. |
| DS-036 | Accordion body-hover and tab/focus regressions; native body hover and close-control arrow locking. Arrows stay on a focused close button, as its test specifies. |
| DS-037, DS-044 | Inverse-card form pixel assertions and dual theme-style checks; native editors remain readable through theme changes. |
| DS-038 | File/image/toast/table scrolling regressions; native retained file-action, restored image and sticky-header focus permit manual scrolling. |
| DS-039 | Unit-boundary and gesture-cap regressions; native picker displays 999,950 bytes as 1.0 MB; Scribble draft cancellation, undo/redo and read-only scrolling. |
| DS-040 | Gallery gap/opacity/tracking assertions and decoder panic unit test; native spaced gaps, lightbox return, loading/retry and successful Picsum display. |
| DS-043 | Duplicate-key repaint and measurement eviction tests; native sorting, centered rows and sticky body scrolling. |

## Native scope and limits

Opened all 31 showcase pages. Inspected the affected views at the native Retina
scale, across all five accents and both themes; used the layout page's narrow
preview and reduced-motion switch. Exercised dropdown arrow/Enter selection,
filtered command activation, form pending/error/valid feedback, card child
actions, toast queue/dismiss/undo, stepper continuation and lightbox endpoint
wrapping. Scribble preview and PNG Save As completed locally with focus
restored to Export PNG. The resulting 1200 × 400 RGBA PNG was independently
checked for chunk CRCs, header and decompressed size (13,038 bytes).

The ledger still retains source contrast and selected-tab focus decisions,
form geometry/keyboard differences, modal/toast call-order and visual limits,
and table row naming/viewport-role work. No VoiceOver, Windows/Linux runtime,
browser build or exhaustive malformed-codec testing was performed. One existing
test file (`tests/data_tables.rs`, 448 lines) exceeds the 400-line soft cap;
production and example Rust files remain below it. No dependency, version,
commit, push or publication change was made.
