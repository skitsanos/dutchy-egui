//! Tests run by `cargo test --all-features` through the `[[example]]` entry:
//! store and format units, keyboard flows and two image baselines.

mod commands;
mod format;
mod imports;
mod keyboard;
mod rendering;
mod store;

use crate::app::App;
use crate::support::data::FIXED_TODAY;
use dutchy_egui::{Accent, Mode};
use egui_kittest::Harness;

/// The seeded application at desktop size with motion disabled.
fn harness() -> Harness<'static, App> {
    harness_with(App::seeded(FIXED_TODAY), Accent::Orange, Mode::Light, 1.0)
}

fn harness_with(mut app: App, accent: Accent, mode: Mode, scale: f32) -> Harness<'static, App> {
    app.theme.accent = accent;
    app.theme.mode = mode;
    app.theme.reduced_motion = true;
    let theme = app.theme.theme();
    let mut h = Harness::builder()
        .with_os(egui::os::OperatingSystem::Mac)
        .with_size([1120.0, 900.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(match mode {
            Mode::Light => egui::Theme::Light,
            Mode::Dark => egui::Theme::Dark,
        })
        .build_eframe(move |cc| {
            dutchy_egui::install(&cc.egui_ctx, theme);
            app
        });
    h.run();
    h
}
