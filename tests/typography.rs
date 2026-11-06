//! Text transformation, wrapping, and real-font contracts.

mod common;

#[cfg(feature = "bundled-fonts")]
use dutchy_egui::{Family, Weight};
use dutchy_egui::{Leading, Text, TextSize, Theme, Tracking};
use egui_kittest::Harness;

#[test]
fn unicode_transform_keeps_layout_ranges_valid() {
    let theme = Theme::default();
    let text = Text::new("Straße café — こんにちは")
        .uppercase(true)
        .size(TextSize::Xl)
        .tracking(Tracking::Widest)
        .leading(Leading::Relaxed);
    let job = text.layout_job(&theme, 240.0);
    job.debug_sanity_check();
    assert_eq!(job.text, "STRASSE CAFÉ — こんにちは");
    assert_eq!(job.sections[0].format.extra_letter_spacing, 2.0);
    assert_eq!(job.sections[0].format.line_height, Some(32.5));
    assert_eq!(job.wrap.max_width, 240.0);
}

#[test]
fn long_text_wraps_within_the_allocated_width() {
    let mut harness = Harness::builder().with_size([240.0, 320.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let available = ui.available_width();
            let response = ui.add(Text::new("A longer paragraph must wrap within a narrow content column without extending the window."));
            assert!(response.rect.width() <= available + 1.0);
            assert!(response.rect.height() > 24.0);
        });
    harness.run();
}

#[cfg(feature = "bundled-fonts")]
#[test]
fn static_weights_have_distinct_real_glyph_metrics() {
    let mut harness = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            for family in [Family::Body, Family::Display] {
                let widths: Vec<_> = [
                    Weight::Regular,
                    Weight::Medium,
                    Weight::SemiBold,
                    Weight::Bold,
                ]
                .into_iter()
                .map(|weight| {
                    let job = Text::new("Hamburgefontsiv")
                        .family(family)
                        .weight(weight)
                        .size(TextSize::Xl4)
                        .layout_job(&Theme::default(), f32::INFINITY);
                    ui.fonts_mut(|fonts| fonts.layout_job(job).size().x)
                })
                .collect();
                assert!(
                    widths.windows(2).all(|pair| pair[0] != pair[1]),
                    "{family:?}: {widths:?}"
                );
            }
        });
    harness.run();
}

#[cfg(feature = "bundled-fonts")]
#[test]
fn extending_fonts_retains_host_faces_and_is_idempotent() {
    let mut definitions = egui::FontDefinitions::empty();
    definitions.font_data.insert(
        "host".into(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/Body-Regular.ttf"
        ))),
    );
    definitions
        .families
        .insert(egui::FontFamily::Proportional, vec!["host".into()]);
    dutchy_egui::fonts::extend(&mut definitions);
    let once = definitions.clone();
    dutchy_egui::fonts::extend(&mut definitions);
    assert_eq!(definitions, once);
    assert_eq!(
        definitions.families[&egui::FontFamily::Proportional]
            .last()
            .map(String::as_str),
        Some("host")
    );
    let regular = dutchy_egui::fonts::font_family(Family::Body, Weight::Regular);
    let bold = dutchy_egui::fonts::font_family(Family::Body, Weight::Bold);
    assert_ne!(definitions.families[&regular], definitions.families[&bold]);
}
