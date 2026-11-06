use crate::{
    ControlSize, Theme,
    interaction::paint_focus,
    tokens::{border, control},
};
use egui::{Id, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, vec2};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Checked {
    No,
    Yes,
    Mixed,
}
pub(super) fn show(ui: &mut Ui, id: Id, label: &str, state: Checked, enabled: bool) -> Response {
    ui.add_enabled(enabled, |ui: &mut Ui| {
        let (_, rect) = ui.allocate_space(egui::Vec2::splat(ControlSize::Small.height()));
        // Use an explicit key independent of labels and row/column order.
        let response = ui.interact(rect, id, Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(
                egui::accesskit::Role::CheckBox,
                ui.is_enabled(),
                state == Checked::Yes,
                label,
            )
        });
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_toggled(match state {
                Checked::No => egui::accesskit::Toggled::False,
                Checked::Yes => egui::accesskit::Toggled::True,
                Checked::Mixed => egui::accesskit::Toggled::Mixed,
            })
        });
        let p = Theme::from_context(ui.ctx()).palette;
        let icon =
            egui::Rect::from_center_size(rect.center(), egui::Vec2::splat(control::INDICATOR));
        let fill = if state == Checked::No {
            p.background
        } else {
            p.primary
        };
        let stroke = if response.hovered() || state != Checked::No {
            p.primary
        } else {
            p.border
        };
        ui.painter().rect(
            icon,
            0,
            fill,
            Stroke::new(border::STANDARD, stroke),
            StrokeKind::Inside,
        );
        if state == Checked::Mixed {
            ui.painter().line_segment(
                [
                    icon.center() - vec2(control::INDICATOR / 4.0, 0.0),
                    icon.center() + vec2(control::INDICATOR / 4.0, 0.0),
                ],
                Stroke::new(border::STANDARD, p.on_primary),
            );
        }
        if state == Checked::Yes {
            let point = |x, y| icon.min + vec2(x * icon.width(), y * icon.height());
            ui.painter().add(egui::Shape::line(
                vec![point(0.2, 0.5), point(0.43, 0.73), point(0.8, 0.28)],
                Stroke::new(border::STANDARD, p.on_primary),
            ));
        }
        if response.enabled() && response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        paint_focus(ui, &response, p.primary);
        response
    })
}
