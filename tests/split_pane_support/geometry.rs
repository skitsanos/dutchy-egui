use super::support::*;
use dutchy_egui::{SplitPane, Theme};
use egui::{Rect, vec2};
use egui_kittest::Harness;

#[test]
fn both_orientations_fill_the_fixed_outer_slot_and_each_closure_runs_once() {
    for vertical in [false, true] {
        let mut h = harness(Page {
            vertical,
            ..Default::default()
        });
        h.run_steps(3);
        let p = h.state();
        let [a, divider, b] = p.rects;
        assert_eq!(p.first_calls, p.second_calls);
        assert!(p.outer.contains_rect(a) && p.outer.contains_rect(b));
        if vertical {
            assert_eq!(a.height(), 128.0);
            assert_eq!(divider.height(), 12.0);
            assert_eq!(b.height(), 100.0);
            assert_eq!(a.bottom(), divider.top());
            assert_eq!(divider.bottom(), b.top());
        } else {
            assert_eq!(a.width(), 160.0);
            assert_eq!(divider.width(), 12.0);
            assert_eq!(a.right(), divider.left());
            assert_eq!(divider.right(), b.left());
        }
        let calls = p.first_calls;
        h.run_steps(1);
        assert_eq!(h.state().first_calls, calls + 1);
        h.check_accessibility();
    }
}
#[test]
fn impossible_minima_shrink_proportionally_and_external_values_are_clamped() {
    let mut h = harness(Page {
        width: 200.0,
        minima: [100.0, 300.0],
        first: 999.0,
        ..Default::default()
    });
    h.run_steps(3);
    let [a, _, b] = h.state().rects;
    assert!((b.width() - 3.0 * a.width()).abs() < 0.001);
    assert!((h.state().first - a.width()).abs() < 0.001);
    h.state_mut().width = 560.0;
    h.state_mut().first = 50.0;
    h.run_steps(3);
    assert_eq!(h.state().first, 100.0);
    assert_eq!(h.state().changes, 2);
}
#[test]
fn invalid_values_and_zero_available_space_keep_all_geometry_finite() {
    for first in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let mut h = harness(Page {
            first,
            height: f32::NAN,
            minima: [f32::INFINITY, -20.0],
            ..Default::default()
        });
        h.run_steps(3);
        assert!(h.state().first.is_finite());
        for rect in h.state().rects {
            assert!(rect.is_finite());
        }
    }
    let mut h = harness(Page {
        vertical: true,
        width: 0.0,
        height: 0.0,
        ..Default::default()
    });
    h.run_steps(3);
    assert_eq!(h.state().first, 0.0);
    for rect in h.state().rects {
        assert!(rect.is_finite());
    }
}
#[test]
fn panes_below_the_fold_of_a_scroll_area_keep_their_requested_height() {
    // Vertical ScrollArea content gets a finite viewport height, so the space
    // left below the fold is not a limit for a scrolling page.
    for spacer in [0.0, 450.0, 700.0] {
        let mut h = Harness::builder()
            .with_size([600.0, 500.0])
            .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
            .build_ui_state(
                move |ui, p: &mut (f32, Rect)| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(spacer);
                        p.1 = SplitPane::horizontal("fold", &mut p.0)
                            .height(300.0)
                            .show(ui, |ui| ui.label("First"), |ui| ui.label("Second"))
                            .response
                            .rect;
                        ui.add_space(400.0);
                    });
                },
                (200.0, Rect::NOTHING),
            );
        h.run_steps(3);
        assert_eq!(h.state().1.height(), 300.0, "spacer {spacer}");
        assert_eq!(h.state().0, 200.0, "spacer {spacer}");
    }
}
#[test]
fn overflowing_content_is_clipped_and_does_not_move_following_controls() {
    let mut h = Harness::builder()
        .with_size([600.0, 400.0])
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, p: &mut (f32, Rect, Rect, Rect)| {
                let r = SplitPane::horizontal("clip", &mut p.0).height(200.0).show(
                    ui,
                    |ui| {
                        p.1 = ui.clip_rect();
                        ui.allocate_space(vec2(4000.0, 4000.0));
                        ui.painter().rect_filled(
                            Rect::from_min_size(ui.min_rect().min, vec2(4000.0, 4000.0)),
                            0,
                            egui::Color32::RED,
                        );
                    },
                    |ui| {
                        p.2 = ui.clip_rect();
                    },
                );
                assert!(r.response.rect.contains_rect(p.1));
                assert!(r.response.rect.contains_rect(p.2));
                p.3 = ui.button("Following control").rect;
            },
            (160.0, Rect::NOTHING, Rect::NOTHING, Rect::NOTHING),
        );
    h.run_steps(3);
    let below = h.state().3;
    h.state_mut().0 = 320.0;
    h.run_steps(3);
    assert_eq!(below, h.state().3);
    assert!(h.state().1.right() < h.state().2.left());
    let clip = h.state().1;
    assert!(h.output().shapes.iter().any(
        |s| matches!(&s.shape,egui::Shape::Rect(r) if r.fill==egui::Color32::RED)
            && s.clip_rect == clip
    ));
}
