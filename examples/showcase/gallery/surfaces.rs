use super::section;
use dutchy_egui::{
    Alert, AlertVariant, Badge, Button, ButtonVariant, Card, CardPadding, CardVariant, ControlSize,
    Text, TextColor, TextSize, tokens::space,
};
use egui::Ui;

pub struct SurfacesState {
    pub opens: u32,
    pub pins: u32,
    pub retries: u32,
    pub alert_visible: bool,
    pub banner_visible: bool,
}

impl Default for SurfacesState {
    fn default() -> Self {
        Self {
            opens: 0,
            pins: 0,
            retries: 0,
            alert_visible: true,
            banner_visible: true,
        }
    }
}

pub fn cards(ui: &mut Ui, state: &mut SurfacesState) {
    ui.add(Text::label("Application essentials / cards").color(TextColor::Primary));
    ui.add(Text::heading("Content with structure.").size(TextSize::Xl4));
    ui.add(Text::new("Six surfaces. One composable foundation.").color(TextColor::Muted));
    section(ui, "01", "Surfaces", |ui| {
        if ui.available_width() >= 640.0 {
            for pair in CardVariant::ALL.chunks(2) {
                ui.columns(2, |columns| {
                    for (column, variant) in columns.iter_mut().zip(pair) {
                        sample_card(column, *variant);
                    }
                });
                ui.add_space(space::MD);
            }
        } else {
            for variant in CardVariant::ALL {
                sample_card(ui, variant);
                ui.add_space(space::MD);
            }
        }
    });
    section(ui, "02", "Interactive content", |ui| {
        let result = Card::new()
            .variant(CardVariant::Accent)
            .padding(CardPadding::Small)
            .show_interactive(ui, "project", "Open Project Phoenix", |ui| {
                Card::title(ui, "Project Phoenix");
                Card::description(
                    ui,
                    "Activate the card with Enter or Space. The Pin action has its own behavior.",
                );
                ui.add_space(space::LG);
                if ui
                    .add(
                        Button::new("Pin project")
                            .size(ControlSize::Small)
                            .variant(ButtonVariant::Outline),
                    )
                    .clicked()
                {
                    state.pins += 1;
                }
            });
        if result.response.clicked() {
            state.opens += 1;
        }
        ui.add(
            Text::label(format!(
                "Opened {} times · Pinned {} times",
                state.opens, state.pins
            ))
            .color(TextColor::Muted),
        );
        Card::new()
            .variant(CardVariant::Bordered)
            .padding(CardPadding::Small)
            .enabled(false)
            .show_interactive(ui, "archived", "Open archived workspace", |ui| {
                Card::title(ui, "Archived workspace");
                Card::description(
                    ui,
                    "Disabled cards preserve their content and cannot activate.",
                );
            });
    });
    section(ui, "03", "Header and footer", |ui| {
        Card::new().variant(CardVariant::Bordered).padding(CardPadding::Small).show(ui, |ui| {
            Card::header(ui, |ui| {
                ui.horizontal_wrapped(|ui| { Card::title(ui, "Workspace summary"); ui.add(Badge::new("Ready")); });
            });
            Card::description(ui, "Use a card to group related information. Header and footer helpers compose normal egui widgets.");
            Card::footer(ui, |ui| { ui.add(Text::label("Last updated just now").color(TextColor::Muted)); });
        });
    });
}

fn sample_card(ui: &mut Ui, variant: CardVariant) {
    Card::new()
        .variant(variant)
        .padding(CardPadding::Small)
        .show(ui, |ui| {
            Card::title(ui, variant.name());
            Card::description(
                ui,
                "Sharp boundaries, deliberate color, and room for your content.",
            );
            // Native text follows the same scoped surface foreground.
            ui.label("Native egui content");
        });
}

pub fn alerts(ui: &mut Ui, state: &mut SurfacesState) {
    ui.add(Text::label("Application essentials / alerts").color(TextColor::Primary));
    ui.add(Text::heading("Clear status. Clear next step.").size(TextSize::Xl4));
    ui.add(
        Text::new("Useful feedback, with actions controlled by your application.")
            .color(TextColor::Muted),
    );
    section(ui, "01", "Four intents", |ui| {
        for (variant, message) in [
            (
                AlertVariant::Info,
                "Your workspace uses the latest component library.",
            ),
            (
                AlertVariant::Success,
                "Your preferences have been saved successfully.",
            ),
            (
                AlertVariant::Warning,
                "Your trial ends soon. Review your workspace settings.",
            ),
            (
                AlertVariant::Error,
                "The last update could not be saved. Your local work is retained.",
            ),
        ] {
            Alert::new(variant.name(), message)
                .variant(variant)
                .show(ui);
            ui.add_space(space::SM);
        }
    });
    section(ui, "02", "Action and dismissal", |ui| {
        let restore = ui.add(
            Button::new("Restore connection alert")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small),
        );
        if restore.clicked() {
            state.alert_visible = true;
        }
        if state.alert_visible {
            let result = Alert::new(
                "connection",
                "The demonstration is offline. Retry increments the host's counter.",
            )
            .variant(AlertVariant::Error)
            .title("Connection interrupted")
            .action("Retry connection")
            .dismissible(true)
            .return_focus_to(restore.id)
            .show(ui);
            if result.action_clicked {
                state.retries += 1;
            }
            if result.dismissed {
                state.alert_visible = false;
            }
        }
        ui.add(Text::label(format!("Retry requests: {}", state.retries)).color(TextColor::Muted));
    });
    section(ui, "03", "Full-width banner", |ui| {
        let restore = ui.add(
            Button::new("Restore banner")
                .variant(ButtonVariant::Outline)
                .size(ControlSize::Small),
        );
        if restore.clicked() {
            state.banner_visible = true;
        }
        if state.banner_visible {
            let result = Alert::new("release", "Cards and alerts are ready to explore.")
                .title("New in the library")
                .banner(true)
                .dismissible(true)
                .return_focus_to(restore.id)
                .show(ui);
            if result.dismissed {
                state.banner_visible = false;
            }
        }
    });
}
