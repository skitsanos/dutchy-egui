//! Shared command metadata, native shortcut routing and additive surface adapters.
pub(crate) mod dispatch;
mod model;
mod surfaces;
pub use dispatch::same_shortcut;
use egui::{Context, Id, IdSalt, KeyboardShortcut};
pub use model::{Command, CommandRequest, CommandScope, ShortcutConflict};

/// Host-owned command catalog. Reuse one stable ID per application command domain.
/// Inserting an existing key replaces its metadata; no command executes here.
#[derive(Clone, Debug)]
pub struct CommandRegistry<T> {
    id: Id,
    entries: Vec<Command<T>>,
}
impl<T: Clone + PartialEq> CommandRegistry<T> {
    /// Build a catalog. Later definitions of a repeated key replace earlier ones.
    pub fn new(id: impl egui::AsIdSalt, commands: impl IntoIterator<Item = Command<T>>) -> Self {
        let mut registry = Self {
            id: Id::unique(("dutchy-commands", IdSalt::new(id))),
            entries: Vec::new(),
        };
        for command in commands {
            registry.insert(command);
        }
        registry
    }
    /// Insert or replace metadata by stable action key.
    pub fn insert(&mut self, command: Command<T>) {
        if let Some(old) = self.get_mut(&command.value) {
            *old = command;
        } else {
            self.entries.push(command);
        }
    }
    /// Remove an action and its binding.
    pub fn remove(&mut self, value: &T) {
        self.entries.retain(|entry| &entry.value != value);
    }
    /// Command descriptions in insertion order.
    pub fn commands(&self) -> &[Command<T>] {
        &self.entries
    }
    /// Read metadata for one command.
    pub fn get(&self, value: &T) -> Option<&Command<T>> {
        self.entries.iter().find(|entry| &entry.value == value)
    }
    /// Update availability, checked state, name or binding in the host.
    pub fn get_mut(&mut self, value: &T) -> Option<&mut Command<T>> {
        self.entries.iter_mut().find(|entry| &entry.value == value)
    }
    /// Validate a surface activation against current host availability.
    pub fn request(&self, value: &T) -> Option<CommandRequest<T>> {
        let entry = self.get(value).filter(|entry| entry.enabled)?;
        Some(CommandRequest {
            value: entry.value.clone(),
            checked: entry.checked.map(|checked| !checked),
        })
    }
    /// Native platform formatting, shared by menu hints, tooltips and settings.
    pub fn shortcut_label(&self, ctx: &Context, value: &T) -> Option<String> {
        self.get(value)?
            .shortcut
            .as_ref()
            .map(|shortcut| ctx.format_shortcut(shortcut))
    }
    /// Bindings of other commands, useful as KeyRecorder's reserved list.
    pub fn shortcuts_except(&self, value: &T) -> Vec<KeyboardShortcut> {
        self.entries
            .iter()
            .filter(|entry| &entry.value != value)
            .filter_map(|entry| entry.shortcut)
            .collect()
    }
}
