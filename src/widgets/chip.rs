use super::close;
use crate::{
    ControlSize, Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus_inside,
    tokens::{border, close as close_tokens, space},
};
use egui::accesskit::{Role, Toggled};
use egui::{
    Align, Galley, IdSalt, Layout, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, pos2, vec2,
};
use std::{borrow::Cow, sync::Arc};

pub(super) fn wrap_chips<R>(ui: &mut Ui, show: impl FnOnce(&mut Ui) -> R) -> R {
    let layout = if ui.layout().prefer_right_to_left() {
        Layout::right_to_left(Align::Min)
    } else {
        Layout::left_to_right(Align::Min)
    };
    // Start one chip high so later rows cannot flow back into the first row.
    let row = vec2(
        ui.available_size_before_wrap().x,
        ControlSize::Small.height(),
    );
    ui.allocate_ui_with_layout(row, layout.with_main_wrap(true), show)
        .inner
}

/// Independent chip controls and the requests raised in this frame.
#[non_exhaustive]
pub struct ChipResponse {
    /// The label or toggle body, excluding the remove button.
    pub response: Response,
    /// The separately focusable remove button, when present.
    pub remove: Option<Response>,
    /// The next selection requested by activating a toggleable body.
    pub toggle_requested: Option<bool>,
    /// Whether the remove button was activated.
    pub remove_requested: bool,
}

/// Compact square label with optional host-owned selection and removal.
///
/// The body and remove button have independent hit targets and stable IDs.
#[must_use = "Call show to render the chip"]
pub struct Chip<'a> {
    id: IdSalt,
    label: Cow<'a, str>,
    selected: Option<bool>,
    removable: bool,
    enabled: bool,
}

impl<'a> Chip<'a> {
    /// Create a passive chip with a stable ID within its parent UI.
    pub fn new(id: impl egui::AsIdSalt, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id: IdSalt::new(id),
            label: label.into(),
            selected: None,
            removable: false,
            enabled: true,
        }
    }

    /// Make the body toggleable and display its current host-owned selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Add a separately named remove button.
    pub fn removable(mut self, removable: bool) -> Self {
        self.removable = removable;
        self
    }

    /// Enable chip actions, honoring disabled ancestors.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the chip and return requests for the host to apply.
    pub fn show(self, ui: &mut Ui) -> ChipResponse {
        let id = ui.make_persistent_id(("dutchy-chip", self.id));
        let theme = Theme::from_context(ui.ctx());
        let height = ControlSize::Small.height();
        let max_width = ui.max_rect().width().max(0.0);
        let remove_width = if self.removable {
            height.min(max_width / 2.0)
        } else {
            0.0
        };
        let galley = self.layout_label(ui, theme, max_width - remove_width - 2.0 * space::MD);
        let width = (height.max(galley.size().x + 2.0 * space::MD) + remove_width).min(max_width);
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let mut child = ui.new_child(UiBuilder::new().scope_id(id).max_rect(rect));
        if !self.enabled {
            child.disable();
        }
        let body_rect =
            Rect::from_min_max(rect.min, pos2(rect.right() - remove_width, rect.bottom()));
        let response = self.body(&child, body_rect, id.with("body"));
        let close_colors = self.paint_body(&child, rect, &response, galley, theme);
        let remove = self.removable.then(|| {
            let remove_rect = Rect::from_min_max(pos2(body_rect.right(), rect.top()), rect.max);
            let button = child.interact(remove_rect, id.with("remove"), Sense::click());
            button.widget_info(|| {
                WidgetInfo::labeled(
                    Role::Button,
                    child.is_enabled(),
                    format!("Remove {}", self.label),
                )
            });
            close::paint_inside(
                &child,
                &button,
                rect.shrink(border::STANDARD),
                close_tokens::BADGE_ICON,
                close_colors,
            );
            button
        });
        ChipResponse {
            toggle_requested: self
                .selected
                .filter(|_| response.clicked())
                .map(|selected| !selected),
            remove_requested: remove.as_ref().is_some_and(Response::clicked),
            response,
            remove,
        }
    }

    fn layout_label(&self, ui: &Ui, theme: Theme, width: f32) -> Arc<Galley> {
        let mut job = Text::new(self.label.as_ref())
            .size(TextSize::Sm)
            .weight(Weight::Bold)
            .tracking(Tracking::Wide)
            .uppercase(true)
            .layout_job(&theme, width.max(0.0));
        job.wrap.max_rows = 1;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    }

    fn body(&self, ui: &Ui, rect: Rect, id: egui::Id) -> Response {
        let (sense, role) = if self.selected.is_some() {
            (Sense::click(), Role::Button)
        } else {
            (Sense::hover(), Role::Label)
        };
        let response = ui.interact(rect, id, sense);
        response.widget_info(|| WidgetInfo::labeled(role, ui.is_enabled(), self.label.as_ref()));
        if let Some(selected) = self.selected {
            response
                .ctx
                .accesskit_node_builder(id, |node| node.set_toggled(Toggled::from(selected)));
        }
        if response.hovered() && self.selected.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response
    }

    fn paint_body(
        &self,
        ui: &Ui,
        rect: Rect,
        response: &Response,
        galley: Arc<Galley>,
        theme: Theme,
    ) -> close::Colors {
        let p = theme.palette;
        let selected = self.selected == Some(true);
        let (fill, ink, edge, focus) = if selected {
            (p.foreground, p.background, p.foreground, p.background)
        } else {
            (p.background, p.foreground, p.border, p.primary)
        };
        if ui.is_rect_visible(rect) {
            ui.painter().rect(
                rect,
                0,
                fill,
                Stroke::new(border::STANDARD, edge),
                StrokeKind::Inside,
            );
            let painter = ui
                .painter()
                .with_clip_rect(ui.clip_rect().intersect(response.rect));
            let hot = self.selected.is_some()
                && (response.hovered() || response.is_pointer_button_down_on());
            if hot {
                let tint = if selected {
                    ink.gamma_multiply(close_tokens::HOVER_TINT)
                } else {
                    p.muted
                };
                painter.rect_filled(rect.shrink(border::STANDARD), 0, tint);
            }
            painter.galley_with_override_text_color(
                pos2(
                    response.rect.left() + space::MD,
                    response.rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                ink,
            );
            paint_focus_inside(ui, response, focus);
        }
        close::Colors {
            idle: ink.gamma_multiply(close_tokens::IDLE_OPACITY),
            foreground: ink,
            focus,
        }
    }
}
