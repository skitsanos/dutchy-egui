# dutchy-egui

Dutchy Design System for egui: sharp geometry, warm colors, and bold typography.

Version **1.0.4** adds stats and empty states to the progress indicators,
skeletons, cards, alerts, form controls, typography, and theme foundation.
The [coverage matrix](docs/coverage.md) tracks the remaining design system.

```toml
[dependencies]
dutchy-egui = "1.0.4"
```

The package is prepared locally; registry publication is a separate release step.
During local development, use a path dependency instead.

```rust
use dutchy_egui::{Accent, Badge, Button, Mode, Text, Theme};

fn initialize(ctx: &egui::Context) {
    dutchy_egui::install(ctx, Theme::new(Accent::Orange, Mode::Light));
}

fn content(ui: &mut egui::Ui) {
    ui.add(Text::heading("Design with clarity."));
    if ui.add(Button::new("Save changes")).clicked() {
        // Update application state.
    }
    ui.add(Badge::new("Ready"));
}
```

Call `install` during application setup. To change colors or motion later, call
`Theme::apply`; font installation is separate and preserves host fonts. A theme
can use Orange, Purple, or Crimson accents in either Light or Dark mode.
Customize semantic colors through `theme.palette` before applying it.

```rust
use dutchy_egui::{Family, Leading, Text, TextSize, Tracking, Weight};

let title = Text::new("Architectural clarity")
    .family(Family::Display)
    .size(TextSize::Xl6)
    .weight(Weight::Bold)
    .tracking(Tracking::Tighter)
    .leading(Leading::Display)
    .uppercase(true);
```

Buttons support Primary, Secondary, Outline, Ghost, and Destructive variants,
Small/Medium/Large sizes, full width, disabled, and loading states. Badges have
eight treatments and optional dismissal; the host removes a closable badge when
its response is clicked. Spinners respect reduced motion and visibility.

The sole direct runtime crate dependency is **egui 0.37.0**. eframe and
egui_kittest are development dependencies. The library does not perform network
requests, manage application state, or require the web design system checkout.

## Forms

`Input`, `Textarea`, `Checkbox`, `RadioGroup`, and `Select` bind to host-owned
values and use explicit IDs. `Field` adds labels, required/help/error/pending
feedback and accessible metadata. Text inputs and select triggers share the
button height tokens. Read-only inputs allow selection/copy; disabled inputs
do not edit. Textarea limits/counts use Unicode scalar values.

`validation::Rule` supplies required, basic email, finite decimal number, length,
and custom predicates. `FieldValidation` tracks explicit change/blur/submit
triggers and rejects stale async tickets; the application executes async work.
See the [form API guide](docs/forms.md) for examples and boundaries.

## Cards and alerts

`Card` provides six surfaces, token padding, header/footer helpers, and an
explicitly named keyboard-accessible action surface. Child controls retain
their own actions. Dutchy and native text inherit inverse surface colors.

`Alert` supports four severities, tinted alerts or solid banners, action and
dismiss results, live metadata, and an explicit dismissal focus target. The host
owns visibility and action execution. See the [cards and alerts guide](docs/cards-alerts.md).

## Progress and skeletons

`Progress` supports determinate/custom-range and indeterminate states, three
track sizes, four fills, labels, and percentages. `Skeleton` supplies five sharp
placeholder shapes; `SkeletonGroup` gives them one named busy region and optional
card/list/content/table compositions. Motion respects reduced motion, pausing,
and clipping. Real work, completion, and content replacement remain host-owned.
See the [loading components guide](docs/progress-skeletons.md).

## Stats and empty states

`Stat` and `StatsGrid` present host-formatted KPIs with responsive surfaces,
compact rows, and explicit trend direction/sentiment. `EmptyState` provides
generic, search, table-body, and dashboard guidance with independent actions,
heading metadata, and optional polite announcements. The host owns values,
navigation, replacement content, and focus after actions.
See the [stats and empty states guide](docs/stats-empty-states.md).

## Showcase

```sh
cargo run --example showcase
```

The native showcase includes theme/motion switches, component navigation,
typography and palette pages, an application form with validation, cards, alerts,
progress, loading-to-content examples, stats, empty states, and narrow-window layouts.
Its renderer is Glow; automated rendering tests use WGPU. Keyboard focus uses
Tab, Enter, and Space. The library itself is renderer-independent.

## Fonts and features

`bundled-fonts` is enabled by default. It embeds twelve upright static faces:
Regular/Medium/SemiBold/Bold for Space Grotesk-derived display, Inter-derived
body, and JetBrains Mono-derived technical text. The derivatives are renamed
Dutchy Display, Dutchy Body, and Dutchy Mono. See [third-party notices](THIRD_PARTY.md).

Use `fonts::extend` to add them to your own FontDefinitions before calling
`Context::set_fonts`, or `fonts::install` to add them to an existing Context.
Additional host fonts/system providers supply glyphs outside the bundled
coverage. Italics use egui's italic treatment.

With `default-features = false`, no font bytes are bundled by the library. The
host supplies proportional and monospace fonts; role-specific weights then
fall back to those host families. The showcase requires bundled fonts.

## Verification

```sh
cargo fmt --all --check
cargo test --all-features
cargo test --no-default-features
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --all-targets --no-default-features -- -D warnings
cargo check --lib --target wasm32-unknown-unknown
cargo doc --no-deps --all-features
cargo package --allow-dirty
```

Rendering tests need a WGPU-capable adapter (a software Vulkan adapter works on
Linux). Review rendering baseline changes visually before accepting updates:
`UPDATE_SNAPSHOTS=true cargo test --test insights_rendering`, then rerun without that
environment variable. Generated showcase previews are under `target/visual-proof/`.

Current platform evidence and limitations are recorded in
[stats and empty states verification](docs/verification-insights.md),
[loading verification](docs/verification-loading.md),
[cards and alerts verification](docs/verification-surfaces.md),
[form verification](docs/verification-forms.md), and the earlier
[visual-proof verification](docs/verification.md). This milestone does not claim whole-system
parity or blanket contrast compliance; [DS-003](docs/issues/DS-003.md) records
inherited source-palette contrast limitations.

## Development

Follow [AGENTS.md](AGENTS.md), the [plan](docs/plan.md), and the
[Issues Ledger](docs/issues/). Keep code modular and generally below 400 lines
per file. New crate dependencies use the most recent stable releases. Prefer
small local implementations of helpers; preserve attribution when adapting
MIT-licensed code.

Versioning starts at 1.0.0. Every push to main advances the version relative to
the last pushed baseline and updates Cargo.lock and CHANGELOG.md.

MIT-licensed code. Bundled fonts retain their SIL Open Font License notices.
