//! Accessible names keep the host's casing; uppercase is a visual treatment only.
mod common;
use dutchy_egui::{
    Accordion, AccordionItem, AccordionState, BreadcrumbItem, Breadcrumbs, Button, Card,
    EmptyState, Tab, Tabs, Theme,
};
use egui::accesskit::Role;
use egui_kittest::{Harness, kittest::Queryable};

struct State {
    accordion: AccordionState<u8>,
    tab: Option<u8>,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 720.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                ui.add(Button::new("Save changes"));
                Tabs::new(
                    "tabs",
                    &mut state.tab,
                    &[Tab::new(1u8, "Workspace settings")],
                )
                .show(ui, |_, _| {});
                Accordion::new(
                    "faq",
                    &mut state.accordion,
                    &[AccordionItem::new(1u8, "More context")],
                )
                .show(ui, |_, _| {});
                Breadcrumbs::new(
                    "path",
                    &[
                        BreadcrumbItem::new(1u8, "Library"),
                        BreadcrumbItem::new(2u8, "Current page"),
                    ],
                )
                .show(ui);
                Card::new().show(ui, |ui| {
                    Card::title(ui, "Project card");
                });
                EmptyState::new("empty", "No results found").show(ui);
            },
            State {
                accordion: AccordionState::Single(None),
                tab: Some(1),
            },
        )
}
#[test]
fn visually_uppercased_controls_expose_original_case_names() {
    let mut h = harness();
    h.run_steps(2);
    for (role, name) in [
        (Role::Button, "Save changes"),
        (Role::Tab, "Workspace settings"),
        (Role::Button, "More context"),
        (Role::Button, "Library"),
        (Role::Label, "Current page"),
        (Role::Heading, "Project card"),
        (Role::Heading, "No results found"),
    ] {
        assert!(
            h.query_by_role_and_label(role, name).is_some(),
            "{name:?} must be exposed with its original casing"
        );
        assert!(
            h.query_by_role_and_label(role, &name.to_uppercase())
                .is_none(),
            "{name:?} must not be exposed uppercased"
        );
    }
}
