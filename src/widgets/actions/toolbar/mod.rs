pub(crate) mod focus;
mod item;
mod popup;
use super::core;
use crate::{
    ControlSize, Theme,
    interaction::reveal_focused,
    tokens::{actions as tokens, border},
    widgets::navigation::{self, Axis},
};
use egui::{
    Id, IdSalt, Rect, Response, Sense, Stroke, Ui, UiBuilder, WidgetInfo,
    accesskit::{Orientation, Role},
    vec2,
};
pub use item::{ToolbarEvent, ToolbarItem};
use std::borrow::Cow;

/// Toolbar's layout and arrow-key navigation axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolbarOrientation {
    /// Left/Right navigation, with horizontal scrolling or optional wrapping.
    #[default]
    Horizontal,
    /// Up/Down navigation and vertical scrolling.
    Vertical,
}
/// Toolbar geometry and a single independent host event.
#[non_exhaustive]
pub struct ToolbarResponse<T> {
    /// Named Toolbar container.
    pub response: Response,
    /// Visible content viewport, excluding scrollbar chrome.
    pub viewport: Rect,
    /// Item keys and native responses, including disabled items.
    pub items: Vec<(T, Response)>,
    /// Application action or controlled toggle request.
    pub event: Option<ToolbarEvent<T>>,
}

/// Typed action/toggle group with one remembered Tab stop and roving focus.
/// Arrow/Home/End navigation never activates items. Hosts own all action state.
#[must_use = "Call show to render the toolbar"]
pub struct Toolbar<'a, T> {
    id: IdSalt,
    items: &'a [ToolbarItem<'a, T>],
    name: Cow<'a, str>,
    orientation: ToolbarOrientation,
    size: ControlSize,
    wrap: bool,
    width: Option<f32>,
    max_height: Option<f32>,
    enabled: bool,
    bordered: bool,
    fallback: Option<Id>,
}
impl<'a, T: Clone> Toolbar<'a, T> {
    /// Bind stable identity and items with unique IDs; Small horizontal defaults.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [ToolbarItem<'a, T>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            name: "Tools".into(),
            orientation: ToolbarOrientation::Horizontal,
            size: ControlSize::Small,
            wrap: false,
            width: None,
            max_height: None,
            enabled: true,
            bordered: true,
            fallback: None,
        }
    }
    /// Group name announced to assistive technology.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Layout and keyboard axis.
    pub fn orientation(mut self, orientation: ToolbarOrientation) -> Self {
        self.orientation = orientation;
        self
    }
    /// Shared item height and typography.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Wrap horizontal items instead of scrolling; navigation remains linear.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    /// Preferred outer width; vertical defaults fit an icon target and padding.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.is_finite().then_some(width.max(0.0));
        self
    }
    /// Bound a vertical toolbar's content viewport and enable overflow scrolling.
    /// Without a bound, vertical tools reserve their natural column height.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height.is_finite().then_some(height.max(0.0));
        self
    }
    /// Disable all items without enabling a disabled parent.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Show/remove the surrounding frame border.
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }
    /// Surviving host target when the focused item is removed and no enabled
    /// item remains. Call the toolbar while empty to apply this policy.
    pub fn focus_when_empty(mut self, id: Id) -> Self {
        self.fallback = Some(id);
        self
    }

    /// Render requests. Tab exits normally; remembered reentry is independent of
    /// toggle state. Disabled/busy entries are skipped by arrow navigation.
    pub fn show(self, ui: &mut Ui) -> ToolbarResponse<T> {
        let root = ui.make_persistent_id(("dutchy-toolbar", self.id));
        let mut memory = ui
            .ctx()
            .data(|d| d.get_temp::<focus::Memory>(root))
            .unwrap_or_default();
        let result = ui.scope_builder(UiBuilder::new().scope_id(root), |ui| {
            if !self.enabled {
                ui.disable();
            }
            let available = ui.available_width();
            let default = if self.orientation == ToolbarOrientation::Vertical {
                tokens::vertical_width(self.size)
            } else {
                available
            };
            let width = self.width.unwrap_or(default).min(available).max(0.0);
            ui.set_width(width);
            let p = Theme::from_context(ui.ctx()).palette;
            egui::Frame::NONE
                .fill(p.background)
                .stroke(if self.bordered {
                    Stroke::new(border::STANDARD, p.border)
                } else {
                    Stroke::NONE
                })
                .inner_margin(egui::Margin::same(tokens::PADDING as i8))
                .show(ui, |ui| {
                    // This group owns its background, including inside inverse cards.
                    ui.visuals_mut().override_text_color = None;
                    ui.set_min_width(ui.available_width());
                    let ids: Vec<_> = self
                        .items
                        .iter()
                        .filter(|i| i.interactive() && ui.is_enabled())
                        .map(|i| root.with(i.id))
                        .collect();
                    let axis = if self.orientation == ToolbarOrientation::Horizontal {
                        Axis::Horizontal
                    } else {
                        Axis::Vertical
                    };
                    popup::tab_exit(ui, &mut memory);
                    memory.prepare(ui, &ids, axis, self.fallback);
                    let result = self.layout(ui, root, &mut memory, axis);
                    memory.remember(ui, &ids);
                    result
                })
                .inner
        });
        result.response.widget_info(|| {
            WidgetInfo::labeled(Role::Toolbar, result.response.enabled(), self.name.as_ref())
        });
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.set_orientation(if self.orientation == ToolbarOrientation::Horizontal {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                })
            });
        ui.ctx().data_mut(|d| d.insert_temp(root, memory));
        ToolbarResponse {
            response: result.response,
            viewport: result.inner.0,
            items: result.inner.1,
            event: result.inner.2,
        }
    }

    fn layout(
        &self,
        ui: &mut Ui,
        root: Id,
        memory: &mut focus::Memory,
        axis: Axis,
    ) -> (Rect, Vec<(T, Response)>, Option<ToolbarEvent<T>>) {
        ui.spacing_mut().item_spacing = vec2(tokens::GAP, tokens::GAP);
        ui.spacing_mut().interact_size.y = self.size.height();
        if self.orientation == ToolbarOrientation::Horizontal && self.wrap {
            let out = ui.horizontal_wrapped(|ui| self.contents(ui, root, memory, axis));
            (out.response.rect, out.inner.0, out.inner.1)
        } else if self.orientation == ToolbarOrientation::Horizontal {
            let out = egui::ScrollArea::horizontal()
                .id_salt("overflow")
                .content_margin(egui::Margin::ZERO)
                .max_height(self.size.height())
                .show(ui, |ui| {
                    ui.horizontal(|ui| self.contents(ui, root, memory, axis))
                        .inner
                });
            (out.inner_rect, out.inner.0, out.inner.1)
        } else {
            let height = self.vertical_height();
            let out = egui::ScrollArea::vertical()
                .id_salt("overflow")
                .content_margin(egui::Margin::ZERO)
                .min_scrolled_height(height)
                .max_height(height)
                .show(ui, |ui| {
                    ui.vertical(|ui| self.contents(ui, root, memory, axis))
                        .inner
                });
            let viewport = out.inner_rect.intersect(Rect::from_min_size(
                out.inner_rect.min,
                vec2(out.inner_rect.width(), out.content_size.y),
            ));
            (viewport, out.inner.0, out.inner.1)
        }
    }
    fn vertical_height(&self) -> f32 {
        let separators = self
            .items
            .iter()
            .skip(1)
            .filter(|i| i.separator_before)
            .count();
        let natural = self.items.len() as f32 * self.size.height()
            + self.items.len().saturating_sub(1) as f32 * tokens::GAP
            + separators as f32 * (tokens::SEPARATOR + tokens::GAP);
        self.max_height.unwrap_or(natural).min(natural)
    }
    fn contents(
        &self,
        ui: &mut Ui,
        root: Id,
        memory: &mut focus::Memory,
        axis: Axis,
    ) -> (Vec<(T, Response)>, Option<ToolbarEvent<T>>) {
        let anchor = ui.max_rect().min;
        let mut items = Vec::new();
        let mut event = None;
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 && item.separator_before {
                self.separator(ui);
            }
            let id = root.with(item.id);
            let response = core::show(
                ui,
                core::Action {
                    id,
                    label: &item.label,
                    icon: item.icon,
                    icon_only: item.icon_only,
                    pressed: item.pressed,
                    variant: item.variant,
                    size: self.size,
                    enabled: item.interactive(),
                    loading: item.loading,
                    tooltip: item.tooltip.as_deref(),
                    inside_focus: true,
                    menu: item.menu.is_some(),
                    role: egui::accesskit::Role::Button,
                },
                if memory.tab == Some(id) {
                    Sense::click()
                } else {
                    Sense::CLICK
                },
                None,
            );
            if response.has_focus() {
                if item.menu.is_some() {
                    crate::widgets::menu::memory::lock(&response, false);
                } else {
                    navigation::lock_arrows(ui, id, axis);
                }
            }
            reveal_focused(ui, &response, anchor);
            if item.menu.is_some() {
                response.ctx.accesskit_node_builder(id, |n| {
                    n.set_has_popup(egui::accesskit::HasPopup::Menu);
                    n.set_expanded(memory.open == Some(id));
                });
            }
            if response.clicked() && event.is_none() && item.menu.is_none() {
                event = Some(match item.pressed {
                    Some(pressed) => ToolbarEvent::Toggled {
                        value: item.value.clone(),
                        pressed: !pressed,
                    },
                    None => ToolbarEvent::Activated(item.value.clone()),
                });
            }
            items.push((item.value.clone(), response));
        }
        let menu_event = popup::show(ui, root, self.items, &items, memory);
        (items, event.or(menu_event))
    }
    fn separator(&self, ui: &mut Ui) {
        let vertical = self.orientation == ToolbarOrientation::Vertical;
        let size = if vertical {
            vec2(ui.available_width(), tokens::SEPARATOR)
        } else {
            vec2(tokens::SEPARATOR, self.size.height())
        };
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let span = if vertical {
            vec2(rect.width(), border::THIN)
        } else {
            vec2(border::THIN, (rect.height() - 2.0 * tokens::GAP).max(0.0))
        };
        ui.painter().rect_filled(
            Rect::from_center_size(rect.center(), span),
            0,
            Theme::from_context(ui.ctx()).palette.border,
        );
    }
}
