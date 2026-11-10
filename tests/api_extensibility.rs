//! Constructors, setters and lists that let hosts keep compiling when
//! non-exhaustive types grow (DS-105).
mod common;
use dutchy_egui::{
    Accent, AlertVariant, BadgeVariant, ButtonVariant, CardVariant, CodeBlock, CodeSpan, Icon,
    OverlayOptions, ProgressVariant, ScribbleExportOptions, StatVariant, Theme,
};
use egui::{Color32, FullOutput, Id, epaint::Shape};
use egui_kittest::Harness;

#[test]
fn overlay_options_chain_every_setting() {
    let opener = Id::unique("opener");
    let field = Id::unique("field");
    let options = OverlayOptions::default()
        .dismiss_on_escape(false)
        .dismiss_on_backdrop(false)
        .show_close_button(false)
        .initial_focus(field)
        .return_focus(opener);
    assert!(!options.dismiss_on_escape);
    assert!(!options.dismiss_on_backdrop);
    assert!(!options.show_close_button);
    assert_eq!(options.initial_focus, Some(field));
    assert_eq!(options.return_focus, Some(opener));
    let cleared = options.initial_focus(None).return_focus(None);
    assert_eq!((cleared.initial_focus, cleared.return_focus), (None, None));
}

#[test]
fn scribble_export_options_take_a_scale_and_an_optional_background() {
    let options = ScribbleExportOptions::new(2.0).background(Color32::WHITE);
    assert_eq!(options.scale, 2.0);
    assert_eq!(options.background, Some(Color32::WHITE));
    assert_eq!(options.background(None).background, None);
    assert_eq!(
        ScribbleExportOptions::new(1.0),
        ScribbleExportOptions::default()
    );
}

#[cfg(feature = "url-images")]
#[test]
fn url_image_options_chain_every_limit() {
    use dutchy_egui::image_loaders::UrlImageOptions;
    use std::time::Duration;
    let options = UrlImageOptions::default()
        .max_download_bytes(64)
        .max_pixels(100)
        .max_entries(2)
        .max_concurrent(3)
        .response_body_timeout(Duration::from_millis(40));
    assert_eq!(options.max_download_bytes, 64);
    assert_eq!(options.max_pixels, 100);
    assert_eq!(options.max_entries, 2);
    assert_eq!(options.max_concurrent, 3);
    assert_eq!(options.response_body_timeout, Duration::from_millis(40));
}

/// Painted text sections whose color is `color`.
fn sections_in(output: &FullOutput, color: Color32) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack: Vec<&Shape> = output.shapes.iter().map(|clipped| &clipped.shape).collect();
    while let Some(shape) = stack.pop() {
        match shape {
            Shape::Vec(shapes) => stack.extend(shapes),
            Shape::Text(text) => {
                let job = &text.galley.job;
                found.extend(
                    job.sections
                        .iter()
                        .filter(|section| section.format.color == color)
                        .map(|section| {
                            job.text[section.byte_range.start.0..section.byte_range.end.0]
                                .to_owned()
                        }),
                );
            }
            _ => {}
        }
    }
    found
}

#[test]
fn host_highlighter_builds_spans_that_color_the_painted_code() {
    let accent = Color32::from_rgb(201, 37, 52);
    let span = CodeSpan::new(4..7, accent);
    assert_eq!((span.range, span.color), (4..7, accent));
    let highlight = move |_: &str, _: &str| Some(vec![CodeSpan::new(4..7, accent)]);
    let mut h = Harness::builder()
        .with_size([320.0, 120.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(move |ui| {
            ui.add(
                CodeBlock::new("code", "let key = 1;")
                    .copy(false)
                    .highlighter(&highlight),
            );
        });
    h.run();
    assert_eq!(sections_in(h.output(), accent), ["key"]);
}

/// A static list with every entry distinct.
fn assert_distinct<T: std::fmt::Debug>(all: &'static [T]) {
    let names: Vec<String> = all.iter().map(|item| format!("{item:?}")).collect();
    assert!(!names.is_empty());
    for (index, name) in names.iter().enumerate() {
        assert!(!names[..index].contains(name), "{name} repeats");
    }
}

#[test]
fn all_lists_are_static_slices_without_repeats() {
    assert_distinct(Icon::ALL);
    assert_distinct(Accent::ALL);
    assert_distinct(AlertVariant::ALL);
    assert_distinct(BadgeVariant::ALL);
    assert_distinct(ButtonVariant::ALL);
    assert_distinct(CardVariant::ALL);
    assert_distinct(ProgressVariant::ALL);
    assert_distinct(StatVariant::ALL);
}
