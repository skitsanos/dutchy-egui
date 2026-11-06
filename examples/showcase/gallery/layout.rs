use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, Card, CardPadding, CardVariant, Checkbox,
    ColumnWidth, Container, ContainerWidth, CrossAlign, Field, Flex, FlexDirection, Input,
    ResponsiveGrid, Spacing, Stat, StatSize, Text, TextColor, TextSize, Tooltip,
};

pub struct LayoutState {
    pub name: String,
    pub updates: bool,
    pub saves: u32,
    pub compact: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            name: "Studio workspace".into(),
            updates: true,
            saves: 0,
            compact: false,
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut LayoutState) {
    ui.add(Text::label("Application essentials / Layout").color(TextColor::Primary));
    ui.add(Text::heading("Space with purpose.").size(TextSize::Xl4));
    ui.add(
        Text::new("Shared gaps, centered content, and stable controls as columns reflow.")
            .color(TextColor::Muted),
    );
    section(ui, "01", "Responsive workspace", |ui| {
        ui.add(Checkbox::new(
            "compact",
            &mut state.compact,
            "Preview a narrow workspace",
        ));
        let max = if state.compact {
            ContainerWidth::Sm
        } else {
            ContainerWidth::Xl4
        };
        Container::new("workspace")
            .max_width(max)
            .padding(Spacing::Sm)
            .show(ui, |ui| {
                ResponsiveGrid::new("workspace-panels")
                    .columns(2)
                    .min_column_width(ColumnWidth::Form)
                    .show(
                        ui,
                        &["settings", "overview"],
                        |key| *key,
                        |ui, key| {
                            if *key == "settings" {
                                settings(ui, state);
                            } else {
                                overview(ui, state);
                            }
                        },
                    );
            });
    });
    section(ui, "02", "Wrapping toolbar", |ui| {
        Flex::row("toolbar")
            .wrap(true)
            .gap(Spacing::Lg)
            .align(CrossAlign::Center)
            .show(ui, |ui| {
                for name in ["Projects", "People", "Activity", "Settings"] {
                    let response = ui.add(Button::new(name).variant(ButtonVariant::Outline));
                    Tooltip::new(format!("Open {name} in your application")).show(&response);
                }
            });
    });
    section(ui, "03", "Native reverse direction", |ui| {
        Flex::row("reverse")
            .direction(FlexDirection::RowReverse)
            .gap(Spacing::Lg)
            .show(ui, |ui| {
                let _ = ui.add(Button::new("First / right").variant(ButtonVariant::Secondary));
                let _ = ui.add(Button::new("Second / left").variant(ButtonVariant::Ghost));
            });
    });
}

fn settings(ui: &mut egui::Ui, state: &mut LayoutState) {
    Card::new()
        .variant(CardVariant::Bordered)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            Card::title(ui, "Workspace settings");
            Flex::column("settings-fields")
                .gap(Spacing::Lg)
                .show(ui, |ui| {
                    Field::new("Workspace name")
                        .help("Your team sees this name.")
                        .show(ui, |ui| {
                            ui.add(Input::new("workspace-name", &mut state.name))
                        });
                    ui.add(Checkbox::new(
                        "updates",
                        &mut state.updates,
                        "Receive workspace updates",
                    ));
                    let response = ui.add(Button::new("Save settings"));
                    Tooltip::new("Save the current settings in this example.").show(&response);
                    if response.clicked() {
                        state.saves += 1;
                    }
                    if state.saves > 0 {
                        Alert::new("saved", "Workspace settings saved")
                            .variant(AlertVariant::Success)
                            .show(ui);
                    }
                });
        });
}

fn overview(ui: &mut egui::Ui, state: &LayoutState) {
    Card::new()
        .variant(CardVariant::Muted)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            Card::title(ui, "Workspace overview");
            Card::description(ui, &state.name);
            ui.add(Stat::new("Saves this session", state.saves.to_string()).size(StatSize::Small));
            ui.add(
                Text::new(if state.updates {
                    "Updates enabled"
                } else {
                    "Updates paused"
                })
                .size(TextSize::Sm),
            );
        });
}
