//! Rename requests in the notebook showcase, including host rejection and reordering.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::Theme;
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn showcase() -> Harness<'static, gallery::Showcase> {
    let mut h = Harness::builder()
        .with_size([1200.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut gallery::Showcase| s.show(ui),
            gallery::Showcase {
                page: gallery::Page::InlineEdit,
                reduced_motion: true,
                ..Default::default()
            },
        );
    h.run_steps(4);
    h
}

#[test]
fn notebook_renames_and_cancelled_drafts_follow_their_record_keys() {
    let mut h = showcase();
    h.get_by_label("Rename Document 1 name").click();
    h.run_steps(3);
    h.get_by_label("Document 1 name")
        .type_text("Retained draft.md");
    h.run_steps(3);
    let id = h.get_by_label("Document 1 name").accesskit_node().id();
    h.get_by_label("Reverse order").click();
    h.run_steps(3);
    assert_eq!(h.get_by_label("Document 1 name").accesskit_node().id(), id);
    assert_eq!(
        h.get_by_label("Document 1 name")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Retained draft.md")
    );
    h.get_by_role_and_label(Role::Button, "Cancel Document 1 name")
        .click();
    h.run_steps(3);
    assert_eq!(
        h.get_by_label("Rename Document 1 name")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Design notes.md")
    );
    h.get_by_label("Rename Document 1 name").click();
    h.run_steps(3);
    h.get_by_label("Document 1 name")
        .type_text("Shipped name.md");
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Save Document 1 name")
        .click();
    h.run_steps(3);
    assert_eq!(
        h.get_by_label("Rename Document 1 name")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("Shipped name.md")
    );
    h.check_accessibility();
}

#[test]
fn the_host_rejects_blank_names_without_losing_the_proposal() {
    let mut h = showcase();
    h.get_by_label("Rename Workspace name").click();
    h.run_steps(3);
    h.get_by_label("Workspace name").type_text("   ");
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Save Workspace name")
        .click();
    h.run_steps(3);
    assert!(h.get_by_label("Workspace name").is_focused());
    assert_eq!(
        h.get_by_label("Workspace name")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("   ")
    );
    assert!(h.query_by_role(Role::Alert).is_some());
    h.get_by_label("Workspace name").type_text("New studio");
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Save Workspace name")
        .click();
    h.run_steps(3);
    assert_eq!(
        h.get_by_label("Rename Workspace name")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("New studio")
    );
    assert!(h.query_by_role(Role::Alert).is_none());
}
