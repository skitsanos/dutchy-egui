use super::ActionsState;
use crate::gallery::section;
use dutchy_egui::{ButtonVariant, ControlSize, Icon, Text, TextColor, ToggleButton};

pub fn show(ui: &mut egui::Ui, state: &mut ActionsState) {
    section(ui, "01", "Text, icon + text, icon only", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(ToggleButton::new("pin", &mut state.pinned, "Pin workspace"));
            ui.add(ToggleButton::new("preview", &mut state.preview, "Preview").icon(Icon::Play));
            ui.add(
                ToggleButton::new("bold", &mut state.bold, "Bold")
                    .icon(Icon::Bold)
                    .icon_only(true),
            );
        });
        ui.add(
            Text::label(format!(
                "Pinned: {} / Preview: {} / Bold: {}",
                state.pinned, state.preview, state.bold
            ))
            .color(TextColor::Muted),
        );
    });
    section(ui, "02", "Independent formatting toggles", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                ToggleButton::new("format-bold", &mut state.bold, "Bold")
                    .icon(Icon::Bold)
                    .size(ControlSize::Small),
            );
            ui.add(
                ToggleButton::new("format-italic", &mut state.italic, "Italic")
                    .icon(Icon::Italic)
                    .size(ControlSize::Small),
            );
            ui.add(
                ToggleButton::new("format-underline", &mut state.underline, "Underline")
                    .icon(Icon::Underline)
                    .size(ControlSize::Small),
            );
        });
        ui.add(
            Text::new("These values are shared with the Toolbars page.").color(TextColor::Muted),
        );
    });
    section(ui, "03", "Read-only and busy state", |ui| {
        let mut on = true;
        let mut off = false;
        ui.horizontal_wrapped(|ui| {
            ui.add(
                ToggleButton::new("disabled-on", &mut on, "Selected, unavailable").enabled(false),
            );
            ui.add(ToggleButton::new("disabled-off", &mut off, "Unavailable").enabled(false));
            ui.add(
                ToggleButton::new("busy", &mut on, "Applying format")
                    .icon(Icon::Bold)
                    .icon_only(true)
                    .loading(true),
            );
        });
    });
    section(ui, "04", "Secondary / visible pressed state", |ui| {
        ui.add(
            ToggleButton::new("secondary-preview", &mut state.preview, "Secondary preview")
                .icon(Icon::Play)
                .variant(ButtonVariant::Secondary),
        );
        ui.add(
            Text::new("The accent edge marks On while the inverse fill stays consistent.")
                .color(TextColor::Muted),
        );
    });
}
