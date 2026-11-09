use crate::Icon;
use egui::{Id, KeyboardShortcut};

/// Where a binding is eligible. Focused bindings take precedence over application bindings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CommandScope {
    /// Any host focus, subject to editing and modal policy.
    #[default]
    Application,
    /// Only while this exact native widget owns focus.
    Focused(Id),
}

/// Application-owned command description. No callback or application data is retained.
#[derive(Clone, Debug)]
pub struct Command<T> {
    /// Stable host action key.
    pub value: T,
    /// Visible and accessible action name.
    pub label: String,
    /// Optional single-chord binding; COMMAND means Command on macOS, Control elsewhere.
    pub shortcut: Option<KeyboardShortcut>,
    /// Host availability, shared by all adapters and keyboard dispatch.
    pub enabled: bool,
    /// Controlled check state; activation requests its inverse.
    pub checked: Option<bool>,
    /// Optional toolbar glyph.
    pub icon: Option<Icon>,
    /// Binding focus scope.
    pub scope: CommandScope,
    /// Explicitly allow this binding while a text editor has focus.
    pub allow_in_text: bool,
    /// Explicitly allow this binding while a modal owns input.
    pub allow_in_overlays: bool,
}
impl<T> Command<T> {
    /// Enabled application action, without a binding or checked state.
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            shortcut: None,
            enabled: true,
            checked: None,
            icon: None,
            scope: CommandScope::Application,
            allow_in_text: false,
            allow_in_overlays: false,
        }
    }
    /// Single logical key chord.
    pub fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
    /// Host availability.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Host-controlled checked state.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }
    /// Optional toolbar glyph.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Restrict binding eligibility to a native focus target.
    pub fn scope(mut self, scope: CommandScope) -> Self {
        self.scope = scope;
        self
    }
    /// Opt into bindings inside text editors; ordinary editing remains the default.
    pub fn allow_in_text(mut self, allow: bool) -> Self {
        self.allow_in_text = allow;
        self
    }
    /// Opt into bindings inside modal overlays.
    pub fn allow_in_overlays(mut self, allow: bool) -> Self {
        self.allow_in_overlays = allow;
        self
    }
}

/// An activation request. The host decides whether to execute or accept it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandRequest<T> {
    /// Stable host action key.
    pub value: T,
    /// Requested check state, or None for an ordinary action.
    pub checked: Option<bool>,
}

/// Two enabled commands with the same chord and equally specific focus scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutConflict<T> {
    /// First conflicting command key.
    pub first: T,
    /// Second conflicting command key.
    pub second: T,
    /// Conflicting binding (platform aliases are considered).
    pub shortcut: KeyboardShortcut,
}
