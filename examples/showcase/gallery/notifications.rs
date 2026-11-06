use super::section;
use dutchy_egui::{
    Button, ButtonVariant, Checkbox, Text, TextColor, Toast, ToastAppearance, ToastDismissal,
    ToastEvent, ToastPosition, ToastQueue, ToastVariant, Toasts, tokens::space,
};
use egui::{Context, Ui};
#[derive(Debug)]
pub enum Action {
    UndoArchive,
    RetrySync,
}
pub struct NotificationState {
    pub queue: ToastQueue<Action>,
    pub position: ToastPosition,
    pub outline: bool,
    pub paused: bool,
    pub persistent: bool,
    pub archives: usize,
    pub retries: usize,
    pub dismissed: usize,
    pub expired: usize,
    pub pending: usize,
    serial: usize,
}
impl Default for NotificationState {
    fn default() -> Self {
        Self {
            queue: ToastQueue::new(),
            position: ToastPosition::BottomRight,
            outline: false,
            paused: false,
            persistent: false,
            archives: 0,
            retries: 0,
            dismissed: 0,
            expired: 0,
            pending: 0,
            serial: 0,
        }
    }
}
impl NotificationState {
    fn appearance(&self) -> ToastAppearance {
        if self.outline {
            ToastAppearance::Outline
        } else {
            ToastAppearance::Solid
        }
    }
    fn notice(&self, title: &str, message: &str, variant: ToastVariant) -> Toast<Action> {
        let toast = Toast::new(message)
            .title(title)
            .variant(variant)
            .appearance(self.appearance());
        if self.persistent {
            toast.timeout(None)
        } else {
            toast
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        ui.add(Text::heading("Feedback. Without friction."));
        ui.add(Text::new(
            "Nonblocking messages with an application-owned queue and explicit host actions.",
        ));
        section(
            ui,
            "01",
            "Four intents / live and documented treatments",
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (label, title, message, variant) in [
                        (
                            "Notify info",
                            "Heads up",
                            "A new version is available.",
                            ToastVariant::Info,
                        ),
                        (
                            "Notify success",
                            "Saved",
                            "Your changes are saved.",
                            ToastVariant::Success,
                        ),
                        (
                            "Notify warning",
                            "Session reminder",
                            "Save your work before leaving.",
                            ToastVariant::Warning,
                        ),
                    ] {
                        if ui
                            .add(Button::new(label).variant(ButtonVariant::Outline))
                            .clicked()
                        {
                            self.queue.push(self.notice(title, message, variant));
                        }
                    }
                    if ui
                        .add(Button::new("Notify error").variant(ButtonVariant::Destructive))
                        .clicked()
                    {
                        self.queue.push(
                            self.notice(
                                "Sync failed",
                                "Retry updates the host counter.",
                                ToastVariant::Error,
                            )
                            .action("Retry sync", Action::RetrySync)
                            .timeout(None),
                        );
                    }
                });
                ui.add_space(space::MD);
                ui.add(Checkbox::new(
                    "outline-notices",
                    &mut self.outline,
                    "Use outlined notifications",
                ));
                ui.add(Checkbox::new(
                    "persistent-notices",
                    &mut self.persistent,
                    "Keep notices until dismissed",
                ));
                ui.add(Checkbox::new(
                    "pause-notices",
                    &mut self.paused,
                    "Pause notification lifetimes",
                ));
                ui.add(Text::new("Ordinary notices last five visible seconds. Hover, keyboard focus, clipping, inactive windows, and modals pause expiry. Action notices in this demo persist.").color(TextColor::Muted));
            },
        );
        section(ui, "02", "Host action / single-use undo", |ui| {
            if ui.add(Button::new("Archive sample item")).clicked() {
                self.archives += 1;
                self.queue.push(
                    self.notice(
                        "Item archived",
                        "Undo applies in the application.",
                        ToastVariant::Success,
                    )
                    .action("Undo archive", Action::UndoArchive)
                    .timeout(None),
                );
            }
            ui.add_space(space::SM);
            ui.add(Text::label(format!(
                "Host archives: {} / retries: {}",
                self.archives, self.retries
            )));
            ui.add(
                Text::new(
                    "A toast returns its typed action once. It never performs application work.",
                )
                .color(TextColor::Muted),
            );
        });
        section(ui, "03", "Four corners / bounded visible stack", |ui| {
            ui.horizontal_wrapped(|ui| {
                for (label, position) in [
                    ("Top left", ToastPosition::TopLeft),
                    ("Top right", ToastPosition::TopRight),
                    ("Bottom left", ToastPosition::BottomLeft),
                    ("Bottom right", ToastPosition::BottomRight),
                ] {
                    let variant = if self.position == position {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    };
                    if ui.add(Button::new(label).variant(variant)).clicked() {
                        self.position = position;
                    }
                }
            });
            ui.add_space(space::SM);
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add(Button::new("Queue five notices").variant(ButtonVariant::Outline))
                    .clicked()
                {
                    for _ in 0..5 {
                        self.serial += 1;
                        self.queue.push(self.notice(
                            &format!("Queued {}", self.serial),
                            "Waiting entries retain their full reading time.",
                            ToastVariant::Info,
                        ));
                    }
                }
                if ui
                    .add(Button::new("Clear notifications").variant(ButtonVariant::Ghost))
                    .clicked()
                {
                    self.queue.clear();
                }
            });
            ui.add_space(space::SM);
            ui.add(
                Text::label(format!(
                    "Queue: {} / waiting: {} / dismissed: {} / expired: {}",
                    self.queue.len(),
                    self.pending,
                    self.dismissed,
                    self.expired
                ))
                .color(TextColor::Muted),
            );
        });
    }
    pub fn render(&mut self, ctx: &Context) {
        let result = Toasts::new("showcase-notifications", &mut self.queue)
            .position(self.position)
            .paused(self.paused)
            .show(ctx);
        self.pending = result.pending;
        for event in result.events {
            match event {
                ToastEvent::Action {
                    action: Action::UndoArchive,
                    ..
                } => self.archives = self.archives.saturating_sub(1),
                ToastEvent::Action {
                    action: Action::RetrySync,
                    ..
                } => self.retries += 1,
                ToastEvent::Dismissed {
                    reason: ToastDismissal::CloseButton,
                    ..
                } => self.dismissed += 1,
                ToastEvent::Dismissed {
                    reason: ToastDismissal::Expired,
                    ..
                } => self.expired += 1,
            }
        }
    }
}
