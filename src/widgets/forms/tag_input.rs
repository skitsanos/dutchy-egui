use super::Input;
use crate::widgets::chip::wrap_chips;
use crate::{
    Button, ButtonVariant, Chip, ControlSize, commands::dispatch::ime_active, tokens::space,
};
use egui::{
    Align, Event, Id, IdSalt, Key, Layout, Response, Ui, UiBuilder, WidgetInfo, accesskit::Role,
    vec2,
};
use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

/// One host-owned tag with a key that remains stable across renames and reordering.
#[derive(Clone, Debug)]
#[must_use = "Pass the item to TagInput"]
pub struct TagItem<'a, K> {
    key: K,
    label: Cow<'a, str>,
    removable: bool,
}

impl<'a, K> TagItem<'a, K> {
    /// Create a removable tag. Keys must be unique within the editor.
    pub fn new(key: K, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            label: label.into(),
            removable: true,
        }
    }

    /// Control whether this tag can be removed from the editor.
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }
}

/// One requested change to the host's tags.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TagInputRequest<K> {
    /// Add the trimmed draft text as a single tag.
    Add(String),
    /// Remove the tag identified by its key.
    Remove(K),
}

/// Draft editor and tag requests raised in the current frame.
#[non_exhaustive]
pub struct TagInputResponse<K> {
    /// The native draft editor, including its change and focus state.
    pub response: Response,
    /// Requests for the host to accept or decline.
    pub requests: Vec<TagInputRequest<K>>,
}

/// Editable draft and wrapping row of host-owned [`Chip`] tags.
///
/// Enter or Add tag submits the trimmed draft. Backspace from the empty,
/// focused editor requests removal of the last removable tag. Each remove
/// button also supports native keyboard, pointer and assistive activation.
/// The host applies requests, validates labels, and clears accepted drafts.
#[must_use = "Call show to render TagInput"]
pub struct TagInput<'a, K> {
    id: IdSalt,
    draft: &'a mut String,
    tags: &'a [TagItem<'a, K>],
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    enabled: bool,
}

impl<'a, K: Clone + Hash> TagInput<'a, K> {
    /// Bind a stable editor ID, an application-owned draft and keyed tags.
    pub fn new(id: impl egui::AsIdSalt, draft: &'a mut String, tags: &'a [TagItem<'a, K>]) -> Self {
        Self {
            id: IdSalt::new(id),
            draft,
            tags,
            name: Cow::Borrowed("Tags"),
            placeholder: Cow::Borrowed("Add a tag…"),
            enabled: true,
        }
    }

    /// Name the group and draft editor for assistive technology.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Show a hint while the draft is empty.
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Enable editing and removal, honoring disabled parent UIs.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the current tags and edit the host draft. The tag list changes
    /// when the host applies a request.
    pub fn show(self, ui: &mut Ui) -> TagInputResponse<K> {
        let id = ui.make_persistent_id(("dutchy-tag-input", self.id));
        let enabled = self.enabled && ui.is_enabled();
        let name = self.name.as_ref();
        let group = ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                if !enabled {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing = vec2(space::SM, space::SM);
                let mut requests = Vec::new();
                let removed_focus = show_tags(ui, self.tags, &mut requests);
                let last = self.tags.iter().rfind(|tag| tag.removable);
                let (response, action) = entry(
                    ui,
                    id.with("entry"),
                    self.draft,
                    name,
                    self.placeholder,
                    last.is_some(),
                );
                if removed_focus || action == Some(EntryAction::Add) {
                    response.request_focus();
                }
                match action {
                    Some(EntryAction::Add) => {
                        requests.push(TagInputRequest::Add(self.draft.trim().to_owned()))
                    }
                    Some(EntryAction::RemoveLast) => {
                        if let Some(tag) = last {
                            requests.push(TagInputRequest::Remove(tag.key.clone()));
                        }
                    }
                    None => {}
                }
                TagInputResponse { response, requests }
            },
        );
        group
            .response
            .widget_info(|| WidgetInfo::labeled(Role::Group, enabled, name));
        group.inner
    }
}

fn show_tags<K: Clone + Hash>(
    ui: &mut Ui,
    tags: &[TagItem<'_, K>],
    requests: &mut Vec<TagInputRequest<K>>,
) -> bool {
    if tags.is_empty() {
        return false;
    }
    wrap_chips(ui, |ui| {
        let mut restore_focus = false;
        for tag in tags {
            let mut hash = DefaultHasher::new();
            tag.key.hash(&mut hash);
            let chip = Chip::new(hash.finish(), tag.label.as_ref())
                .removable(tag.removable)
                .show(ui);
            if chip.remove_requested {
                requests.push(TagInputRequest::Remove(tag.key.clone()));
                restore_focus |= chip.remove.as_ref().is_some_and(Response::has_focus);
            }
        }
        restore_focus
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryAction {
    Add,
    RemoveLast,
}

fn entry(
    ui: &mut Ui,
    id: Id,
    draft: &mut String,
    name: &str,
    placeholder: Cow<'_, str>,
    can_remove: bool,
) -> (Response, Option<EntryAction>) {
    let composing = ime_active(ui.ctx());
    let was_empty = draft.is_empty();
    let size = ControlSize::Small;
    let button = Button::new("Add tag")
        .variant(ButtonVariant::Outline)
        .size(size);
    let width = ui.available_width();
    let button_width = button.natural_width(ui);
    let editor_width = width - button_width - space::SM;
    let stacked = editor_width < button_width;
    let (layout, editor_width) = if stacked {
        (Layout::top_down(Align::Min), width)
    } else {
        (Layout::left_to_right(Align::Min), editor_width)
    };
    ui.scope_builder(UiBuilder::new().scope_id(id).layout(layout), |ui| {
        let editor_id = ui.make_persistent_id(IdSalt::new("editor"));
        let focused = ui.is_enabled() && ui.memory(|memory| memory.has_focus(editor_id));
        let response = ui.add(Input::new("editor", draft)
            .accessible_name(name)
            .placeholder(placeholder)
            .size(size)
            .width(editor_width)
            .without_return_key());
        response.ctx.accesskit_node_builder(response.id, |node| {
            node.set_description(if can_remove {
                "Enter adds the draft tag. Backspace in an empty draft removes the last removable tag"
            } else {
                "Enter adds the draft tag"
            });
        });
        let can_add = !composing && !draft.trim().is_empty();
        let keys = if focused && response.has_focus() && !composing {
            // egui can report an empty Backspace as changed. Check the draft itself.
            entry_keys(ui, can_add, can_remove && was_empty && draft.is_empty())
        } else {
            None
        };
        let add = ui.add(button.enabled(can_add).full_width(stacked).shrink_padding_to_fit())
            .accessible_name(format!("Add tag to {name}"));
        let action = if can_add && (add.clicked() || keys == Some(EntryAction::Add)) {
            Some(EntryAction::Add)
        } else {
            keys.filter(|key| *key == EntryAction::RemoveLast)
        };
        (response, action)
    }).inner
}

fn entry_keys(ui: &Ui, can_add: bool, can_remove: bool) -> Option<EntryAction> {
    let mut action = None;
    ui.input_mut(|input| {
        let can_remove = can_remove
            && !input.events.iter().any(
                |event| matches!(event, Event::Text(text) | Event::Paste(text) if !text.is_empty()),
            );
        input.events.retain(|event| {
            let Event::Key {
                key,
                pressed: true,
                repeat,
                modifiers,
                ..
            } = event
            else {
                return true;
            };
            if !modifiers.is_none() {
                return true;
            }
            let next = match key {
                Key::Enter => can_add.then_some(EntryAction::Add),
                Key::Backspace if can_remove => Some(EntryAction::RemoveLast),
                _ => return true,
            };
            if !repeat && action.is_none() {
                action = next;
            }
            false
        })
    });
    action
}
