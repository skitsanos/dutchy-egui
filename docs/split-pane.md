# SplitPane

`SplitPane` composes two clipped, resizable regions. The application owns the
first pane's size in logical points; the second fills the remaining space.
No helper crate, persistence service or hidden pane-size state is involved.

```rust
use dutchy_egui::{egui, SplitPane};

fn workspace(ui: &mut egui::Ui, folders_width: &mut f32) {
    let result = SplitPane::horizontal("workspace", folders_width)
        .accessible_name("Folders")
        .min_sizes(160.0, 240.0)
        .height(400.0)
        .show(ui, |ui| ui.label("Folders"), |ui| ui.label("Document"));
    if result.response.changed() {
        // The host can persist the updated folders_width.
    }
}
```

`horizontal` lays out left/right panes; `vertical` lays out top/bottom panes.
Both fill the available width. Height defaults to 320 points and is allocated
as given, also below the fold of a ScrollArea, whose content only sees the
viewport as available height. Use `.height(ui.available_height())` to fill a
bounded host region. Unbounded or invalid dimensions get finite defaults, and
dimensions are capped at 32768 points. Ordering is physical, including in
right-to-left UIs.

## Sizing and composition

The divider reserves 12 points. Each minimum defaults to 48 points; pass
`.min_sizes(first, second)` to change them. If their sum cannot fit, both
minima shrink in proportion. The first size is clamped to the resulting range
on every draw, including disabled/read-only draws. NaN/infinite sizes start at
half the available pane space. Invalid or negative minima become zero.

`response.changed()` and `divider.changed()` report a changed first size,
including normalization. Resizing the host can clamp that stored size; growing
it again preserves the clamped size, rather than restoring a hidden preference.
The host can keep a separate preferred size if it wants that policy.

Each content closure runs once per egui pass in its own stable ID scope. Child
results and allocated rectangles are available through `first` and `second`;
`response` is the fixed outer region and `divider` the resizing control.
The region below the component stays in place while the divider moves.

Children must lay out within their available size. Painting and pointer input
are clipped to each slot; oversized content cannot cover the neighboring pane.
Use ScrollArea for overflow and explicit IDs for dynamic child controls. Nest
another SplitPane to make a three-pane workspace. For an editor and preview of
one string, capture a preview copy before passing a mutable editor reference.

## Interaction and accessibility

- Drag the divider to resize. The grab offset is preserved; release applies the
  final pointer position. Locking or changing orientation cancels an active
  drag.
- Matching-axis arrows resize by 8 points; Shift+arrow uses 32 points. Home and
  End reach the allowed minimum and maximum. Cross-axis arrows keep focus on
  the divider. Control/Command/Alt shortcuts remain with the host.
- Tab visits first-pane controls, the divider, then second-pane controls.
  Child editing/focus stays native, with stable identities while panes resize.
- `read_only(true)` locks resizing and keeps child controls usable.
  `enabled(false)` disables the complete composition, as does a disabled parent.

The divider uses AccessKit's Splitter role, with its actual orientation, numeric
value/range/steps in points, and a controls relation to the first pane. Name it
after that pane with `accessible_name`. Assistive increment, decrement and set
value requests follow the same bounds as pointer/keyboard edits. Focus and hover
use shared Dutchy tokens.

The [W3C splitter pattern](https://www.w3.org/WAI/ARIA/apg/patterns/windowsplitter/)
informs the directional keyboard behavior. This initial scope has no Enter
collapse/restore, F6 pane cycling, docking, animated resizing, ratio storage or
automatic child scrollbars. Zero-area content is invisible and non-interactive.

The **Split panes** showcase composes TreeView, nested splitters, editable
Markdown sources and live previews. Notebook drafts and both sizes stay in the
application; without the Markdown feature the preview uses plain text.
