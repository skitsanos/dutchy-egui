mod paint;
mod state;

use super::navigation::{self, Axis};
use crate::{
    Theme,
    tokens::{border, disclosure as geometry},
};
use egui::{Frame, Id, IdSalt, InnerResponse, Margin, Response, Stroke, Ui, UiBuilder, WidgetInfo};
pub use state::AccordionState;
use std::borrow::Cow;

/// One disclosure item. Values must be unique within the accordion.
#[derive(Clone, Debug)]
pub struct AccordionItem<'a, T> {
    /// Host's durable key, also used as the default widget identity.
    pub value: T,
    /// Header text.
    pub label: Cow<'a, str>,
    /// Whether the header and its body controls are enabled.
    pub enabled: bool,
    id: IdSalt,
}

impl<'a, T: egui::AsIdSalt> AccordionItem<'a, T> {
    /// Create an item whose identity is independent of its label and position.
    pub fn new(value: T, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(&value),
            value,
            label: label.into(),
            enabled: true,
        }
    }
    /// Disable this item without changing its expansion state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Accordion surface treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionVariant {
    /// Separate items with a four-point accent edge on hover.
    #[default]
    Default,
    /// One two-point outer border with shared dividers.
    Bordered,
}

/// Responses and callback results, in item order.
pub struct AccordionResponse<R> {
    /// Whole container; changed when a header toggles expansion.
    pub response: Response,
    /// Independently focusable header responses.
    pub headers: Vec<Response>,
    /// Open panel results; closed panels do not call the renderer.
    pub bodies: Vec<Option<InnerResponse<R>>>,
}

/// Controlled disclosure group. The application owns all expansion state.
#[must_use = "Call show to render the accordion"]
pub struct Accordion<'a, T> {
    id: IdSalt,
    state: &'a mut AccordionState<T>,
    items: &'a [AccordionItem<'a, T>],
    variant: AccordionVariant,
    enabled: bool,
    name: Cow<'a, str>,
}

impl<'a, T: Clone + PartialEq> Accordion<'a, T> {
    /// Bind a group ID, host state, and keyed items.
    pub fn new(
        id: impl egui::AsIdSalt,
        state: &'a mut AccordionState<T>,
        items: &'a [AccordionItem<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            state,
            items,
            variant: AccordionVariant::default(),
            enabled: true,
            name: Cow::Borrowed("Accordion"),
        }
    }
    /// Surface treatment.
    pub fn variant(mut self, variant: AccordionVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Group accessibility name.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable all headers and body controls, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Render each open panel once. Enter/Space toggle; Up/Down/Home/End move
    /// among enabled headers. Every header also remains a normal Tab stop.
    pub fn show<R>(
        mut self,
        ui: &mut Ui,
        mut contents: impl FnMut(&mut Ui, &T) -> R,
    ) -> AccordionResponse<R> {
        let id = ui.make_persistent_id(("dutchy-accordion", self.id));
        let content_spacing = ui.spacing().item_spacing;
        let mut changed = false;
        let result = ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                if !self.enabled {
                    ui.disable();
                }
                let ids: Vec<_> = self
                    .items
                    .iter()
                    .filter(|item| item.enabled)
                    .map(|item| id.with(item.id).with("trigger"))
                    .collect();
                navigation::navigate(ui, &ids, Axis::Vertical);
                let theme = Theme::from_context(ui.ctx());
                let frame = if self.variant == AccordionVariant::Bordered && !self.items.is_empty()
                {
                    Frame::NONE.stroke(Stroke::new(border::STANDARD, theme.palette.border))
                } else {
                    Frame::NONE
                };
                frame
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        let mut headers = Vec::with_capacity(self.items.len());
                        let mut bodies = Vec::with_capacity(self.items.len());
                        for (index, item) in self.items.iter().enumerate() {
                            self.separator(ui, index);
                            let result = ui
                                .scope_builder(UiBuilder::new().scope_id(id.with(item.id)), |ui| {
                                    if !item.enabled {
                                        ui.disable();
                                    }
                                    self.item(
                                        ui,
                                        id.with(item.id),
                                        item,
                                        &mut contents,
                                        &mut changed,
                                        content_spacing,
                                    )
                                })
                                .inner;
                            headers.push(result.0);
                            bodies.push(result.1);
                        }
                        (headers, bodies)
                    })
                    .inner
            },
        );
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Group,
                ui.is_enabled() && self.enabled,
                self.name.as_ref(),
            )
        });
        let mut response = result.response;
        if changed {
            response.mark_changed();
            ui.ctx().request_repaint();
        }
        AccordionResponse {
            response,
            headers: result.inner.0,
            bodies: result.inner.1,
        }
    }

    fn separator(&self, ui: &mut Ui, index: usize) {
        if index == 0 {
            return;
        }
        if self.variant == AccordionVariant::Default {
            ui.add_space(geometry::ACCORDION_GAP);
        } else {
            divider(ui, border::STANDARD);
        }
    }

    fn item<R>(
        &mut self,
        ui: &mut Ui,
        id: Id,
        item: &AccordionItem<'_, T>,
        contents: &mut impl FnMut(&mut Ui, &T) -> R,
        changed: &mut bool,
        content_spacing: egui::Vec2,
    ) -> (Response, Option<InnerResponse<R>>) {
        let theme = Theme::from_context(ui.ctx());
        let result = Frame::NONE.fill(theme.palette.background).show(ui, |ui| {
            let mut header = paint::header(
                ui,
                id.with("trigger"),
                &item.label,
                self.state.is_open(&item.value),
                self.variant,
            );
            if header.clicked() {
                self.state.toggle(item.value.clone());
                header.mark_changed();
                header.request_focus();
                *changed = true;
            }
            navigation::lock_arrows(ui, header.id, Axis::Vertical);
            let open = self.state.is_open(&item.value);
            let body = open.then(|| {
                self.body(
                    ui,
                    id.with("panel"),
                    &header,
                    item,
                    contents,
                    content_spacing,
                )
            });
            header.ctx.accesskit_node_builder(header.id, |node| {
                node.set_expanded(open);
                if let Some(body) = &body {
                    node.push_controlled(body.response.id.accesskit_id());
                }
            });
            (header, body)
        });
        if self.variant == AccordionVariant::Default {
            let mut edge = result.response.rect;
            edge.max.x = edge.min.x + border::ACCENT;
            let fill = if result.inner.0.hovered() {
                theme.palette.primary
            } else {
                theme.palette.border
            };
            ui.painter().rect_filled(edge, 0, fill);
        }
        result.inner
    }

    fn body<R>(
        &self,
        ui: &mut Ui,
        id: Id,
        header: &Response,
        item: &AccordionItem<'_, T>,
        contents: &mut impl FnMut(&mut Ui, &T) -> R,
        content_spacing: egui::Vec2,
    ) -> InnerResponse<R> {
        let vertical = if self.variant == AccordionVariant::Bordered {
            geometry::BORDERED_PADDING_Y as i8
        } else {
            0
        };
        if self.variant == AccordionVariant::Bordered {
            divider(ui, border::THIN);
        }
        let result = ui.scope_builder(UiBuilder::new().scope_id(id), |ui| {
            ui.spacing_mut().item_spacing = content_spacing;
            ui.visuals_mut().override_text_color =
                Some(Theme::from_context(ui.ctx()).palette.muted_foreground);
            Frame::NONE
                .inner_margin(Margin {
                    left: geometry::PADDING_X as i8,
                    right: geometry::PADDING_X as i8,
                    top: vertical,
                    bottom: if vertical == 0 {
                        geometry::ACCORDION_PADDING_Y as i8
                    } else {
                        vertical
                    },
                })
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    contents(ui, &item.value)
                })
                .inner
        });
        result.response.widget_info(|| {
            WidgetInfo::labeled(
                egui::accesskit::Role::Region,
                ui.is_enabled(),
                item.label.to_uppercase(),
            )
        });
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.push_labelled_by(header.id.accesskit_id())
            });
        result
    }
}

fn divider(ui: &mut Ui, width: f32) {
    let (_, rect) = ui.allocate_space(egui::vec2(ui.available_width(), width));
    ui.painter()
        .rect_filled(rect, 0, Theme::from_context(ui.ctx()).palette.border);
}
