# Verification — native color picker

Date: 2026-11-07. Local, unreleased development on the 1.0.21 checkout;
no version bump, commit or push. No runtime or development dependency changes.

## Automated coverage

The final check results below cover the public ColorPicker/ColorPreset API,
showcase integration and the existing suite. The component adds one parsing
unit test, eleven interaction tests and three rendering/layout tests.

Final checks passed:

- `cargo fmt --all --check`.
- `cargo test --all-features`: 421 passed; `cargo test --no-default-features`:
  314 passed. No failures or ignored tests.
- `cargo clippy --all-targets -- -D warnings`, plus the `--all-features` and
  `--no-default-features` configurations, each with warnings denied.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features`.
- `cargo check --lib --target wasm32-unknown-unknown` with all features and
  no default features, and `cargo build --example showcase --all-features`.

Focused evidence:

- Strict CSS-style RGB/RGBA hex, optional alpha, invalid Unicode and partial
  drafts, canonical copy, immediate host updates and meaningful changed flags.
- Palette, slider and stepped-number edits share the bound value; invisible
  straight RGB survives alpha zero while editing. Merely inspecting a
  premultiplied host color never reencodes it.
- Enter/Escape/Done focus, stable IDs across inserted siblings, external host
  changes, disabled parents, read-only inspection/copy, covering modals and an
  outside host action without duplicate activation.
- Reviewed new snapshots: Orange/light at 1x, Orange/dark at 1.5x and Blue/light
  at 2x. All picker rectangles are sharp and unblurred. Small/Medium/Large
  swatches are vertically centered. A 240 × 420 viewport constrains the popup
  and scrolls keyboard-focused footer actions into view.
- The showcase navigation now contains 33 pages, sorted within its groups.
  Existing tracked snapshot baselines remain unchanged.

## Native macOS review

Built and ran the showcase in a task-owned app. Palette and typed hex edits
updated the host counter, shared controls and live preview. Transparency was
tested at alpha 0, 1 and 255, retaining red across the zero-alpha edit. Read-only
typing and disabled triggers did not change the color. Escape and Done restored
trigger focus. Native accessibility exposed named ColorWells, a named dialog,
palette selection and channel values/bounds.

The final rebuilt app also verified centered previews at every trigger size,
the custom brand palette and its independent host value. Light and dark
appearances and Blue accent were inspected. The task-owned preview was closed;
unrelated applications, ignored `/media/` work and storyboard outputs were
preserved.

## Limits

No Windows/Linux native, browser runtime or screen-reader acceptance run.
WASM compilation is not browser interaction proof. Color32 retains its normal
premultiplied conversion limits; zero-alpha RGB retention is temporary editor
state. HSV/HSL, eyedropper, persistence/history and wide-gamut/HDR support remain
outside this implementation. See the [editing contract](color-picker.md) and
[component roadmap](component-roadmap.md).

## Trailing spacing follow-up — DS-057, 2026-11-08

Moved the picker chevron 8 points inward for a 16-point nominal trailing inset,
with 12 points reserved between text and the glyph. Reviewed the updated 1x,
1.5x and 2x baselines, all three trigger sizes and the narrow popup. Only the
three local color-picker baselines changed; existing tracked baselines did not.

Repeated the complete automated check list above: 421/314 tests passed with
no failures or ignored tests, all three Clippy configurations passed, and
formatting, rustdoc, native build and both WASM checks passed. This paint-only
follow-up did not repeat the earlier native interaction review. Live Foliopress
control/card observations are recorded in
[the reference notes](references/foliopress-showcase.md).

## Repository release 1.0.22 — 2026-11-08

Repeated all checks against the 1.0.22 package and lockfile: 421/314 tests,
formatting, all three Clippy configurations, rustdoc with warnings denied,
native build and both WASM checks passed. The distributable package includes
the color-picker sources, tests, guides and bundled font notices; demo-video
work and storyboard outputs are excluded. Registry publication is a separate
step from the authorized main-branch commit and push.
