use super::geometry::{Axis, Bounds};
use crate::{
    Theme,
    interaction::paint_focus_inside,
    tokens::{border, split_pane as tokens},
};
use egui::{
    Id, Response, Stroke, Ui,
    accesskit::{Action, Orientation, Role},
    vec2,
};

pub(super) struct Info<'a> {
    pub name: &'a str,
    pub first: Id,
    pub axis: Axis,
    pub bounds: Bounds,
    pub value: f32,
    pub read_only: bool,
}
pub(super) fn metadata(ui: &Ui, response: &Response, info: Info<'_>) {
    let Info {
        name,
        first,
        axis,
        bounds,
        value,
        read_only,
    } = info;
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::new(Role::Splitter);
        info.enabled = response.enabled();
        info.label = Some(name.to_owned());
        info.value = Some(value as f64);
        info
    });
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(Role::Splitter);
        node.set_orientation(if axis == Axis::Horizontal {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        });
        node.set_controls(vec![first.accesskit_id()]);
        node.set_min_numeric_value(bounds.min as f64);
        node.set_max_numeric_value(bounds.max as f64);
        node.set_numeric_value_step(tokens::STEP as f64);
        node.set_numeric_value_jump(tokens::LARGE_STEP as f64);
        node.set_value(format!("{value:.0} points"));
        if read_only {
            node.set_read_only();
        }
        if response.enabled() && !read_only && bounds.max > bounds.min {
            node.add_action(Action::SetValue);
            if value < bounds.max {
                node.add_action(Action::Increment);
            }
            if value > bounds.min {
                node.add_action(Action::Decrement);
            }
        }
    });
    if response.has_focus() {
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                response.id,
                egui::EventFilter {
                    horizontal_arrows: axis == Axis::Horizontal,
                    vertical_arrows: axis == Axis::Vertical,
                    ..Default::default()
                },
            )
        });
    }
}
pub(super) fn divider(ui: &Ui, response: &Response, axis: Axis, read_only: bool) {
    if !ui.is_rect_visible(response.rect) {
        return;
    }
    let p = Theme::from_context(ui.ctx()).palette;
    let hot = response.enabled() && !read_only && (response.hovered() || response.dragged());
    if hot {
        response
            .clone()
            .on_hover_and_drag_cursor(if axis == Axis::Horizontal {
                egui::CursorIcon::ResizeHorizontal
            } else {
                egui::CursorIcon::ResizeVertical
            });
        ui.painter().rect_filled(response.rect, 0, p.muted);
    }
    let color = if hot { p.primary } else { p.border };
    let rect = response.rect;
    let line = if axis == Axis::Horizontal {
        [
            egui::pos2(rect.center().x, rect.top()),
            egui::pos2(rect.center().x, rect.bottom()),
        ]
    } else {
        [
            egui::pos2(rect.left(), rect.center().y),
            egui::pos2(rect.right(), rect.center().y),
        ]
    };
    ui.painter()
        .line_segment(line, Stroke::new(border::STANDARD, color));
    let size = if axis == Axis::Horizontal {
        vec2(border::ACCENT, tokens::GRIP.min(rect.height()))
    } else {
        vec2(tokens::GRIP.min(rect.width()), border::ACCENT)
    };
    ui.painter().rect_filled(
        egui::Rect::from_center_size(rect.center(), size),
        0,
        if hot {
            p.primary
        } else if !response.enabled() || read_only {
            p.muted_foreground
        } else {
            p.foreground
        },
    );
    paint_focus_inside(ui, response, p.primary);
}
