use crate::{
    Input, SearchState, SelectOption,
    tokens::{border, menu as geometry},
    widgets::search::{
        Cursor, Results, SearchKeys, Selection, input_metadata, option_items, options_popup,
    },
};
use egui::{IdSalt, Response, Ui, UiBuilder};
use std::borrow::Cow;

/// Native input, popup geometry, and explicit committed-selection feedback.
#[non_exhaustive]
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
            let selected = self
                .options
                .iter()
                .find(|option| self.value.as_ref() == Some(&option.value))
                .map(|option| option.id);
            let mut cursor = Cursor::load(ui.ctx(), root);
            let selection = Selection::Single(selected);
            let items = option_items(self.options, selection);
            let shown = options_popup(
                &response,
                root,
                &mut self.state.open,
                Results {
                    items: &items,
                    query: &self.state.query,
                    name: &format!("{} options", self.name),
                    selection,
                    height: geometry::RESULTS_HEIGHT
                        .min((ui.ctx().content_rect().height() - border::STANDARD * 2.0).max(0.0)),
                },
                &mut cursor,
                &keys,
            );
            let (popup, result) = match shown {
                Some(shown) => (Some(shown.response), Some(shown.inner)),
                None => (None, None),
            };
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
