use crate::{
    Input, MenuItem, SearchState, SelectOption, Theme,
    tokens::{border, menu as geometry},
    widgets::{
        menu::paint,
        search::{Cursor, Results, SearchKeys, input_metadata},
    },
};
use egui::{IdSalt, Response, Ui, UiBuilder};
use std::borrow::Cow;

/// Native input, popup geometry, and explicit committed-selection feedback.
pub struct SearchableSelectResponse {
    /// Editable query response; changed includes query editing.
    pub response: Response,
    /// Whether a choice was committed, even if it matched the previous value.
    pub committed: bool,
    /// Whether the host's selected value changed.
    pub selection_changed: bool,
    /// Visible popup geometry, if shown.
    pub popup: Option<Response>,
}

/// Editable combobox with a host-owned query, open flag, and typed selection.
#[must_use = "Call show to render the searchable select"]
pub struct SearchableSelect<'a, T> {
    id: IdSalt,
    value: &'a mut Option<T>,
    state: &'a mut SearchState,
    options: &'a [SelectOption<'a, T>],
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    enabled: bool,
}
impl<'a, T: Clone + PartialEq> SearchableSelect<'a, T> {
    /// Bind stable identity and host-owned state. Typing filters without selecting.
    pub fn new(
        id: impl egui::AsIdSalt,
        value: &'a mut Option<T>,
        state: &'a mut SearchState,
        options: &'a [SelectOption<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            state,
            options,
            name: Cow::Borrowed("Search options"),
            placeholder: Cow::Borrowed("Search options…"),
            enabled: true,
        }
    }
    /// Input accessibility name; Field can also supply its visible label.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Empty-query hint.
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    /// Disable query editing and results, closing the popup.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Up/Down/Home/End choose an active result; Enter commits. Escape dismisses
    /// without selecting, and Tab exits normally. No missing value is normalized.
    pub fn show(self, ui: &mut Ui) -> SearchableSelectResponse {
        let root = ui.make_persistent_id(("dutchy-searchable-select", self.id));
        let previous = self.value.clone();
        ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            let keys = SearchKeys::take(
                ui,
                ui.make_persistent_id(IdSalt::new("query")),
                self.state.open,
            );
            let response = ui.add(
                Input::new("query", &mut self.state.query)
                    .accessible_name(self.name.as_ref())
                    .placeholder(self.placeholder.as_ref()),
            );
            if response.clicked() || response.changed() || keys.opens() {
                self.state.open = true;
            }
            if !ui.is_enabled() || keys.escape || keys.tab {
                self.state.open = false;
            }
            let items: Vec<_> = self
                .options
                .iter()
                .map(|option| MenuItem {
                    checked: None,
                    children: None,
                    value: option.value.clone(),
                    label: option.label.clone(),
                    group: None,
                    shortcut: None,
                    variant: crate::MenuItemVariant::Default,
                    enabled: option.enabled,
                    separator_before: false,
                    icon: None,
                    id: option.id,
                })
                .collect();
            let selected = self
                .options
                .iter()
                .find(|option| self.value.as_ref() == Some(&option.value))
                .map(|option| option.id);
            let mut cursor = Cursor::load(ui.ctx(), root);
            let mut result = None;
            let mut popup = None;
            if self.state.open {
                let shown = egui::Popup::from_response(&response)
                    .id(root.with("popup"))
                    .open_bool(&mut self.state.open)
                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                    .gap(geometry::POPUP_GAP)
                    .frame(paint::frame(Theme::from_context(ui.ctx())))
                    .width(response.rect.width())
                    .accessible_name(format!("{} options", self.name))
                    .show(|ui| {
                        ui.set_width((response.rect.width() - border::STANDARD * 2.0).max(0.0));
                        Results {
                            items: &items,
                            query: &self.state.query,
                            name: &format!("{} options", self.name),
                            selected,
                            height: geometry::RESULTS_HEIGHT.min(
                                (ui.ctx().content_rect().height() - border::STANDARD * 2.0)
                                    .max(0.0),
                            ),
                        }
                        .show(ui, root.with("results"), &mut cursor, &keys)
                    });
                if let Some(shown) = shown {
                    popup = Some(shown.response);
                    result = Some(shown.inner);
                }
            }
            let committed = result
                .as_ref()
                .is_some_and(|result| result.action.is_some());
            if let Some(result) = &result
                && let Some(value) = &result.action
            {
                *self.value = Some(value.clone());
                self.state.query = result.label.clone().unwrap_or_default();
                self.state.open = false;
                response.request_focus();
                ui.ctx().request_repaint();
            }
            if keys.escape {
                response.request_focus();
            }
            input_metadata(
                &response,
                self.state.open,
                result.as_ref().map(|r| r.response.id),
                result.as_ref().and_then(|r| r.active),
            );
            cursor.was_open = self.state.open;
            cursor.store(ui.ctx(), root);
            SearchableSelectResponse {
                response,
                committed,
                selection_changed: previous != *self.value,
                popup,
            }
        })
        .inner
    }
}
