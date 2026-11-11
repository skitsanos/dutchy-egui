# Native Rust component roadmap

The Dutchy web system defines the visual language; the Rust library aims
wider, at useful controls for native tools, dashboards and editors.
[Coverage](coverage.md) lists what ships today. This roadmap sets priorities
rather than promising parity with a widget count, and each milestone gets an
Issues Ledger ticket before work starts.

## Recommended next milestones

1. **Workbench building blocks.** [PathInput](forms.md#path-input) pairs a path
   field with a typed browse request and demonstrates the three picker modes
   in Showcase. [InlineEdit](inline-edit.md), [StatusBar](status-bar.md) and
   [Avatar](avatar.md) also ship for this milestone. See
   [example applications](example-applications.md) for workbench compositions.
2. **Powerful filtering.** [Chip](chips.md) provides independent toggle and
   remove actions, and [FilterChipGroup](chips.md#filter-chip-groups) groups
   keyed filters with host-owned selection. [TagInput](chips.md#tag-input)
   adds a draft and keyed add and remove requests for host-owned tags.
   **MultiSelect** will build on these controls. **RangeSlider** edits
   two endpoints on the existing rail;
   **Combobox** accepts free text with suggestions where SearchableSelect only
   commits an option. A small repository-owned fuzzy scorer with highlighted
   matched glyphs upgrades CommandPalette, SearchableSelect and List type-ahead.
   Compose a filter-builder example with SearchInput and the MediaCard browser,
   with keyboard removal, empty states, explicit commit behavior and typed
   host values.
3. **Small wins.** **Dialog presets** (confirm, alert and prompt with typed
   results, Enter and Escape mapping and optional type-to-confirm for
   destructive actions) built on Modal; **DescriptionList** for label and value
   rows with alignment, wrapping and copy; **Activity**, the port of the web
   activity panel (accent-edged surface, colored square markers, title and
   time) with a vertical connector variant; **ThemePicker** combining accent
   swatches, mode and reduced motion in one settings control; and an icon-rail
   **Sidebar** mode if an application needs one. List and TreeView already
   compose a navigation sidebar, as the Showcase pages and DutchyMail folders
   show.
4. **Dashboards.** A repository-owned charts family: sparkline, bar, donut,
   gauge and calendar heatmap with flat fills, square markers, token colors,
   hover tooltips, host-formatted labels and an accessible text summary. No
   plotting crate. **BackgroundTasks** lists running operations with progress,
   cancel and retry and feeds a StatusBar segment. **Calendar** exposes a month
   grid with event markers and range selection over the date math already
   behind DatePicker.

**Prototype before scheduling:** a **TitleBar** for frameless eframe windows
with a drag region, square window controls and a brand slot. It depends on
viewport commands for dragging, minimizing, maximizing and closing, behaves
differently on each desktop platform (macOS keeps native window buttons), and
is a no-op on the web. Verify it per platform before committing to a release.

## Example applications

DutchyMail will demonstrate the workbench milestone and DutchyDash the
dashboards milestone; see [example applications](example-applications.md).

## Further candidates

| Addition | Useful outcome / implementation basis |
| --- | --- |
| CodeBlock highlighting and diff views | Extend the shipped host highlighter hook when applications need richer code presentation |
| JsonInspector / PropertyGrid | Inspect structured application state; compose TreeView, DescriptionList and form primitives |
| LogViewer | Filter, pause and follow logs with List virtualization and bounded host buffers |
| NotificationCenter | Browse dismissed toasts from a host-owned history in a Drawer; Toasts stay the live channel |
| TreeTable | Tree rows with DataTable columns for outlines and nested records |
| AvatarGroup | Stacked avatars with an overflow count and one accessible summary; compose Avatar |
| Reorderable lists / richer drag and drop | Arrange application-owned records with keyboard alternatives |
| PinInput / MaskedInput | Fixed-length code boxes and formatted entry for sign-in and identifier forms |
| Popover | Anchored rich content beyond menus, reusing the overlay foundations |
| HSV/HSL color entry and palette history | Extend the picker; keep persistence with the host |

## Not planned from the web inventory

Hero, Footer, PricingCard, FeatureGrid, NewsletterForm, PageNotFound and the
website header and footer are marketing-page patterns. Their native
equivalents are MenuBar, Toolbar, TitleBar and Sidebar, so they are not ported.

## Existing components worth extending

- DataTable: virtualization on the List row engine, column resizing, inline
  cell editing and a column chooser menu (checked commands from a host
  CommandRegistry) are the most valuable desktop upgrades; pinning and multicolumn sorting follow concrete application needs.
- DatePicker: ranges and locale-aware display; TimePicker: optional 12-hour entry.
- Tabs: drag reordering and an overflow menu; Textarea: interactive resizing.
- Toasts: exit transitions, and the history service noted above.
- General popovers and richer image zoom/pan can reuse existing overlay/media
  foundations. They are distinct from navigation/routing and file transport,
  which remain application concerns.
- Select: optional leading icons and shared trailing-slot spacing for sort and
  filter triggers.

Each milestone includes a compelling showcase, accessible names and keyboard
behavior, disabled/read-only handling, state stability, clipping, multiple
themes/scales and useful documentation. Keep runtime dependencies small and
implement focused primitives in repository-owned modules.
