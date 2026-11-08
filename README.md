# dutchy-egui

Dutchy Design System for egui: sharp geometry, warm colors and bold typography.
Composable native widgets with application-owned state and minimal dependencies.

## Quick start

Use a path dependency while the package is being prepared for registry publication:

```toml
[dependencies]
dutchy-egui = { path = "../dutchy-egui" }
```

```rust
use dutchy_egui::{Button, Theme, egui};

fn setup(ctx: &egui::Context) {
    dutchy_egui::install(ctx, Theme::default());
}

fn content(ui: &mut egui::Ui) {
    if ui.add(Button::new("Save changes")).clicked() {
        // Update application state.
    }
}
```

Run the native showcase:

```sh
cargo run --example showcase --all-features
```

## Documentation

[Setup, themes and features](docs/getting-started.md) ·
[Component guides](docs/index.md) · [Coverage](docs/coverage.md) ·
[Development](docs/development.md) · [Changelog](CHANGELOG.md)

MIT-licensed code. Bundled fonts retain their SIL Open Font License notices;
see [third-party notices](THIRD_PARTY.md).
