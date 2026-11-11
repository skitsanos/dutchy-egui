use crate::{Chip, ControlSize, tokens::space};
use egui::{Align, IdSalt, Layout, Response, Ui, UiBuilder, accesskit::Role, vec2};
use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

/// One keyed choice in a [`FilterChipGroup`].
#[derive(Clone, Debug)]
#[must_use = "Pass the item to a FilterChipGroup"]
pub struct FilterChipItem<'a, K> {
    key: K,
    label: Cow<'a, str>,
    enabled: bool,
}

impl<'a, K> FilterChipItem<'a, K> {
    /// Create an enabled choice. Keys must be unique within each group.
    pub fn new(key: K, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            key,
            label: label.into(),
            enabled: true,
        }
    }

    /// Enable the choice, subject to the group's and parent UI's enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// A requested change to one filter choice.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FilterChipRequest<K> {
    /// Key of the activated choice.
    pub key: K,
    /// Requested selection after activation.
    pub selected: bool,
}

/// Group response and selection requests for this frame.
#[non_exhaustive]
pub struct FilterChipGroupResponse<K> {
    /// The named region containing the filter chips.
    pub response: Response,
    /// Requested changes in item order. The host decides which to apply.
    pub requests: Vec<FilterChipRequest<K>>,
}

/// A named, wrapping group of toggleable [`Chip`] controls.
///
/// The host owns selection and can apply its own selection and filtering policy.
/// Each enabled chip participates in the native Tab order.
#[must_use = "Call show to render the filter chip group"]
pub struct FilterChipGroup<'a, K> {
    id: IdSalt,
    items: &'a [FilterChipItem<'a, K>],
    name: Cow<'a, str>,
    enabled: bool,
}

impl<'a, K: Clone + Hash> FilterChipGroup<'a, K> {
    /// Bind a stable group ID and a slice of keyed choices.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [FilterChipItem<'a, K>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            name: Cow::Borrowed("Filters"),
            enabled: true,
        }
    }

    /// Name the group for assistive technology.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    /// Enable all otherwise enabled choices, honoring disabled parent UIs.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the host's current selection and report activation requests.
    pub fn show(
        self,
        ui: &mut Ui,
        mut is_selected: impl FnMut(&K) -> bool,
    ) -> FilterChipGroupResponse<K> {
        let id = ui.make_persistent_id(("dutchy-filter-chip-group", self.id));
        let layout = if ui.layout().prefer_right_to_left() {
            Layout::right_to_left(Align::Min)
        } else {
            Layout::left_to_right(Align::Min)
        };
        let enabled = self.enabled && ui.is_enabled();
        let items = self.items;
        let group = ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            if !enabled {
                ui.disable();
            }
            ui.spacing_mut().item_spacing = vec2(space::SM, space::SM);
            // Start with one chip-high row. An unbounded first row would absorb
            // the rows below it, sending later chips back to the top.
            let row = vec2(
                ui.available_size_before_wrap().x,
                ControlSize::Small.height(),
            );
            ui.allocate_ui_with_layout(row, layout.with_main_wrap(true), |ui| {
                chips(ui, items, &mut is_selected)
            })
            .inner
        });
        let name = self.name;
        group
            .response
            .widget_info(|| egui::WidgetInfo::labeled(Role::Group, enabled, name.as_ref()));
        FilterChipGroupResponse {
            response: group.response,
            requests: group.inner,
        }
    }
}

fn chips<K: Clone + Hash>(
    ui: &mut Ui,
    items: &[FilterChipItem<'_, K>],
    is_selected: &mut impl FnMut(&K) -> bool,
) -> Vec<FilterChipRequest<K>> {
    let mut requests = Vec::new();
    for item in items {
        let mut hash = DefaultHasher::new();
        item.key.hash(&mut hash);
        let chip = Chip::new(hash.finish(), item.label.as_ref())
            .selected(is_selected(&item.key))
            .enabled(item.enabled)
            .show(ui);
        if let Some(selected) = chip.toggle_requested {
            requests.push(FilterChipRequest {
                key: item.key.clone(),
                selected,
            });
        }
    }
    requests
}
