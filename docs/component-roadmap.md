# Native Rust component roadmap

The Dutchy web system defines our visual language. The Rust library's scope is
broader: useful, attractive controls for native tools, dashboards and editors.
The web registry's core groups are implemented; individual variants and native
developer features remain opportunities. [Coverage](coverage.md) records actual
support, and this roadmap prioritizes additions rather than promising parity
with an arbitrary widget count. Each milestone gets an Issues Ledger ticket
before implementation starts.

## Shipped additions

- 1.0.22: **ColorPicker**, a port addition with host-owned color, flat named
  palettes, hex entry, RGB/alpha, copy and transparency preview; see
  [the editing contract](color-picker.md).
- 1.0.23: **MediaCard** and **SearchInput**, with a searchable artwork-browser
  showcase; see the [composition guide](media-card-search-input.md).
- 1.0.24: **IconButton**, **ToggleButton** and **Toolbar** with 22 built-in
  vector icons and host painters; see [actions and toolbars](actions-toolbars.md).
  The same release integrates [Markdown and CodeBlock](markdown.md).
- 1.0.26: checked menu actions, nested submenus, an in-window **MenuBar** and
  Toolbar menu buttons with a connected command-surface showcase; see
  [richer menus](richer-menus.md). The milestones below build on them.
- 1.0.28: shared commands and shortcuts, with **CommandRegistry**,
  **SplitButton**, **KeyRecorder**, nested command search, Todo and Canal
  Studio integration and automated CI; see [the contract](commands-shortcuts.md),
  [the plan](plan-1.0.28.md) and [the release record](verification-release-1.0.28.md).
  Workbench primitives then enable DutchyMail.

## Current application milestone

[DutchyAgent](dutchy-agent.md) implements OpenAI Responses and an explicit offline
mock, independent conversations, streaming, Stop/Retry and Markdown/CodeBlock
integration. See [the milestone plan](plan-dutchy-agent.md), original
[offline verification](verification-dutchy-agent.md) (DS-099) and
[live verification](verification-dutchy-agent-live.md) (DS-100).
The work is unreleased, planned for 1.0.30.

## Recommended next milestones

1. **Workbench building blocks.** **List** provides keyed rows, none/single/
   multiple selection with Shift and Control ranges, type-ahead, roving focus,
   list box semantics and fixed-height virtualization over egui's row-range
   scrolling; it replaces the separate VirtualList idea. **TreeView** reuses the
   same row engine with keyed expansion, Left/Right expand and collapse and tree
   semantics. **SplitPane** keeps pane sizes in the host, supports keyboard
   resizing and minimum sizes. **InlineEdit** renames a label in place with
   commit and cancel. **PathInput** pairs a text field with a browse button that
   returns a typed file, folder or save request. **StatusBar** is a bottom strip
   of keyed segments (text, icon, progress, clickable) with overflow priority and
   status semantics. The DutchyMail example demonstrates the set; see
   [example applications](example-applications.md).
2. **Powerful filtering.** A square **Chip** primitive with optional remove and
   toggle states is the shared base for **FilterChipGroup**, **TagInput** and
   **MultiSelect**; **RangeSlider** edits two endpoints on the existing rail;
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
   swatches, mode and reduced motion in one settings control; and **Sidebar**
   with its icon-rail mode, the port of the web navigation surface, so the
   showcase can use the library's own navigation.
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

Scenario applications under `examples/` complement the component gallery:
they prove composition under application-owned state and surface gaps for
this roadmap. DutchyTodo and DutchyAgent are buildable with shipped
components; DutchyMail demonstrates the command-plumbing and workbench
milestones; DutchyDash demonstrates the dashboards milestone. The shared
rules (dev-only dependencies, fictional data, environment-only secrets, an
offline mock provider and tests inside the examples), scopes and limits are
in [example applications](example-applications.md).

## Further candidates

| Addition | Useful outcome / implementation basis |
| --- | --- |
| CodeBlock highlighting and diff views | Extend the shipped host highlighter hook when applications need richer code presentation |
| JsonInspector / PropertyGrid | Inspect structured application state; compose TreeView, DescriptionList and form primitives |
| LogViewer | Filter, pause and follow logs with List virtualization and bounded host buffers |
| NotificationCenter | Browse dismissed toasts from a host-owned history in a Drawer; Toasts stay the live channel |
| TreeTable | Tree rows with DataTable columns for outlines and nested records |
| Avatar / AvatarGroup | Identity, fallback initials and presence; compose existing image loading |
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
  CommandRegistry, deferred from the 1.0.28 command work) are the most
  valuable desktop upgrades; pinning and multicolumn sorting follow concrete application needs.
- DatePicker: ranges and locale-aware display; TimePicker: optional 12-hour entry.
- Tabs: drag reordering and an overflow menu; Textarea: interactive resizing.
- Toasts: exit transitions, and the history service noted above.
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
