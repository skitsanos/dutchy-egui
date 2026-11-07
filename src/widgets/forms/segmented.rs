use super::{SelectOption, style};
use crate::{
    ControlSize, Text, TextColor, Theme, Weight,
    interaction::{focus_is_visible, paint_focus_inside},
    tokens::{border, control},
};
use egui::accesskit::Role;
use egui::{
    Id, IdSalt, Key, Modifiers, Rect, Response, Sense, Stroke, StrokeKind, Ui, UiBuilder, Widget,
    WidgetInfo, pos2, vec2,
};
use std::borrow::Cow;

/// Exclusive typed choice drawn as one bordered row of equal segments.
///
/// A radio group with one tab stop: Left/Right wrap across enabled segments and
/// select as they move; Home/End reach the first and last enabled segment.
#[must_use = "Add this segmented control to a Ui"]
pub struct Segmented<'a, T> {
    id: IdSalt,
    value: &'a mut Option<T>,
    options: &'a [SelectOption<'a, T>],
    name: Cow<'a, str>,
    size: ControlSize,
    enabled: bool,
    full_width: bool,
}

impl<'a, T: Clone + PartialEq> Segmented<'a, T> {
    /// Bind a stable group ID and application-owned selection.
    pub fn new(
        id: impl egui::AsIdSalt,
        value: &'a mut Option<T>,
        options: &'a [SelectOption<'a, T>],
    ) -> Self {
        Self {
            id: IdSalt::new(id),
            value,
            options,
            name: Cow::Borrowed("Options"),
            size: ControlSize::Medium,
            enabled: true,
            full_width: false,
        }
    }
    /// Group name; Field can supply the visible label instead.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Shared control height and type scale.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// Disable the entire group.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Spread the segments across the available width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    fn show(mut self, ui: &mut Ui) -> Response {
        if !self.enabled {
            ui.disable();
        }
        let theme = Theme::from_context(ui.ctx());
        let id = ui.make_persistent_id(self.id);
        let mut changed = self.navigate(ui, id);
        let galleys: Vec<_> = self
            .options
            .iter()
            .map(|option| {
                let selected = self.value.as_ref() == Some(&option.value);
                let job = Text::new(option.label.as_ref())
                    .size(self.size.text_size())
                    .weight(Weight::Bold)
                    .color(if selected {
                        TextColor::Background
                    } else {
                        TextColor::Foreground
                    })
                    .layout_job(&theme, f32::INFINITY);
                ui.fonts_mut(|fonts| fonts.layout_job(job))
            })
            .collect();
        let count = self.options.len().max(1) as f32;
        let natural = galleys
            .iter()
            .map(|galley| galley.size().x + 2.0 * control::SEGMENT_PADDING_X)
            .fold(0.0, f32::max);
        let available = (ui.available_width() - 2.0 * border::STANDARD).max(0.0);
        let segment = if self.full_width {
            available / count
        } else {
            natural.min(available / count)
        };
        let size = vec2(segment * count + 2.0 * border::STANDARD, self.size.height());
        let (outer, mut response) = ui.allocate_exact_size(size, Sense::hover());
        let tab_index = self
            .options
            .iter()
            .position(|o| o.enabled && self.value.as_ref() == Some(&o.value))
            .or_else(|| self.options.iter().position(|o| o.enabled));
        let inner = outer.shrink(border::STANDARD);
        for (index, (option, galley)) in self.options.iter().zip(galleys).enumerate() {
            let rect = Rect::from_min_size(
                pos2(inner.left() + segment * index as f32, inner.top()),
                vec2(segment, inner.height()),
            );
            let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
            if !option.enabled {
                child.disable();
            }
            let selected = self.value.as_ref() == Some(&option.value);
            let sense = if Some(index) == tab_index {
                Sense::click()
            } else {
                Sense::CLICK
            };
            let item = child.interact(rect, id.with(option.id), sense);
            if item.clicked() && !selected {
                *self.value = Some(option.value.clone());
                changed = true;
                item.request_focus();
            }
            item.widget_info(|| {
                WidgetInfo::selected(
                    Role::RadioButton,
                    child.is_enabled(),
                    selected,
                    option.label.as_ref(),
                )
            });
            crate::text::name_as_written(&item, option.label.as_ref());
            if ui.is_rect_visible(rect) {
                paint_segment(&child, &item, rect, galley, selected, index > 0);
            }
        }
        if ui.is_rect_visible(outer) {
            ui.painter().rect_stroke(
                outer,
                0,
                Stroke::new(border::STANDARD, style::inherited_border(ui)),
                StrokeKind::Inside,
            );
        }
        response.widget_info(|| {
            WidgetInfo::labeled(Role::RadioGroup, ui.is_enabled(), self.name.as_ref())
        });
        if changed {
            response.mark_changed();
        }
        response
    }

    fn navigate(&mut self, ui: &mut Ui, id: Id) -> bool {
        if !ui.is_enabled() {
            return false;
        }
        let enabled: Vec<_> = self
            .options
            .iter()
            .filter(|option| option.enabled)
            .collect();
        let focused = ui.memory(|memory| memory.focused());
        let Some(current) = enabled
            .iter()
            .position(|option| focused == Some(id.with(option.id)))
        else {
            return false;
        };
        let direction = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
                Some(1)
            } else if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                Some(-1)
            } else if input.consume_key(Modifiers::NONE, Key::Home) {
                Some(-(current as isize))
            } else if input.consume_key(Modifiers::NONE, Key::End) {
                Some(enabled.len() as isize - 1 - current as isize)
            } else {
                None
            }
        });
        let Some(direction) = direction else {
            return false;
        };
        let next = (current as isize + direction).rem_euclid(enabled.len() as isize) as usize;
        let option = enabled[next];
        ui.memory_mut(|memory| {
            memory.move_focus(egui::FocusDirection::None);
            memory.request_focus(id.with(option.id));
        });
        let changed = self.value.as_ref() != Some(&option.value);
        *self.value = Some(option.value.clone());
        changed
    }
}

fn paint_segment(
    ui: &Ui,
    response: &Response,
    rect: Rect,
    galley: std::sync::Arc<egui::Galley>,
    selected: bool,
    divided: bool,
) {
    let p = Theme::from_context(ui.ctx()).palette;
    let fill = if selected {
        p.foreground
    } else if response.hovered() || focus_is_visible(response) {
        p.muted
    } else {
        p.background
    };
    ui.painter().rect_filled(rect, 0, fill);
    if divided && !selected {
        let divider = Rect::from_min_size(rect.min, vec2(border::THIN, rect.height()));
        ui.painter().rect_filled(divider, 0, p.border);
    }
    ui.painter().galley(
        pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        if selected { p.background } else { p.foreground },
    );
    paint_focus_inside(
        ui,
        response,
        if selected { p.background } else { p.primary },
    );
}

impl<T: Clone + PartialEq> Widget for Segmented<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.scope_builder(UiBuilder::new().id_salt(self.id), |ui| self.show(ui))
            .inner
    }
}
