//! Deterministic example checks; no developer credentials or external network.
mod config;
mod interaction;
mod lifecycle;
mod live_protocol;
mod live_routing;
mod live_transport;
mod live_watchdog;
mod provider;
mod rendering;
mod review;
mod shell;

use crate::{app::App, config::Config};
use dutchy_egui::{Accent, Mode};
use egui_kittest::Harness;

pub(super) fn config() -> Config {
    Config::from_sources(None, |_| None)
}
pub(super) fn finish(app: &mut App) {
    for _ in 0..30 {
        app.chat.poll(100.0);
    }
    assert!(app.chat.next_due().is_none());
}

pub(super) fn harness(app: App, width: f32, mode: Mode, scale: f32) -> Harness<'static, App> {
    let mut app = app;
    app.theme.mode = mode;
    app.theme.accent = Accent::Orange;
    app.theme.reduced_motion = true;
    let theme = app.theme.theme();
    let mut h = Harness::builder()
        .with_os(egui::os::OperatingSystem::Mac)
        .with_max_steps(300)
        .with_size([width, 900.0])
        .with_pixels_per_point(scale)
        .with_render_cursor(false)
        .with_theme(if mode == Mode::Light {
            egui::Theme::Light
        } else {
            egui::Theme::Dark
        })
        .build_eframe(move |cc| {
            dutchy_egui::install(&cc.egui_ctx, theme);
            app
        });
    h.run_steps(4);
    h
}
