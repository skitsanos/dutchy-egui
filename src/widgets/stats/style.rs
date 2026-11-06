use super::StatVariant;
use crate::{Theme, tokens::border};
use egui::{Color32, Painter, Rect, Shape, Stroke, StrokeKind, Ui};

pub(super) fn colors(ui: &mut Ui, variant: StatVariant) -> Color32 {
    let p = Theme::from_context(ui.ctx()).palette;
    let (fill, text) = match variant {
        StatVariant::Light => (p.background, None),
        StatVariant::Muted => (p.muted, None),
        StatVariant::Dark => (p.foreground, Some(p.background)),
        StatVariant::Primary => (p.primary, Some(p.on_primary)),
    };
    ui.visuals_mut().override_text_color = text;
    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = text.unwrap_or(p.foreground);
    fill
}

pub(super) struct Tile {
    pub rect: Rect,
    pub response: egui::Response,
    pub painter: Painter,
    pub shape: egui::layers::ShapeIdx,
    pub fill: Color32,
    pub edge: Color32,
    pub accent: bool,
    pub bordered: bool,
}

impl Tile {
    pub fn finish(&self, bottom: f32) {
        let rect = Rect::from_min_max(self.rect.min, egui::pos2(self.rect.right(), bottom));
        let stroke = if self.bordered && !self.accent {
            Stroke::new(border::ACCENT, self.edge)
        } else {
            Stroke::NONE
        };
        let mut shapes = vec![
            Shape::rect_filled(rect, 0, self.fill),
            Shape::rect_stroke(rect, 0, stroke, StrokeKind::Inside),
        ];
        if self.accent {
            let mut edge = rect;
            edge.max.x = edge.left() + border::ACCENT;
            shapes.push(Shape::rect_filled(edge, 0, self.edge));
        }
        self.painter.set(self.shape, Shape::Vec(shapes));
    }
}
