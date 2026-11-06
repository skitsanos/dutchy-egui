use dutchy_egui::Theme;

pub fn install(ctx: &egui::Context, theme: Theme) {
    #[cfg(not(feature = "bundled-fonts"))]
    {
        let mut definitions = egui::FontDefinitions::empty();
        definitions.font_data.insert(
            "host-font".into(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/Body-Regular.ttf"
            ))),
        );
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            definitions
                .families
                .insert(family, vec!["host-font".into()]);
        }
        ctx.set_fonts(definitions);
    }
    dutchy_egui::install(ctx, theme);
}
