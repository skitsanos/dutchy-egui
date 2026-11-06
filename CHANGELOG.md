# Changelog

## 1.0.11 — 2026-11-06

- Application-owned toast queues with stable keys, replacement/removal, typed
  single-use action payloads, and manual/expiry dismissal events.
- Four corner stacks, four severities, solid/outlined treatments, native controls,
  live metadata, focus retention/return, and scrolling for narrow overflow.
- Visible-only lifetimes, pending FIFO slots, hover/focus/inactive/modal pause,
  persistent notifications, and reduced-motion-aware entrance fading.
- Host Undo/Retry showcase, timing/interaction regressions, and multi-theme/scale
  rendering; completes the core interaction milestone. No new dependencies.

## 1.0.10 — 2026-11-06

- Centered modals and full-height left/right overlay drawers with token widths,
  sharp borders, named titles, native input blocking, and optional divided footers.
- Host-owned visibility/content, configurable close/Escape/backdrop dismissal,
  initial focus and deferred return focus, including replacement targets.
- Viewport-constrained scrolling with a persistent header, native Tab traversal,
  popup-first Escape, and reduced-motion-aware drawer entrances.
- Wider confirmation with inline desktop actions, validated profile, and retained
  preference showcase flows;
  interaction/motion regressions and theme/scale baselines. No new dependencies.

## 1.0.9 — 2026-11-06

- Keyed, host-controlled stepper with horizontal, vertical, compact, and position-
  progress presentations; explicit not-started, current, and completed states.
- Adaptive columns, wrapped descriptions, compact scrolling, finite empty/single/
  unknown-key behavior, and optional native navigation requests.
- Accessible status/current-step metadata, stable identities, keyboard focus,
  and a validated host workflow with retained input and explicit submission.
- Interaction regressions and multi-theme/scale rendering. No new dependencies.

## 1.0.8 — 2026-11-06

- Keyed breadcrumbs with slash/chevron separators, decorative icon roots, a muted
  bar, wrapping labels, and host-controlled collapsed-ancestor menus.
- Numbered/ellipsis and compact pagination with typed page requests, current-page
  metadata, disabled endpoints, empty/invalid ranges, and bounded large counts.
- Native keyboard behavior, stable navigation identities, host-owned route/data
  examples, and multi-theme/scale rendering. No new dependencies.

## 1.0.7 — 2026-11-06

- Shared flat hover/pressed feedback and pointing cursors for alert/banner, tab,
  and removable-tag close controls; disabled controls retain their resting state.
- Alert dismissal uses a real 32x32-point target with equal 16-point top/right
  insets, correcting the zero-width blank button and misplaced visible glyph.
- Grouped menus with stable action IDs, shortcut hints, optional decorative icons,
  separators, disabled states, and destructive treatment.
- Anchored and context dropdowns with native constrained placement, scrolling,
  keyboard navigation, dismissal, and focus return/Tab exit.
- Searchable selection with separate host-owned query/selection, active-descendant
  accessibility, filtering, no-match feedback, and explicit commit results.
- Native modal command palette with optional Cmd/Ctrl+K, grouped results, input
  blocking, focus confinement, and deferred focus restoration after closing.
- Menus/Dropdowns showcase pages, host-action examples, interaction tests, and
  light/dark rendering at 1x/1.5x/2x. No new dependencies.

## 1.0.6 — 2026-11-06

- Controlled accordions with single/multiple expansion, accent-edge/bordered
  surfaces, wrapping headers, named regions, and native keyboard actions.
- Underline/segmented tabs with manual keyboard activation, roving focus,
  disabled items, stable panel scopes, and horizontal overflow.
- Independent close requests, Delete shortcut, focus after host removal, and an
  explicit focus destination when no tabs remain. The host retains all data.
- Selected tabs use their active underline/surface without an extra focus outline;
  unselected keyboard targets and close controls retain focus cues.
- Shared focus-visible behavior distinguishes pointer and keyboard input;
  accordion focus rings sit inside the header with all four edges visible.
- Accordions/Tabs showcase pages, focused interaction and motion checks, and
  light/dark rendering at 1x/1.5x/2x. No new dependencies.

## 1.0.5 — 2026-11-06

- Inverted text tooltips with four preferred sides, native constrained placement,
  hover/focus presentation, accessible descriptions, and Escape dismissal.
- Token-based Flex rows/columns, reverse directions, row wrapping, cross-axis
  alignment, centered containers, and keyed responsive grids.
- Tooltips/Layout showcase pages and a host-owned workspace settings/dashboard
  example; the existing form now uses responsive grid composition.
- Focus/reflow/interaction tests and light/dark rendering at 1x/1.5x/2x.
  No new dependencies; egui remains the sole direct runtime crate.

## 1.0.4 — 2026-11-06

- Host-formatted statistics, four surfaces, regular/small type scales, optional
  accent borders, detailed context, and responsive grid/compact layouts.
- Explicit trend direction and sentiment, visible text alternatives, and grouped
  accessibility names/values/context; no inferred arithmetic or formatting.
- Generic/search/table-body/dashboard empty states with decorative icons, wrapping
  guidance, independent native actions, disabled states, and optional announcements.
- Stats/Empty States showcase pages, host-controlled content replacement/focus,
  behavior tests, and multiple-scale/theme rendering. No new runtime dependencies.

## 1.0.3 — 2026-11-06

- Determinate/custom-range and indeterminate progress, three track heights, four
  fills, labels, percentages, stable IDs, and consistent sanitized accessibility ranges.
- Five skeleton shapes with counts/dimensions, one named busy group, and
  card/list/content/table placeholder compositions.
- Visible-only animation scheduling, explicit pause, reduced motion, and shared
  loading geometry/motion tokens. No additional runtime dependencies.
- Progress/Skeletons showcase pages, reserved-height content replacement example,
  range/accessibility/motion tests, and light/dark rendering at multiple scales.

## 1.0.2 — 2026-11-06

- Six card surfaces, four padding tiers, composable headings/descriptions/headers/
  footers, and explicitly named interactive cards with native keyboard behavior.
- Scoped surface foreground inheritance for Dutchy and native text; nested cards
  restore their own foreground and custom text colors remain explicit.
- Four alert severities, tinted alerts, solid banners, host-owned action/dismiss
  results, live metadata, and explicit focus restoration after dismissal.
- Cards/Alerts showcase pages, stable-ID and interaction tests, light/dark and
  multiple-scale rendering baselines. No additional runtime dependencies.

## 1.0.1 — 2026-11-06

- Application essentials, first step: form controls and field validation.
- Host-owned field values and validation state; shared control geometry and accessible feedback.
- Inputs/password, textarea with Unicode limits/counting, checkbox, radio group,
  typed select, field help/required/error/success/pending presentation.
- Change/blur/submit validation, pure required/email/number/length/custom rules,
  and opaque tickets that reject stale/cross-field async results.
- Interactive Forms showcase, light/dark render baselines, keyboard and lifecycle
  tests. No additional runtime dependencies.

## 1.0.0 — 2026-11-05

- Initial visual-proof library on egui 0.37.0.
- Shared warm palettes, light/dark themes, spacing, borders, control sizes, motion,
  and typography tokens.
- Bundled licensed static fonts, explicit font weights, and LayoutJob text builder.
- Five button variants, eight badge variants, and a visibility-aware square spinner.
- Native eframe showcase, interaction tests, rendering baselines, and package docs.
- Implementation plan, coverage matrix, project rules, and Issues Ledger.

Remaining component groups and source-palette limitations are documented in
`docs/coverage.md` and `docs/issues/`.
