//! The bottom StatusBar: counts, a clickable sync segment and shortcut hints.

use crate::{app::App, commands::Action};
use dutchy_egui::{Icon, StatusAlign, StatusBar, StatusItem, StatusTone};
use egui::Ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Segment {
    Folder,
    Unread,
    Selected,
    Sync,
    Compose,
    Archive,
    Delete,
    Search,
}

pub fn show(ui: &mut Ui, app: &App) -> Option<Action> {
    let ctx = ui.ctx().clone();
    let folder = app
        .mailbox
        .folder(app.folder)
        .map_or_else(String::new, |f| f.name.clone());
    let sync = if app.syncing.is_some() {
        StatusItem::labeled(Segment::Sync, "Sync", "Checking mail").busy()
    } else {
        StatusItem::labeled(Segment::Sync, "Sync", "Up to date")
            .icon(Icon::Check)
            .tone(StatusTone::Success)
            .clickable(true)
            .tooltip("Check for new mail")
    };
    let items = [
        StatusItem::labeled(Segment::Folder, "Folder", folder).priority(4),
        StatusItem::labeled(
            Segment::Unread,
            "Unread",
            app.mailbox.unread(app.folder).to_string(),
        )
        .priority(3),
        StatusItem::labeled(
            Segment::Selected,
            "Selected",
            app.list.selected.len().to_string(),
        )
        .priority(1),
        sync.live(true).priority(2),
        hint(app, &ctx, Segment::Compose, Action::Compose),
        hint(app, &ctx, Segment::Archive, Action::Archive),
        hint(app, &ctx, Segment::Delete, Action::Delete),
        hint(app, &ctx, Segment::Search, Action::FocusSearch),
    ];
    let mut intent = None;
    egui::Panel::bottom("mail-status")
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            let result = StatusBar::new("mail-status-bar", &items)
                .accessible_name("Mailbox status")
                .show(ui);
            if result.clicked == Some(Segment::Sync) {
                intent = Some(Action::Sync);
            }
        });
    intent
}

fn hint<'a>(
    app: &'a App,
    ctx: &egui::Context,
    segment: Segment,
    action: Action,
) -> StatusItem<'a, Segment> {
    let registry = &app.commands.registry;
    let chord = registry.shortcut_label(ctx, &action).unwrap_or_default();
    let label = &registry
        .get(&action)
        .expect("mail command is registered")
        .label;
    StatusItem::hint(segment, chord, label.as_str()).align(StatusAlign::End)
}
