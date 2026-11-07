use crate::{
    ControlSize, Text, TextColor, TextSize, Theme,
    interaction::paint_focus,
    tokens::{border, control, space},
};
use egui::accesskit::Role;
use egui::{Id, Rect, Response, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};

pub(super) fn show(
    ui: &mut Ui,
    id: Id,
    label: &str,
    selected: &mut bool,
    radio: bool,
    tab_stop: bool,
) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let side = control::INDICATOR;
    let gap = control::LABEL_GAP;
    let text_width = (ui.available_width() - side - gap).max(0.0);
    // Inverse surfaces install a scoped text color; the label follows it.
    let color = ui
        .visuals()
        .override_text_color
        .map_or(TextColor::Foreground, TextColor::Custom);
    let job = Text::new(label)
        .size(TextSize::Sm)
        .color(color)
        .layout_job(&theme, text_width);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = vec2(
        (side + gap + galley.size().x).min(ui.available_width()),
        ControlSize::Small.height().max(galley.size().y),
    );
    let (_, rect) = ui.allocate_space(size);
    let sense = if tab_stop {
        Sense::click()
    } else {
        Sense::CLICK
    };
    let mut response = ui.interact(rect, id, sense);
    if response.clicked() && (!radio || !*selected) {
        *selected = if radio { true } else { !*selected };
        response.mark_changed();
        response.request_focus();
    }
    let role = if radio {
        Role::RadioButton
    } else {
        Role::CheckBox
    };
    response.widget_info(|| WidgetInfo::selected(role, ui.is_enabled(), *selected, label));
    if ui.is_rect_visible(rect) {
        let indicator = Rect::from_center_size(
            pos2(rect.left() + side / 2.0, rect.center().y),
            vec2(side, side),
        );
        paint_indicator(ui, indicator, *selected, radio, response.hovered());
        ui.painter().galley(
            pos2(
                indicator.right() + gap,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            theme.palette.foreground,
        );
        let mut focus = response.clone();
        focus.rect = indicator;
        paint_focus(ui, &focus, theme.palette.primary);
    }
    response
}

fn paint_indicator(ui: &Ui, rect: Rect, checked: bool, radio: bool, hovered: bool) {
    let p = Theme::from_context(ui.ctx()).palette;
    let inherited = super::style::inherited_border(ui);
    let edge = if checked || hovered {
        p.primary
    } else {
        inherited
    };
    let fill = if checked && !radio {
        p.primary
    } else {
        p.background
    };
    ui.painter().rect(
        rect,
        0,
        fill,
        Stroke::new(border::STANDARD, edge),
        StrokeKind::Inside,
    );
    if !checked {
        return;
    }
    if radio {
        ui.painter()
            .rect_filled(rect.shrink(space::XS + border::THIN), 0, p.primary);
    } else {
        let side = rect.width();
        let point = |x, y| rect.min + vec2(x * side, y * side);
        ui.painter().add(egui::Shape::line(
            vec![point(0.2, 0.5), point(0.42, 0.72), point(0.8, 0.25)],
            Stroke::new(border::STANDARD, p.on_primary),
        ));
    }
}
