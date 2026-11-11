//! The compose Modal: To, Subject and a Markdown body; Send files the
//! message in Sent, Discard closes with an Undo that restores the draft.

use crate::app::{App, Undo};
use crate::mailbox::{Message, MessageId, SENT};
use dutchy_egui::{
    Button, ButtonVariant, ControlSize, Field, Input, Modal, ModalWidth, Textarea, Toast,
    tokens::space,
};
use egui::Context;

/// Sender shown for messages written in the example.
pub const ME: &str = "You";
pub const MY_ADDRESS: &str = "you@example.com";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl Draft {
    fn is_empty(&self) -> bool {
        self.to.trim().is_empty() && self.subject.trim().is_empty() && self.body.trim().is_empty()
    }
}

#[derive(Default)]
pub struct Compose {
    pub open: bool,
    pub draft: Draft,
}

impl Compose {
    pub fn open_new(&mut self) {
        if !self.open {
            self.draft = Draft::default();
            self.open = true;
        }
    }

    pub fn reopen(&mut self, draft: Draft) {
        self.draft = draft;
        self.open = true;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Choice {
    Send,
    Discard,
}

pub fn show(app: &mut App, ctx: &Context) {
    let can_send = !app.compose.draft.to.trim().is_empty();
    let Compose { open, draft } = &mut app.compose;
    let result = Modal::new("compose", open, "New message")
        .width(ModalWidth::Large)
        .show_with_footer(
            ctx,
            |ui| {
                ui.spacing_mut().item_spacing.y = space::MD;
                Field::new("To").required(true).show(ui, |ui| {
                    ui.add(Input::new("compose-to", &mut draft.to).accessible_name("To"))
                });
                Field::new("Subject").show(ui, |ui| {
                    ui.add(
                        Input::new("compose-subject", &mut draft.subject)
                            .accessible_name("Subject"),
                    )
                });
                Field::new("Message").show(ui, |ui| {
                    ui.add(
                        Textarea::new("compose-body", &mut draft.body)
                            .accessible_name("Message")
                            .rows(8)
                            .show_count(false),
                    )
                });
            },
            |ui| footer(ui, can_send),
        );
    let Some(result) = result else { return };
    match result.inner.1 {
        Some(Choice::Send) => send(app),
        // Escape or the close button discards too, with the same Undo.
        Some(Choice::Discard) => discard(app),
        None if result.dismissed.is_some() => discard(app),
        None => {}
    }
}

fn footer(ui: &mut egui::Ui, can_send: bool) -> Option<Choice> {
    let mut choice = None;
    ui.horizontal(|ui| {
        let send = Button::new("Send")
            .size(ControlSize::Small)
            .enabled(can_send);
        if ui.add(send).clicked() {
            choice = Some(Choice::Send);
        }
        let discard = Button::new("Discard")
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Small);
        if ui.add(discard).clicked() {
            choice = Some(Choice::Discard);
        }
    });
    choice
}

fn send(app: &mut App) {
    let draft = std::mem::take(&mut app.compose.draft);
    app.compose.open = false;
    let subject = if draft.subject.trim().is_empty() {
        "(No subject)".to_owned()
    } else {
        draft.subject.trim().to_owned()
    };
    app.mailbox.add(Message {
        id: MessageId(0),
        folder: SENT,
        from: ME.into(),
        address: MY_ADDRESS.into(),
        subject: subject.clone(),
        body: format!("To: {}\n\n{}", draft.to.trim(), draft.body),
        date: app.today,
        minute: 23 * 60 + 58,
        unread: false,
    });
    app.toasts
        .push(Toast::new(format!("Sent \u{201c}{subject}\u{201d}.")));
}

fn discard(app: &mut App) {
    let draft = std::mem::take(&mut app.compose.draft);
    app.compose.open = false;
    if !draft.is_empty() {
        app.toasts
            .push(Toast::new("Draft discarded.").action("Undo", Undo::Discard(draft)));
    }
}
