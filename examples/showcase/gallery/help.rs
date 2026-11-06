use super::section;
use dutchy_egui::{
    Button, ButtonVariant, ControlSize, Field, Flex, Input, Spacing, Text, TextColor, TextSize,
    Tooltip, TooltipPosition,
};

#[derive(Default)]
pub struct HelpState {
    pub actions: u32,
    pub alias: String,
}

pub fn show(ui: &mut egui::Ui, state: &mut HelpState) {
    ui.add(Text::label("Application essentials / Tooltips").color(TextColor::Primary));
    ui.add(Text::heading("A little more clarity.").size(TextSize::Xl4));
    ui.add(Text::new("Hover or focus a control to see its hint. Escape dismisses it; leave and return to see it again.").color(TextColor::Muted));
    section(ui, "01", "Four preferred positions", |ui| {
        Flex::row("placements")
            .wrap(true)
            .gap(Spacing::Xxl)
            .show(ui, |ui| {
                for (side, label, help) in [
                    (TooltipPosition::Top, "Above", "A hint above the control"),
                    (TooltipPosition::Bottom, "Below", "A hint below the control"),
                    (TooltipPosition::Left, "Left side", "A hint to the left"),
                    (TooltipPosition::Right, "Right side", "A hint to the right"),
                ] {
                    let response = ui.add(
                        Button::new(label)
                            .variant(ButtonVariant::Outline)
                            .size(ControlSize::Small),
                    );
                    Tooltip::new(help).position(side).show(&response);
                    if response.clicked() {
                        state.actions += 1;
                    }
                }
            });
        ui.add(Text::label(format!("Example actions: {}", state.actions)).color(TextColor::Muted));
    });
    section(ui, "02", "Disabled pointer help", |ui| {
        let response = ui.add_enabled(false, Button::new("Publish workspace"));
        Tooltip::new("Publishing is available after the workspace has been saved.").show(&response);
        ui.add(
            Text::new("Save the workspace before publishing.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    });
    section(ui, "03", "Field help stays visible", |ui| {
        let response = Field::new("Workspace alias")
            .help("Use a short, memorable name.")
            .show(ui, |ui| {
                ui.add(Input::new("alias", &mut state.alias).placeholder("studio"))
            });
        Tooltip::new("This alias appears in workspace navigation.").show(&response);
        ui.add(Text::new("Hints supplement visible labels and instructions. They do not contain links or buttons.").size(TextSize::Sm).color(TextColor::Muted));
    });
}
