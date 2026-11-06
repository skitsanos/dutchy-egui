# Changelog

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
