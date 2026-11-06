//! Range, identity, semantics, and layout contracts for progress indicators.

mod common;
use dutchy_egui::{Progress, ProgressSize, Theme};
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn ranges_are_clamped_consistently_and_invalid_maxima_have_a_finite_fallback() {
    for (value, max, expected, expected_max) in [
        (-5.0, 20.0, 0.0, 20.0),
        (50.0, 20.0, 20.0, 20.0),
        (5.0, 20.0, 5.0, 20.0),
        (f64::NAN, 20.0, 0.0, 20.0),
        (f64::INFINITY, 20.0, 20.0, 20.0),
        (f64::NEG_INFINITY, 20.0, 0.0, 20.0),
        (25.0, 0.0, 25.0, 100.0),
        (25.0, -10.0, 25.0, 100.0),
        (25.0, f64::NAN, 25.0, 100.0),
        (25.0, f64::INFINITY, 25.0, 100.0),
        (f64::MAX, f64::MAX, f64::MAX, f64::MAX),
    ] {
        let mut h = Harness::builder()
            .with_theme(egui::Theme::Light)
            .with_context_setup(|ctx| common::install(ctx, Theme::default()))
            .build_ui(move |ui| {
                ui.add(Progress::new(value).max(max).accessible_name("Upload"));
            });
        h.run();
        let node = h.get_by_role_and_label(Role::ProgressIndicator, "Upload");
        let node = node.accesskit_node();
        assert_eq!(node.numeric_value(), Some(expected));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(expected_max));
        assert_eq!(
            node.value(),
            Some(format!(
                "{:.0}% complete",
                100.0 * (expected / expected_max)
            ))
        );
    }
}

#[derive(Default)]
struct State {
    unknown: bool,
    inserted: bool,
}

#[test]
fn switching_to_unknown_clears_range_metadata_without_stealing_focus_or_identity() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let _ = ui.button("Keep focus");
                if state.inserted {
                    ui.label("Inserted sibling");
                }
                let progress = if state.unknown {
                    Progress::indeterminate()
                } else {
                    Progress::new(25.0)
                };
                ui.add(
                    progress
                        .id_salt("upload")
                        .label("Upload")
                        .show_value(true)
                        .animate(false),
                );
            },
            State::default(),
        );
    h.get_by_label("Keep focus").focus();
    h.run();
    let id = h
        .get_by_role_and_label(Role::ProgressIndicator, "Upload")
        .accesskit_node()
        .id();
    h.state_mut().unknown = true;
    h.state_mut().inserted = true;
    h.run_steps(2);
    let node = h.get_by_role_and_label(Role::ProgressIndicator, "Upload");
    assert_eq!(node.accesskit_node().id(), id);
    assert_eq!(node.accesskit_node().numeric_value(), None);
    assert_eq!(node.accesskit_node().min_numeric_value(), None);
    assert_eq!(node.accesskit_node().max_numeric_value(), None);
    assert!(node.accesskit_node().is_busy());
    assert!(h.get_by_label("Keep focus").is_focused());
    h.state_mut().unknown = false;
    h.run_steps(2);
    let node = h.get_by_role_and_label(Role::ProgressIndicator, "Upload");
    assert!(!node.accesskit_node().is_busy());
    assert_eq!(node.accesskit_node().numeric_value(), Some(25.0));
}

#[test]
fn tracks_fit_narrow_columns_and_header_text_wraps_without_changing_track_height() {
    let mut h = Harness::builder()
        .with_size([240.0, 500.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            let bounds = ui.available_rect_before_wrap();
            for size in [
                ProgressSize::Small,
                ProgressSize::Medium,
                ProgressSize::Large,
            ] {
                let response = ui.add(
                    Progress::new(2.0)
                        .max(3.0)
                        .label("A longer operation label that wraps")
                        .show_value(true)
                        .size(size)
                        .width(900.0),
                );
                assert_eq!(response.rect.height(), size.height());
                assert!(response.rect.right() <= bounds.right() + 0.1);
                assert!(response.rect.is_finite());
            }
            for width in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
                let response = ui.add(Progress::new(50.0).width(width));
                assert_eq!(response.rect.width(), 0.0);
                assert!(response.rect.is_finite());
            }
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
}
