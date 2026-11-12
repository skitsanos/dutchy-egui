//! Rail, fill, handle geometry and value column shared by Slider and RangeSlider.

use crate::{
    Family, Text, TextColor, TextSize, Theme,
    interaction::paint_focus,
    tokens::{border, control},
};
use egui::layers::ShapeIdx;
use egui::{Align, Id, Layout, Rangef, Rect, Response, Shape, Stroke, TextStyle, Ui, pos2, vec2};
use std::ops::RangeInclusive;

/// Zero-height egui rail (painted here instead), square handle colors, no strokes.
/// Item spacing is cleared so rail, gap and value column add up to the width asked for.
pub(super) fn configure(ui: &mut Ui, theme: Theme, rail_width: f32) {
    let p = theme.palette;
    let spacing = ui.spacing_mut();
    spacing.item_spacing.x = 0.0;
    spacing.slider_width = rail_width;
    spacing.slider_rail_height = 0.0;
    spacing.interact_size.y = control::SLIDER_HEIGHT;
    let visuals = ui.visuals_mut();
    visuals.slider_trailing_fill = false;
    let widgets = &mut visuals.widgets;
    for (state, fill) in [
        (&mut widgets.noninteractive, p.primary),
        (&mut widgets.inactive, p.primary),
        (&mut widgets.hovered, p.foreground),
        (&mut widgets.active, p.foreground),
    ] {
        state.bg_fill = fill;
        state.fg_stroke = Stroke::NONE;
        state.expansion = 0.0;
    }
}

/// Rail and fill shapes reserved below the handles egui paints next.
pub(super) struct Rail {
    rail: ShapeIdx,
    fill: ShapeIdx,
}

impl Rail {
    pub(super) fn reserve(ui: &Ui) -> Self {
        Self {
            rail: ui.painter().add(Shape::Noop),
            fill: ui.painter().add(Shape::Noop),
        }
    }

    /// Paint the 4-point rail across `rect` and the primary fill over `span`.
    pub(super) fn paint(self, ui: &Ui, theme: Theme, rect: Rect, span: Rangef) {
        let p = theme.palette;
        let rail = Rect::from_center_size(rect.center(), vec2(rect.width(), border::ACCENT));
        let fill = Rect::from_x_y_ranges(span, rail.y_range());
        ui.painter()
            .set(self.rail, Shape::rect_filled(rail, 0, p.border));
        ui.painter()
            .set(self.fill, Shape::rect_filled(fill, 0, p.primary));
    }
}

/// The square handle egui paints for `value` inside the control's `rect`.
pub(super) fn handle_rect(rect: Rect, range: &RangeInclusive<f64>, value: f64) -> Rect {
    let radius = handle_radius(rect.height());
    let positions = rect.x_range().shrink(radius);
    let (start, end) = (*range.start(), *range.end());
    let fraction = if start == end {
        0.5
    } else {
        ((value - start) / (end - start)).clamp(0.0, 1.0) as f32
    };
    let center_x = positions.min + (positions.max - positions.min) * fraction;
    Rect::from_center_size(
        pos2(center_x, rect.center().y),
        vec2(2.0 * radius, 2.0 * radius),
    )
}

fn handle_radius(height: f32) -> f32 {
    // egui derives the handle radius from the control height.
    height / 2.5
}

/// Leave room for both handles even when the slider has a narrow column.
pub(super) fn minimum_rail_width(ui: &Ui) -> f32 {
    let height = ui
        .text_style_height(&TextStyle::Body)
        .max(control::SLIDER_HEIGHT);
    4.0 * handle_radius(height)
}

/// Ring the handle while the widget with `id` holds keyboard focus.
pub(super) fn paint_handle_focus(ui: &Ui, control: &Response, id: Id, handle: Rect, theme: Theme) {
    let mut focus = control.clone();
    focus.id = id;
    focus.rect = handle;
    paint_focus(ui, &focus, theme.palette.primary);
}

/// Host-formatted value text in a reserved column, so the rail never resizes.
pub(super) fn value_column(ui: &mut Ui, width: f32, height: f32, text: String) {
    ui.allocate_ui_with_layout(
        vec2(width, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                Text::new(text)
                    .family(Family::Mono)
                    .size(TextSize::Sm)
                    .color(TextColor::Foreground),
            );
        },
    );
}
