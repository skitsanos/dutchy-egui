//! The reading pane: one message with its sender, or guidance otherwise.

use crate::{
    commands::Action,
    mailbox::{Mailbox, Message, MessageId},
    text,
};
use dutchy_egui::{
    Avatar, AvatarSize, EmptyState, EmptyStateIcon, Markdown, Text, TextColor, TextSize,
    calendar::Date, tokens::space,
};
use egui::Ui;

pub fn show(ui: &mut Ui, mailbox: &Mailbox, selected: &[MessageId], today: Date) -> Option<Action> {
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(space::XXL as i8))
        .show(ui, |ui| {
            let selected: Vec<&Message> = selected
                .iter()
                .filter_map(|id| mailbox.message(*id))
                .collect();
            match selected.as_slice() {
                [] => {
                    let _ = EmptyState::new("no-message", "No message selected")
                        .icon(EmptyStateIcon::Inbox)
                        .description("Choose a message to read it here.")
                        .show(ui);
                    None
                }
                [message] => {
                    message_view(ui, message, today);
                    None
                }
                many => several(ui, many.len()),
            }
        })
        .inner
}

fn message_view(ui: &mut Ui, message: &Message, today: Date) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = space::MD;
        ui.add(Avatar::new(message.from.as_str()).size(AvatarSize::Large));
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = space::MICRO;
            text::line(
                ui,
                &message.from,
                TextSize::Base,
                true,
                TextColor::Foreground,
            );
            text::line(ui, &message.address, TextSize::Sm, false, TextColor::Muted);
            text::line(
                ui,
                &text::date(message.date, today),
                TextSize::Xs,
                false,
                TextColor::Muted,
            );
        });
    });
    ui.add_space(space::LG);
    ui.add(Text::heading(message.subject.as_str()).size(TextSize::Xl2));
    ui.add_space(space::LG);
    egui::ScrollArea::vertical()
        .id_salt(("message-body", message.id.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            Markdown::new(("message", message.id.0), &message.body)
                .accessible_name("Message body")
                .show(ui);
        });
}

fn several(ui: &mut Ui, count: usize) -> Option<Action> {
    let result = EmptyState::new("several", format!("{count} messages selected"))
        .icon(EmptyStateIcon::Inbox)
        .description("Archive or delete them together, or choose one to read it.")
        .action("Archive")
        .secondary_action("Delete")
        .show(ui);
    if result.action_clicked {
        Some(Action::Archive)
    } else if result.secondary_clicked {
        Some(Action::Delete)
    } else {
        None
    }
}
