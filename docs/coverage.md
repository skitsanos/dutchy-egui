# Component coverage

Version 1.0.15 adds Scribble and SVG/optional PNG export after file selection,
images, galleries, lightboxes, data tables and the core
interaction and application-essential groups. Registry groups are not a count
of individual controls. "Planned" means no public implementation is shipped.

| Web registry group | Status | Supported scope / next work |
| --- | --- | --- |
| Buttons | Visual proof | Five variants; three sizes; disabled/loading/full width; focus and keyboard. Compound/icon-only APIs remain planned. |
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
| Menu | Interaction components | Keyed actions, consecutive named groups, separators, shortcut hints, optional decorative icons, disabled/destructive states, native pointer/assistive/keyboard input. Nested submenus, checked actions, and menu bars remain planned. |
| Text | Visual proof | Three roles; four actual weights; 12 sizes; tracking, leading, color, Unicode uppercase, italics, wrapping |
| Scribble | Advanced components | Pointer/dot and keyboard drawing, host-owned bounded strokes/history, undoable clear, fixed paper, captured brushes, read-only/disabled, immutable snapshots, SVG/CPU raster/optional PNG and native demo Save As. No pressure/tilt, multitouch, eraser, smoothing, history persistence or shape editor. |
| Textarea | Application essentials (partial) | Multiline, row sizing, Unicode count/limit, read-only/disabled and field feedback. Interactive resize remains planned. |

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
