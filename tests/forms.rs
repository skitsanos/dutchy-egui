//! Editing, accessible feedback, keyboard selection, IDs, and control geometry.

mod common;

use dutchy_egui::{
    Checkbox, ControlSize, Field, Input, RadioGroup, Select, SelectOption, Textarea, Theme,
    validation::{FieldValidation, Rule, ValidationState, ValidationTrigger},
};
use egui::accesskit::{Invalid, Role, Toggled};
use egui::{Key, Rect};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    email: String,
    notes: String,
    password: String,
    readonly: String,
    disabled: String,
    accepted: bool,
    delivery: Option<u8>,
    region: Option<u8>,
    validation: FieldValidation,
    trigger: ValidationTrigger,
}

fn form(ui: &mut egui::Ui, state: &mut State) {
    let response = Field::new("Email")
        .required(true)
        .help("Use a work address")
        .validation(state.validation.state())
        .show(ui, |ui| ui.add(Input::new("email", &mut state.email)));
    state.validation.update(
        &response,
        &state.email,
        state.trigger,
        &[
            Rule::required("Email is required"),
            Rule::email("Enter a valid email"),
        ],
    );
    Field::new("Notes").show(ui, |ui| {
        ui.add(
            Textarea::new("notes", &mut state.notes)
                .char_limit(3)
                .show_count(true),
        )
    });
    Field::new("Password").show(ui, |ui| {
        ui.add(Input::new("password", &mut state.password).password(true))
    });
    Field::new("Read only").show(ui, |ui| {
        ui.add(Input::new("readonly", &mut state.readonly).read_only(true))
    });
    Field::new("Disabled").show(ui, |ui| {
        ui.add(Input::new("disabled", &mut state.disabled).enabled(false))
    });
    ui.add(Checkbox::new(
        "accepted",
        &mut state.accepted,
        "Accept terms",
    ));
    let delivery = [
        SelectOption::new(0, "Economy"),
        SelectOption::new(1, "Unavailable").enabled(false),
        SelectOption::new(2, "Priority"),
    ];
    Field::new("Delivery").show(ui, |ui| {
        ui.add(RadioGroup::new("delivery", &mut state.delivery, &delivery))
    });
    let regions = [
        SelectOption::new(0, "North"),
        SelectOption::new(1, "South"),
        SelectOption::new(2, "Closed").enabled(false),
    ];
    Field::new("Region").show(ui, |ui| {
        ui.add(Select::new("region", &mut state.region, &regions))
    });
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([720.0, 1200.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            form,
            State {
                readonly: "Reference".into(),
                password: "Secret".into(),
                delivery: Some(0),
                ..Default::default()
            },
        )
}

#[test]
fn blur_and_change_validation_expose_label_required_error_and_help() {
    let mut h = harness();
    h.get_by_label("Email").click();
    h.run();
    assert!(h.get_by_label("Email").is_focused());
    assert!(h.get_by_label("Email").accesskit_node().is_required());
    assert_eq!(
        h.get_by_label("Email")
            .accesskit_node()
            .description()
            .as_deref(),
        Some("Use a work address")
    );
    h.get_by_label("Email").focus();
    h.run();
    h.get_by_label("Email").type_text("wrong");
    h.run();
    assert_eq!(h.state().validation.state(), &ValidationState::Unchecked);
    h.get_by_label("Notes").focus();
    h.run();
    assert_eq!(
        h.state().validation.state(),
        &ValidationState::Invalid("Enter a valid email".into())
    );
    assert_eq!(
        h.get_by_label("Email").accesskit_node().data().invalid(),
        Some(Invalid::True)
    );
    assert!(
        h.query_by_role_and_label(Role::Alert, "Enter a valid email")
            .is_some()
    );
    h.state_mut().trigger = ValidationTrigger::OnChange;
    h.get_by_label("Email").focus();
    h.run();
    h.get_by_label("Email").type_text("@example.test");
    h.run();
    assert!(h.state().validation.is_valid());
    assert_eq!(
        h.get_by_label("Email").accesskit_node().data().invalid(),
        None
    );
}

#[test]
fn textarea_paste_uses_unicode_count_and_keeps_host_values() {
    let mut h = harness();
    // Count/edit semantics are independent of the bundled font's glyph coverage.
    h.ctx
        .set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    h.get_by_label("Notes").focus();
    h.run();
    h.event(egui::Event::Paste("é🦊ab".into()));
    h.run();
    assert_eq!(h.state().notes, "é🦊a");
    assert!(h.query_by_label("3 / 3").is_some());
    h.state_mut().notes = "Externally supplied long value".into();
    h.run_steps(2);
    assert_eq!(h.state().notes, "Externally supplied long value");
}

#[test]
fn readonly_and_disabled_inputs_do_not_edit_and_password_is_protected() {
    let mut h = harness();
    assert!(h.get_by_label("Read only").accesskit_node().is_read_only());
    h.get_by_label("Read only").focus();
    h.run();
    h.get_by_label("Read only").type_text("changed");
    h.run();
    assert_eq!(h.state().readonly, "Reference");
    assert!(h.get_by_label("Disabled").accesskit_node().is_disabled());
    h.get_by_label("Disabled").click();
    h.run();
    h.event(egui::Event::Text("ignored".into()));
    h.run();
    assert!(h.state().disabled.is_empty());
    assert_ne!(
        h.get_by_label("Password").value().as_deref(),
        Some("Secret")
    );
}

#[test]
fn checkbox_and_radio_group_have_keyboard_semantics() {
    let mut h = harness();
    h.get_by_label("Accept terms").focus();
    h.run();
    h.key_press(Key::Space);
    h.run();
    assert!(h.state().accepted);
    assert_eq!(
        h.get_by_label("Accept terms").accesskit_node().toggled(),
        Some(Toggled::True)
    );
    h.get_by_label("Economy").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(h.state().delivery, Some(2));
    assert!(h.get_by_label("Priority").is_focused());
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(h.state().delivery, Some(0));
    h.key_press(Key::End);
    h.run();
    assert_eq!(h.state().delivery, Some(2));
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_label("Region").is_focused());
}

#[test]
fn select_supports_keyboard_selection_and_escape() {
    let mut h = harness();
    h.get_by_label("Region").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    h.key_press(Key::Tab);
    h.run();
    assert!(h.get_by_role_and_label(Role::Button, "North").is_focused());
    h.key_press(Key::ArrowDown);
    h.run();
    assert!(h.get_by_role_and_label(Role::Button, "South").is_focused());
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().region, Some(1));
    assert!(h.get_by_label("Region").is_focused());
    h.key_press(Key::Enter);
    h.run();
    assert!(
        h.get_by_role_and_label(Role::Button, "Closed")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Escape);
    h.run();
    assert_eq!(h.state().region, Some(1));
    assert!(h.query_by_role_and_label(Role::Button, "North").is_none());
}

#[test]
fn editing_a_pending_value_rejects_its_async_result() {
    let mut h = harness();
    let ticket = h.state_mut().validation.begin_async();
    h.run_steps(2);
    assert!(h.get_by_label("Email").accesskit_node().is_busy());
    h.get_by_label("Email").focus();
    h.run();
    h.get_by_label("Email").type_text("new");
    h.run();
    assert!(
        !h.state_mut()
            .validation
            .finish_async(ticket, Err("Old result".into()))
    );
    assert_eq!(h.state().validation.state(), &ValidationState::Unchecked);
}

#[test]
fn text_and_select_heights_match_the_shared_size_contract() {
    let mut h = Harness::builder()
        .with_size([640.0, 600.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, rects: &mut Vec<Rect>| {
                rects.clear();
                for (index, size) in [ControlSize::Small, ControlSize::Medium, ControlSize::Large]
                    .into_iter()
                    .enumerate()
                {
                    let mut value = String::new();
                    rects.push(
                        ui.add(
                            Input::new(index, &mut value)
                                .size(size)
                                .accessible_name("Sized input"),
                        )
                        .rect,
                    );
                    let mut selected = None;
                    let options = [SelectOption::new(0, "One")];
                    rects.push(
                        ui.add(
                            Select::new(index, &mut selected, &options)
                                .size(size)
                                .accessible_name("Sized select"),
                        )
                        .rect,
                    );
                }
            },
            Vec::new(),
        );
    h.run();
    for (rect, expected) in h.state().iter().zip([32.0, 32.0, 48.0, 48.0, 56.0, 56.0]) {
        assert!(
            (rect.height() - expected).abs() <= 1.0,
            "{rect:?} vs {expected}"
        );
    }
}
