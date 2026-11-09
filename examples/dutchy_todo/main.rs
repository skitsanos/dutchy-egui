//! DutchyTodo: the smallest complete Dutchy application.
//!
//! Run with `cargo run --example dutchy_todo --all-features`. Projects live in
//! a sidebar, tasks in a keyboard-operable list, and editing happens in a
//! Drawer. The application owns every value; the library draws it.
//!
//! `DUTCHY_EXAMPLE_ZOOM`, `DUTCHY_EXAMPLE_MODE` and `DUTCHY_EXAMPLE_CAPTURE`
//! drive native verification runs; see `support::capture`.

#[path = "../support/mod.rs"]
mod support;

mod actions;
mod app;
mod data;
mod editor;
mod format;
mod history;
mod io;
mod row;
mod sidebar;
mod store;
mod tasks;
#[cfg(test)]
mod tests;

fn main() -> eframe::Result {
    // A zoom override scales the initial window too, so the logical layout
    // stays comparable across display scales.
    let zoom = support::capture::zoom().unwrap_or(1.0);
    eframe::run_native(
        "DutchyTodo",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1120.0 * zoom, 760.0 * zoom])
                .with_min_inner_size([560.0, 420.0]),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| {
            if let Some(zoom) = support::capture::zoom() {
                cc.egui_ctx.set_zoom_factor(zoom);
            }
            let mut app = app::App::seeded(support::data::today());
            if let Some(mode) = support::capture::mode() {
                app.theme.mode = mode;
            }
            app.capture = support::capture::Capture::from_env();
            dutchy_egui::install(&cc.egui_ctx, app.theme.theme());
            Ok(Box::new(app))
        }),
    )
}
