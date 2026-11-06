# Dutchy for egui — implementation plan

## Objective

Ship `dutchy-egui`, an egui-native design system with reusable tokens, theming,
typography, composable widgets, and a separate runnable showcase. Start at 1.0.0;
that version identifies the first visual-proof release, not completion of all
components. Every subsequent main-branch push advances the version.

## Reference contract

The web source is `skitsanos/dutchy-design-system`, revision
`e91ca1d506e1c7a11b58a5d6857d82208229812a` (reviewed 2026-11-05).
Its local checkout is `/Users/skitsanos/FTP/Projects/design-systems/dutchy-design-system`.
The library must build independently of that checkout.

- Appearance: `src/styles/dutchy.css`, component implementations, and rendered
  localhost reference (`http://localhost:4545/`).
- Intended behavior: `docs/01-design-system/`, component demos, and handlers.
- Scope: the registry's 27 groups, with individual controls/variants tracked
  separately. Forms and dropdowns contain multiple controls.
- Preserve warm neutrals: background #FAF8F5, foreground #120F0C, border #E7DFDA.
  Resolve HSL tokens once to sRGB rather than copying inconsistent old hex examples.
- Zero radius, flat fills, no gradients or blurred shadows. Soft-shadow examples
  in the web effects documentation are an intentional difference.
- Share Small/Medium/Large control heights across buttons, inputs, and selects.
  Initial contract: 32/48/56 logical points; future text controls reuse it.
- Orange, purple, crimson accents are independent of light/dark mode. Dark
  neutrals follow the documented dark palette; complete semantic mappings
  explicitly because the current web stylesheet has no dark overrides.

## Architecture

`tokens` owns semantic palette, spacing, border, motion, control-size, and text
tokens. `theme` adapts tokens to egui defaults. `fonts` installs named static
faces without weight/fallback ambiguity. `text` builds LayoutJobs for family,
size, weight, tracking, leading, and color. `interaction` centralizes focus
outlines. `widgets` contains modular component implementations. `layout` composes
native layouts with shared tokens and stable scopes. Add optional `integrations`
only when later milestones require them.

Leaf widgets implement egui::Widget and return Response. Containers use
closure-based show methods. Values and durable state remain application-owned;
stateful controls accept stable IDs. Theme updates do not reinstall fonts.

Use the latest stable egui release family and verify current versions whenever
adding dependencies. egui is the sole direct runtime crate for this milestone.
eframe and egui_kittest are development dependencies. Implement small helpers
locally; adapt compatible MIT code only with provenance and notices. Keep code
files near or below 400 lines and favor small functions over complex branching.

## Milestones

| Milestone | Deliverable | Exit criterion |
| --- | --- | --- |
| Reference specification | Token contract, coverage matrix, API and dependency policy | Differences and supported behavior explicit |
| Visual proof | Theme, bundled fonts, text, button, badge, square spinner, showcase | Rendering, focus, sizing, states, and theme changes verified at multiple scales |
| Application essentials | Inputs, textarea, checkbox, radio, select, field feedback; cards, alerts, tags, stats, empty states, progress, skeletons, tooltips, layout | Form/settings/dashboard example uses public APIs |
| Interaction components | Accordion, tabs, menus, dropdowns, searchable select, command palette, breadcrumbs, pagination, stepper, modal, drawer, toasts | Keyboard operation, focus restoration, dismissal, overflow, and controlled state verified |
| Advanced components | Tables, image/gallery/lightbox, file selection/drop zones, Scribble/export | Optional integrations work on declared platforms |
| Release hardening | Docs, examples, licenses, CI, compatibility and packaging | Package builds and examples work from packaged sources |

Use built-in egui layout/Grid before adding layout abstractions. An overlay
drawer needs a backdrop and input blocking; a docked panel is a separate pattern.
Sorting/filtering, upload transport, and async validation execution belong to
the application. Repaint only while visible animation needs it.

## Verification and release

The showcase exposes themes, text scales, widget variants/states, narrow layouts,
and application examples as coverage expands. Test interactions separately from
focused rendering snapshots. Check 1x, 1.5x, and 2x scales, accessible names,
keyboard focus, disabled/loading activation, text wrapping, and control sizes.
Measure contrast and record source-palette limitations without silently changing
brand colors. Bundle original font licenses and reproducible provenance.

Run formatting, tests, Clippy with warnings denied, feature configurations,
doctests, documentation, WASM library compilation, and package verification.
Native GUI verification and automated headless rendering are separate evidence.

The first implementation is the visual-proof milestone. Subsequent work follows
the coverage matrix and Issues Ledger; version numbers do not imply full parity.

Progress: 1.0.0 visual proof was pushed to main as `f944089` on 2026-11-06.
The 1.0.1 application-essentials forms/validation step was pushed to main as
`cefe635` on 2026-11-06. The 1.0.2 cards/alerts step was pushed to main as
`3c3b5ae` on 2026-11-06. The 1.0.3 progress/skeletons step was pushed to main as
`d750373` on 2026-11-06. The 1.0.4 stats/empty-states step was pushed to main as
`c8c8747` on 2026-11-06. The 1.0.5 tooltips/layout step was pushed to main as
`954ca23` on 2026-11-06, completing the core application-essentials scope with
a composed workspace settings/dashboard example. The 1.0.6 accordion/tab step was
pushed to main as `4840e10` on 2026-11-06, including revised composite focus cues:
selected tabs use their active mark; accordion keyboard outlines are inset and
complete, with pointer selection leaving shared borders clean.
The 1.0.7 menus/dropdowns and close-control step was pushed to main as `198ab48`
on 2026-11-06. The 1.0.8 breadcrumbs/pagination step was pushed to main as
`59a80b4` on 2026-11-06. The 1.0.9 step is developed locally on `ticket/stepper`:
four workflow presentations, explicit host-owned position/completion, optional
navigation requests, and a form-validated setup example with retained values.
See [coverage](coverage.md), [accordion/tab decisions](accordions-tabs.md),
[menu/dropdown decisions](menus-dropdowns.md),
[navigation decisions](breadcrumbs-pagination.md), and [stepper decisions](stepper.md)
for behavior and scope boundaries.
