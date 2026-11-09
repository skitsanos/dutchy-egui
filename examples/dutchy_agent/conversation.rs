//! Transcript composition: stable message documents, native code copy and host requests.
use crate::{
    app::App,
    chat::{Message, Status},
    provider::{Mode, Role, UsageKind},
};
use dutchy_egui::{
    Alert, AlertVariant, Button, ButtonVariant, ControlSize, EmptyState, Flex, LinkPolicy,
    Markdown, Skeleton, SkeletonGroup, Text, TextColor, TextSize, Theme,
    tokens::{border, space},
};
use egui::{Frame, Margin, Stroke, Ui, accesskit::Role as AccessibleRole};

pub(crate) fn show(ui: &mut Ui, app: &mut App, now: f64) {
    ui.add(Text::heading(&app.chat.current().title).size(TextSize::Xl2));
    Flex::row("chat-heading").wrap(true).show(ui, |ui| {
        ui.add(
            Text::new(if app.config.provider == Mode::Mock {
                "Scripted replies · no network required"
            } else {
                "Responses API · model replies"
            })
            .size(TextSize::Sm)
            .color(TextColor::Muted),
        );
        if ui
            .add(
                Button::new("Delete conversation")
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small),
            )
            .clicked()
        {
            app.delete_chat();
        }
    });
    ui.add_space(space::LG);
    let id = app.chat.selected;
    let mut retry = false;
    let mut sample = false;
    let mut link = false;
    egui::ScrollArea::vertical()
        .id_salt(("agent-transcript", id))
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if app.chat.current().messages.is_empty() {
                sample = EmptyState::new(("empty-chat", id), "Make room for a good question.")
                    .description(if app.config.provider == Mode::Mock {
                        "Try an offline reply with headings, a checklist and copyable Rust code."
                    } else {
                        "Ask your OpenAI model a question, or request a Rust code example."
                    })
                    .action("Try the Rust example")
                    .show(ui)
                    .action_clicked;
            }
            let current = app.chat.current();
            for (index, message) in current.messages.iter().enumerate() {
                let outcome = show_message(
                    ui,
                    message,
                    current.retryable() && index + 1 == current.messages.len(),
                );
                retry |= outcome.0;
                link |= outcome.1;
                ui.add_space(space::LG);
            }
            if let Some(usage) = &current.usage {
                let label = match usage.kind {
                    UsageKind::Estimated => format!(
                        "Mock token estimate · {} input / {} output",
                        usage.input, usage.output
                    ),
                    UsageKind::Reported => format!(
                        "OpenAI tokens · {} input / {} output · {}",
                        usage.input,
                        usage.output,
                        usage.model.as_deref().unwrap_or("configured model")
                    ),
                    UsageKind::Unavailable => "Token usage unavailable".into(),
                };
                ui.add(Text::label(label).color(TextColor::Muted));
            }
        });
    if retry {
        app.retry(now);
    }
    if sample {
        // The sample is a one-off request: keep anything the user has typed.
        let draft = std::mem::replace(
            &mut app.chat.current_mut().draft,
            "Show a Rust example".into(),
        );
        app.send(now);
        app.chat.current_mut().draft = draft;
    }
    if link {
        app.toasts.push(dutchy_egui::Toast::new(
            "Link reported locally; no browser was opened.",
        ));
    }
}

fn show_message(ui: &mut Ui, message: &Message, retryable: bool) -> (bool, bool) {
    let theme = Theme::from_context(ui.ctx());
    let fill = if message.role == Role::User {
        theme.palette.muted
    } else {
        theme.palette.background
    };
    let mut retry = false;
    let mut link = false;
    let frame = Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(border::THIN, theme.palette.border))
        .inner_margin(Margin::same(space::LG as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // The transcript scrolls vertically. Its messages need natural
            // height so nested code scrollers do not shrink to leftover space.
            ui.set_max_height(f32::INFINITY);
            let author = if message.role == Role::User {
                "You"
            } else {
                message.provider.label()
            };
            ui.add(Text::label(author));
            ui.add_space(space::MD);
            if message.text.is_empty() && message.status == Status::Receiving {
                SkeletonGroup::new(
                    ("waiting", message.id),
                    if message.provider == Mode::Mock {
                        "Waiting for offline reply"
                    } else {
                        "Waiting for OpenAI reply"
                    },
                )
                .show(ui, |ui| {
                    ui.add(Skeleton::new().count(3));
                });
            } else if message.role == Role::Assistant && !message.text.is_empty() {
                link = !Markdown::new(("reply", message.id), &message.text)
                    .accessible_name("Assistant reply")
                    .heading_offset(1)
                    .links(LinkPolicy::Report)
                    .images(false)
                    .line_numbers(true)
                    .show(ui)
                    .links
                    .is_empty();
            } else if !message.text.is_empty() {
                ui.add(Text::new(&message.text));
            }
            match message.status {
                Status::Failed(error) => {
                    let mut alert = Alert::new(("reply-error", message.id), error)
                        .title("Reply unavailable")
                        .variant(AlertVariant::Error);
                    if retryable {
                        alert = alert.action("Retry reply");
                    }
                    retry = alert.show(ui).action_clicked;
                }
                Status::Stopped => {
                    ui.add(Text::new("Reply stopped.").color(TextColor::Muted));
                    if retryable {
                        retry = ui
                            .add(
                                Button::new("Retry reply")
                                    .variant(ButtonVariant::Outline)
                                    .size(ControlSize::Small),
                            )
                            .clicked();
                    }
                }
                _ => {}
            }
        });
    frame
        .response
        .ctx
        .accesskit_node_builder(frame.response.id, |node| {
            node.set_role(AccessibleRole::Group);
            node.set_label(if message.role == Role::User {
                "User message"
            } else {
                "Assistant message"
            });
            if message.status == Status::Receiving {
                node.set_busy();
            }
        });
    (retry, link)
}
