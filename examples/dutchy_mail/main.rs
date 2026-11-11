//! DutchyMail: a three-pane mail client over a fictional in-memory mailbox.
//!
//! Run with `cargo run --example dutchy_mail --all-features`. Folders are a
//! TreeView, messages a virtualized List, the reading pane renders Markdown,
//! and a StatusBar closes the window. The application owns every value.
//!
//! `DUTCHY_EXAMPLE_ZOOM`, `DUTCHY_EXAMPLE_MODE` and `DUTCHY_EXAMPLE_CAPTURE`
//! drive native verification runs; see `support::capture`.
//! Set `DUTCHY_MAIL_ICONS=phosphor` to use the bundled toolbar icon font.

#[path = "../support/mod.rs"]
mod support;

mod app;
mod commands;
mod compose;
mod context_menu;
mod data;
mod folders;
mod icons;
mod mailbox;
mod messages;
mod phosphor;
mod reader;
mod status;
#[cfg(test)]
mod tests;
mod text;

fn main() -> eframe::Result {
    let zoom = support::capture::zoom().unwrap_or(1.0);
    eframe::run_native(
        "DutchyMail",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0 * zoom, 800.0 * zoom])
                .with_min_inner_size([960.0, 560.0]),
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
            if std::env::var("DUTCHY_MAIL_ICONS").as_deref() == Ok("phosphor") {
                app.commands.use_phosphor_icons(&cc.egui_ctx);
            }
            Ok(Box::new(app))
        }),
    )
}
