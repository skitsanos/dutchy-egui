use crate::{
    ControlSize, Theme,
    interaction::paint_focus,
    tokens::{border, space},
};
use egui::{Color32, Frame, Margin, Response, Stroke, StrokeKind, Ui};

pub(super) fn frame(ui: &Ui, size: ControlSize) -> Frame {
    let theme = Theme::from_context(ui.ctx());
    let fill = if ui.is_enabled() {
        theme.palette.background
    } else {
        theme.palette.muted
    };
    let vertical =
        ((size.height() - size.text_size().points() * 1.5) / 2.0 - border::STANDARD).max(0.0);
    Frame::NONE
        .fill(fill)
        .stroke(Stroke::new(border::STANDARD, inherited_border(ui)))
        .inner_margin(Margin::symmetric(space::MD as i8, vertical.round() as i8))
}

pub(super) fn inherited_border(ui: &Ui) -> Color32 {
    ui.visuals().widgets.inactive.bg_stroke.color
}

pub(super) fn finish(ui: &Ui, response: &Response) {
    let theme = Theme::from_context(ui.ctx());
    let inherited = inherited_border(ui);
    if response.has_focus() {
        let color = if inherited == theme.palette.border {
            theme.palette.foreground
        } else {
            inherited
        };
        ui.painter().rect_stroke(
            response.rect,
            0,
            Stroke::new(border::STANDARD, color),
            StrokeKind::Inside,
        );
    }
    paint_focus(ui, response, theme.palette.primary);
}

pub(super) fn set_border(ui: &mut Ui, color: Color32) {
    let widgets = &mut ui.style_mut().visuals.widgets;
    for visual in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        visual.bg_stroke.color = color;
    }
}

/// Paint a composite control's frame: light or muted fill with the inherited border.
pub(super) fn paint_frame(ui: &Ui, rect: egui::Rect, theme: Theme) {
    let fill = if ui.is_enabled() {
        theme.palette.background
    } else {
        theme.palette.muted
    };
    ui.painter().rect(
        rect,
        0,
        fill,
        Stroke::new(border::STANDARD, inherited_border(ui)),
        StrokeKind::Inside,
    );
}
