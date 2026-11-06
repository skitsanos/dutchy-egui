use super::{BreadcrumbItem, BreadcrumbSeparator};
use crate::{
    Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus,
    tokens::{border, navigation as geometry},
    widgets::nav_icon,
};
use egui::{Id, Rect, Response, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

pub(super) fn galley(ui: &Ui, label: &str, width: f32) -> std::sync::Arc<egui::Galley> {
    let job = Text::new(label)
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(Tracking::Wide)
        .uppercase(true)
        .layout_job(&Theme::from_context(ui.ctx()), width);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}
pub(super) fn width<T>(ui: &Ui, item: &BreadcrumbItem<'_, T>) -> f32 {
    let width = if item.icon.is_some() {
        geometry::ICON_SIZE
    } else {
        galley(ui, &item.label, f32::INFINITY).size().x
    };
    width + geometry::LINK_PADDING * 2.0
}
pub(super) fn item<T>(
    ui: &mut Ui,
    id: Id,
    item: &BreadcrumbItem<'_, T>,
    current: bool,
    width: f32,
) -> Response {
    let p = Theme::from_context(ui.ctx()).palette;
    let galley = item.icon.is_none().then(|| {
        galley(
            ui,
            &item.label,
            (width - geometry::LINK_PADDING * 2.0).max(0.0),
        )
    });
    let height = galley.as_ref().map_or(geometry::BREADCRUMB_HEIGHT, |text| {
        geometry::BREADCRUMB_HEIGHT.max(text.size().y)
    });
    let (_, rect) = ui.allocate_space(vec2(width, height));
    let response = ui.interact(
        rect,
        id,
        if current {
            Sense::hover()
        } else {
            Sense::click()
        },
    );
    response.widget_info(|| {
        WidgetInfo::labeled(
            if current {
                egui::accesskit::Role::Label
            } else {
                egui::accesskit::Role::Button
            },
            ui.is_enabled(),
            item.label.to_uppercase(),
        )
    });
    if current {
        ui.ctx().accesskit_node_builder(id, |node| {
            node.set_aria_current(egui::accesskit::AriaCurrent::Page)
        });
    }
    let color = if current {
        p.foreground
    } else if response.hovered() || response.has_focus() {
        p.primary
    } else {
        p.muted_foreground
    };
    if ui.is_rect_visible(rect) {
        let painter = ui.painter().with_clip_rect(ui.clip_rect().intersect(rect));
        if let Some(icon) = item.icon {
            icon(
                &painter,
                Rect::from_center_size(
                    rect.center(),
                    vec2(geometry::ICON_SIZE, geometry::ICON_SIZE),
                ),
                color,
            );
        } else if let Some(galley) = galley {
            painter.galley_with_override_text_color(
                pos2(
                    rect.left() + geometry::LINK_PADDING,
                    rect.center().y - galley.size().y / 2.0,
                ),
                galley,
                color,
            );
        }
    }
    if !current {
        paint_focus(ui, &response, p.primary);
    }
    response.on_hover_cursor(if current {
        egui::CursorIcon::Default
    } else {
        egui::CursorIcon::PointingHand
    })
}
pub(super) fn separator(ui: &mut Ui, separator: BreadcrumbSeparator) {
    let (_, rect) = ui.allocate_space(vec2(geometry::ICON_SIZE, geometry::BREADCRUMB_HEIGHT));
    let color = Theme::from_context(ui.ctx()).palette.muted_foreground;
    let icon = Rect::from_center_size(
        rect.center(),
        vec2(geometry::ICON_SIZE, geometry::ICON_SIZE),
    );
    match separator {
        BreadcrumbSeparator::Chevron => nav_icon::chevron(ui.painter(), icon, color, true),
        BreadcrumbSeparator::Slash => {
            ui.painter().line_segment(
                [icon.left_bottom(), icon.right_top()],
                Stroke::new(border::THIN, color),
            );
        }
    };
}
pub(super) fn collapsed(ui: &mut Ui, id: Id) -> Response {
    let p = Theme::from_context(ui.ctx()).palette;
    let (rect, _) = ui.allocate_exact_size(
        vec2(geometry::BREADCRUMB_HEIGHT, geometry::BREADCRUMB_HEIGHT),
        Sense::hover(),
    );
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::Button,
            ui.is_enabled(),
            "Show collapsed ancestors",
        )
    });
    let color = if response.hovered() || response.has_focus() {
        p.primary
    } else {
        p.muted_foreground
    };
    ui.painter().rect_stroke(
        rect,
        0,
        Stroke::new(border::STANDARD, color),
        egui::StrokeKind::Inside,
    );
    let text = galley(ui, "…", f32::INFINITY);
    ui.painter()
        .galley_with_override_text_color(rect.center() - text.size() / 2.0, text, color);
    paint_focus(ui, &response, p.primary);
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
