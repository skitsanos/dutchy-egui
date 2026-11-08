# Getting started

Dutchy is an egui library. Applications own values, selection, visibility,
navigation and action execution. The core UI dependency is egui; the default
`markdown` feature adds its parser. eframe and egui_kittest are development-only.
No web checkout is required.

The package is prepared locally; registry publication is a separate step.
Use a path dependency while working from this checkout:

```toml
[dependencies]
dutchy-egui = { path = "../dutchy-egui" }
```

Install once during application setup, then compose widgets:

```rust
use dutchy_egui::{Accent, Button, Mode, Text, Theme, egui};

fn initialize(ctx: &egui::Context) {
    dutchy_egui::install(ctx, Theme::new(Accent::Orange, Mode::Light));
}

fn content(ui: &mut egui::Ui) {
    ui.add(Text::heading("Design with clarity."));
    if ui.add(Button::new("Save changes")).clicked() {
        // Update application state.
    }
}
```

## Themes and typography

Use `Theme::apply` for subsequent theme/motion changes. Choose Orange, Purple,
Crimson, Blue or Green (`Accent::ALL`) in Light/Dark mode; customize semantic
values through `theme.palette`. Uppercase is visual; accessible names retain
the host's casing. Text supports family, size, actual weight, tracking, leading,
color, wrapping and egui's italic treatment.

`bundled-fonts` embeds twelve upright faces: Regular/Medium/SemiBold/Bold for
Dutchy Display, Dutchy Body and Dutchy Mono, derived from Space Grotesk, Inter
and JetBrains Mono. Font licenses and renamed-face details are in
[THIRD_PARTY.md](../THIRD_PARTY.md). Use `fonts::extend` with host FontDefinitions,
or `fonts::install` on an existing Context. Host fonts/providers supply glyphs
outside bundled coverage. With `default-features = false`, the host supplies
proportional and monospace fonts and the library embeds no font bytes.

## Features and showcase

| Feature | Behavior |
| --- | --- |
| `bundled-fonts` (default) | Bundled display, body and mono faces |
| `markdown` (default) | CommonMark parsing via pulldown-cmark; CodeBlock is unconditional |
| `url-images` | ehttp/image-backed HTTP(S) PNG/JPEG loading |
| `png-export` | Image-backed PNG drawing export without HTTP/JPEG |

```sh
cargo run --example showcase --all-features
```

The showcase requires bundled fonts; its Markdown page appears when `markdown`
is enabled. Disable default features and enable only `bundled-fonts` for the
parser-free showcase. It uses Glow; rendering tests use WGPU.
The library is renderer-independent. Media pages start with original offline
artwork; select **Use Picsum photos** to request online samples, and **Reload
photos** for an explicit cache refresh. File browsing and drawing Save As are
native host integrations. Tab/Enter/Space and component-specific arrow keys
retain native egui behavior.

See the [documentation index](index.md) for component contracts and examples.
