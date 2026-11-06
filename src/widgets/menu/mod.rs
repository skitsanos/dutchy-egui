mod item;
pub(crate) mod paint;

use crate::{
    MenuWidth, Text, TextColor, TextSize, Theme, Tracking, Weight,
    tokens::{border, menu as geometry},
    widgets::navigation::{self, Axis},
};
use egui::{Id, IdSalt, Response, Ui, UiBuilder, WidgetInfo};
pub use item::{MenuIconPainter, MenuItem, MenuItemVariant};
use std::borrow::Cow;

/// Named action results, with no application action executed by the widget.
pub struct MenuResponse<T> {
    /// Whole menu response.
    pub response: Response,
    /// Action chosen by pointer, Enter, Space, or an assistive click.
    pub action: Option<T>,
    /// Item responses in source order, including disabled items.
    pub items: Vec<Response>,
}

/// Grouped action list with stable item IDs and native interaction.
#[must_use = "Call show to render the menu"]
pub struct Menu<'a, T> {
    id: IdSalt,
    items: &'a [MenuItem<'a, T>],
    width: MenuWidth,
    enabled: bool,
    name: Cow<'a, str>,
}
impl<'a, T: Clone> Menu<'a, T> {
    /// A 320-point menu, constrained to its parent.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [MenuItem<'a, T>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            width: MenuWidth::default(),
            enabled: true,
            name: Cow::Borrowed("Actions"),
        }
    }
    /// Preferred menu width.
    pub fn width(mut self, width: MenuWidth) -> Self {
        self.width = width;
        self
    }
    /// Menu's accessible name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable the menu, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render an inline menu. Up/Down/Home/End move among enabled items.
    /// Shortcut strings are hints; they never register application key bindings.
    pub fn show(self, ui: &mut Ui) -> MenuResponse<T> {
        let root = ui.make_persistent_id(("dutchy-menu", self.id));
        let width = self.width.points().min(ui.available_width());
        self.render(ui, root, Some(width))
    }
    pub(crate) fn show_unframed(self, ui: &mut Ui, root: Id) -> MenuResponse<T> {
        self.render(ui, root, None)
    }

    fn render(self, ui: &mut Ui, root: Id, framed_width: Option<f32>) -> MenuResponse<T> {
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(root)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                ui.spacing_mut().item_spacing.y = 0.0;
                let ids: Vec<_> = self
                    .items
                    .iter()
                    .filter(|item| item.enabled)
                    .map(|item| root.with(item.id))
                    .collect();
                navigation::navigate(ui, &ids, Axis::Vertical);
                if let Some(width) = framed_width {
                    ui.set_width(width.max(0.0));
                    paint::frame(Theme::from_context(ui.ctx()))
                        .show(ui, |ui| {
                            ui.set_width((width - border::STANDARD * 2.0).max(0.0));
                            self.contents(ui, root)
                        })
                        .inner
                } else {
                    self.contents(ui, root)
                }
            },
        );
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Menu,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        MenuResponse {
            response: result.response,
            action: result.inner.0,
            items: result.inner.1,
        }
    }

    fn contents(&self, ui: &mut Ui, root: Id) -> (Option<T>, Vec<Response>) {
        let mut action = None;
        let mut responses = Vec::with_capacity(self.items.len());
        for (group_index, group) in self.items.chunk_by(|a, b| a.group == b.group).enumerate() {
            if group_index > 0 {
                paint::divider(ui, true);
            }
            let result = ui.scope_builder(
                UiBuilder::new().scope_id(root.with(("group", group[0].id))),
                |ui| {
                    if let Some(label) = &group[0].group {
                        egui::Frame::NONE
                            .inner_margin(egui::Margin::symmetric(
                                geometry::PADDING_X as i8,
                                geometry::GROUP_PADDING,
                            ))
                            .show(ui, |ui| {
                                ui.add(
                                    Text::new(label.as_ref())
                                        .size(TextSize::Xs)
                                        .weight(Weight::Bold)
                                        .tracking(Tracking::Wider)
                                        .uppercase(true)
                                        .color(TextColor::Muted),
                                );
                            });
                    }
                    for (index, item) in group.iter().enumerate() {
                        if index > 0 || item.separator_before {
                            paint::divider(ui, item.separator_before);
                        }
                        let response = ui
                            .add_enabled_ui(item.enabled, |ui| {
                                paint::row(
                                    ui,
                                    root.with(item.id),
                                    item,
                                    paint::RowState {
                                        role: egui::accesskit::Role::MenuItem,
                                        active: false,
                                        selected: false,
                                        focusable: true,
                                    },
                                )
                            })
                            .inner;
                        navigation::lock_arrows(ui, response.id, Axis::Vertical);
                        if response.clicked() {
                            action = Some(item.value.clone());
                            response.request_focus();
                        }
                        if response.gained_focus() {
                            response
                                .scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
                        }
                        responses.push(response);
                    }
                },
            );
            if let Some(label) = &group[0].group {
                result.response.widget_info(|| {
                    WidgetInfo::labeled(
                        egui::accesskit::Role::Group,
                        ui.is_enabled(),
                        label.as_ref(),
                    )
                });
            }
        }
        (action, responses)
    }
}
