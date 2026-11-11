# Component coverage

What each component supports today. "Web" rows follow a group of the reference
web registry; a group is not a count of individual controls. "Port addition"
marks a control with no web counterpart, and "partial" marks a row with known
gaps, which the row names. The [roadmap](component-roadmap.md) lists what may
come next, and each component guide states its limits.

| Component | Origin | Supported scope and gaps |
| --- | --- | --- |
| Buttons | Web | Five variants; three sizes; disabled/loading/full width; padded left/center/right text alignment; focus and keyboard. IconButton adds named square actions, original/custom vector icons and shared variants/sizes. |
| Forms | Web | Input/password, checkbox, radio group, typed select, read-only/disabled; field labels/help/errors/required/pending/success; validation triggers and async tickets |
| Cards | Web | Six surfaces, four padding tiers, headers/footers, scoped text colors, interactive and disabled cards; host-owned actions |
| Badges & Tags | Web | Eight variants, uppercase labels, keyboard-accessible close affordance |
| Alerts | Web | Four severities, tinted/solid banners, accessible announcements, action/dismiss results, explicit focus-return target |
| Empty State | Web | Generic/search/table-body/dashboard patterns, heading/context, decorative icons, independent actions, disabled controls, optional polite announcements |
| Skeleton | Web | Five shapes, count/dimension overrides, named busy groups, card/list/content/table patterns, pause/reduced motion and clipped scheduling |
| Spinner | Web | Three-sided square; 12/16/20 sizes; color, pause, reduced motion, clipped-widget scheduling |
| Progress | Web | Determinate/custom maximum and indeterminate; 4/8/16-point tracks; four fills; labels/percentages; sanitized range metadata and reduced motion |
| Stats | Web | Host-formatted KPIs, four surfaces, detailed context/accent, explicit direction/sentiment, responsive grids/compact rows and stable identity |
| Pagination | Web | Numbered/ellipsis and compact controls; current-page metadata, stable focus, disabled endpoints, empty/invalid ranges, bounded large counts, wrapping, and host-owned page/data requests |
| Accordion | Web | Controlled single/multiple expansion, accent-edge/bordered surfaces, keyboard headers, named regions, disabled items, wrapping and stable body scopes |
| Tabs | Web | Underline/segmented, full width or natural width, horizontal overflow, manual keyboard activation, roving focus, disabled items, independent close requests and focus after removal. No docking, reorder dragging, overflow menu, or arbitrary icons yet. |
| Toasts | Web | Host-owned FIFO queue, typed single-use actions, manual/expiry events, four corners/severities, solid/outline, 1–4 visible slots, persistent/pause policies, live metadata, stable focus, constrained scrolling, and entrance fade. No exit/slide transitions or notification-history service. |
| Modal / Drawer | Web | Centered dialogs and full-height left/right drawers; three widths each, native blocking/focus confinement, configurable dismissal, replacement focus targets, divided footers, constrained scrolling, and reduced-motion-aware drawer entrances. No docked/top/bottom drawers, resizing, or exit animation. |
| Dropdowns | Web | Anchored/context actions, start/end alignment, viewport constraints, scrolling, keyboard/dismissal/focus return; searchable select and native modal command palette with host-owned query/open/selection/actions; the palette searches nested command leaves with breadcrumbs |
| Data Table | Web | Keyed custom cells, fixed/flexible widths, alignment, sticky headers, two-axis scrolling, three header surfaces, compact/striped/hover/selected rows, native sort/selection requests, disabled rows, busy/empty guidance and host filter/pagination composition. Nonvirtualized; no resizing/pinning, dragging, arbitrary spans or multicolumn sort yet. |
| Stepper | Web | Horizontal/vertical/compact/position progress; keyed host state, explicit completion, optional navigation requests, current/status metadata, narrow adaptation, compact scrolling, empty/single/unknown states and validated host workflow |
| Tooltips | Web | Text hints; four preferred sides, native constrained placement/hover timing, keyboard focus, Escape dismissal, and merged accessible descriptions |
| File Upload | Web | Dashed/compact browse requests, lazy spatially claimed drop handles, shared type/size/count metadata validation, keyed file lists and progress/errors/remove/cancel/retry with focus return. Native picker is a demo-host integration. Transport, byte reads, folder traversal, browser picker and content verification stay host-owned. |
| Image | Web | Native source/texture presentation, sharp frames, explicit aspect/contain/cover, caption/source, busy/error/retry, keyed responsive galleries and constrained modal lightboxes with keyboard navigation, caption scrolling and focus return. Optional `url-images` installs HTTP(S) PNG/JPEG; other URI/byte loaders stay host-owned. No zoom/pan, slideshow, masonry or drag reorder. |
| Breadcrumbs | Web | Keyed destinations, current-page metadata, slash/chevron, icon root, muted bar, wrapping and collapsed-ancestor menus; host-owned routing and collapse policy |
| Flex | Web | Token gaps, row/column/reverse, row wrapping, native cross alignment; centered containers and keyed responsive grids. CSS baseline/distribution modes are outside this scope. |
| Menu | Web | Keyed actions, groups, separators, hints/icons and disabled/destructive states. Controlled checks, nested submenus, viewport placement and one-level keyboard dismissal. Radio/mixed checks remain outside scope. |
| Text | Web | Three roles; four actual weights; 12 sizes; tracking, leading, color, Unicode uppercase, italics, wrapping |
| Scribble | Web | Pointer/dot and keyboard drawing, host-owned bounded strokes/history, undoable clear, fixed paper, captured brushes, read-only/disabled, immutable snapshots, SVG/CPU raster/optional PNG, opt-in pointer smoothing and native demo Save As. No pressure/tilt, multitouch, eraser, tapered ends, history persistence or shape editor. |
| Textarea | Web (partial) | Multiline, row sizing, Unicode count/limit, read-only/disabled and field feedback. Interactive resize remains planned. |
| Switch | Port addition | 36 × 20 track with a sliding square thumb, Switch role, Space/Enter activation, disabled state, reduced motion |
| Kbd | Port addition | Mono uppercase key chip on the muted surface; a label, composed per key |
| Segmented | Port addition | Typed exclusive segments at a shared height, foreground selection, RadioGroup/RadioButton semantics, one tab stop, Left/Right/Home/End roving that selects, disabled segments, full width |
| Slider | Port addition | egui slider interaction and Slider semantics under a 4-point rail, primary trailing fill, 20-point square handle, step snapping, host-formatted value label, disabled state. Single value; no range or vertical slider yet. |
| NumberInput | Port addition | Editor with square decrease/increase buttons, Up/Down stepping, bounds, decimals, read-only, SpinButton value/min/max/step; unparsable text reverts on blur |
| DatePicker | Port addition | ISO `YYYY-MM-DD` editor, caret-segment stepping, bounds, host-supplied today, calendar popup with month navigation, arrows/Home/End/PageUp/PageDown, Enter/Escape and focus return. No week numbers, ranges or locale formats yet. |
| TimePicker | Port addition | `HH:MM` or `HH:MM:SS` editor with wrapping caret-segment steps and strict parsing; no time zones or 12-hour display. |
| ColorPicker | Port addition | Named theme/host palettes, RGB/optional alpha sliders and numbers, hex editing/copy, transparency previews, constrained popup, disabled/read-only and keyboard/accessibility. No HSV/HSL, eyedropper, history service or HDR yet. |
| MediaCard | Port addition | Image/URI/byte sources, category/title/description, independent preview/metadata/save/retry, host-owned saved state, responsive width and native loading/error pipeline. No custom metadata slots or masonry. |
| [InlineEdit](inline-edit.md) | Port addition | Host-owned drafts, stable keys, click/Enter/Space/F2 start, native text editing, explicit commit requests and cancel, focus return, retained blur drafts, Unicode limits, disabled/read-only states and compact actions. Validation, persistence and rename conflicts stay with the host. |
| SearchInput | Port addition | Native text/IME/clipboard, magnifier, conditional clear with focus return, optional Kbd hint, all sizes, disabled/read-only, char limits and narrow adaptation. Filtering, submission and shortcuts remain host-owned. |
| [Chip](chips.md) | Port addition | Square label, optional controlled toggle and independent remove button, native Tab/Enter/Space, pressed state and named actions. The host owns selection, removal and filtering. |
| [FilterChipGroup](chips.md#filter-chip-groups) | Port addition | Named wrapping group of keyed toggle Chips with native Tab/Enter/Space, disabled choices and typed selection requests. The host owns selection policy and filtering. |
| [PathInput](forms.md#path-input) | Port addition | Editable host path, distinct Browse button, typed open-file, open-folder and save-file requests, stacked narrow layout, and disabled/read-only states. Picker, validation and file operations stay host-owned. |
| ToggleButton | Port addition | Host bool, text/icon/icon-only, pressed metadata, stable name/identity, pointer/Enter/Space, disabled/busy and shared variants/sizes. Two-state only. |
| Toolbar | Port addition | Typed action/toggle requests, named/oriented group, one remembered Tab stop, arrow/Home/End focus, disabled/busy skipping, separators, horizontal overflow/wrap, vertical layout and focus repair. Shared-tree menu buttons. Arbitrary child inputs and shortcuts stay host-owned. |
| CommandRegistry | Port addition | Host-owned command metadata, typed requests, enabled/checked state, single-chord bindings, exact Command/Control/Alt/Shift matching with platform aliases, focus scopes, conflict reports and once-per-frame dispatch; Menu, Toolbar and CommandPalette adapters share labels and hints. Text editing, IME, overlays, recording and a focused control's own activation/navigation keys keep their input. No chord sequences, system hotkeys or OS menu integration. |
| SplitButton | Port addition | Primary action plus a separately named alternatives arrow with Dropdown keyboard, Escape/Tab and focus return; Down/Alt+Down opens from the primary; shared variants and sizes; disabled/busy states fade the joined control once. |
| KeyRecorder | Port addition | Captures one portable chord into a host value; Escape/Tab cancel, explicit Clear, disabled/read-only, accessible rejection of reserved chords and of bare Enter/Space/arrow/Home/End/page keys (host opt-in), macOS Control kept with Command. Session-local; OS-reserved chords may not reach the app. |
| MenuBar | Port addition | Host-controlled in-window menus, one remembered Tab stop, switching, checked/nested commands, popup metadata and dynamic focus repair. No operating-system menu integration or mnemonics. |
| Markdown | Port addition | CommonMark plus tables, tasks, strikethrough, heading anchors and callouts; token typography, named focusable links, native/report policies, fragment scrolling, lists, quotes, figures and cached parsing. No footnotes, math, raw HTML rendering, inline image widgets or column spans. |
| CodeBlock | Port addition | Selectable code, language label, copy feedback, line numbers, host highlight spans, wrapping/horizontal scrolling and height cap. No bundled highlighter or diff view. |
| [List](lists.md) | Port addition | Keyed host state; none/single/multiple selection; modifier ranges, type-ahead, roving focus and fixed-height virtualization; elided labels and host empty text. Passive row composition; no variable-height or embedded interactive controls. |
| [TreeView](tree-view.md) | Port addition | List row engine; host-owned keyed expansion, Left/Right navigation, disclosure clicks, retained hidden selection and virtualized tree/group semantics. Passive fixed-height rows; no lazy loading or drag/reorder. |
| [SplitPane](split-pane.md) | Port addition | Host-owned logical-point sizes; left/right or top/bottom panes, nested compositions, proportional minimum constraints, clipped children, keyboard and assistive resizing. No docking, ratio storage or collapse/restore. |
| [StatusBar](status-bar.md) | Port addition | Keyed host-formatted text, label/value and Kbd hint segments; icons, tone markers and inline progress; start/end groups; priority overflow with hidden keys and lone-segment elision; clickable, disabled and polite live segments in a named group. No overflow menu or multi-line segments. |
| [Avatar](avatar.md) | Port addition | Four square sizes and fills; Unicode-aware initials with an override; cover-cropped images with initials while loading or after failure; described presence marker; passive inside clickable rows. No groups, overflow counts or name-derived colors. |

Foundation: ten accent and mode combinations, a custom semantic palette,
standard egui styling, shared 32/48/56-point control heights, a four-point
spacing rhythm, zero-radius borders and reduced motion. Form validation covers
required, email, number, length and custom rules, host-owned triggers and stale
async protection. Header and sidebar, hero and footer composition and other web
page patterns are not provided, and the matrix doesn't imply whole-system parity
with the web reference.

Interaction and rendering tests run headless on macOS and in Linux CI; native
windows are inspected on macOS. Windows and Linux desktops, browser runtimes
and screen readers are not exercised beyond compiling and the accessibility
tree the tests read.
