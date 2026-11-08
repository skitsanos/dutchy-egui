use super::ActionsState;
use crate::gallery::section;
use dutchy_egui::{
    ButtonVariant, Card, CardVariant, ControlSize, Icon, IconButton, Text, TextColor,
};
use egui::{Color32, Painter, Rect, Shape, Stroke, pos2};

pub fn show(ui: &mut egui::Ui, state: &mut ActionsState) {
    section(
        ui,
        "01",
        "Original vector icons / no icon dependency",
        |ui| {
            ui.horizontal_wrapped(|ui| {
                for icon in Icon::ALL {
                    if ui
                        .add(IconButton::new(
                            icon.name(),
                            format!("Use {} tool", icon.name()),
                            icon,
                        ))
                        .clicked()
                    {
                        state.actions += 1;
                        state.last_icon = icon.name().into();
                    }
                }
                if ui
                    .add(IconButton::new(
                        "custom",
                        "Use diamond tool",
                        Icon::Custom(diamond),
                    ))
                    .clicked()
                {
                    state.actions += 1;
                    state.last_icon = "Custom diamond".into();
                }
            });
            ui.add(
                Text::label(format!(
                    "Actions: {} / Last: {}",
                    state.actions,
                    if state.last_icon.is_empty() {
                        "None"
                    } else {
                        &state.last_icon
                    }
                ))
                .color(TextColor::Muted),
            );
        },
    );
    section(ui, "02", "Shared variants and control sizes", |ui| {
        for variant in ButtonVariant::ALL {
            ui.horizontal_wrapped(|ui| {
                for (index, size) in [ControlSize::Small, ControlSize::Medium, ControlSize::Large]
                    .into_iter()
                    .enumerate()
                {
                    if ui
                        .add(
                            IconButton::new(
                                (variant.name(), index),
                                format!("{} add {}", variant.name(), index + 1),
                                Icon::Plus,
                            )
                            .variant(variant)
                            .size(size),
                        )
                        .clicked()
                    {
                        state.actions += 1;
                    }
                }
                ui.add(Text::label(variant.name()));
            });
        }
    });
    section(ui, "03", "Busy, disabled and inverse surfaces", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(IconButton::new("busy-save", "Saving document", Icon::Save).loading(true));
            ui.add(
                IconButton::new("disabled-trash", "Delete document", Icon::Trash).enabled(false),
            );
            ui.add(
                Text::new("Busy and disabled controls cannot activate.").color(TextColor::Muted),
            );
        });
        Card::new().variant(CardVariant::Dark).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(IconButton::new(
                        "inverse-download",
                        "Export document",
                        Icon::Download,
                    ))
                    .clicked()
                {
                    state.actions += 1;
                }
                ui.add(Text::new("Transparent actions inherit the surface ink."));
            });
        });
    });
}

// Host painters receive semantic ink and a clip constrained to their icon box.
fn diamond(painter: &Painter, rect: Rect, color: Color32) {
    let r = rect.shrink(2.0);
    painter.add(Shape::closed_line(
        vec![
            pos2(r.center().x, r.top()),
            pos2(r.right(), r.center().y),
            pos2(r.center().x, r.bottom()),
            pos2(r.left(), r.center().y),
        ],
        Stroke::new(dutchy_egui::tokens::border::STANDARD, color),
    ));
}
