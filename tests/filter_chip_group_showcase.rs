//! Showcase composes single and multiple selection policies over filter requests.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::Theme;
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn showcase_combines_multiple_media_types_with_one_collection() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut gallery::Showcase| state.show(ui),
            gallery::Showcase {
                page: gallery::Page::FilterChipGroup,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h.get_by_role_and_label(Role::Group, "Media types");
    h.get_by_role_and_label(Role::Group, "Collection");
    h.get_by_label("Waterline sketches");
    assert!(
        h.get_by_role_and_label(Role::Button, "Archive")
            .accesskit_node()
            .is_disabled()
    );

    h.get_by_label("Prints").click();
    h.run_steps(2);
    h.get_by_label("Canal district").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(2);
    h.get_by_label("No assets match these filters.");

    h.get_by_label("Photography").click();
    h.run_steps(2);
    h.get_by_label("Canal at dusk");
    assert!(h.query_by_label("Studio colour proof").is_none());

    h.get_by_label("Studio work").click();
    h.run_steps(2);
    h.get_by_label("Studio colour proof");
    h.get_by_label("Light meter study");
    assert!(h.query_by_label("Canal at dusk").is_none());
    h.check_accessibility();
}
