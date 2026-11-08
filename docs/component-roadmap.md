# Native Rust component roadmap

The Dutchy web system defines our visual language. The Rust library's scope is
broader: useful, attractive controls for native tools, dashboards and editors.
The web registry's core groups are implemented; individual variants and native
developer features remain opportunities. [Coverage](coverage.md) records actual
support, and this roadmap prioritizes additions rather than promising parity
with an arbitrary widget count.

## Current addition

**Markdown and CodeBlock**: CommonMark with GitHub extensions rendered from the
tokens, host-reported links with native opening and fragment scrolling, and a
standalone code block with copy, line numbers and host highlighting. Both are
port additions; see [the Markdown guide](markdown.md). The previous addition,
**ColorPicker**, is documented in [the editing contract](color-picker.md).

## Recommended next milestones

1. **Actions and toolbars:** public IconButton, ToggleButton and Toolbar APIs.
   The library currently has text buttons and private close/icon painting.
   Add named icon-only actions, pressed state, consistent hit targets, tooltip
   composition and roving toolbar focus. Include a small polished icon gallery
   and host-provided icon painters. Follow with a MediaCard composition with
   independent preview/save actions, informed by the
   [Foliopress showcase](references/foliopress-showcase.md).
   Extend menus with checked actions/submenus
   and an application menu bar as a follow-up.
2. **Powerful filtering:** MultiSelect with removable chips, TagInput and
   RangeSlider. Current Select/SearchableSelect choose one value and Slider
   edits one endpoint. Ship a useful filter-builder example with keyboard
   removal, empty states, explicit commit behavior and typed host values.
   Add SearchInput with a magnifier, clear action and optional key hint; combine
   it with media cards and explicit result counts in a gallery-browser example.
3. **Workbench building blocks:** TreeView, VirtualList and resizable SplitPane.
   Provide keyed selection/expansion, large collections, keyboard navigation
   and host-owned pane sizes. A small file/record explorer demonstrates the
   combination. These also support future property inspectors and log viewers.

## Further developer candy

| Addition | Useful outcome / implementation basis |
| --- | --- |
| Syntax highlighting for CodeBlock | Bundled or optional highlighter over the shipped `CodeHighlighter` span hook; diff and line-focus views |
| Markdown extensions | Footnotes, inline images in text and clickable linked figures, once hosts ask for them |
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
