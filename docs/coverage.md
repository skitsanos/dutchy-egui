# Component coverage

Version 1.0.7 adds menus, dropdowns, searchable selection, and command search after
accordions and tabs, following
the core application essentials. Registry groups are not a count of individual
controls. "Planned" means no public implementation is shipped.

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
| Pagination | Planned | Numbered, ellipsis, compact controls |
| Accordion | Interaction components | Controlled single/multiple expansion, accent-edge/bordered surfaces, keyboard headers, named regions, disabled items, wrapping and stable body scopes |
| Tabs | Interaction components | Underline/segmented, full width or natural width, horizontal overflow, manual keyboard activation, roving focus, disabled items, independent close requests and focus after removal. No docking, reorder dragging, overflow menu, or arbitrary icons yet. |
| Toasts | Planned | Stack, expiry, dismiss/action behavior |
| Modal / Drawer | Planned | Blocking overlays, backdrop, dismissal, focus restoration |
| Dropdowns | Interaction components | Anchored/context actions, start/end alignment, viewport constraints, scrolling, keyboard/dismissal/focus return; searchable select and native modal command palette with host-owned query/open/selection/actions |
| Data Table | Planned | Table primitives, empty/loading, host-controlled sort/filter |
| Stepper | Planned | Horizontal/vertical progress |
| Tooltips | Application essentials | Text hints; four preferred sides, native constrained placement/hover timing, keyboard focus, Escape dismissal, and merged accessible descriptions |
| File Upload | Planned | File selection/drop zone/list; application-owned transfer |
| Image | Planned | Caption/gallery/lightbox; optional loaders |
| Breadcrumbs | Planned | Standard/truncated hierarchy |
| Flex | Application essentials | Token gaps, row/column/reverse, row wrapping, native cross alignment; centered containers and keyed responsive grids. CSS baseline/distribution modes are outside this scope. |
| Menu | Interaction components | Keyed actions, consecutive named groups, separators, shortcut hints, optional decorative icons, disabled/destructive states, native pointer/assistive/keyboard input. Nested submenus, checked actions, and menu bars remain planned. |
| Text | Visual proof | Three roles; four actual weights; 12 sizes; tracking, leading, color, Unicode uppercase, italics, wrapping |
| Scribble | Planned | Pointer drawing, clear, host-owned strokes, optional export |
| Textarea | Application essentials (partial) | Multiline, row sizing, Unicode count/limit, read-only/disabled and field feedback. Interactive resize remains planned. |

Foundation support: six accent/mode combinations, custom semantic palette,
standard egui styling, shared 32/48/56-point control heights, four-point spacing
rhythm, zero-radius borders, and reduced motion. Form validation includes
required/email/number/length/custom rules, host-owned triggers, and stale async
protection. Header/sidebar, hero/footer composition, and other web patterns still
require work beyond these groups. The application-essentials milestone's core
scope is implemented, including a public-API form and workspace settings/dashboard
composition. Partial controls and broader web parity remain explicit above;
interaction components are underway. General overlays, toasts, and the other
planned navigation groups remain outstanding. The command palette uses a native
modal internally without exposing a general modal API.
