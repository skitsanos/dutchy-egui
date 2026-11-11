//! Application state and the frame. Panes only read the mailbox and return
//! intents; the frame applies them afterwards, so the nested SplitPane
//! closures never need the mailbox mutably at the same time.

use crate::{
    compose::{self, Compose},
    data, folders,
    mailbox::{ARCHIVE, FolderId, INBOX, Mailbox, MessageId, Moved, TRASH},
    messages, reader, status,
    support::{capture::Capture, shell, theme::ThemeSettings},
};
use dutchy_egui::{
    Flex, ListState, SplitPane, Text, TextColor, TextSize, Toast, ToastEvent, ToastQueue, Toasts,
    TreeState, calendar::Date, tokens::space,
};
use egui::{Context, Key, KeyboardShortcut, Modifiers, Ui};

/// One requested change, applied after the panes are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intent {
    Compose,
    Archive,
    Delete,
    ToggleRead,
    Sync,
    FocusSearch,
}

/// What a notification's Undo button puts back.
#[derive(Clone, Debug)]
pub enum Undo {
    Move(Moved),
    Discard(compose::Draft),
}

/// Seconds a simulated sync takes.
pub const SYNC_SECONDS: f64 = 1.5;

pub const COMPOSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const ARCHIVE_KEY: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::E);
pub const SEARCH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);

pub struct App {
    pub mailbox: Mailbox,
    pub today: Date,
    pub theme: ThemeSettings,
    /// Folder shown in the message list.
    pub folder: FolderId,
    pub folders: TreeState<FolderId>,
    pub list: ListState<MessageId>,
    pub query: String,
    /// Folder column width in points.
    pub folders_width: f32,
    /// Message column width in points.
    pub list_width: f32,
    pub toasts: ToastQueue<Undo>,
    pub compose: Compose,
    /// When a running sync started, in egui seconds.
    pub syncing: Option<f64>,
    /// Messages delivered by sync so far.
    pub delivered: usize,
    /// The message last opened in the reader, so marking it unread sticks.
    pub opened: Option<MessageId>,
    pub focus_search: bool,
    pub capture: Capture,
}

impl App {
    pub fn seeded(today: Date) -> Self {
        let mut folders = TreeState::default();
        folders.rows.selected.insert(INBOX);
        folders.rows.focused = Some(INBOX);
        folders.expanded.insert(data::PROJECTS);
        Self {
            mailbox: data::seeded(today),
            today,
            theme: ThemeSettings::default(),
            folder: INBOX,
            folders,
            list: ListState::default(),
            query: String::new(),
            folders_width: 264.0,
            list_width: 420.0,
            toasts: ToastQueue::new(),
            compose: Compose::default(),
            syncing: None,
            delivered: 0,
            opened: None,
            focus_search: false,
            capture: Capture::default(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let mut intents = self.shortcuts(&ctx);
        self.finish_sync(&ctx);
        self.header(ui);
        intents.extend(status::show(ui, self));
        egui::CentralPanel::default()
            .frame(shell::frame(ui, 0))
            .show(ui, |ui| intents.extend(self.panes(ui)));
        self.follow_folder();
        self.mark_opened_read();
        for intent in intents {
            self.apply(intent, &ctx);
        }
        compose::show(self, &ctx);
        self.notifications(&ctx);
        self.capture.tick(&ctx);
    }

    /// Selected messages that still exist in the current folder view.
    pub fn selection(&self) -> Vec<MessageId> {
        let mut ids: Vec<MessageId> = self.list.selected.iter().copied().collect();
        ids.sort();
        ids
    }

    fn panes(&mut self, ui: &mut Ui) -> Vec<Intent> {
        let Self {
            mailbox,
            folder,
            folders,
            list,
            query,
            folders_width,
            list_width,
            focus_search,
            today,
            ..
        } = self;
        let mailbox: &Mailbox = mailbox;
        // The reader shows the selection the list had when the frame began;
        // a change repaints once more so both panes agree.
        let reading: Vec<MessageId> = list.selected.iter().copied().collect();
        let height = ui.available_height();
        let outer = SplitPane::horizontal("mail-folders", folders_width)
            .accessible_name("Folders")
            .min_sizes(160.0, 520.0)
            .height(height)
            .show(
                ui,
                |ui| folders::show(ui, mailbox, folders),
                |ui| {
                    SplitPane::horizontal("mail-reader", list_width)
                        .accessible_name("Messages")
                        .min_sizes(280.0, 240.0)
                        .height(ui.available_height())
                        .show(
                            ui,
                            |ui| {
                                messages::show(
                                    ui,
                                    mailbox,
                                    *folder,
                                    list,
                                    query,
                                    focus_search,
                                    *today,
                                )
                            },
                            |ui| reader::show(ui, mailbox, &reading, *today),
                        )
                },
            );
        if list.selected.len() != reading.len()
            || reading.iter().any(|id| !list.selected.contains(id))
        {
            ui.ctx().request_repaint();
        }
        let inner = outer.second.inner;
        inner
            .first
            .inner
            .into_iter()
            .chain(inner.second.inner)
            .collect()
    }

    /// Follow the folder tree's single selection.
    fn follow_folder(&mut self) {
        let selected = self.folders.rows.selected.iter().next().copied();
        if let Some(folder) = selected
            && folder != self.folder
        {
            self.folder = folder;
            self.list.selected.clear();
            self.list.focused = None;
            self.opened = None;
        }
    }

    /// Opening one message marks it read once; Mark unread then sticks.
    fn mark_opened_read(&mut self) {
        let single = (self.list.selected.len() == 1)
            .then(|| self.list.selected.iter().next().copied())
            .flatten();
        if single != self.opened {
            self.opened = single;
            if let Some(id) = single {
                self.mailbox.set_unread(id, false);
            }
        }
    }

    fn apply(&mut self, intent: Intent, ctx: &Context) {
        match intent {
            Intent::Compose => self.compose.open_new(),
            Intent::Archive if self.folder != ARCHIVE => self.move_selection(ARCHIVE, "Archived"),
            Intent::Delete if self.folder != TRASH => self.move_selection(TRASH, "Moved to Trash"),
            Intent::ToggleRead => {
                let ids = self.selection();
                let unread = ids
                    .iter()
                    .any(|id| self.mailbox.message(*id).is_some_and(|m| !m.unread));
                for id in ids {
                    self.mailbox.set_unread(id, unread);
                }
            }
            Intent::Sync if self.syncing.is_none() => {
                self.syncing = Some(ctx.input(|i| i.time));
                ctx.request_repaint();
            }
            Intent::FocusSearch => self.focus_search = true,
            _ => {}
        }
    }

    fn move_selection(&mut self, to: FolderId, verb: &str) {
        let moved = self.mailbox.move_to(&self.selection(), to);
        if moved.ids.is_empty() {
            return;
        }
        let count = moved.ids.len();
        self.list.selected.clear();
        let noun = if count == 1 { "message" } else { "messages" };
        self.toasts
            .push(Toast::new(format!("{verb} {count} {noun}.")).action("Undo", Undo::Move(moved)));
    }

    fn finish_sync(&mut self, ctx: &Context) {
        let Some(started) = self.syncing else { return };
        if ctx.input(|i| i.time) - started < SYNC_SECONDS {
            ctx.request_repaint();
            return;
        }
        self.syncing = None;
        self.delivered += 1;
        let message = data::arrival(self.today, self.delivered);
        let subject = message.subject.clone();
        self.mailbox.add(message);
        self.toasts
            .push(Toast::new(format!("New message: {subject}")));
    }

    /// Command chords, so plain letters stay with List type-ahead.
    fn shortcuts(&mut self, ctx: &Context) -> Vec<Intent> {
        if self.compose.open {
            return Vec::new();
        }
        let mut intents = Vec::new();
        ctx.input_mut(|input| {
            for (shortcut, intent) in [
                (COMPOSE, Intent::Compose),
                (ARCHIVE_KEY, Intent::Archive),
                (SEARCH, Intent::FocusSearch),
            ] {
                if input.consume_shortcut(&shortcut) {
                    intents.push(intent);
                }
            }
        });
        // Delete edits text while a field has focus; otherwise it moves mail.
        let typing = ctx.text_edit_focused();
        if !typing && ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Delete)) {
            intents.push(Intent::Delete);
        }
        intents
    }

    fn header(&mut self, ui: &mut Ui) {
        shell::header(ui, "mail-header", |ui| {
            Flex::row("brand").wrap(true).show(ui, |ui| {
                ui.add(Text::heading("DutchyMail").size(TextSize::Xl2));
                ui.add_space(space::LG);
                ui.add(Text::label("Example application").color(TextColor::Muted));
            });
            ui.add_space(space::SM);
            self.theme.controls(ui);
        });
    }

    fn notifications(&mut self, ctx: &Context) {
        let result = Toasts::new("dutchy-mail-toasts", &mut self.toasts).show(ctx);
        for event in result.events {
            if let ToastEvent::Action { action, .. } = event {
                match action {
                    Undo::Move(moved) => self.mailbox.restore(&moved),
                    Undo::Discard(draft) => self.compose.reopen(draft),
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
