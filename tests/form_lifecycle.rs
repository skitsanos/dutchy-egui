//! Trigger timing and stable identities through layout changes.

mod common;
use dutchy_egui::{
    Field, Input, Select, SelectOption, Theme,
    validation::{FieldValidation, Rule, ValidationState, ValidationTrigger},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
struct State {
    value: String,
    validation: FieldValidation,
    banner: bool,
    selection: Option<u8>,
}

#[test]
fn on_submit_waits_for_the_application_and_edits_clear_the_old_result() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let response = Field::new("Email")
                    .validation(state.validation.state())
                    .show(ui, |ui| ui.add(Input::new("email", &mut state.value)));
                state.validation.update(
                    &response,
                    &state.value,
                    ValidationTrigger::OnSubmit,
                    &[Rule::email("Email error")],
                );
                let _ = ui.button("Other action");
            },
            State::default(),
        );
    h.get_by_label("Email").focus();
    h.run();
    h.get_by_label("Email").type_text("wrong");
    h.run();
    h.get_by_label("Other action").focus();
    h.run();
    assert_eq!(h.state().validation.state(), &ValidationState::Unchecked);
    let state = h.state_mut();
    state
        .validation
        .validate(&state.value, &[Rule::email("Email error")]);
    h.run_steps(2);
    assert_eq!(
        h.state().validation.state(),
        &ValidationState::Invalid("Email error".into())
    );
    h.get_by_label("Email").focus();
    h.run();
    h.get_by_label("Email").type_text("@example.test");
    h.run();
    assert_eq!(h.state().validation.state(), &ValidationState::Unchecked);
}

#[test]
fn rules_run_for_edits_instead_of_each_render_frame() {
    let calls = Rc::new(Cell::new(0));
    let captured = calls.clone();
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            move |ui, state: &mut State| {
                let response =
                    Field::new("Name").show(ui, |ui| ui.add(Input::new("name", &mut state.value)));
                let rules = [Rule::custom(
                    |_| {
                        captured.set(captured.get() + 1);
                        true
                    },
                    "Error",
                )];
                state.validation.update(
                    &response,
                    &state.value,
                    ValidationTrigger::OnChange,
                    &rules,
                );
            },
            State::default(),
        );
    h.get_by_label("Name").focus();
    h.run();
    assert_eq!(calls.get(), 0);
    h.get_by_label("Name").type_text("Ada");
    h.run();
    assert_eq!(calls.get(), 1);
    h.run_steps(5);
    assert_eq!(calls.get(), 1);
}

#[test]
fn labels_focus_editors_and_stable_ids_survive_inserted_siblings() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.banner {
                    ui.label("An extra message before the controls");
                }
                Field::new("Account name")
                    .show(ui, |ui| ui.add(Input::new("name", &mut state.value)));
                let options = [SelectOption::new(0, "One")];
                ui.add(
                    Select::new("choice", &mut state.selection, &options).accessible_name("Choice"),
                );
            },
            State::default(),
        );
    h.get_by_role_and_label(egui::accesskit::Role::Label, "ACCOUNT NAME")
        .click();
    h.run();
    assert!(h.get_by_label("Account name").is_focused());
    let editor_id = h.get_by_label("Account name").accesskit_node().id();
    let select_id = h.get_by_label("Choice").accesskit_node().id();
    h.state_mut().banner = true;
    h.run_steps(2);
    assert_eq!(
        h.get_by_label("Account name").accesskit_node().id(),
        editor_id
    );
    assert_eq!(h.get_by_label("Choice").accesskit_node().id(), select_id);
    assert!(h.get_by_label("Account name").is_focused());
}
