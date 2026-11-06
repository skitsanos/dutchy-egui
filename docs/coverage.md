# Component coverage

Version 1.0.1 adds the first application-essentials step: form controls and
validation. Registry groups are not a count of individual controls. "Planned"
means no public implementation is shipped.

| Web registry group | Status | Supported scope / next work |
| --- | --- | --- |
| Buttons | Visual proof | Five variants; three sizes; disabled/loading/full width; focus and keyboard. Compound/icon-only APIs remain planned. |
| Forms | Application essentials | Input/password, checkbox, radio group, typed select, read-only/disabled; field labels/help/errors/required/pending/success; validation triggers and async tickets |
| Cards | Planned | Container and interactive variants |
| Badges & Tags | Visual proof | Eight variants, uppercase labels, keyboard-accessible close affordance |
| Alerts | Planned | Severity, banner, dismiss/action behavior |
| Empty State | Planned | Generic, search, and table patterns |
| Skeleton | Planned | Card/list/content/table placeholders |
| Spinner | Visual proof | Three-sided square; 12/16/20 sizes; color, pause, reduced motion, clipped-widget scheduling |
| Progress | Planned | Sizes, status, labels, max value |
| Stats | Planned | KPI, trends, compact/detailed variants |
| Pagination | Planned | Numbered, ellipsis, compact controls |
| Accordion | Planned | Single/multiple open behavior |
| Tabs | Planned | Underline/boxed, overflow, optional close |
| Toasts | Planned | Stack, expiry, dismiss/action behavior |
| Modal / Drawer | Planned | Blocking overlays, backdrop, dismissal, focus restoration |
| Dropdowns | Planned | Action menu, searchable select, command palette |
| Data Table | Planned | Table primitives, empty/loading, host-controlled sort/filter |
| Stepper | Planned | Horizontal/vertical progress |
| Tooltips | Planned | Placement and keyboard presentation |
| File Upload | Planned | File selection/drop zone/list; application-owned transfer |
| Image | Planned | Caption/gallery/lightbox; optional loaders |
| Breadcrumbs | Planned | Standard/truncated hierarchy |
| Flex | Planned | Token-based layout over native egui primitives |
| Menu | Planned | Groups, separators, shortcuts, destructive intent |
| Text | Visual proof | Three roles; four actual weights; 12 sizes; tracking, leading, color, Unicode uppercase, italics, wrapping |
| Scribble | Planned | Pointer drawing, clear, host-owned strokes, optional export |
| Textarea | Application essentials (partial) | Multiline, row sizing, Unicode count/limit, read-only/disabled and field feedback. Interactive resize remains planned. |

Foundation support: six accent/mode combinations, custom semantic palette,
standard egui styling, shared 32/48/56-point control heights, four-point spacing
rhythm, zero-radius borders, and reduced motion. Form validation includes
required/email/number/length/custom rules, host-owned triggers, and stale async
protection. Header/sidebar, hero/footer composition, and other web patterns still
require work beyond these groups. Remaining application essentials are cards,
alerts, progress, skeletons, stats, empty states, tooltips, and layout helpers.
