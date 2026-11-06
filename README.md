# dutchy-egui

Dutchy Design System for egui: sharp geometry, warm colors, and bold typography.

Version **1.0.1** adds application form controls and field validation to the
theme, typography, buttons, badges, and square spinner introduced in 1.0.0.
The [coverage matrix](docs/coverage.md) tracks the remaining design system.

```toml
[dependencies]
dutchy-egui = "1.0.1"
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

## Showcase

```sh
cargo run --example showcase
```

The native showcase includes theme/motion switches, component navigation,
typography and palette pages, an application form with validation, interactive
controls, and narrow-window layouts.
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
`UPDATE_SNAPSHOTS=true cargo test --test rendering`, then rerun without that
environment variable. Generated showcase previews are under `target/visual-proof/`.

Current platform evidence and limitations are recorded in
[form verification](docs/verification-forms.md) and the earlier
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
