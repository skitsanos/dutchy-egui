# Data tables

`DataTable` presents a host-supplied view of records. Stable row and column keys
preserve native cell controls across sorting, filtering, insertion and reordering.
The host owns record order, typed comparisons, search, pagination, selection,
loading and business actions. No helper crate is required.

```rust
use dutchy_egui::{DataTable, TableAlign, TableColumn, TableSort, TableSortDirection, Text};
use std::collections::HashSet;

struct Record { id: u64, name: String, cents: i64 }

fn records_ui(
    ui: &mut egui::Ui,
    rows: &mut [Record],
    selected: &mut HashSet<u64>,
    sort: &mut Option<TableSort<&'static str>>,
) {
    // Compare typed values in the host, before presenting its current view.
    if let Some(order) = sort.as_ref() {
        rows.sort_by(|a, b| {
            let comparison = match order.column {
                "amount" => a.cents.cmp(&b.cents),
                _ => a.name.cmp(&b.name),
            };
            if order.direction == TableSortDirection::Descending {
                comparison.reverse()
            } else { comparison }
        });
    }
    let columns = [
        TableColumn::new("name", "Name").sortable(true),
        TableColumn::new("amount", "Amount").sortable(true).align(TableAlign::Right),
    ];
    let result = DataTable::new("records", &columns, rows)
        .accessible_name("Project records")
        .sort(sort.as_ref())
        .selection(|row| selected.contains(&row.id))
        .show(ui, |row| row.id, |ui, row, column| {
            match *column {
                "amount" => { ui.add(Text::new(format!("{} cents", row.cents))); }
                _ => { ui.add(Text::new(row.name.as_str())); }
            }
        });
    if let Some(request) = result.sort_requested {
        *sort = Some(request);
        ui.ctx().request_repaint();
    }
    for request in result.selection_requested {
        if request.selected { selected.insert(request.key); }
        else { selected.remove(&request.key); }
    }
}
```

Filter and paginate **before** passing the slice. The showcase composes `Input`,
`Pagination`, custom badges, monospaced quantities and independent View buttons.
It sorts numeric cents directly, retains selection outside the current view and
keeps all durable data in `TableState` in the application.

## Columns and cells

`TableColumn::new(key, label)` defaults to left alignment, a flexible 160-point
minimum and no sorting. `TableColumnWidth::Fixed(width)` keeps that width;
`Flexible(minimum)` shares surplus viewport space equally with other flexible
columns. Widths clamp to 96–4096 points; nonfinite inputs use 160. An all-fixed
table uses its natural total width. Minima never shrink to fit a narrow viewport;
the table scrolls horizontally instead.

Keys must be unique within their row/column collection and remain immutable.
A repeated row key receives an occurrence salt so the rows keep separate
identities and measurements instead of looping; only unique keys retain
identity across reordering.
Do not use display positions as record keys or mutate `TableColumn.key` after
construction. Labels may change while keys stay fixed. A cell closure receives
the record and column key, runs once per cell per egui pass, and composes ordinary
egui/Dutchy controls. Observe normal egui side-effect rules; execute business work
only on activation, never merely because the closure runs.

Column rectangles remain aligned when text wraps or custom content exceeds its
width. Content wider than its column is clipped; make host content responsive or
choose a larger minimum. Row height follows its tallest cell; all header/body
content blocks share the row's vertical center, including native controls and
wrapped text. Keyed temporary height measurements request an egui layout pass
when content changes; centering offsets do not contribute to natural row height.
`TableAlign::{Left, Center, Right}` applies horizontally to headings and cells.
`DataTableResponse` includes the viewport, semantic header/row/cell responses,
optional selection controls, and host requests.
Measurements of rows that leave the view are evicted at the end of the pass.

## Sort, selection and focus

`.sortable(true)` exposes a named native header button. A new column requests
ascending order; activating it again toggles descending/ascending. `.sort(...)`
reports the host-applied sort, including the arrow and accessibility metadata.
There is one active sort column, no internal comparison or unsorted third state.
Column `.enabled(false)` disables only that sort button.

`.selection(predicate)` adds a 64-point checkbox column. Row toggles return one
`TableSelection { key, selected }`. Select-all returns changes for enabled records
in **the supplied current view**; it never addresses unseen pages. Its native
checkbox exposes mixed state when some enabled rows are selected.
`.row_enabled(predicate)` disables selection and nested cell controls for specific
records; `.enabled(false)` disables the whole component without enabling a
disabled parent. Independent cell buttons do not select a row.

Native Tab/Shift+Tab traverse controls; Enter/Space activate sort buttons and
checkboxes. This is a semantic table containing native controls, not an editable
ARIA grid with arrow-key navigation. Focused controls reveal themselves in both
scroll axes. Stable cell scopes preserve editor identity when records/columns
move. `.return_focus_to(actual_widget_id)` supplies a surviving enabled host
target when a focused row/column is removed, filtered out, or replaced by a state
body. Without that target the host manages removal focus. No automatic business
selection or arbitrary row focus is introduced.

## Surfaces and state bodies

`TableHeader::{Muted, Inverse, Plain}` explicitly supports the documented muted
header, striped reference's inverse header and live component's plain header.
Header labels use the bold body face at 12 points, uppercase, with wider
(0.05 em) tracking; the muted header keeps foreground text while the plain
header uses muted-foreground like the live `th`. Sort glyphs are the live
12-point double arrow at 30 % opacity, primary at full strength when the column
is sorted and rotated for descending order.
Comfortable density uses 24x16 body padding and 24x12 header padding; Compact uses
16x8 for both. `.striped(true)` alternates rows with the live 30 % muted blend
(the Markdown spec says 50 %). Hover uses a flat muted
fill; selection adds a subtle accent fill and leading edge. Outer borders are
two points, header dividers two, row separators one. `.bordered(false)` removes
only the outer border. Geometry and palette values come from shared tokens.

The header stays above the vertically scrolling body; horizontal scrolling moves
headings and cells together. `.max_height(...)` caps the body at 384 points by
default, clamped to 96–4096. Nonfinite height requests use the default.

Empty tables display `EmptyState` guidance, configurable through `.empty_text`;
its inbox glyph is the port's own, drawn by neither reference. Rows and
selection checkboxes are named by display index.
`.loading(true)` replaces records with a named busy skeleton group and disables
header controls; stale cell closures do not run. This deliberately differs from
the source's translucent overlay on stale data. Empty/loading guidance is sized
to the visible horizontal viewport, including after scrolling to a far column.
Skeleton motion follows the shared reduced-motion and visibility policy.
Zero columns display configuration guidance without fabricating table semantics.

AccessKit provides Table/Row/ColumnHeader/Cell roles, current rendered counts,
indices, a spanning state cell, sort direction, row selection, mixed checkboxes,
named viewport and loading busy state. Header count is included in `row_count`;
empty/loading each use one placeholder row. Child native controls keep their
names and roles. Metadata tests do not establish screen-reader acceptance.

## Reference and limits

Reviewed `docs/01-design-system/components/data-table.md`,
`src/components/DataTable/index.tsx` and `public/assets/js/sortable-table.js` at
web revision `104e4dab59fd94130b44fb276736c6be731e1ebf`. Differences between live
and documented header treatments are explicit. Native keyed selection, sticky
body scrolling and focus handling extend that reference; sort comparisons stay
typed and application-owned rather than parsing formatted display strings.
The implementation is original repository-owned code using egui only.

The table is **not virtualized**: all supplied rows and cells are visited
each pass. Bound or paginate the slice for large datasets. Column resizing,
reordering, pinning, grouped/multirow headers, arbitrary spans, multicolumn sort,
row dragging, copy/export and a grid editing model are out of scope.
