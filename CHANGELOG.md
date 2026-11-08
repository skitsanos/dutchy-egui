# Changelog

## 1.0.22 — 2026-11-08

- Add a native Dutchy `ColorPicker`: named palettes, hex entry, RGB and optional
  alpha sliders/number inputs, transparency previews, copy, read-only/disabled
  states and accessible keyboard-operated popups over host-owned colors.
- Add a color-picker showcase and a Rust-focused component roadmap beyond the
  web inventory (DS-056).
- Increase the color-picker arrow's trailing inset to 16 points and reserve its
  text gap; record the Foliopress card, dropdown and search references (DS-057).

## 1.0.21 — 2026-11-07

- Upgrade the showcase-only native file-dialog dependency `rfd` from 0.17.2
  to 0.18.0 through Renovate PR #2. Refresh its lockfile entries and current
  documentation; the library adds no runtime dependency (DS-055).
- Ignore local demo-video work under `/media/`.

## 1.0.20 — 2026-11-07

- Form primitives: `Switch` (sliding square thumb, Switch role), `Kbd` key
  chips, `Segmented` exclusive choice (radio semantics, roving Left/Right),
  `Slider` over egui's slider with Dutchy rail and square handle, `NumberInput`
  with square step buttons, Up/Down stepping and SpinButton semantics,
  `DatePicker` with ISO text, caret-segment stepping and a keyboard-navigable
  month grid, and `TimePicker` with wrapping segment steps.
- `calendar::Date` and `calendar::Time` values with civil arithmetic and strict
  ISO parsing; no calendar dependency. Hosts supply `today`.
- Showcase "Form controls" page; primitives baselines in light, dark and green
  at 1x, 1.5x and 2x plus the open calendar.
- Single-line `Input` and `TimePicker` rows are vertically centered in their
  frame; textareas stay top-aligned (DS-047). The date picker's calendar
  hangs below the whole field, aligned with its start (DS-048).
- Follow-up verification fixes disabled typed-field stepping and calendar
  dismissal, preserves slider focus when siblings change, bounds calendar
  keyboard navigation and defers entry focus until after sizing, guards extreme
  numeric/date/time arithmetic, and restores visible month arrows
  (DS-049–DS-053).
- Enable Renovate dependency proposals through the reviewed onboarding
  configuration, with automatic merging explicitly disabled (PR #1, DS-054).

## 1.0.19 — 2026-11-07

- Opt-in Scribble pointer smoothing: `Scribble::smoothing` with tokenized
  `ScribbleSmoothing::{Off, Light, Regular, Strong}` levels. A distance-based
  pulled-string stabilizer, a midpoint quadratic B-spline fit and bounded
  simplification are baked into committed points, so history and SVG/raster/PNG
  exports are unchanged polylines. The live stroke draws the same curve; keyboard
  strokes stay exact. `ScribbleSmoothing::smooth` exposes the pipeline to hosts.
  The default stays `Off`; the showcase adds a "Smooth pointer strokes" toggle
  (DS-046).

## 1.0.18 — 2026-11-07

- The showcase sidebar reveals page buttons when keyboard or assistive focus
  arrives, while preserving manual scrolling with retained focus (DS-045).
- The tables guide links the implemented media, file-selection and Scribble
  guides instead of calling them future work (DS-041).

## 1.0.17 — 2026-11-07

- Delft Blue (hsl 221 83% 53%) and Emerald Green (hsl 160 84% 31%) accents,
  additions of this port recorded in the reference; `Accent::ALL` lists the
  five presets for theme pickers; Blue/light 1.5x and Green/dark 2x baselines.
- The showcase navigation leads with Overview, Typography and Tokens, then the
  component pages sorted by name; the switcher offers all five accents. The
  sidebar pins its foundation footer and scrolls the page list above it.

## 1.0.16 — 2026-11-07

- Focused file-row actions, image expand/retry controls, toast controls and
  table cells reveal once (focus arrival or layout move) and never pull a
  scroll area back; the table body ignores focus inside its sticky header.
- Field, checkbox and radio labels follow inverse card and banner text colors;
  editors and select triggers keep foreground text on their light fill.
- Duplicate table row keys receive occurrence salts instead of looping
  repaints; row height measurements are evicted when rows leave the view.
- The command palette and overlays stop repainting while another modal is on
  top; overlays enter once they are the topmost modal.
- Accessible names keep the host's casing for buttons, tabs, accordions,
  breadcrumbs, badges, headings, field labels and uppercase text.
- The theme style is installed for both egui themes, so preference changes no
  longer drop native widgets to egui defaults; six baselines refreshed.
- Spaced galleries paint nothing between tiles and border each tile; source
  credits use wider tracking; gallery expand controls use the live 70 % fill;
  URL image decoding reports a codec panic as a failure instead of hanging.
- Toast entrance fades follow the queue's guarded clock; file sizes round
  before choosing a unit; a capped Scribble gesture commits its points; the
  drop zone and file rows follow the live component.
- Field, progress, card, skeleton and empty-state gaps render as documented
  with item spacing absorbed; card footers use a one-point border divider and
  card titles drop heading tracking.
- Tooltips follow the live stylesheet: 12/6 padding, 256-point text width
  within the viewport minus 32, 0.02 em letter spacing.
- Tinted alerts keep foreground titles and muted messages, status glyphs are
  circle-backed, and the resting dismiss glyph is muted.
- Bordered accordion headers use wide tracking, the accent edge lights over
  the open body, arrows lock on focused tab close buttons, and text, IME and
  clipboard events restore the keyboard focus cue.
- Table headers use the bold body face with wider tracking, muted headers keep
  foreground text, and sort glyphs follow the live double arrow.
- Compact pagination summaries bold their numbers, page numbers rest on the
  foreground, collapsed breadcrumb buttons rest on the border color, and
  compact stat labels are bold with a 6-point trend gap.
- Ledger verification audit (DS-028) with entries DS-027 to DS-044; DS-003 and
  DS-027 closed as design decisions (source status and accent colors kept, host
  palette overrides for stricter ratios); DS-029, DS-030, DS-033 to DS-036 and
  DS-039 to DS-044 resolved or resolved for their defects; baselines refreshed.

## 1.0.15 — 2026-11-07

- Scribble canvas with ordered primary-pointer capture, dots, keyboard drawing,
  fixed paper/ink tokens, per-stroke brushes, read-only/disabled and lifecycle guards.
- Host-owned bounded vector data and undoable clear/undo/redo; immutable snapshots
  support background export without retaining UI controls or draft gestures.
- Static SVG and bounded CPU raster export with transparency/paper backgrounds;
  independent optional `png-export` reuses image 0.25.10 for RGBA PNG bytes.
- Manual scrolling no longer snaps to a focused canvas/toolbar; keyboard reveal
  remains available, keyboard drafts survive layout changes, and read-only empty
  views use an accurate hint.
- Native arrow-focus retention with Tab/Escape exit, and picker focus restoration
  after export controls re-enable, preserving intervening focus changes.
- Drawing/history, async preview and native SVG/PNG Save As showcase; model,
  interaction, export, host-workflow and theme/scale/narrow rendering checks.
  Completes the core advanced inventory with broader parity limitations explicit.

## 1.0.14 — 2026-11-06

- Dashed and compact file selection surfaces, native keyboard browse requests,
  lazy egui 0.37 drop handles, spatial/modal/disabled guards and one claim per frame.
- Pure metadata policies for extension/MIME alternatives, byte-size boundaries,
  unknown sizes and retained-count limits, shared by picker/drop batches.
- Keyed file rows with sizes, ready/determinate/indeterminate/complete/error states,
  independent remove/cancel/retry actions, hover feedback and removal focus return.
- Native picker and background metadata showcase using development-only rfd 0.17.2;
  clearly labelled local transfer simulation, retained host state and retry flows.
- Theme/scale, narrow-layout, drop routing, keyboard/focus and host-workflow tests.
  Runtime dependencies are unchanged. Added the post-push cleanup convention.

## 1.0.13 — 2026-11-06

- Bordered images on native egui sources with explicit aspect/contain/cover,
  caption/source typography, named loading/error states and expand/retry controls.
- Keyed responsive galleries with joined/spaced tiles, stable identities,
  disabled items, empty guidance and host-owned opening requests.
- Fullscreen in-app lightboxes with native modality, keyed host selection,
  enabled-item navigation, optional wrapping, constrained captions and focus return.
- Images/Galleries/Lightboxes showcase pages using original raster illustrations;
  loader, interaction, fitting, pixel, host-workflow and theme/scale verification.
- Pending images show a named square spinner and loading label, respecting
  reduced motion, clipping and completion without moving the image box.
- Optional `url-images`: asynchronous HTTP(S) PNG/JPEG loading, bounded streamed
  bodies/decoded pixels/concurrency/cache, explicit retry and safe stale requests.
  Adds optional ehttp 0.7.3 and image 0.25.10; default dependencies stay unchanged.
- Picsum source switch and reload across media demo pages, photographer credits,
  deterministic HTTP integration tests, and native live-provider verification.

## 1.0.12 — 2026-11-06

- Start advanced components with keyed `DataTable` composition, custom native
  cells, fixed/flexible widths, alignment, sticky headers and two-axis scrolling.
- Host-owned typed sort and current-view selection requests, disabled records,
  stable editor identity and explicit focus return after row/column removal.
- Muted/inverse/plain headers, compact density, striped/hover/selected rows,
  named busy skeletons, constrained empty guidance and semantic table metadata.
- Vertically centered header/body content, including mixed native controls and
  wrapped cells; keyed natural-height measurements preserve stable reflow.
- Data Tables showcase with host search, numeric sorting, pagination, retained
  selection and independent actions; interaction, geometry and theme/scale checks.
  No new dependencies. Initial tables are nonvirtualized; hosts bound/page slices.

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
