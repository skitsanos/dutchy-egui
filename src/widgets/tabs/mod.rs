mod bar;
mod focus;
mod item;
mod paint;

use crate::{Theme, interaction::paint_focus, tokens::disclosure as geometry};
use egui::{Frame, Id, IdSalt, Margin, Rect, Response, Sense, Ui, UiBuilder, WidgetInfo};
pub use item::Tab;
use std::borrow::Cow;

/// Tab-strip appearance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// Content navigation with a four-point active underline.
    #[default]
    Underline,
    /// Muted segmented control with an inverted active surface.
    Segmented,
}

/// Selection, close requests, and the active panel's callback result.
pub struct TabsResponse<T, R> {
    /// Whole group; changed when selection changes or is normalized.
    pub response: Response,
    /// Individual tab responses, in source order.
    pub tabs: Vec<Response>,
    /// Visible tab-strip bounds, excluding any horizontal scrollbar.
    pub viewport: Rect,
    /// Active panel response. Inactive panels are not rendered.
    pub panel: Option<Response>,
    /// Active panel's callback result, if an enabled tab exists.
    pub inner: Option<R>,
    /// The host decides whether and when to remove this tab.
    pub close_requested: Option<T>,
}

/// Controlled tab navigation with manual keyboard activation and stable panels.
#[must_use = "Call show to render the tabs"]
pub struct Tabs<'a, T> {
    id: IdSalt,
    selected: &'a mut Option<T>,
    items: &'a [Tab<'a, T>],
    variant: TabsVariant,
    name: Cow<'a, str>,
    enabled: bool,
    full_width: Option<bool>,
    empty_focus: Option<Id>,
}

impl<'a, T: Clone + PartialEq> Tabs<'a, T> {
    /// Bind a stable group ID, host selection, and tabs with unique values.
    /// Missing/disabled selections choose the first enabled tab; none if empty.
    pub fn new(
        id: impl egui::AsIdSalt,
        selected: &'a mut Option<T>,
        items: &'a [Tab<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            selected,
            items,
            variant: TabsVariant::default(),
            name: Cow::Borrowed("Tabs"),
            enabled: true,
            full_width: None,
            empty_focus: None,
        }
    }
    /// Underline or segmented appearance.
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Name of the tab list.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable the tab strip and active panel's controls.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Equal-width tabs filling the strip, scrolling if their labels cannot fit.
    /// Defaults to true for segmented controls and false for underline tabs.
    pub fn full_width(mut self, full: bool) -> Self {
        self.full_width = Some(full);
        self
    }
    /// Focus destination when the focused tab/close button is removed and no
    /// enabled tab remains. The host must keep this target present and enabled.
    pub fn focus_when_empty(mut self, id: Id) -> Self {
        self.empty_focus = Some(id);
        self
    }
    /// Render the strip and only the active panel. Left/Right/Home/End move focus
    /// without selecting; Enter/Space activate. Delete requests closing.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        contents: impl FnOnce(&mut Ui, &T) -> R,
    ) -> TabsResponse<T, R> {
        let root = ui.make_persistent_id(("dutchy-tabs", self.id));
        let content_spacing = ui.spacing().item_spacing;
        let previous = self.selected.clone();
        self.normalize();
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(root)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing.y = 0.0;
                let list = ui.scope_builder(UiBuilder::new().scope_id(root.with("list")), |ui| {
                    bar::show(ui, root, &mut self)
                });
                list.response.widget_info(|| {
                    WidgetInfo::labeled(
                        egui::accesskit::Role::TabList,
                        ui.is_enabled(),
                        self.name.as_ref(),
                    )
                });
                list.response
                    .ctx
                    .accesskit_node_builder(list.response.id, |node| {
                        node.set_orientation(egui::accesskit::Orientation::Horizontal)
                    });
                let panel = self
                    .selected_index()
                    .map(|index| self.panel(ui, root, index, contents, content_spacing));
                (list.inner, panel)
            },
        );
        let mut response = result.response;
        if previous != *self.selected {
            response.mark_changed();
        }
        let (bar, panel) = result.inner;
        let (inner, panel) = panel.map_or((None, None), |result| {
            (Some(result.inner), Some(result.response))
        });
        TabsResponse {
            response,
            tabs: bar.tabs,
            viewport: bar.viewport,
            panel,
            inner,
            close_requested: bar.close.map(|index| self.items[index].value.clone()),
        }
    }

    fn normalize(&mut self) {
        if self.selected_index().is_none() {
            *self.selected = self
                .items
                .iter()
                .find(|item| item.enabled)
                .map(|item| item.value.clone());
        }
    }
    fn selected_index(&self) -> Option<usize> {
        self.items
            .iter()
            .position(|item| item.enabled && self.selected.as_ref() == Some(&item.value))
    }
    fn panel<R>(
        &self,
        ui: &mut Ui,
        root: Id,
        index: usize,
        contents: impl FnOnce(&mut Ui, &T) -> R,
        content_spacing: egui::Vec2,
    ) -> egui::InnerResponse<R> {
        let item = &self.items[index];
        let panel = ui.scope_builder(
            UiBuilder::new()
                .scope_id(focus::panel(root, item.id))
                .sense(Sense::focusable_noninteractive()),
            |ui| {
                ui.spacing_mut().item_spacing = content_spacing;
                Frame::NONE
                    .inner_margin(Margin::same(geometry::PANEL_PADDING))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        contents(ui, &item.value)
                    })
                    .inner
            },
        );
        let trigger = focus::trigger(root, item.id);
        panel.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::TabPanel,
                ui.is_enabled(),
                item.label.to_string(),
            )
        });
        ui.ctx().accesskit_node_builder(panel.response.id, |node| {
            node.push_labelled_by(trigger.accesskit_id())
        });
        ui.ctx().accesskit_node_builder(trigger, |node| {
            node.push_controlled(panel.response.id.accesskit_id())
        });
        paint_focus(
            ui,
            &panel.response,
            Theme::from_context(ui.ctx()).palette.primary,
        );
        panel
    }
}
