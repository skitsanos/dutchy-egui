//! Showcase applies Chip requests to filters and tags and restores keyboard focus.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::Theme;
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn status_filters_and_tag_removal_change_only_their_own_host_state() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut gallery::Showcase| state.show(ui),
            gallery::Showcase {
                page: gallery::Page::Chips,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h.get_by_label("Color studies");
    h.get_by_role_and_label(Role::Button, "In review").click();
    h.run_steps(2);
    h.get_by_label("Type specimens");
    assert!(h.query_by_label("Color studies").is_none());
    assert!(h.query_by_label("Canal district guide").is_none());

    h.get_by_role_and_label(Role::Label, "Layout").click();
    h.run_steps(2);
    h.get_by_label("Remove Layout").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert!(h.query_by_label("Remove Layout").is_none());
    assert!(h.get_by_label("Restore tags").is_focused());
    h.get_by_label("Remove Icons");
    h.get_by_label("Type specimens");
    assert!(h.query_by_label("Color studies").is_none());
    h.key_press(Key::Enter);
    h.run_steps(3);
    h.get_by_label("Remove Layout");
    h.check_accessibility();
}
