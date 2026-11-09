mod icons;
mod toggles;
mod toolbars;

use dutchy_egui::{Text, TextColor, TextSize};

#[derive(Default)]
pub struct ActionsState {
    pub actions: u32,
    pub last_icon: String,
    pub pinned: bool,
    pub preview: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub alignment: usize,
    pub saves: u32,
}

impl ActionsState {
    pub fn icons(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "Icon buttons",
            "Small targets. Clear actions.",
            "Every icon action has a descriptive name, keyboard activation and a hover/focus hint.",
        );
        icons::show(ui, self);
    }
    pub fn toggles(&mut self, ui: &mut egui::Ui) {
        heading(
            ui,
            "Toggle buttons",
            "Press. Keep. Release.",
            "Pressed state belongs to the application. A toggle keeps the same accessible name in both states.",
        );
        toggles::show(ui, self);
    }
    pub fn toolbars(
        &mut self,
        ui: &mut egui::Ui,
        commands: &mut super::command_surfaces::CommandState,
    ) {
        heading(
            ui,
            "Toolbars",
            "A place for your tools.",
            "Tab enters once. Arrows move between tools. Enter or Space performs an action; moving focus never does.",
        );
        toolbars::show(ui, self);
        super::section(ui, "04", "Menus / connected command surfaces", |ui| {
            super::command_surfaces::show(ui, commands);
        });
    }
}

fn heading(ui: &mut egui::Ui, label: &str, title: &str, description: &str) {
    ui.add(Text::label(format!("Native additions / {label}")).color(TextColor::Primary));
    ui.add(Text::heading(title).size(TextSize::Xl4));
    ui.add(Text::new(description).color(TextColor::Muted));
}
