use super::{
    Step, StepperVariant,
    state::{Model, Status},
};
use crate::{
    Text, TextSize, Theme, Tooltip, Tracking, Weight,
    interaction::paint_focus,
    tokens::{border, stepper as geometry},
};
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, WidgetInfo, pos2, vec2};
use std::sync::Arc;

pub(super) struct Content {
    title: Option<Arc<egui::Galley>>,
    detail: Option<Arc<egui::Galley>>,
    pub size: egui::Vec2,
}
pub(super) struct Cell<'a, 'b, T> {
    pub index: usize,
    pub step: &'a Step<'b, T>,
    pub model: &'a Model,
    pub variant: StepperVariant,
}
fn text(
    ui: &Ui,
    label: &str,
    width: f32,
    size: TextSize,
    bold: bool,
    caps: bool,
) -> Arc<egui::Galley> {
    let job = Text::new(label)
        .size(size)
        .weight(if bold { Weight::Bold } else { Weight::Regular })
        .tracking(if caps {
            Tracking::Wide
        } else {
            Tracking::Normal
        })
        .uppercase(caps)
        .layout_job(&Theme::from_context(ui.ctx()), width.max(0.0));
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}
impl<T> Cell<'_, '_, T> {
    pub fn measure(&self, ui: &Ui, width: f32) -> Content {
        if self.variant == StepperVariant::Compact {
            return Content {
                title: None,
                detail: None,
                size: egui::Vec2::splat(geometry::COMPACT_INDICATOR),
            };
        }
        let vertical = self.variant == StepperVariant::Vertical;
        let text_width = if vertical {
            width - geometry::INDICATOR - geometry::TEXT_GAP
        } else {
            width - geometry::COLUMN_PADDING * 2.0
        };
        let title = text(
            ui,
            &self.step.label,
            text_width,
            if vertical { TextSize::Sm } else { TextSize::Xs },
            self.variant != StepperVariant::Progress
                || self.model.status(self.index) == Status::Current,
            true,
        );
        let detail = if vertical {
            self.step
                .description
                .as_ref()
                .map(|detail| text(ui, detail, text_width, TextSize::Sm, false, false))
        } else {
            None
        };
        let height = match self.variant {
            StepperVariant::Horizontal => {
                geometry::INDICATOR + geometry::LABEL_GAP + title.size().y
            }
            StepperVariant::Vertical => geometry::INDICATOR.max(
                title.size().y
                    + detail
                        .as_ref()
                        .map_or(0.0, |detail| geometry::DESCRIPTION_GAP + detail.size().y),
            ),
            _ => geometry::COMPACT_INDICATOR.max(title.size().y),
        };
        Content {
            title: Some(title),
            detail,
            size: vec2(width, height),
        }
    }
    pub fn show(&self, ui: &mut Ui, root: Id, width: f32) -> Response {
        let content = self.measure(ui, width);
        ui.allocate_ui_with_layout(
            content.size,
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let item = ui.scope_builder(
                    UiBuilder::new().scope_id(root.with(("list-item", self.step.id))),
                    |ui| {
                        if !self.step.enabled {
                            ui.disable();
                        }
                        let (_, rect) = ui.allocate_space(content.size);
                        let response = ui.interact(
                            rect,
                            root.with(self.step.id),
                            if self.step.navigable {
                                Sense::click()
                            } else {
                                Sense::hover()
                            },
                        );
                        self.metadata(&response);
                        self.draw(ui, &response, &content);
                        if self.step.navigable {
                            paint_focus(
                                ui,
                                &response,
                                Theme::from_context(ui.ctx()).palette.primary,
                            );
                        }
                        if self.variant == StepperVariant::Compact {
                            Tooltip::new(format!(
                                "{} — {}",
                                self.step.label,
                                self.model.status(self.index).name()
                            ))
                            .show(&response);
                        }
                        if response.gained_focus() {
                            response
                                .scroll_to_me_animation(None, egui::style::ScrollAnimation::none());
                        }
                        if self.step.navigable {
                            response.on_hover_cursor(egui::CursorIcon::PointingHand)
                        } else {
                            response
                        }
                    },
                );
                item.response.widget_info(|| {
                    WidgetInfo::labeled(egui::accesskit::Role::ListItem, ui.is_enabled(), "")
                });
                ui.ctx().accesskit_node_builder(item.response.id, |node| {
                    node.set_position_in_set(self.index)
                });
                item.inner
            },
        )
        .inner
    }
    fn metadata(&self, response: &Response) {
        response.widget_info(|| {
            WidgetInfo::labeled(
                if self.step.navigable {
                    egui::accesskit::Role::Button
                } else {
                    egui::accesskit::Role::Label
                },
                response.enabled(),
                self.step.label.as_ref(),
            )
        });
        response.ctx.accesskit_node_builder(response.id, |node| {
            let state = self.model.status(self.index);
            node.set_description(format!(
                "{}. Step {} of {}. {}",
                state.name(),
                self.index + 1,
                self.model.total,
                self.step.description.as_deref().unwrap_or("")
            ));
            if state == Status::Current {
                node.set_aria_current(egui::accesskit::AriaCurrent::Step);
            }
        });
    }
    fn draw(&self, ui: &Ui, response: &Response, content: &Content) {
        let rect = response.rect;
        if !ui.is_rect_visible(rect) {
            return;
        }
        let p = Theme::from_context(ui.ctx()).palette;
        let status = self.model.status(self.index);
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
        if self.step.navigable && response.enabled() && response.hovered() {
            let width = if response.is_pointer_button_down_on() {
                border::STANDARD
            } else {
                border::THIN
            };
            painter.rect_stroke(rect, 0, Stroke::new(width, p.primary), StrokeKind::Inside);
        }
        if self.variant != StepperVariant::Progress {
            indicator(
                &painter,
                self.marker(rect),
                status,
                self.index + 1,
                self.variant,
                ui,
            );
        }
        let color = if status == Status::Upcoming {
            p.muted_foreground
        } else {
            p.foreground
        };
        if let Some(title) = &content.title {
            let position = match self.variant {
                StepperVariant::Horizontal => pos2(
                    rect.center().x - title.size().x / 2.0,
                    rect.top() + geometry::INDICATOR + geometry::LABEL_GAP,
                ),
                StepperVariant::Vertical => pos2(
                    rect.left() + geometry::INDICATOR + geometry::TEXT_GAP,
                    rect.top(),
                ),
                _ => rect.center() - title.size() / 2.0,
            };
            painter.galley_with_override_text_color(position, title.clone(), color);
            if let Some(detail) = &content.detail {
                painter.galley_with_override_text_color(
                    position + vec2(0.0, title.size().y + geometry::DESCRIPTION_GAP),
                    detail.clone(),
                    p.muted_foreground,
                );
            }
        }
    }
    pub fn marker(&self, rect: Rect) -> Rect {
        let size = if self.variant == StepperVariant::Compact {
            geometry::COMPACT_INDICATOR
        } else {
            geometry::INDICATOR
        };
        let min = if self.variant == StepperVariant::Vertical {
            rect.min
        } else {
            pos2(rect.center().x - size / 2.0, rect.top())
        };
        Rect::from_min_size(min, egui::Vec2::splat(size))
    }
}
fn indicator(
    painter: &egui::Painter,
    rect: Rect,
    status: Status,
    index: usize,
    variant: StepperVariant,
    ui: &Ui,
) {
    let p = Theme::from_context(ui.ctx()).palette;
    let (fill, color, stroke) = match status {
        Status::Completed => (p.primary, p.on_primary, Stroke::NONE),
        Status::Current if variant == StepperVariant::Horizontal => {
            (p.primary, p.on_primary, Stroke::NONE)
        }
        Status::Current => (
            p.background,
            p.primary,
            Stroke::new(border::ACCENT, p.primary),
        ),
        Status::Upcoming if variant == StepperVariant::Horizontal => {
            (p.muted, p.muted_foreground, Stroke::NONE)
        }
        Status::Upcoming => (
            p.background,
            p.muted_foreground,
            Stroke::new(border::STANDARD, p.border),
        ),
    };
    painter.rect(rect, 0, fill, stroke, StrokeKind::Inside);
    if status == Status::Completed && variant != StepperVariant::Compact {
        let icon = Rect::from_center_size(rect.center(), egui::Vec2::splat(geometry::CHECK));
        painter.add(egui::Shape::line(
            vec![
                pos2(icon.left(), icon.center().y),
                pos2(icon.center().x, icon.bottom()),
                pos2(icon.right(), icon.top()),
            ],
            Stroke::new(border::STANDARD, color),
        ));
    } else {
        let galley = text(
            ui,
            &index.to_string(),
            rect.width(),
            if variant == StepperVariant::Compact {
                TextSize::Xs
            } else {
                TextSize::Sm
            },
            true,
            false,
        );
        painter.galley_with_override_text_color(rect.center() - galley.size() / 2.0, galley, color);
    }
}
