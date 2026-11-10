use dutchy_egui::{
    Icon, Slider, StatusAlign, StatusBar, StatusItem, StatusTone, Switch, Text, TextColor,
    TextSize, Theme, tokens::border, tokens::space,
};
use egui::{Frame, Margin, Stroke, Ui};

/// How long the simulated sync runs, in seconds.
const SYNC_SECONDS: f64 = 2.0;
const KEYS: &str = "Activate Sync with a click, Enter or Space. Narrow the bar to watch low-priority segments step aside.";

#[derive(Clone, Copy, PartialEq)]
enum Mail {
    Current,
    Running { started: f64 },
    Failed,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Segment {
    Unread,
    Selected,
    Sync,
    Storage,
    Commands,
    Compose,
}

impl Segment {
    fn name(self) -> &'static str {
        match self {
            Self::Unread => "Unread",
            Self::Selected => "Selected",
            Self::Sync => "Sync",
            Self::Storage => "Storage",
            Self::Commands => "Commands",
            Self::Compose => "Compose",
        }
    }
}

pub struct StatusState {
    sync: Mail,
    fail_next: bool,
    disabled: bool,
    width: f64,
    syncs: u32,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            sync: Mail::Current,
            fail_next: false,
            disabled: false,
            // On the slider's 8-point grid from 240.
            width: 960.0,
            syncs: 0,
        }
    }
}

impl StatusState {
    /// Finish a running sync once its simulated time has passed.
    fn tick(&mut self, ui: &Ui) {
        let Mail::Running { started } = self.sync else {
            return;
        };
        let now = ui.input(|i| i.time);
        if now - started >= SYNC_SECONDS {
            self.sync = if self.fail_next {
                Mail::Failed
            } else {
                Mail::Current
            };
            self.syncs += 1;
        } else {
            ui.ctx().request_repaint();
        }
    }

    fn items(&self) -> Vec<StatusItem<'static, Segment>> {
        let sync = match self.sync {
            Mail::Current => StatusItem::labeled(Segment::Sync, "Sync", "Up to date")
                .icon(Icon::Check)
                .tone(StatusTone::Success),
            Mail::Running { .. } => {
                StatusItem::labeled(Segment::Sync, "Sync", "Checking mail").busy()
            }
            Mail::Failed => StatusItem::labeled(Segment::Sync, "Sync", "Failed · retry")
                .tone(StatusTone::Destructive),
        };
        vec![
            StatusItem::labeled(Segment::Unread, "Unread", "42").priority(3),
            StatusItem::labeled(Segment::Selected, "Selected", "3").priority(1),
            sync.clickable(!matches!(self.sync, Mail::Running { .. }))
                .live(true)
                .tooltip("Check for new mail")
                .priority(2),
            StatusItem::labeled(Segment::Storage, "Storage", "62%").progress(62.0),
            StatusItem::hint(Segment::Commands, "Ctrl+K", "Commands").align(StatusAlign::End),
            StatusItem::hint(Segment::Compose, "N", "Compose").align(StatusAlign::End),
        ]
    }
}

pub fn show(ui: &mut Ui, state: &mut StatusState) {
    state.tick(ui);
    ui.add(Text::heading("Know where things stand.").size(TextSize::Xl5));
    ui.add(
        Text::new("Counts, sync state and shortcuts in one quiet line at the edge of the window.")
            .color(TextColor::Muted),
    );
    ui.add_space(space::LG);
    ui.horizontal_wrapped(|ui| {
        ui.add(Switch::new(
            "status-fail",
            &mut state.fail_next,
            "Fail next sync",
        ));
        ui.add(Switch::new(
            "status-disabled",
            &mut state.disabled,
            "Disabled",
        ));
    });
    ui.add_space(space::MD);
    ui.add(Text::label("Bar width").color(TextColor::Muted));
    ui.add(
        Slider::new("status-width", &mut state.width, 240.0..=1200.0)
            .step(8.0)
            .accessible_name("Bar width")
            .value_label(|width| format!("{width:.0} pt")),
    );
    ui.add_space(space::LG);
    let items = state.items();
    let width = (state.width as f32).min(ui.available_width());
    let result = window(ui, width, |ui| {
        StatusBar::new("mail-status", &items)
            .accessible_name("Mailbox status")
            .enabled(!state.disabled)
            .show(ui)
    });
    if result.clicked == Some(Segment::Sync) {
        state.sync = Mail::Running {
            started: ui.input(|i| i.time),
        };
        ui.ctx().request_repaint();
    }
    ui.add_space(space::MD);
    let hidden: Vec<&str> = result.hidden.iter().map(|s| s.name()).collect();
    ui.add(Text::label(format!(
        "{} syncs / hidden: {}",
        state.syncs,
        if hidden.is_empty() {
            "none".to_owned()
        } else {
            hidden.join(", ")
        }
    )));
    ui.add(Text::new(KEYS).size(TextSize::Sm).color(TextColor::Muted));
}

/// A bordered mock window whose bottom edge holds the bar.
fn window<R>(ui: &mut Ui, width: f32, bar: impl FnOnce(&mut Ui) -> R) -> R {
    let p = Theme::from_context(ui.ctx()).palette;
    Frame::NONE
        .stroke(Stroke::new(border::THIN, p.border))
        .show(ui, |ui| {
            ui.set_width(width);
            Frame::NONE
                .inner_margin(Margin::same(space::LG as i8))
                .show(ui, |ui| {
                    ui.set_width(width - 2.0 * space::LG);
                    ui.add(Text::heading("Inbox").size(TextSize::Xl));
                    ui.add(
                        Text::new("Messages, folders and the reading pane live up here.")
                            .color(TextColor::Muted),
                    );
                    ui.add_space(space::HERO);
                });
            ui.spacing_mut().item_spacing.y = 0.0;
            bar(ui)
        })
        .inner
}
