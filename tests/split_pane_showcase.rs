//! Notebook drafts and navigation compose with nested SplitPane controls.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn both_notebook_dividers_accept_pointer_drags_beside_scroll_areas() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| {
            common::install(ctx, dutchy_egui::Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            |ui, s: &mut gallery::Showcase| s.show(ui),
            gallery::Showcase {
                page: gallery::Page::SplitPanes,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    for (name, delta) in [
        ("Notebook", egui::vec2(50.0, 0.0)),
        ("Document preview", egui::vec2(0.0, 30.0)),
    ] {
        let divider = h.get_by_role_and_label(Role::Splitter, name);
        let start = divider.rect().center();
        let before = divider.accesskit_node().numeric_value().unwrap();
        h.hover_at(start);
        h.run_steps(2);
        h.drag_at(start);
        h.run_steps(2);
        h.hover_at(start + delta);
        h.run_steps(3);
        h.drop_at(start + delta);
        h.run_steps(3);
        assert_eq!(
            h.get_by_role_and_label(Role::Splitter, name)
                .accesskit_node()
                .numeric_value(),
            Some(before + f64::from(delta.x + delta.y)),
            "{name}"
        );
    }
}

#[test]
fn switching_documents_and_resizing_preserve_notebook_drafts() {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| {
            common::install(ctx, dutchy_egui::Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            |ui, s: &mut gallery::Showcase| s.show(ui),
            gallery::Showcase {
                page: gallery::Page::SplitPanes,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h.get_by_label("Note source").focus();
    h.run_steps(3);
    h.get_by_label("Note source").type_text("Retained draft. ");
    h.run_steps(3);
    assert!(
        h.get_by_label("Note source")
            .accesskit_node()
            .value()
            .unwrap()
            .contains("Retained draft.")
    );
    h.get_by_role_and_label(Role::TreeItem, "Release checklist.md")
        .click();
    h.run_steps(4);
    assert!(
        h.get_by_label("Note source")
            .accesskit_node()
            .value()
            .unwrap()
            .contains("Ready to ship")
    );
    h.get_by_role_and_label(Role::TreeItem, "Design notes.md")
        .click();
    h.run_steps(4);
    assert!(
        h.get_by_label("Note source")
            .accesskit_node()
            .value()
            .unwrap()
            .contains("Retained draft.")
    );
    let divider = h.get_by_role_and_label(Role::Splitter, "Notebook");
    let before = divider.accesskit_node().numeric_value().unwrap();
    divider.focus();
    h.run_steps(3);
    h.key_press(Key::ArrowRight);
    h.run_steps(3);
    assert_eq!(
        h.get_by_role_and_label(Role::Splitter, "Notebook")
            .accesskit_node()
            .numeric_value(),
        Some(before + 8.0)
    );
    assert!(
        h.get_by_label("Note source")
            .accesskit_node()
            .value()
            .unwrap()
            .contains("Retained draft.")
    );
    h.check_accessibility();
}
