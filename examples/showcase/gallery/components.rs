use super::{Showcase, section};
use dutchy_egui::tokens::space;
use dutchy_egui::{
    Badge, BadgeVariant, Button, ButtonVariant, ControlSize, Family, Leading, Spinner, SpinnerSize,
    Text, TextColor, TextSize, Weight,
};
use egui::Ui;

pub(super) fn overview(ui: &mut Ui, app: &mut Showcase) {
    ui.add(Text::label("Dutch-inspired. Egui-native.").color(TextColor::Primary));
    let scale = if ui.available_width() < 600.0 {
        TextSize::Xl5
    } else {
        TextSize::Xl7
    };
    ui.add(Text::heading("BOLD BY").size(scale).leading(Leading::Hero));
    ui.add(
        Text::heading("DESIGN.")
            .size(scale)
            .leading(Leading::Hero)
            .color(TextColor::Primary),
    );
    ui.add_space(space::SM);
    ui.add(Text::new("Sharp geometry, warm color, and a clear visual hierarchy.\nA composable design system for Rust applications.").color(TextColor::Muted));
    section(ui, "01", "Actions", |ui| {
        ui.horizontal_wrapped(|ui| {
            for &variant in ButtonVariant::ALL {
                if ui
                    .add(
                        Button::new(variant.name())
                            .variant(variant)
                            .size(ControlSize::Small),
                    )
                    .clicked()
                {
                    app.clicks += 1;
                }
            }
        });
        ui.add_space(space::SM);
        ui.horizontal_wrapped(|ui| {
            ui.add(
                Button::new("Working")
                    .loading(true)
                    .size(ControlSize::Small),
            );
            ui.add(
                Button::new("Unavailable")
                    .enabled(false)
                    .size(ControlSize::Small),
            );
            ui.add(
                Text::new(format!("{} actions activated", app.clicks))
                    .size(TextSize::Sm)
                    .color(TextColor::Muted),
            );
        });
    });
    section(ui, "02", "Status", |ui| {
        badge_row(ui);
    });
    section(ui, "03", "Three voices. One system.", |ui| {
        ui.add(Text::heading("Space Grotesk.").size(TextSize::Xl4));
        ui.add(Text::new(
            "Inter brings clarity to everyday interfaces. Regular, medium, semibold, bold.",
        ));
        ui.add(Text::label("JetBrains Mono / 0123456789").color(TextColor::Primary));
    });
    section(ui, "04", "Native egui compatibility", |ui| {
        ui.label("Standard egui controls inherit the Dutchy theme:");
        let label = ui.add(Text::label("Example input"));
        ui.add(
            egui::TextEdit::singleline(&mut app.native_text).desired_width(ui.available_width()),
        )
        .labelled_by(label.id);
        ui.checkbox(&mut app.native_checked, "Use a consistent visual language");
    });
}

pub(super) fn buttons(ui: &mut Ui, app: &mut Showcase) {
    ui.add(Text::heading("Actions.").size(TextSize::Xl6));
    ui.add(Text::new(
        "Five variants. Three shared control sizes. Pointer and keyboard activation.",
    ));
    for &variant in ButtonVariant::ALL {
        section(ui, "01", variant.name(), |ui| {
            ui.horizontal_wrapped(|ui| {
                for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
                    if ui
                        .add(Button::new(variant.name()).variant(variant).size(size))
                        .clicked()
                    {
                        app.clicks += 1;
                    }
                }
            });
            ui.add_space(space::MD);
            ui.horizontal_wrapped(|ui| {
                ui.add(Button::new("Disabled").variant(variant).enabled(false));
                ui.add(Button::new("Loading").variant(variant).loading(true));
            });
        });
    }
    section(ui, "02", "Full width", |ui| {
        if ui.add(Button::new("Continue").full_width(true)).clicked() {
            app.clicks += 1;
        }
        ui.add(Text::new(format!(
            "Activated {} times. Press Tab, then Enter or Space.",
            app.clicks
        )));
    });
}

pub(super) fn badges(ui: &mut Ui, app: &mut Showcase) {
    ui.add(Text::heading("Make status clear.").size(TextSize::Xl5));
    section(ui, "01", "Eight treatments", badge_row);
    section(ui, "02", "Application-owned dismissal", |ui| {
        if app.badge_visible {
            if ui
                .add(
                    Badge::new("Removable tag")
                        .variant(BadgeVariant::Primary)
                        .closable(true),
                )
                .clicked()
            {
                app.badge_visible = false;
            }
        } else if ui
            .add(Button::new("Restore tag").size(ControlSize::Small))
            .clicked()
        {
            app.badge_visible = true;
        }
        ui.add(
            Text::new("Only the close control dismisses the tag. It also works with the keyboard.")
                .size(TextSize::Sm)
                .color(TextColor::Muted),
        );
    });
}

fn badge_row(ui: &mut Ui) {
    ui.horizontal_wrapped(|ui| {
        for &variant in BadgeVariant::ALL {
            ui.add(Badge::new(variant.name()).variant(variant));
        }
    });
}

pub(super) fn motion(ui: &mut Ui, app: &mut Showcase) {
    ui.add(Text::heading("Keep it moving.").size(TextSize::Xl5));
    section(ui, "01", "The square spinner", |ui| {
        ui.horizontal(|ui| {
            for size in [SpinnerSize::Small, SpinnerSize::Medium, SpinnerSize::Large] {
                ui.add(Spinner::new().size(size).color(app.theme().palette.primary));
                ui.add_space(space::MD);
            }
            ui.add(
                Text::new("12 / 16 / 20")
                    .family(Family::Mono)
                    .weight(Weight::Medium),
            );
        });
        ui.add_space(space::LG);
        ui.add(Button::new("Processing").loading(true));
        ui.add(
            Text::new("Use Reduce motion in the toolbar for a static loading indicator.")
                .color(TextColor::Muted),
        );
    });
    section(ui, "02", "Explicitly paused", |ui| {
        ui.horizontal(|ui| {
            ui.add(Spinner::new().animate(false));
            ui.add(Text::new(
                "A paused indicator schedules no animation frames.",
            ));
        });
    });
}
