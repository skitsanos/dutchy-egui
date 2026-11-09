//! Shell, command routing and host-owned UI state.
use crate::{
    chat::Chat,
    composer,
    config::Config,
    conversation, header,
    provider::Scenario,
    settings, sidebar,
    support::{capture::Capture, shell, theme::ThemeSettings},
};
use dutchy_egui::{Command, CommandRegistry, Toast, ToastQueue, Toasts, tokens::space};
use egui::{Context, Id, Key, KeyboardShortcut, Modifiers, Ui};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    New,
    Settings,
    Stop,
}
pub(crate) struct App {
    pub chat: Chat,
    pub config: Config,
    pub scenario: Scenario,
    pub theme: ThemeSettings,
    pub settings_open: bool,
    pub settings_return: Option<Id>,
    pub composer_id: Option<Id>,
    pub composing: bool,
    pub focus_composer: bool,
    pub capture: Capture,
    pub toasts: ToastQueue<()>,
    copy_frame: Option<u64>,
}
impl App {
    pub fn new(config: Config) -> Self {
        let mut toasts = ToastQueue::new();
        if let Some(warning) = config.warning {
            toasts.push(Toast::new(warning));
        }
        Self {
            chat: Chat::default(),
            config,
            scenario: Scenario::Normal,
            theme: ThemeSettings::default(),
            settings_open: false,
            settings_return: None,
            composer_id: None,
            composing: false,
            focus_composer: false,
            capture: Capture::default(),
            toasts,
            copy_frame: None,
        }
    }
    pub fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        self.chat.context(ctx.clone());
        for message in self.chat.poll(now) {
            self.toasts.push(Toast::new(message));
        }
        self.shortcuts(&ctx);
        header::show(ui, self);
        self.status(ui);
        let wide = ui.available_width() >= 960.0;
        if wide {
            sidebar::show(ui, self);
        }
        egui::CentralPanel::default()
            .frame(shell::frame(ui, space::LG as i8))
            .show(ui, |ui| {
                if !wide {
                    sidebar::compact(ui, self);
                }
                composer::show(ui, self, now);
                conversation::show(ui, self, now);
            });
        settings::show(self, &ctx);
        self.clipboard_notice(&ctx);
        Toasts::new("agent-notifications", &mut self.toasts).show(&ctx);
        if let Some(due) = self.chat.next_due() {
            ctx.request_repaint_after(Duration::from_secs_f64((due - now).clamp(0.0, 1.0)));
        }
        self.capture.tick(&ctx);
    }
    pub fn send(&mut self, now: f64) {
        let result = self.chat.send(&self.config, self.scenario, now);
        self.report(result);
        self.focus_composer = true;
    }
    pub fn retry(&mut self, now: f64) {
        let result = self.chat.retry(&self.config, self.scenario, now);
        self.report(result);
        self.focus_composer = true;
    }
    pub fn new_chat(&mut self) {
        let result = self.chat.new_conversation();
        self.report(result);
        self.focus_composer = true;
    }
    pub fn delete_chat(&mut self) {
        let result = self.chat.delete_current();
        self.report(result);
        self.focus_composer = true;
    }
    fn report(&mut self, result: Result<(), &'static str>) {
        if let Err(error) = result {
            self.toasts.push(Toast::new(error));
        }
    }
    fn shortcuts(&mut self, ctx: &Context) {
        let commands = CommandRegistry::new(
            "agent-commands",
            [
                Command::new(Action::New, "New conversation")
                    .shortcut(KeyboardShortcut::new(Modifiers::COMMAND, Key::N))
                    .allow_in_text(true),
                Command::new(Action::Settings, "Chat settings")
                    .shortcut(KeyboardShortcut::new(Modifiers::COMMAND, Key::Comma))
                    .allow_in_text(true),
                Command::new(Action::Stop, "Stop reply")
                    .shortcut(KeyboardShortcut::new(Modifiers::COMMAND, Key::Period))
                    .allow_in_text(true)
                    .enabled(self.chat.current().active.is_some()),
            ],
        );
        for request in commands.dispatch(ctx) {
            match request.value {
                Action::New => self.new_chat(),
                Action::Settings => {
                    // Return to where the shortcut was pressed, not to an
                    // icon clicked earlier; fall back to the message field.
                    self.settings_return = ctx.memory(|m| m.focused()).or(self.composer_id);
                    self.settings_open = true;
                }
                Action::Stop => {
                    self.chat.stop();
                }
            }
        }
    }
    fn status(&self, ui: &mut Ui) {
        let running = self
            .chat
            .conversations
            .iter()
            .filter(|c| c.active.is_some())
            .count();
        let segments = [
            ("Provider", self.config.provider.label().into()),
            ("Running", running.to_string()),
        ];
        shell::status_strip(
            ui,
            "agent-status",
            &segments,
            &[
                shell::Hint {
                    key: "Enter",
                    action: "Send",
                },
                shell::Hint {
                    key: "Shift+Enter",
                    action: "New line",
                },
            ],
        );
    }
    fn clipboard_notice(&mut self, ctx: &Context) {
        let copied = ctx.output(|o| {
            o.commands
                .iter()
                .any(|c| matches!(c, egui::OutputCommand::CopyText(_)))
        });
        let frame = ctx.cumulative_frame_nr();
        if copied && self.copy_frame != Some(frame) {
            self.copy_frame = Some(frame);
            self.toasts.push(Toast::new("Copied to clipboard."));
        }
    }
}
impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}
