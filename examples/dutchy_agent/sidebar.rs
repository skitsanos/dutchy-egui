//! Sidebar and compact conversation navigation share the same stable records.
use crate::{app::App, support::shell};
use dutchy_egui::{
    Button, ButtonVariant, ControlSize, Select, SelectOption, Text, TextColor, tokens::space,
};
use egui::Ui;

pub(crate) fn show(ui: &mut Ui, app: &mut App) {
    shell::sidebar(ui, "agent-sidebar", 264.0, |ui| {
        ui.add(Text::label("Conversations").color(TextColor::Muted));
        ui.add_space(space::MD);
        egui::ScrollArea::vertical()
            .id_salt("conversation-list")
            .show(ui, |ui| {
                let rows: Vec<_> = app
                    .chat
                    .conversations
                    .iter()
                    .map(|c| (c.id, c.title.clone(), c.active.is_some()))
                    .collect();
                for (id, title, busy) in rows {
                    ui.push_id(id, |ui| {
                        let label = if busy {
                            format!("{title} · replying")
                        } else {
                            title.clone()
                        };
                        let variant = if id == app.chat.selected {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Ghost
                        };
                        let row = ui
                            .add(
                                Button::new(label)
                                    .variant(variant)
                                    .size(ControlSize::Small)
                                    .full_width(true)
                                    .text_align(egui::Align::Min),
                            )
                            .accessible_name(format!("Open conversation: {title}"));
                        row.ctx.accesskit_node_builder(row.id, |node| {
                            node.set_selected(id == app.chat.selected);
                            if busy {
                                node.set_description("Reply running");
                            }
                        });
                        if row.clicked() {
                            app.chat.select(id);
                            app.focus_composer = true;
                        }
                    });
                }
            });
    });
}
pub(crate) fn compact(ui: &mut Ui, app: &mut App) {
    let options: Vec<_> = app
        .chat
        .conversations
        .iter()
        .map(|c| SelectOption::new(c.id, c.title.clone()).id_salt(c.id))
        .collect();
    let mut selected = Some(app.chat.selected);
    if ui
        .add(
            Select::new("compact-conversations", &mut selected, &options)
                .accessible_name("Conversation"),
        )
        .changed()
        && let Some(id) = selected
    {
        app.chat.select(id);
        app.focus_composer = true;
    }
    ui.add_space(space::SM);
}
