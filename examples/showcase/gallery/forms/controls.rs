use super::{
    FormState,
    state::{email_rules, name_rules, notes_rules, password_rules, role_rules},
};
use dutchy_egui::validation::ValidationTrigger;
use dutchy_egui::{Checkbox, Field, Input, RadioGroup, Select, SelectOption, Textarea};

pub(super) fn identity(ui: &mut egui::Ui, state: &mut FormState) {
    let trigger = state.trigger.unwrap_or_default();
    let response = Field::new("Full name")
        .required(true)
        .validation(state.name_check.state())
        .help("The name shown to your team.")
        .show(ui, |ui| {
            ui.add(Input::new("name", &mut state.name).placeholder("Ada Developer"))
        });
    state
        .name_check
        .update(&response, &state.name, trigger, &name_rules());
    let response = Field::new("Work email")
        .required(true)
        .validation(state.email_check.state())
        .help("Use taken@example.test to exercise the demo check.")
        .show(ui, |ui| {
            ui.add(Input::new("email", &mut state.email).placeholder("ada@example.test"))
        });
    state
        .email_check
        .update(&response, &state.email, trigger, &email_rules());
    let response = Field::new("Access phrase")
        .validation(state.password_check.state())
        .help("Optional. Eight or more characters when provided.")
        .show(ui, |ui| {
            ui.add(Input::new("phrase", &mut state.password).password(!state.reveal_password))
        });
    state
        .password_check
        .update(&response, &state.password, trigger, &password_rules());
    ui.add(Checkbox::new(
        "reveal",
        &mut state.reveal_password,
        "Show access phrase",
    ));
}

pub(super) fn preferences(ui: &mut egui::Ui, state: &mut FormState) {
    let options = [
        SelectOption::new("editor", "Editor"),
        SelectOption::new("viewer", "Viewer"),
        SelectOption::new("owner", "Owner (managed)").enabled(false),
    ];
    let response = Field::new("Workspace role")
        .required(true)
        .validation(state.role_check.state())
        .show(ui, |ui| {
            ui.add(Select::new("role", &mut state.role, &options))
        });
    state.role_check.update(
        &response,
        if state.role.is_some() { "chosen" } else { "" },
        ValidationTrigger::OnChange,
        &role_rules(),
    );
    let digests = [
        SelectOption::new(0, "Daily digest"),
        SelectOption::new(1, "Weekly digest"),
        SelectOption::new(2, "No digest"),
    ];
    Field::new("Notifications").show(ui, |ui| {
        ui.add(RadioGroup::new("digest", &mut state.digest, &digests))
    });
    ui.add(Checkbox::new(
        "updates",
        &mut state.updates,
        "Send product updates",
    ));
    Field::new("Workspace reference")
        .help("Read-only values still allow selection and copy.")
        .show(ui, |ui| {
            ui.add(Input::new("reference", &mut state.reference).read_only(true))
        });
}

pub(super) fn notes(ui: &mut egui::Ui, state: &mut FormState) {
    let response = Field::new("Team notes")
        .validation(state.notes_check.state())
        .help("Keep it direct. Character limits count Unicode scalar values.")
        .show(ui, |ui| {
            ui.add(
                Textarea::new("notes", &mut state.notes)
                    .rows(4)
                    .char_limit(280)
                    .show_count(true)
                    .placeholder("What should the team know?"),
            )
        });
    state.notes_check.update(
        &response,
        &state.notes,
        state.trigger.unwrap_or_default(),
        &notes_rules(),
    );
}
