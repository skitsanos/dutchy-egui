use super::AccordionVariant;
use crate::{
    Family, Text, TextSize, Theme, Tracking, Weight,
    interaction::paint_focus_inside,
    tokens::{border, disclosure as geometry, motion},
};
use egui::{Id, Response, Sense, Stroke, Ui, WidgetInfo, pos2, vec2};

pub(super) fn header(
    ui: &mut Ui,
    id: Id,
    label: &str,
    open: bool,
    variant: AccordionVariant,
) -> Response {
    let theme = Theme::from_context(ui.ctx());
    let width = ui.available_width().max(0.0);
    let text_width =
        (width - geometry::PADDING_X * 2.0 - geometry::CHEVRON - geometry::CHEVRON_GAP).max(0.0);
    let family = if variant == AccordionVariant::Default {
        Family::Display
    } else {
        Family::Body
    };
    let job = Text::new(label)
        .family(family)
        .size(TextSize::Sm)
        .weight(Weight::Bold)
        .tracking(if variant == AccordionVariant::Default {
            Tracking::Wider
        } else {
            Tracking::Wide
        })
        .uppercase(true)
        .layout_job(&theme, text_width);
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let padding_y = if variant == AccordionVariant::Default {
        geometry::ACCORDION_PADDING_Y
    } else {
        geometry::BORDERED_PADDING_Y
    };
    let (_, rect) = ui.allocate_space(vec2(
        width,
        galley.size().y.max(geometry::CHEVRON) + padding_y * 2.0,
    ));
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            egui::accesskit::Role::Button,
            ui.is_enabled(),
            label.to_string(),
        )
    });
    if !ui.is_rect_visible(rect) {
        return response;
    }
    if response.hovered() && variant == AccordionVariant::Bordered {
        ui.painter().rect_filled(rect, 0, theme.palette.muted);
    }
    ui.painter().galley(
        pos2(
            rect.left() + geometry::PADDING_X,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        theme.palette.foreground,
    );
    let openness = if theme.reduced_motion {
        f32::from(open)
    } else {
        ui.ctx()
            .animate_bool_with_time(id.with("chevron-motion"), open, motion::NORMAL)
    };
    let center = pos2(
        rect.right() - geometry::PADDING_X - geometry::CHEVRON / 2.0,
        rect.center().y,
    );
    let rotation = egui::emath::Rot2::from_angle(openness * std::f32::consts::PI);
    let side = geometry::CHEVRON / 2.0;
    let points = [
        vec2(-side / 2.0, -side / 4.0),
        vec2(0.0, side / 4.0),
        vec2(side / 2.0, -side / 4.0),
    ]
    .map(|point| center + rotation * point);
    ui.painter().add(egui::Shape::line(
        points.to_vec(),
        Stroke::new(border::STANDARD, theme.palette.foreground),
    ));
    paint_focus_inside(ui, &response, theme.palette.primary);
    response
}
