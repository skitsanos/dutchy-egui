//! DutchyAgent: a chat application with an offline mock and an optional OpenAI
//! provider, exercising Markdown and CodeBlock.
mod app;
mod chat;
mod composer;
mod config;
mod conversation;
mod header;
mod provider;
mod settings;
mod sidebar;
#[path = "../support/mod.rs"]
mod support;
#[cfg(test)]
mod tests;

fn main() -> eframe::Result {
    let zoom = support::capture::zoom().unwrap_or(1.0);
    eframe::run_native(
        "DutchyAgent",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1160.0 * zoom, 820.0 * zoom])
                .with_min_inner_size([480.0, 540.0]),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| {
            if let Some(zoom) = support::capture::zoom() {
                cc.egui_ctx.set_zoom_factor(zoom);
            }
            let mut app = app::App::new(config::Config::load());
            if let Some(mode) = support::capture::mode() {
                app.theme.mode = mode;
            }
            app.capture = support::capture::Capture::from_env();
            dutchy_egui::install(&cc.egui_ctx, app.theme.theme());
            Ok(Box::new(app))
        }),
    )
}
