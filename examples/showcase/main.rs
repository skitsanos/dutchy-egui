//! Run with `cargo run --example showcase`.

mod gallery;

fn main() -> eframe::Result {
    eframe::run_native(
        "Dutchy — component showcase",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 940.0])
                .with_min_inner_size([440.0, 560.0])
                .with_drag_and_drop(true),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| {
            dutchy_egui::install(&cc.egui_ctx, dutchy_egui::Theme::default());
            Ok(Box::new(gallery::Showcase::default()))
        }),
    )
}
