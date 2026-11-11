use crate::{
    Chip, Input, SearchState, SelectOption,
    commands::dispatch::ime_active,
    tokens::{border, menu as geometry, space},
    widgets::{
        chip::wrap_chips,
        search::{
            Cursor, Results, SearchKeys, Selection, input_metadata, option_items, options_popup,
        },
    },
};
use egui::{
    Align, Event, Id, IdSalt, Key, Layout, Modifiers, Response, Ui, UiBuilder, WidgetInfo,
    accesskit::Role, vec2,
};
use std::{borrow::Cow, collections::HashSet};

/// One requested change to the host's selection.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct MultiSelectRequest<T> {
    /// Option whose selection was requested.
    pub value: T,
    /// Whether the host should select or remove this option.
    pub selected: bool,
}

/// Native query response, popup geometry, and requests from this frame.
#[non_exhaustive]
pub struct MultiSelectResponse<T> {
    /// Editable query, with native focus and change information.
    pub response: Response,
    /// Visible result popup, if rendered this frame.
    pub popup: Option<Response>,
    /// Requests the host can accept or decline.
    pub requests: Vec<MultiSelectRequest<T>>,
}

/// Searchable choice list with removable chips for the current selection.
///
/// The host owns the selection, query, and popup flag. Result activation
/// requests a toggle and leaves the query and popup open for further choices.
/// Options must have unique values and stable, unique salts. Chips appear
/// below the query in option order, so a choice never moves the query or the
/// open results. Selected values missing from the catalog remain with the host.
#[must_use = "Call show to render the multi-select"]
pub struct MultiSelect<'a, T> {
    id: IdSalt,
    selected: &'a [T],
    state: &'a mut SearchState,
    options: &'a [SelectOption<'a, T>],
    name: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    enabled: bool,
}

impl<'a, T: Clone + PartialEq> MultiSelect<'a, T> {
    /// Bind the host's selection and search state to a stable widget ID.
    pub fn new(
        id: impl egui::AsIdSalt,
        selected: &'a [T],
        state: &'a mut SearchState,
        options: &'a [SelectOption<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            selected,
            state,
            options,
            name: Cow::Borrowed("Select options"),
            placeholder: Cow::Borrowed("Search options…"),
            enabled: true,
        }
    }

    /// Set the accessible name of the group and query.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the query hint.
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Enable interaction, honoring disabled ancestors.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the host selection and return toggle and remove requests.
    pub fn show(mut self, ui: &mut Ui) -> MultiSelectResponse<T> {
        let root = ui.make_persistent_id(("dutchy-multi-select", self.id));
        let enabled = self.enabled && ui.is_enabled();
        let name = self.name.clone();
        let group = ui.scope_builder(
            UiBuilder::new()
                .scope_id(root)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                if !enabled {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing = vec2(space::SM, space::SM);
                self.render(ui, root)
            },
        );
        group
            .response
            .widget_info(|| WidgetInfo::labeled(Role::Group, enabled, name.as_ref()));
        group.inner
    }

    fn render(&mut self, ui: &mut Ui, root: Id) -> MultiSelectResponse<T> {
        let chosen: Vec<_> = self
            .options
            .iter()
            .filter(|option| self.selected.contains(&option.value))
            .collect();
        let input_id = ui.make_persistent_id(IdSalt::new("query"));
        let (keys, enter_opens) = query_keys(ui, input_id, self.state.open);
        let response = ui.add(
            Input::new("query", &mut self.state.query)
                .accessible_name(self.name.as_ref())
                .placeholder(self.placeholder.as_ref())
                .without_return_key(),
        );
        if response.clicked() || response.changed() || keys.opens() || enter_opens {
            self.state.open = true;
        }
        if !ui.is_enabled() || keys.escape || keys.tab {
            self.state.open = false;
        }

        let mut cursor = Cursor::load(ui.ctx(), root);
        let mut requests = Vec::new();
        let shown = self.popup(&response, root, &chosen, &mut cursor, &keys);
        let (popup, list, active) = if let Some(shown) = shown {
            if let Some(value) = shown.inner.action {
                let selected = !self.selected.contains(&value);
                requests.push(MultiSelectRequest { value, selected });
            }
            (
                Some(shown.response),
                Some(shown.inner.response.id),
                shown.inner.active,
            )
        } else {
            (None, None, None)
        };
        // Chips follow the query so a choice never moves it or the open results.
        let removed = chips(ui, &chosen);
        if !removed.is_empty() {
            self.state.open = false;
        }
        requests.extend(removed);
        if keys.escape || !requests.is_empty() {
            response.request_focus();
        }
        input_metadata(&response, self.state.open, list, active);
        cursor.was_open = self.state.open;
        cursor.store(ui.ctx(), root);
        MultiSelectResponse {
            response,
            popup,
            requests,
        }
    }

    fn popup(
        &mut self,
        response: &Response,
        root: Id,
        chosen: &[&SelectOption<'_, T>],
        cursor: &mut Cursor,
        keys: &SearchKeys,
    ) -> Option<egui::InnerResponse<crate::widgets::search::ResultsResponse<T>>> {
        if !self.state.open {
            return None;
        }
        let ids: HashSet<_> = chosen.iter().map(|option| option.id).collect();
        let selection = Selection::Multiple(&ids);
        let items = option_items(self.options, selection);
        options_popup(
            response,
            root,
            &mut self.state.open,
            Results {
                items: &items,
                query: &self.state.query,
                name: &format!("{} options", self.name),
                selection,
                height: geometry::RESULTS_HEIGHT
                    .min((response.ctx.content_rect().height() - border::STANDARD * 2.0).max(0.0)),
            },
            cursor,
            keys,
        )
    }
}

fn chips<T: Clone>(ui: &mut Ui, chosen: &[&SelectOption<'_, T>]) -> Vec<MultiSelectRequest<T>> {
    if chosen.is_empty() {
        return Vec::new();
    }
    wrap_chips(ui, |ui| {
        let mut requests = Vec::new();
        for option in chosen {
            let chip = Chip::new(option.id, option.label.as_ref())
                .removable(option.enabled)
                .show(ui);
            if chip.remove_requested {
                requests.push(MultiSelectRequest {
                    value: option.value.clone(),
                    selected: false,
                });
            }
        }
        requests
    })
}

fn query_keys(ui: &Ui, input_id: Id, open: bool) -> (SearchKeys, bool) {
    if !ui.is_enabled() || ime_active(ui.ctx()) {
        return (SearchKeys::default(), false);
    }
    // Holding Enter must not toggle the same option repeatedly.
    let enter = ui.input(|input| {
        input.events.iter().any(|event| {
            matches!(event,
                Event::Key { key: Key::Enter, pressed: true, repeat: false, modifiers, .. }
                if modifiers.is_none()
            )
        })
    });
    let mut keys = SearchKeys::take(ui, input_id, open);
    keys.activate &= enter;
    let enter_opens = !open
        && enter
        && ui.memory(|memory| memory.has_focus(input_id))
        && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
    (keys, enter_opens)
}
