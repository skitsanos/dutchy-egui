//! Showcase applies accepted requests and shows a host validation error.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;

use dutchy_egui::Theme;
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn showcase_adds_removes_and_validates_project_tags() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut gallery::Showcase| state.show(ui),
            gallery::Showcase {
                page: gallery::Page::TagInput,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    let draft = h.get_by_role_and_label(Role::TextInput, "Project tags");
    draft.click();
    draft.type_text("  Color  ");
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Remove Color");
    let draft = h.get_by_role_and_label(Role::TextInput, "Project tags");
    assert_eq!(draft.accesskit_node().value().as_deref(), Some(""));
    assert!(draft.is_focused());
    h.key_press(Key::Backspace);
    h.run_steps(3);
    assert!(h.query_by_label("Remove Color").is_none());

    h.get_by_role_and_label(Role::TextInput, "Project tags")
        .type_text("layout");
    h.run_steps(2);
    h.key_press(Key::Enter);
    h.run_steps(3);
    h.get_by_role_and_label(Role::Alert, "This tag already exists.");
    assert_eq!(
        h.get_by_role_and_label(Role::TextInput, "Project tags")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("layout")
    );
    assert!(h.query_by_label("Remove layout").is_none());
    assert!(h.query_by_label("Remove Design system").is_none());

    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Remove Layout").is_focused());
    h.key_press(Key::Space);
    h.run_steps(3);
    assert!(h.query_by_label("Remove Layout").is_none());
    assert!(h.query_by_label("This tag already exists.").is_none());
    let draft = h.get_by_role_and_label(Role::TextInput, "Project tags");
    assert!(draft.is_focused());
    assert_eq!(draft.accesskit_node().value().as_deref(), Some("layout"));
    h.key_press(Key::Enter);
    h.run_steps(3);
    h.get_by_label("Remove layout");
    h.check_accessibility();
}
