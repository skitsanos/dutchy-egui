# Data table verification — 2026-11-06

Package: **dutchy-egui 1.0.12**, developed locally on `ticket/data-tables`.
Version 1.0.11 was committed and pushed to main as
`543e7f5eec16505c1a822fbd90e32e3afb4de8e3`; the remote reference was verified.
This record was captured before the 1.0.12 release commit. Registry publication
was not performed. No dependencies were added/upgraded. The crates.io API
confirmed stable egui, eframe and egui_kittest at 0.37.0; egui remains the sole
direct runtime crate.

## Executed checks

- `cargo fmt --all --check`
- `cargo test --all-features`: 200 integration tests, eight unit tests, one doctest.
- `cargo test --no-default-features`: 144 integration tests, eight unit tests, one doctest.
- `cargo clippy --all-targets -- -D warnings`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy --all-targets --no-default-features -- -D warnings`
- `cargo build --example showcase`
- `cargo check --lib --target wasm32-unknown-unknown`
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`
- `cargo package --allow-dirty`, including extracted-library verification.
- `cargo check --manifest-path target/package/dutchy-egui-1.0.12/Cargo.toml --example showcase --locked`: packaged-source example with cached dependencies.
- `git diff --check`; all 185 Rust/Python source files remain below 400 lines
  (largest: 366). Local documentation links resolve. Packaged source/docs/font
  licenses and notices match the working tree byte for byte.

Host: macOS aarch64, Rust/Cargo 1.99.1. Both bundled fonts and host-font fallback
were checked. Logs: `target/visual-proof/{checks,package}-1.0.12.log`.
The centering follow-up is in `checks-table-centering.log`,
`table-centering-rendering.log` and `package-table-centering.log` in that directory.

## Interaction and geometry evidence

Six interaction tests cover semantic table counts/indices, native sort
Enter/Space, host-owned requests without implicit record/selection changes,
mixed select-all, disabled rows/columns/table, independent cell actions, stable
editor identities and values through row/column changes, explicit focus return
after removal, and no cell callbacks during empty/loading/missing-column states.
Two model unit tests cover fixed/flexible surplus allocation, nonfinite/negative
widths, and sort-key/direction transitions.

Five geometry tests cover fractional widths, oversized host content that cannot
shift later columns, full-height aligned cell rectangles, horizontal focus
reveal, vertically revealed body controls with a stationary header, header focus
without resetting body scroll, spanning state cells, and empty/loading guidance
remaining inside a 280-point viewport after a far column was focused.

The public-API showcase test applies host numeric comparisons and selection,
executes independent View actions, filters to no matches while retaining
selection, clears search, toggles busy loading, and filters back to Ada.

Three additional alignment tests cover mixed-height cell/control centers and
wrapped headings at both densities and 1x/1.5x/2x. Width/content/density/column
changes grow and shrink rows without settled geometry drift or lost native
focus. Pointer selection/actions and keyboard sorting remain independent.

## Rendered evidence

Reviewed WGPU baselines:
`tests/snapshots/table-{light-1x,dark-1_5x,purple-2x}.png` and
`table-row-hover.png`. Matrices cover custom text/badge/action cells, right-aligned
monospaced amounts, selection/mixed controls, disabled rows, keyboard focus,
muted/inverse headers, stripes, compact density, empty guidance and loading.
Baseline checks pass without snapshot-update mode.

Desktop and 480-point showcase previews are
`target/visual-proof/table-{desktop,narrow}.png`; the 280-point standalone matrix
is `table-matrix-narrow.png`. Five rendering/example tests include narrow chrome
and guidance bounds and accessibility checks. Review corrected guidance that
was centered in off-screen table width. State bodies now follow the visible
horizontal viewport; data headings and cells still scroll together. Explicit
state-cell/loading-region bounds retain meaningful geometry for assistive APIs.
The [DS-020 correction](issues/DS-020.md) updates these baselines so header and
body content share each row's vertical center. Keyed natural-height measurements
place controls before rendering; centering offsets never increase measured height.

## Native evidence

The macOS Glow showcase displayed 1.0.12 and the Data Tables page. Selecting Ada
and activating View Ada produced one selected record and a separate host-inspected
detail. Select-all increased selection to five, leaving disabled record four
unchecked. Repeated Budget activation placed $12,240.25 before $11,509.25,
confirming host numeric sorting rather than formatted-string ordering.

Typing `zzzz` displayed No Matching Records, zero matching records and No Pages,
while five host selections remained. Replacing it with `Ada` displayed one
selected matching record. Loading replaced that record with a named loading
region and disabled all sort/select-all controls.

Dark mode, stripes, compact cells and the inverse header were inspected with
aligned custom cells and independent View buttons. Native Tab reached Sort by
Name; Space sorted ascending, Enter toggled descending and preserved header
focus. Page 2 changed the host view and current-page metadata while selection and
the inspected detail survived. Scrolling the body retained its header; the
compact inverse sample was also inspected. Native Glow and headless WGPU are
separate evidence.

After DS-020, the rebuilt native showcase was inspected again in light
comfortable and dark compact/inverse modes. Text, badges, amounts, checkboxes
and View controls align vertically with their row centers; header labels align
with select-all. Native row selection and View Ada still updated their separate
host outcomes. Changing density recomputed centered placement and reduced height.

## Limits

This table is not virtualized: it visits every supplied row/cell each pass.
Hosts must bound or paginate large datasets; no large-dataset benchmark is
claimed. Resizing/pinning/reordering, multicolumn sort, arbitrary spans, dragging,
copy/export and spreadsheet arrow-key editing remain outside this step. Loading
replaces stale data rather than overlaying enabled stale controls. Cell content
is vertically centered and clipped to its configured column width.

Native GUI execution is macOS-only without an active screen reader. Windows/
Linux GUI, browser WASM execution and a Rust 1.95 minimum-toolchain run remain
outstanding. Metadata tests/native tree inspection do not establish screen-reader
acceptance, whole-system/browser parity or blanket contrast compliance; inherited
palette limits remain in [DS-003](issues/DS-003.md).
