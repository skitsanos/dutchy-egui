# Native Rust component roadmap

The Dutchy web system defines our visual language. The Rust library's scope is
broader: useful, attractive controls for native tools, dashboards and editors.
The web registry's core groups are implemented; individual variants and native
developer features remain opportunities. [Coverage](coverage.md) records actual
support, and this roadmap prioritizes additions rather than promising parity
with an arbitrary widget count.

## Current addition

**ColorPicker**: host-owned color, flat named palettes, hex entry, RGB/alpha,
copy, transparency preview, controlled disabled/read-only states and a showcase.
It is a port addition; see [the editing contract](color-picker.md).

Version 1.0.23 adds **MediaCard** and **SearchInput**, with a
searchable artwork-browser showcase. See the
[composition guide](media-card-search-input.md). Version 1.0.24 adds
**IconButton**, **ToggleButton** and **Toolbar**, with 22
built-in vector icons, host painters and three showcase pages. See
[actions and toolbars](actions-toolbars.md). Version 1.0.24 also integrates
[Markdown and CodeBlock](markdown.md) from the parallel implementation branch.

## Recommended next milestones

1. **Richer menus:** checked actions, nested submenus and an application menu
   bar. Compose these with the new named icon actions and toolbars. Preserve
   host-owned command execution, stable identity and native dismissal/focus.
2. **Powerful filtering:** MultiSelect with removable chips, TagInput and
   RangeSlider. Current Select/SearchableSelect choose one value and Slider
   edits one endpoint. Ship a useful filter-builder example with keyboard
   removal, empty states, explicit commit behavior and typed host values.
   Compose these with the existing SearchInput and MediaCard browser example.
3. **Workbench building blocks:** TreeView, VirtualList and resizable SplitPane.
   Provide keyed selection/expansion, large collections, keyboard navigation
   and host-owned pane sizes. A small file/record explorer demonstrates the
   combination. These also support future property inspectors and log viewers.

## Further developer candy

| Addition | Useful outcome / implementation basis |
| --- | --- |
| CodeBlock highlighting and diff views | Extend the shipped host highlighter hook when applications need richer code presentation |
| JsonInspector / PropertyGrid | Inspect structured application state; compose tree and form primitives |
| LogViewer | Filter, pause and follow logs with virtualization and bounded host buffers |
| Sparklines / small bar and line charts | Compact dashboard context using Painter and shared tokens |
| Avatar / AvatarGroup | Identity, fallback initials and presence; compose existing image loading |
| Reorderable lists / richer drag and drop | Arrange application-owned records with keyboard alternatives |
| HSV/HSL color entry and palette history | Extend the picker; keep persistence with the host |

## Existing components worth extending

- DataTable: virtualization and column resizing are the most valuable desktop
  upgrades; pinning and multicolumn sorting follow concrete application needs.
- DatePicker: ranges and locale-aware display; TimePicker: optional 12-hour entry.
- Tabs: drag reordering and an overflow menu; Textarea: interactive resizing.
- General popovers and richer image zoom/pan can reuse existing overlay/media
  foundations. They are distinct from navigation/routing and file transport,
  which remain application concerns.
- Select: optional leading icons and shared trailing-slot spacing for sort and
  filter triggers, following the Foliopress reference.

Each milestone includes a compelling showcase, accessible names and keyboard
behavior, disabled/read-only handling, state stability, clipping, multiple
themes/scales and useful documentation. Keep runtime dependencies small and
implement focused primitives in repository-owned modules. Release hardening
also includes automated CI and package/API documentation checks.
