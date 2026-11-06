//! Busy-group semantics and placeholder geometry.

mod common;
use dutchy_egui::{Skeleton, SkeletonGroup, SkeletonVariant, Theme};
use egui::accesskit::{Live, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    loaded: bool,
    inserted: bool,
}

#[test]
fn loading_is_announced_once_and_host_replacement_preserves_focus() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let _ = ui.button("Keep focus");
                if state.inserted {
                    ui.label("Inserted sibling");
                }
                if state.loaded {
                    ui.label("Loaded content");
                } else {
                    SkeletonGroup::new("article", "Loading article").show(ui, |ui| {
                        ui.add(Skeleton::new().variant(SkeletonVariant::Title));
                        ui.add(Skeleton::new().count(3));
                    });
                }
            },
            State::default(),
        );
    h.get_by_label("Keep focus").focus();
    h.run();
    let group = h.get_by_role_and_label(Role::Status, "Loading article");
    assert!(group.accesskit_node().is_busy());
    assert_eq!(group.accesskit_node().live(), Live::Polite);
    assert!(
        group
            .children()
            .all(|child| child.accesskit_node().is_hidden())
    );
    let id = group.accesskit_node().id();
    h.state_mut().inserted = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Loading article").accesskit_node().id(), id);
    h.state_mut().loaded = true;
    h.run_steps(2);
    assert!(h.query_by_label("Loading article").is_none());
    assert!(h.query_by_label("Loaded content").is_some());
    assert!(h.get_by_label("Keep focus").is_focused());
}

#[test]
fn decorative_blocks_are_hidden_and_groups_can_disable_announcements() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            SkeletonGroup::new("static", "Loading preview")
                .announce(false)
                .show(ui, |ui| {
                    ui.add(Skeleton::new().variant(SkeletonVariant::Avatar));
                });
            let _ = ui.button("Next action");
        });
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(h.get_by_label("Next action").is_focused());
    let group = h.get_by_role_and_label(Role::Group, "Loading preview");
    assert!(group.accesskit_node().is_busy());
    assert_eq!(group.accesskit_node().live(), Live::Off);
    assert!(
        group
            .children()
            .all(|child| child.accesskit_node().is_hidden())
    );
}

#[test]
fn shapes_and_patterns_fit_narrow_columns_and_invalid_dimensions_are_finite() {
    let mut h = Harness::builder()
        .with_size([240.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            let before = ui.style().clone();
            let bounds = ui.available_rect_before_wrap();
            for variant in [
                SkeletonVariant::Text,
                SkeletonVariant::Title,
                SkeletonVariant::Avatar,
                SkeletonVariant::Card,
                SkeletonVariant::TableRow,
            ] {
                let response = ui.add(Skeleton::new().variant(variant).width(900.0));
                assert!(response.rect.right() <= bounds.right() + 0.1);
                assert!(response.rect.is_finite());
            }
            for width in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
                let response = ui.add(Skeleton::new().width(width).height(width));
                assert!(response.rect.is_finite());
            }
            let zero = ui.add(Skeleton::new().count(0));
            assert_eq!(zero.rect.height(), 0.0);
            let patterns = [
                SkeletonGroup::new("card", "Card").show_card(ui),
                SkeletonGroup::new("list", "List").show_list(ui, 2),
                SkeletonGroup::new("content", "Content").show_content(ui),
                SkeletonGroup::new("table", "Table").show_table(ui, 2),
            ];
            for response in patterns {
                assert!(response.rect.right() <= bounds.right() + 0.1);
            }
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
}
