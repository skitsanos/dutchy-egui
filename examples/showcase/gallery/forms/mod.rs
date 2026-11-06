mod controls;
mod state;

use super::section;
use dutchy_egui::tokens::space;
use dutchy_egui::{
    Button, ButtonVariant, ControlSize, Field, Input, Select, SelectOption, Text, TextColor,
    TextSize, validation::ValidationTrigger,
};
pub use state::FormState;

pub fn show(ui: &mut egui::Ui, state: &mut FormState) {
    ui.add(Text::label("Application essentials / Forms").color(TextColor::Primary));
    ui.add(Text::heading("Clear input. Useful feedback.").size(TextSize::Xl5));
    ui.add(Text::new("Labeled controls, deliberate states, and validation that stays under your application's control.")
        .color(TextColor::Muted));
    let triggers = [
        SelectOption::new(ValidationTrigger::OnBlur, "On blur"),
        SelectOption::new(ValidationTrigger::OnChange, "On change"),
        SelectOption::new(ValidationTrigger::OnSubmit, "On submit"),
    ];
    Field::new("Validate text fields").show(ui, |ui| {
        ui.add(Select::new("trigger", &mut state.trigger, &triggers).size(ControlSize::Small))
    });
    if ui.available_width() >= 720.0 {
        ui.columns(2, |columns| {
            section(&mut columns[0], "01", "Identity", |ui| {
                controls::identity(ui, state)
            });
            section(&mut columns[1], "02", "Preferences", |ui| {
                controls::preferences(ui, state)
            });
        });
    } else {
        section(ui, "01", "Identity", |ui| controls::identity(ui, state));
        section(ui, "02", "Preferences", |ui| {
            controls::preferences(ui, state)
        });
    }
    section(ui, "03", "Context", |ui| controls::notes(ui, state));
    actions(ui, state);
    sizes(ui);
}

fn actions(ui: &mut egui::Ui, state: &mut FormState) {
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        if ui.add(Button::new("Validate form")).clicked() {
            state.validate();
        }
        if ui
            .add(Button::new("Demo availability check").variant(ButtonVariant::Outline))
            .clicked()
        {
            state.begin_demo_check(ui.ctx());
        }
        if ui
            .add(Button::new("Reset form").variant(ButtonVariant::Ghost))
            .clicked()
        {
            *state = FormState::default();
        }
    });
    if state.submitted && state.is_valid() {
        ui.add(Text::new("All fields passed.").color(TextColor::Success));
    }
    ui.add(Text::new("The availability demo completes after one second. Edit the email while it runs to discard its old result.")
        .size(TextSize::Sm).color(TextColor::Muted));
}

fn sizes(ui: &mut egui::Ui) {
    section(ui, "04", "Shared control heights", |ui| {
        for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
            let mut text = format!("{} points / read only", size.height());
            Field::new(format!("{:?} input", size)).show(ui, |ui| {
                ui.add(
                    Input::new(size.height().to_bits(), &mut text)
                        .size(size)
                        .read_only(true),
                )
            });
        }
        let mut disabled = String::from("Unavailable in this workspace");
        Field::new("Disabled input").show(ui, |ui| {
            ui.add(Input::new("disabled", &mut disabled).enabled(false))
        });
    });
}
