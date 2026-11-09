# Component coverage

Version 1.0.20 adds the form primitives the web registry never listed, after
1.0.15 completed Scribble and SVG/optional PNG export, file selection, images,
galleries, lightboxes, data tables and the core interaction and
application-essential groups. Registry groups are not a count of individual
controls. "Planned" means no public implementation is shipped; "Port addition"
marks a control with no web counterpart.

Version 1.0.22 adds ColorPicker. The native library intentionally
extends beyond the web inventory; see the [component roadmap](component-roadmap.md).
Version 1.0.23 adds MediaCard and SearchInput. Version 1.0.24 adds
IconButton, ToggleButton and Toolbar; see [their contract](actions-toolbars.md).
Version 1.0.24 also integrates [Markdown and CodeBlock](markdown.md).

| Web registry group | Status | Supported scope / next work |
| --- | --- | --- |
| Buttons | Visual proof | Five variants; three sizes; disabled/loading/full width; focus and keyboard. IconButton adds named square actions, original/custom vector icons and shared variants/sizes. |
| Forms | Application essentials | Input/password, checkbox, radio group, typed select, read-only/disabled; field labels/help/errors/required/pending/success; validation triggers and async tickets |
| Cards | Application essentials | Six surfaces, four padding tiers, headers/footers, scoped text colors, interactive and disabled cards; host-owned actions |
| Badges & Tags | Visual proof | Eight variants, uppercase labels, keyboard-accessible close affordance |
| Alerts | Application essentials | Four severities, tinted/solid banners, accessible announcements, action/dismiss results, explicit focus-return target |
| Empty State | Application essentials | Generic/search/table-body/dashboard patterns, heading/context, decorative icons, independent actions, disabled controls, optional polite announcements |
| Skeleton | Application essentials | Five shapes, count/dimension overrides, named busy groups, card/list/content/table patterns, pause/reduced motion and clipped scheduling |
| Spinner | Visual proof | Three-sided square; 12/16/20 sizes; color, pause, reduced motion, clipped-widget scheduling |
| Progress | Application essentials | Determinate/custom maximum and indeterminate; 4/8/16-point tracks; four fills; labels/percentages; sanitized range metadata and reduced motion |
| Stats | Application essentials | Host-formatted KPIs, four surfaces, detailed context/accent, explicit direction/sentiment, responsive grids/compact rows and stable identity |
| Pagination | Interaction components | Numbered/ellipsis and compact controls; current-page metadata, stable focus, disabled endpoints, empty/invalid ranges, bounded large counts, wrapping, and host-owned page/data requests |
| Accordion | Interaction components | Controlled single/multiple expansion, accent-edge/bordered surfaces, keyboard headers, named regions, disabled items, wrapping and stable body scopes |
| Tabs | Interaction components | Underline/segmented, full width or natural width, horizontal overflow, manual keyboard activation, roving focus, disabled items, independent close requests and focus after removal. No docking, reorder dragging, overflow menu, or arbitrary icons yet. |
| Toasts | Interaction components | Host-owned FIFO queue, typed single-use actions, manual/expiry events, four corners/severities, solid/outline, 1–4 visible slots, persistent/pause policies, live metadata, stable focus, constrained scrolling, and entrance fade. No exit/slide transitions or notification-history service. |
| Modal / Drawer | Interaction components | Centered dialogs and full-height left/right drawers; three widths each, native blocking/focus confinement, configurable dismissal, replacement focus targets, divided footers, constrained scrolling, and reduced-motion-aware drawer entrances. No docked/top/bottom drawers, resizing, or exit animation. |
| Dropdowns | Interaction components | Anchored/context actions, start/end alignment, viewport constraints, scrolling, keyboard/dismissal/focus return; searchable select and native modal command palette with host-owned query/open/selection/actions |
| Data Table | Advanced components | Keyed custom cells, fixed/flexible widths, alignment, sticky headers, two-axis scrolling, three header surfaces, compact/striped/hover/selected rows, native sort/selection requests, disabled rows, busy/empty guidance and host filter/pagination composition. Nonvirtualized; no resizing/pinning, dragging, arbitrary spans or multicolumn sort yet. |
| Stepper | Interaction components | Horizontal/vertical/compact/position progress; keyed host state, explicit completion, optional navigation requests, current/status metadata, narrow adaptation, compact scrolling, empty/single/unknown states and validated host workflow |
| Tooltips | Application essentials | Text hints; four preferred sides, native constrained placement/hover timing, keyboard focus, Escape dismissal, and merged accessible descriptions |
| File Upload | Advanced components | Dashed/compact browse requests, lazy spatially claimed drop handles, shared type/size/count metadata validation, keyed file lists and progress/errors/remove/cancel/retry with focus return. Native picker is a demo-host integration. Transport, byte reads, folder traversal, browser picker and content verification stay host-owned. |
| Image | Advanced components | Native source/texture presentation, sharp frames, explicit aspect/contain/cover, caption/source, busy/error/retry, keyed responsive galleries and constrained modal lightboxes with keyboard navigation, caption scrolling and focus return. Optional `url-images` installs HTTP(S) PNG/JPEG; other URI/byte loaders stay host-owned. No zoom/pan, slideshow, masonry or drag reorder. |
| Breadcrumbs | Interaction components | Keyed destinations, current-page metadata, slash/chevron, icon root, muted bar, wrapping and collapsed-ancestor menus; host-owned routing and collapse policy |
| Flex | Application essentials | Token gaps, row/column/reverse, row wrapping, native cross alignment; centered containers and keyed responsive grids. CSS baseline/distribution modes are outside this scope. |
| Menu | Interaction components | Keyed actions, groups, separators, hints/icons and disabled/destructive states. Unreleased DS-073 adds controlled checks, nested submenus, viewport placement and one-level keyboard dismissal. Radio/mixed checks remain outside scope. |
| Text | Visual proof | Three roles; four actual weights; 12 sizes; tracking, leading, color, Unicode uppercase, italics, wrapping |
| Scribble | Advanced components | Pointer/dot and keyboard drawing, host-owned bounded strokes/history, undoable clear, fixed paper, captured brushes, read-only/disabled, immutable snapshots, SVG/CPU raster/optional PNG, opt-in pointer smoothing and native demo Save As. No pressure/tilt, multitouch, eraser, tapered ends, history persistence or shape editor. |
| Textarea | Application essentials (partial) | Multiline, row sizing, Unicode count/limit, read-only/disabled and field feedback. Interactive resize remains planned. |
| Switch | Port addition | 36 × 20 track with a sliding square thumb, Switch role, Space/Enter activation, disabled state, reduced motion |
| Kbd | Port addition | Mono uppercase key chip on the muted surface; a label, composed per key |
| Segmented | Port addition | Typed exclusive segments at a shared height, foreground selection, RadioGroup/RadioButton semantics, one tab stop, Left/Right/Home/End roving that selects, disabled segments, full width |
| Slider | Port addition | egui slider interaction and Slider semantics under a 4-point rail, primary trailing fill, 20-point square handle, step snapping, host-formatted value label, disabled state. Single value; no range or vertical slider yet. |
| NumberInput | Port addition | Editor with square decrease/increase buttons, Up/Down stepping, bounds, decimals, read-only, SpinButton value/min/max/step; unparsable text reverts on blur |
| DatePicker | Port addition | ISO `YYYY-MM-DD` editor, caret-segment stepping, bounds, host-supplied today, calendar popup with month navigation, arrows/Home/End/PageUp/PageDown, Enter/Escape and focus return. No week numbers, ranges or locale formats yet. |
| TimePicker | Port addition | `HH:MM` or `HH:MM:SS` editor with wrapping caret-segment steps and strict parsing; no time zones or 12-hour display. |
| ColorPicker | Port addition | Named theme/host palettes, RGB/optional alpha sliders and numbers, hex editing/copy, transparency previews, constrained popup, disabled/read-only and keyboard/accessibility. No HSV/HSL, eyedropper, history service or HDR yet. |
| MediaCard | Port addition | Image/URI/byte sources, category/title/description, independent preview/metadata/save/retry, host-owned saved state, responsive width and native loading/error pipeline. No custom metadata slots or masonry. |
| SearchInput | Port addition | Native text/IME/clipboard, magnifier, conditional clear with focus return, optional Kbd hint, all sizes, disabled/read-only, char limits and narrow adaptation. Filtering, submission and shortcuts remain host-owned. |
| ToggleButton | Port addition (1.0.24) | Host bool, text/icon/icon-only, pressed metadata, stable name/identity, pointer/Enter/Space, disabled/busy and shared variants/sizes. Two-state only. |
| Toolbar | Port addition (1.0.24) | Typed action/toggle requests, named/oriented group, one remembered Tab stop, arrow/Home/End focus, disabled/busy skipping, separators, overflow/wrap and focus repair. Unreleased DS-073 adds shared-tree menu buttons. Arbitrary child inputs and shortcuts stay host-owned. |
| MenuBar | Port addition (Unreleased) | Host-controlled in-window menus, one remembered Tab stop, switching, checked/nested commands, popup metadata and dynamic focus repair. No operating-system menu integration or mnemonics. |

| Markdown | Port addition (1.0.24) | CommonMark plus tables, tasks, strikethrough, heading anchors and callouts; token typography, named focusable links, native/report policies, fragment scrolling, lists, quotes, figures and cached parsing. No footnotes, math, raw HTML rendering, inline image widgets or column spans. |
| CodeBlock | Port addition (1.0.24) | Selectable code, language label, copy feedback, line numbers, host highlight spans, wrapping/horizontal scrolling and height cap. No bundled highlighter or diff view. |

Foundation support: ten accent/mode combinations, custom semantic palette,
standard egui styling, shared 32/48/56-point control heights, four-point spacing
rhythm, zero-radius borders, and reduced motion. Form validation includes
required/email/number/length/custom rules, host-owned triggers, and stale async
protection. Header/sidebar, hero/footer composition, and other web patterns still
require work beyond these groups. The application-essentials milestone's core
scope is implemented, including a public-API form and workspace settings/dashboard
composition. Partial controls and broader web parity remain explicit above;
the core interaction-components scope is implemented, including a public-API
notification queue with host Undo/Retry actions. Advanced work includes data
tables, images/galleries/lightboxes, file selection/drop zones and Scribble/export.
The core advanced-component inventory is implemented. Partial variants and broader
web parity remain explicit in the matrix; milestone completion does not imply whole-system parity.
