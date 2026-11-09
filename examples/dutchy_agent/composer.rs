//! Composer keeps native multiline editing and consumes only plain Enter.
use crate::{app::App, chat::MAX_PROMPT, support::shell};
use dutchy_egui::{
    Button, ButtonVariant, Flex, Text, TextColor, TextSize, Textarea, Theme, tokens::space,
};
use egui::{Event, ImeEvent, Key, Ui};

pub(crate) fn show(ui: &mut Ui, app: &mut App, now: f64) {
    egui::Panel::bottom("agent-composer-panel")
        .frame(shell::frame(ui, space::SM as i8))
        .show(ui, |ui| {
            shell::divider(ui);
            ui.add_space(space::MD);
            let ready = app.chat.current().active.is_none();
            let focused = app
                .composer_id
                .is_some_and(|id| ui.memory(|m| m.has_focus(id)));
            let ime = composition(ui, focused && !app.settings_open, &mut app.composing);
            let enter = !app.settings_open && focused && ready && !ime && take_enter(ui);
            let reply = ui.add(
                Textarea::new(
                    ("message", app.chat.selected),
                    &mut app.chat.current_mut().draft,
                )
                .placeholder("Write a message…")
                .accessible_name("Message")
                .char_limit(MAX_PROMPT)
                .rows(2),
            );
            app.composer_id = Some(reply.id);
            if app.focus_composer && !app.settings_open {
                reply.request_focus();
                app.focus_composer = false;
            }
            let can_send = ready && !app.chat.current().draft.trim().is_empty();
            let mut send = enter && can_send;
            Flex::row("composer-actions").wrap(true).show(ui, |ui| {
                send |= ui
                    .add(Button::new("Send message").enabled(can_send))
                    .clicked();
                if !ready
                    && ui
                        .add(Button::new("Stop reply").variant(ButtonVariant::Outline))
                        .clicked()
                {
                    app.chat.stop();
                    app.focus_composer = true;
                }
                hint(
                    ui,
                    Text::new(if ready {
                        "Enter sends · Shift+Enter adds a line"
                    } else {
                        "Reply running · your next draft stays here"
                    })
                    .size(TextSize::Xs)
                    .color(TextColor::Muted),
                );
            });
            if send {
                app.send(now);
            }
        });
}

/// Keep the hint whole: egui continues text after the actions and wraps the
/// rest under them, so a hint that does not fit starts its own row instead.
fn hint(ui: &mut Ui, text: Text<'_>) {
    let theme = Theme::from_context(ui.ctx());
    let width = ui
        .fonts_mut(|fonts| fonts.layout_job(text.layout_job(&theme, f32::INFINITY)))
        .size()
        .x;
    if ui.available_size_before_wrap().x < width {
        ui.end_row();
    }
    ui.add(text);
}

fn take_enter(ui: &mut Ui) -> bool {
    let mut taken = false;
    ui.input_mut(|input| {
        input.events.retain(|event| {
            if let Event::Key {
                key: Key::Enter,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } = event
                && modifiers.is_none()
            {
                taken = true;
                return false;
            }
            true
        })
    });
    taken
}

fn composition(ui: &Ui, focused: bool, active: &mut bool) -> bool {
    if !focused {
        *active = false;
    }
    let mut observed = false;
    ui.input(|i| {
        for event in &i.events {
            if let Event::Ime(event) = event {
                observed = true;
                match event {
                    ImeEvent::Preedit { text, .. } => *active = !text.is_empty(),
                    ImeEvent::Commit(_) => *active = false,
                    _ => {}
                }
            }
        }
    });
    observed || *active
}
