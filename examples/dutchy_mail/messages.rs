//! The message column: actions, search and a virtualized message list.

use crate::{
    app::{Intent, SEARCH},
    mailbox::{ARCHIVE, FolderId, Mailbox, Message, MessageId, TRASH},
    text,
};
use dutchy_egui::{
    Avatar, AvatarSize, ButtonVariant, ControlSize, EmptyState, EmptyStateIcon, EmptyStateVariant,
    Icon, List, ListItem, ListSelectionMode, ListState, SearchInput, TextColor, TextSize, Toolbar,
    ToolbarEvent, ToolbarItem, calendar::Date, tokens::space,
};
use egui::{Align, Layout, Ui, vec2};

/// Two lines of text beside a 32-point avatar.
const ROW_HEIGHT: f32 = 64.0;
/// Height of one small text line.
const LINE_HEIGHT: f32 = space::XL;

pub fn show(
    ui: &mut Ui,
    mailbox: &Mailbox,
    folder: FolderId,
    list: &mut ListState<MessageId>,
    query: &mut String,
    focus_search: &mut bool,
    today: Date,
) -> Vec<Intent> {
    let mut intents = Vec::new();
    egui::Frame::NONE
        .inner_margin(egui::Margin::same(space::LG as i8))
        .show(ui, |ui| {
            intents.extend(actions(ui, folder, !list.selected.is_empty()));
            ui.add_space(space::MD);
            let hint = ui.ctx().format_shortcut(&SEARCH);
            let search = ui.add(
                SearchInput::new("mail-search", query)
                    .accessible_name("Search messages")
                    .placeholder("Search sender or subject")
                    .key_hint(hint)
                    .size(ControlSize::Small),
            );
            if std::mem::take(focus_search) {
                search.request_focus();
            }
            ui.add_space(space::MD);
            let view = mailbox.view(folder, query);
            if view.is_empty() {
                // The empty state replaces List, so it must clear the row state.
                *list = ListState::default();
                empty(ui, mailbox, folder, query);
                return;
            }
            let items: Vec<ListItem<'_, MessageId>> = view
                .iter()
                .map(|m| ListItem::new(m.id, format!("{}, {}", m.from, m.subject)))
                .collect();
            List::new(("messages", folder.0), list, &items)
                .accessible_name("Messages")
                .selection_mode(ListSelectionMode::Multiple)
                .row_height(ROW_HEIGHT)
                .max_height(ui.available_height())
                .show_with(ui, |ui, _, row| message_row(ui, view[row.index], today));
        });
    intents
}

/// Avatar, then sender and date over the subject. Unread mail is bold.
fn message_row(ui: &mut Ui, message: &Message, today: Date) {
    ui.add(Avatar::new(message.from.as_str()).size(AvatarSize::Medium));
    ui.add_space(space::SM);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = space::MICRO;
        // Center both lines beside the avatar.
        let lines = 2.0 * LINE_HEIGHT + space::MICRO;
        ui.add_space(((ui.available_height() - lines) / 2.0).max(0.0));
        // A fixed-height first line; a bare right-to-left layout would take
        // the whole row height and push the subject out of the row.
        let first = vec2(ui.available_width(), LINE_HEIGHT);
        ui.allocate_ui_with_layout(first, Layout::right_to_left(Align::Center), |ui| {
            let date = text::date(message.date, today);
            text::line(ui, &date, TextSize::Xs, false, TextColor::Muted);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                text::line(
                    ui,
                    &message.from,
                    TextSize::Sm,
                    message.unread,
                    TextColor::Foreground,
                );
            });
        });
        let color = if message.unread {
            TextColor::Foreground
        } else {
            TextColor::Muted
        };
        text::line(ui, &message.subject, TextSize::Sm, false, color);
    });
}

fn actions(ui: &mut Ui, folder: FolderId, selected: bool) -> Option<Intent> {
    let items = [
        ToolbarItem::action(Intent::Compose, "Compose")
            .icon(Icon::Plus)
            .variant(ButtonVariant::Primary),
        ToolbarItem::action(Intent::Archive, "Archive")
            .enabled(selected && folder != ARCHIVE)
            .separator_before(true),
        ToolbarItem::action(Intent::Delete, "Delete")
            .icon(Icon::Trash)
            .enabled(selected && folder != TRASH),
        ToolbarItem::action(Intent::ToggleRead, "Read / unread").enabled(selected),
    ];
    let result = Toolbar::new("mail-actions", &items)
        .accessible_name("Message actions")
        .size(ControlSize::Small)
        .wrap(true)
        .show(ui);
    match result.event {
        Some(ToolbarEvent::Activated(intent)) => Some(intent),
        _ => None,
    }
}

fn empty(ui: &mut Ui, mailbox: &Mailbox, folder: FolderId, query: &mut String) {
    let name = mailbox
        .folder(folder)
        .map_or("This folder", |f| f.name.as_str());
    if query.trim().is_empty() {
        EmptyState::new("empty-folder", format!("{name} is empty"))
            .icon(EmptyStateIcon::Inbox)
            .description("Messages you move here will show up in this list.")
            .show(ui);
        return;
    }
    let result = EmptyState::new("no-matches", "No matching messages")
        .variant(EmptyStateVariant::Search)
        .icon(EmptyStateIcon::Search)
        .description(format!(
            "Nothing in {name} matches \u{201c}{query}\u{201d}."
        ))
        .action("Clear search")
        .show(ui);
    if result.action_clicked {
        query.clear();
    }
}
