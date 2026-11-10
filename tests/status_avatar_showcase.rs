//! The Status bars and Avatars showcase pages compose their components.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use egui::accesskit::{Action, ActionData, ActionRequest, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

/// One showcase page; each step advances the clock by a quarter second.
fn page(page: gallery::Page) -> Harness<'static, gallery::Showcase> {
    let mut h = Harness::builder()
        .with_size([1200.0, 1400.0])
        .with_context_setup(|ctx| {
            common::install(ctx, dutchy_egui::Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            |ui, s: &mut gallery::Showcase| s.show(ui),
            gallery::Showcase {
                page,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h
}

#[test]
fn the_sync_segment_runs_a_simulated_sync_and_reports_the_result() {
    let mut h = page(gallery::Page::StatusBars);
    h.get_by_role_and_label(Role::Button, "Sync").click();
    h.run_steps(2);
    let running = h.get_by_role_and_label(Role::ProgressIndicator, "Sync");
    assert!(running.accesskit_node().is_busy());
    h.run_steps(12);
    let sync = h.get_by_role_and_label(Role::Button, "Sync");
    assert_eq!(sync.value().as_deref(), Some("Up to date"));
    h.get_by_label_contains("1 syncs / hidden: none");
}
#[test]
fn a_failed_sync_is_stated_in_words_and_stays_retryable() {
    let mut h = page(gallery::Page::StatusBars);
    h.get_by_role_and_label(Role::Switch, "Fail next sync")
        .click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Sync").click();
    h.run_steps(14);
    let sync = h.get_by_role_and_label(Role::Button, "Sync");
    assert_eq!(sync.value().as_deref(), Some("Failed · retry"));
    h.check_accessibility();
}
#[test]
fn a_narrow_bar_hides_low_priority_segments_and_says_so() {
    let mut h = page(gallery::Page::StatusBars);
    let (target_node, target_tree) = h
        .get_by_role_and_label(Role::Slider, "Bar width")
        .accesskit_node()
        .locate();
    h.event(egui::Event::AccessKitActionRequest(ActionRequest {
        target_node,
        target_tree,
        action: Action::SetValue,
        data: Some(ActionData::NumericValue(320.0)),
    }));
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Label, "Unread: 42")
            .is_some()
    );
    assert!(h.query_by_role_and_label(Role::Label, "Compose").is_none());
    let summary = h
        .get_by_label_contains("hidden: ")
        .value()
        .unwrap_or_default();
    assert!(summary.ends_with("Storage, Commands, Compose"), "{summary}");
}
#[test]
fn directory_rows_stay_clickable_through_their_avatars() {
    let mut h = page(gallery::Page::Avatars);
    let selected = |h: &Harness<'_, gallery::Showcase>| {
        h.get_by_role_and_label(Role::ListBoxOption, "Joost de Wit")
            .accesskit_node()
            .is_selected()
    };
    assert_eq!(selected(&h), Some(false));
    // The variants section shows the same person; take the avatar in his row.
    let row = h
        .get_by_role_and_label(Role::ListBoxOption, "Joost de Wit")
        .rect();
    let avatar = h
        .get_all_by_role(Role::Image)
        .map(|n| n.rect())
        .find(|rect| row.contains_rect(*rect))
        .expect("Joost de Wit's row shows his avatar")
        .center();
    h.hover_at(avatar);
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: avatar,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.run_steps(3);
    assert_eq!(selected(&h), Some(true));
    h.check_accessibility();
}
