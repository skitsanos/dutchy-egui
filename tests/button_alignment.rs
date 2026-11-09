//! Alignment keeps the label/spinner group inside a full-width native button.
mod common;
use dutchy_egui::{Button, ControlSize, SpinnerSize, Theme};
use egui::{Align, Rect, Shape};
use egui_kittest::Harness;

#[test]
fn padded_alignment_preserves_truncation_and_loading_group_position() {
    let mut h = Harness::builder()
        .with_size([400.0, 700.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, rows: &mut Vec<(Rect, Align, bool, String)>| {
                rows.clear();
                for align in [Align::Min, Align::Center, Align::Max] {
                    for loading in [false, true] {
                        let label = format!("{align:?} {loading}");
                        let rect = ui
                            .add(
                                Button::new(&label)
                                    .size(ControlSize::Small)
                                    .full_width(true)
                                    .text_align(align)
                                    .loading(loading),
                            )
                            .rect;
                        rows.push((rect, align, loading, label));
                    }
                }
                let label =
                    "A long conversation title that should truncate inside the padded control";
                let rect = ui
                    .add(
                        Button::new(label)
                            .size(ControlSize::Small)
                            .full_width(true)
                            .text_align(Align::Min),
                    )
                    .rect;
                rows.push((rect, Align::Min, false, label.into()));
            },
            Vec::new(),
        );
    h.run();
    for (rect, align, loading, label) in h.state() {
        let text = h
            .output()
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                Shape::Text(text) if text.galley.job.text == label.to_uppercase() => Some(text),
                _ => None,
            })
            .expect("label painted");
        let width = text.galley.size().x;
        let pad = ControlSize::Small.padding();
        let expected = match align {
            Align::Min => rect.left() + pad,
            Align::Center => rect.center().x - width / 2.0,
            Align::Max => rect.right() - pad - width,
        };
        assert!(
            (text.pos.x - expected).abs() < 1.0,
            "{align:?} loading={loading}: {} vs {expected}",
            text.pos.x
        );
        assert!(text.pos.x + width <= rect.right() - pad + 1.0);
        if *loading {
            let center = h
                .output()
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    Shape::Path(path) if rect.contains_rect(Rect::from_points(&path.points)) => {
                        Some(Rect::from_points(&path.points).center())
                    }
                    _ => None,
                })
                .expect("loading spinner painted inside the control");
            assert!((center.x - expected - SpinnerSize::Small.extent() / 2.0).abs() < 1.0);
        }
    }
    h.check_accessibility();
}
