# dutchy-egui

Dutchy Design System for egui: sharp geometry, warm colors, and bold typography.

Version **1.0.7** continues interaction components with menus, action dropdowns,
searchable selection, and command search, following controlled accordions and tabs.
The [coverage matrix](docs/coverage.md) tracks the remaining design system.

```toml
[dependencies]
dutchy-egui = "1.0.7"
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
its response is clicked. Close buttons on badges, tabs, and alerts share flat
hover/pressed feedback, pointing cursors, and keyboard focus cues. Spinners respect reduced motion and visibility.

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

## Tooltips and layout

`Tooltip` attaches concise hover/focus help to an existing response, merges its
accessible description, supports four preferred positions, and dismisses on
Escape. Native egui placement, hover timing, and focus policy still apply.
Keep essential guidance visible and controls explicitly named.

`Flex` composes native rows/columns with typed spacing, cross alignment, reverse
directions, and row wrapping. `Container` centers bounded content with shared
gutters. `ResponsiveGrid` reflows equal-width columns using host-supplied item
keys, preserving child identities across layout changes. The host owns panels,
values, and actions. See the [tooltips and layout guide](docs/tooltips-layout.md).

## Accordions and tabs

`Accordion` binds keyed items to host-owned single/multiple `AccordionState`.
Headers expose expansion and their visible regions, wrap in narrow columns, and
support Enter/Space plus Up/Down/Home/End navigation. Bodies can contain normal
controls; their data remains with the host when collapsed.

`Tabs` binds keyed `Tab` items to an `Option<T>` selection. Underline and
segmented strips scroll horizontally as needed. Left/Right/Home/End move focus;
Enter/Space select. Optional close buttons and Delete return a close request.
The host removes items and retains or discards their data. Stable panels and
focus handling support reordering and removal. See the
[accordions and tabs guide](docs/accordions-tabs.md).

## Menus and dropdowns

`Menu` renders grouped, keyed actions with shortcut hints, decorative icons,
disabled states, and destructive intent. `Dropdown` anchors these actions to a
native trigger or context target, with keyboard navigation, constrained placement,
scrolling, dismissal, and focus return.

`SearchableSelect` keeps editable queries separate from committed values.
`CommandPalette` provides native modal command search and an optional Cmd/Ctrl+K
shortcut. The host owns state and executes returned actions. See the
[menus and dropdowns guide](docs/menus-dropdowns.md).

## Showcase

```sh
cargo run --example showcase
```

The native showcase includes theme/motion switches, component navigation,
typography and palette pages, an application form with validation, cards, alerts,
progress, loading-to-content examples, stats, empty states, tooltips, and a
responsive workspace settings/dashboard composition. Accordions and tabbed
settings/documents demonstrate host-owned state and independent close requests.
Menus/Dropdowns pages demonstrate actions, context menus, searchable selection,
and a global command palette. Its renderer is Glow; automated rendering tests use WGPU. Keyboard focus uses
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
`UPDATE_SNAPSHOTS=true cargo test --test menu_rendering`, then rerun without that
environment variable. Generated showcase previews are under `target/visual-proof/`.

Current platform evidence and limitations are recorded in
[close-control verification](docs/verification-close-controls.md),
[menus and dropdowns verification](docs/verification-menus.md),
[accordions and tabs verification](docs/verification-disclosures.md),
[tooltips and layout verification](docs/verification-layout.md),
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
