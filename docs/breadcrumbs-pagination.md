# Breadcrumbs and pagination

Reference: `docs/01-design-system/components/breadcrumbs.md` and `pagination.md`,
plus their live components at web revision
`e91ca1d506e1c7a11b58a5d6857d82208229812a`.
These are repository-owned egui implementations without new dependencies.

## Breadcrumbs

```rust
use dutchy_egui::{BreadcrumbItem, Breadcrumbs};

fn trail(ui: &mut egui::Ui, ancestors_open: &mut bool) -> Option<u8> {
    let path = [
        BreadcrumbItem::new(0, "Home"),
        BreadcrumbItem::new(1, "Library"),
        BreadcrumbItem::new(2, "Components"),
        BreadcrumbItem::new(3, "Navigation"),
    ];
    Breadcrumbs::new("workspace-path", &path)
        .collapse(2, ancestors_open)
        .show(ui)
        .navigate
}
```

The host supplies ordered keyed destinations and handles `navigate`; no path is
mutated and no URL opens automatically. Keys derive from immutable values using
Hash/Debug. Labels may change without changing identity. The final item is
current, noninteractive, and marked `AriaCurrent::Page`. Ancestor actions use
native button semantics inside a named Navigation/List region. Tab/Shift+Tab and
Enter/Space retain native behavior; separators are decorative paint.

Use `BreadcrumbSeparator::{Chevron,Slash}` and `.bar(true)` for the muted bar
with a bottom divider. Items can be disabled. An optional `BreadcrumbIconPainter`
replaces visible text with a decorative icon while retaining the label as its
accessible name. Text wraps within available columns; separator and destination
are allocated together so a separator does not end the preceding row.

`.collapse(visible, &mut open)` retains the root and trailing destinations, keeping
at least two actual items. Middle ancestors appear in the existing dropdown/menu
pattern, preserving source order, names, disabled state, and typed requests.
The host owns its open flag. Up/Down opens/navigates, Enter/Space activates,
Escape dismisses, and normal dropdown focus return/Tab exit applies.
Without collapse configuration, the full path wraps. The host may choose a
visibility limit from available width. Empty/single paths are safe; no synthetic
Home item is inserted.

When the host changes the path or removes a focused destination, it chooses a
surviving focus target. The showcase resets focus to Restore Deep Path after
applying an ancestor request. The response exposes visible `items`, the optional
`collapsed` trigger, and the named navigation `response` for host coordination.

## Pagination

```rust
use dutchy_egui::Pagination;

fn pages(ui: &mut egui::Ui, page: &mut usize, total_pages: usize) {
    let result = Pagination::new("records", *page, total_pages)
        .accessible_name("Record pages")
        .show(ui);
    if let Some(request) = result.requested_page {
        *page = request;
        // Load or select records in the host.
    }
}
```

Pages are one-based. The widget receives current/count values and returns a
`requested_page`; loading, filtering, counts, and data replacement belong to the
host. `.enabled(false)` supports host loading states. The collapsed-ancestor
button rests on the border color and turns primary on hover/focus; the slash is
a one-point diagonal and chevrons are 8-point strokes rather than the web's
typographic glyphs. Distinct IDs and accessible
names identify multiple pagers. `PaginationVariant::Compact` replaces number
buttons with Previous, a centered page summary, and Next.

The standard variant uses the live component's 40-point square controls and
primary active fill (the older documentation's foreground-filled illustration
is not used). Page numbers rest on the foreground as in the live component (the
Markdown spec says muted) and direction controls rest muted; the compact summary
bolds its numbers on a muted base. Up to seven pages show in full. Larger ranges retain endpoints and
the current page's neighbors, with decorative ellipses. Single-page gaps show the
actual number. Work remains bounded for `usize::MAX`; no full-range allocation
or overflowing next/previous arithmetic is used. Long numbers expand their
minimum target width, and controls wrap within narrow columns.

Previous/Next disable at endpoints. Numbered controls retain native Tab and
Enter/Space behavior. The current button remains focusable and marked
`AriaCurrent::Page`; activating it produces no navigation request. This native
adaptation keeps focus stable when the host applies a requested page and that
button becomes current. Buttons are keyed by page number; direction controls
have stable identities across range changes and sibling insertions.

An out-of-range current value is clamped for display only. `current_page` exposes
the effective view. A zero count produces None, “No pages,” and disabled direction
controls. The host input is not silently rewritten. If external count changes
remove the focused control, the host chooses the new focus destination.

## Boundaries

No browser link/URL construction, routing, data fetch, page-size selector, jump
to page input, virtualized dataset, or automatic post-load content focus is
provided. Native Tab order is used rather than adding arrow-key roving behavior
to pagination. The host owns responsive collapse policy and focus after external
removals. The coverage matrix tracks later interaction groups, and source palette
contrast limitations remain in DS-003.
