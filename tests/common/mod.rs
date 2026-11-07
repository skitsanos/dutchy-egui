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

/// Every text galley painted in `shapes`, including nested shape groups.
#[allow(dead_code)]
pub fn galleys(shapes: &[egui::epaint::ClippedShape]) -> Vec<&egui::Galley> {
    fn collect<'a>(shape: &'a egui::Shape, out: &mut Vec<&'a egui::Galley>) {
        match shape {
            egui::Shape::Text(text) => out.push(&text.galley),
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| collect(shape, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for clipped in shapes {
        collect(&clipped.shape, &mut out);
    }
    out
}
