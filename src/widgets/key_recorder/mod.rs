mod state;
use crate::{
    Alert, AlertVariant, Button, ButtonVariant, ControlSize, Icon, IconButton,
    commands::{
        dispatch::{capturing, ime_active, operates_focus, set_capture},
        same_shortcut,
    },
    tokens::space,
};
use egui::{IdSalt, KeyboardShortcut, Response, Ui};
use state::{Captured, Memory, Rejection};
use std::borrow::Cow;

/// Shortcut edit outcome. Only an accepted chord or explicit Clear changes the value.
#[non_exhaustive]
pub struct KeyRecorderResponse {
    /// Named recording button; changed() reflects accepted host-value edits.
    pub response: Response,
    /// Whether the next eligible key chord is being recorded.
    pub recording: bool,
    /// An uncommitted chord was rejected: another binding reserves it, or a
    /// bare activation or navigation key needs Command, Control or Alt.
    pub conflict: bool,
    /// Recording ended through Escape, Tab, focus loss, IME or disabling.
    pub cancelled: bool,
}
/// Capture one portable shortcut into an application-owned optional binding.
#[must_use = "Call show to render the key recorder"]
pub struct KeyRecorder<'a> {
    id: IdSalt,
    value: &'a mut Option<KeyboardShortcut>,
    name: Cow<'a, str>,
    reserved: &'a [KeyboardShortcut],
    enabled: bool,
    read_only: bool,
    navigation: bool,
    size: ControlSize,
}
impl<'a> KeyRecorder<'a> {
    /// Bind stable identity and an optional host chord. No global binding is installed.
    pub fn new(id: impl egui::AsIdSalt, value: &'a mut Option<KeyboardShortcut>) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            name: "Record shortcut".into(),
            reserved: &[],
            enabled: true,
            read_only: false,
            navigation: false,
            size: ControlSize::Medium,
        }
    }
    /// Accessible recording-button name, independent of the displayed key chord.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Other host bindings that cannot be assigned, including platform aliases.
    pub fn reserved(mut self, reserved: &'a [KeyboardShortcut]) -> Self {
        self.reserved = reserved;
        self
    }
    /// Disable editing and cancel any active capture.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Keep controls focusable while preventing changes/capture.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Allow Enter, Space, arrow, Home, End and page keys without Command,
    /// Control or Alt. Off by default: focused controls use those keys, and
    /// registry dispatch leaves them to a focused control even when bound.
    pub fn allow_navigation_keys(mut self, allow: bool) -> Self {
        self.navigation = allow;
        self
    }
    /// Shared target height.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Render capture and explicit Clear controls. Escape cancels; Tab cancels
    /// and continues navigation. Modifier-only presses and repeats never commit.
    pub fn show(mut self, ui: &mut Ui) -> KeyRecorderResponse {
        let root = ui.make_persistent_id(("dutchy-key-recorder", self.id));
        let mut memory = ui
            .ctx()
            .data(|data| data.get_temp::<Memory>(root).unwrap_or_default());
        let frame = ui.ctx().cumulative_frame_nr();
        if memory.consumed_frame == Some(frame) {
            state::suppress(ui.ctx());
        }
        let was = memory.recording;
        let editable = self.enabled && ui.is_enabled() && !self.read_only;
        let (mut changed, cancelled) = if memory.recording {
            self.record(ui, &mut memory, editable, frame)
        } else {
            (false, false)
        };
        let label = if memory.recording {
            "Press a key chord…".into()
        } else {
            self.value.as_ref().map_or_else(
                || "Set shortcut".into(),
                |key| ui.ctx().format_shortcut(key),
            )
        };
        let mut response = ui
            .push_id(root, |ui| {
                ui.horizontal(|ui| {
                    let button = ui
                        .add(
                            Button::new(label)
                                .size(self.size)
                                .variant(ButtonVariant::Outline)
                                .enabled(self.enabled),
                        )
                        .accessible_name(self.name.to_string());
                    let clear = ui.add(
                        IconButton::new("clear", format!("Clear {}", self.name), Icon::Close)
                            .size(self.size)
                            .enabled(editable && self.value.is_some())
                            .tooltip("Clear shortcut"),
                    );
                    if clear.clicked() {
                        changed = self.value.is_some();
                        *self.value = None;
                        memory.recording = false;
                        memory.error = None;
                        button.request_focus();
                    }
                    button
                })
                .inner
            })
            .inner;
        memory.button = Some(response.id);
        if response.clicked() && editable && !was {
            memory.recording = true;
            memory.error = None;
            response.request_focus();
        }
        ui.memory_mut(|memory_state| {
            memory_state.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    escape: memory.recording,
                    horizontal_arrows: memory.recording,
                    vertical_arrows: memory.recording,
                    ..Default::default()
                },
            )
        });
        if changed {
            response.mark_changed();
        }
        let description = if memory.recording {
            "Recording shortcut. Escape cancels; Tab cancels and moves focus.".to_owned()
        } else {
            self.value.as_ref().map_or_else(
                || "No shortcut assigned.".into(),
                |key| format!("Current shortcut: {}", response.ctx.format_shortcut(key)),
            )
        };
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_description(description);
            if self.read_only {
                node.set_read_only();
            }
        });
        if let Some(rejection) = memory.error {
            let (title, body) = match rejection {
                Rejection::Reserved => (
                    "Shortcut already in use",
                    "Choose another chord or cancel recording.",
                ),
                Rejection::Modifier => (
                    "Key needs a modifier",
                    "Enter, Space, arrow, Home, End and page keys operate focused controls. Add Command, Control or Alt.",
                ),
            };
            ui.add_space(space::SM);
            Alert::new(root.with("conflict"), body)
                .title(title)
                .variant(AlertVariant::Error)
                .show(ui);
        }
        set_capture(ui.ctx(), root, memory.recording);
        let result = KeyRecorderResponse {
            response,
            recording: memory.recording,
            conflict: memory.error.is_some(),
            cancelled,
        };
        ui.ctx().data_mut(|data| data.insert_temp(root, memory));
        result
    }
    /// Apply one frame of capture; returns whether the value changed and
    /// whether recording was cancelled.
    fn record(&mut self, ui: &Ui, memory: &mut Memory, editable: bool, frame: u64) -> (bool, bool) {
        let focused = memory
            .button
            .is_some_and(|id| ui.memory(|m| m.has_focus(id)));
        if !editable || !focused || ime_active(ui.ctx()) {
            memory.recording = false;
            memory.error = None;
            return (false, true);
        }
        match state::take(ui.ctx()) {
            Captured::None => (false, false),
            Captured::Cancel => {
                memory.consumed_frame = Some(frame);
                memory.recording = false;
                memory.error = None;
                (false, true)
            }
            Captured::Shortcut(shortcut) => {
                memory.consumed_frame = Some(frame);
                memory.error = self.rejection(ui.ctx(), shortcut);
                if memory.error.is_some() {
                    return (false, false);
                }
                let changed = *self.value != Some(shortcut);
                *self.value = Some(shortcut);
                memory.recording = false;
                (changed, false)
            }
        }
    }
    fn rejection(&self, ctx: &egui::Context, shortcut: KeyboardShortcut) -> Option<Rejection> {
        if !self.navigation && operates_focus(shortcut.logical_key, shortcut.modifiers) {
            Some(Rejection::Modifier)
        } else if self
            .reserved
            .iter()
            .any(|other| same_shortcut(ctx, shortcut, *other))
        {
            Some(Rejection::Reserved)
        } else {
            None
        }
    }
    /// Whether any recently drawn recorder currently owns command input.
    pub fn is_recording(ctx: &egui::Context) -> bool {
        capturing(ctx)
    }
}
