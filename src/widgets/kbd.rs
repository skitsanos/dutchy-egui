use crate::{
    Family, Text, TextSize, Theme, Tracking, Weight,
    tokens::{border, control},
};
use egui::accesskit::Role;
use egui::{Response, Sense, Stroke, StrokeKind, Ui, Widget, WidgetInfo, pos2, vec2};
use std::borrow::Cow;

/// Keyboard key chip: mono, uppercase, extra-small text on the muted surface.
///
/// One chip per key; compose sequences in a row. The accessible name keeps the
/// host's casing. This is a label, not a control.
#[derive(Clone, Debug)]
#[must_use = "Add this chip to a Ui"]
pub struct Kbd<'a> {
    text: Cow<'a, str>,
}

impl<'a> Kbd<'a> {
    /// A chip for one key or chord, such as `Ctrl+K`.
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self { text: text.into() }
    }

    pub(crate) fn desired_size(&self, ui: &Ui) -> egui::Vec2 {
        let galley = self.galley(ui);
        vec2(
            galley.size().x + 2.0 * control::KBD_PADDING_X,
            control::INDICATOR,
        )
    }

    fn galley(&self, ui: &Ui) -> std::sync::Arc<egui::Galley> {
        let theme = Theme::from_context(ui.ctx());
        let job = Text::new(self.text.as_ref())
            .family(Family::Mono)
            .size(TextSize::Xs)
            .weight(Weight::Bold)
            .tracking(Tracking::Wider)
            .uppercase(true)
            .color(crate::TextColor::Foreground)
            .layout_job(&theme, f32::INFINITY);
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    }
}

impl Widget for Kbd<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.desired_size(ui), Sense::hover());
        response
            .widget_info(|| WidgetInfo::labeled(Role::Label, ui.is_enabled(), self.text.as_ref()));
        crate::text::name_as_written(&response, self.text.as_ref());
        self.paint(ui, rect);
        response
    }
}

impl Kbd<'_> {
    /// Paint the chip into `rect` with `ui`'s painter, without a response.
    pub(crate) fn paint(&self, ui: &Ui, rect: egui::Rect) {
        if !ui.is_rect_visible(rect) {
            return;
        }
        let p = Theme::from_context(ui.ctx()).palette;
        let galley = self.galley(ui);
        ui.painter().rect(
            rect,
            0,
            p.muted,
            Stroke::new(border::THIN, p.border),
            StrokeKind::Inside,
        );
        ui.painter().galley(
            pos2(
                rect.center().x - galley.size().x / 2.0,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            p.foreground,
        );
    }
}
