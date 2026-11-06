use super::CardVariant;
use crate::{CardPadding, Theme, tokens::border};
use egui::{Color32, Frame, Margin, Stroke, Ui};

pub(super) struct Surface {
    pub fill: Color32,
    pub edge: Color32,
    pub text: Option<Color32>,
}

pub(super) fn surface(theme: Theme, variant: CardVariant, hovered: bool) -> Surface {
    let p = theme.palette;
    if hovered {
        return Surface {
            fill: p.muted,
            edge: p.primary,
            text: None,
        };
    }
    let (fill, text) = match variant {
        CardVariant::Default | CardVariant::Accent | CardVariant::Bordered => (p.background, None),
        CardVariant::Muted => (p.muted, None),
        CardVariant::Dark => (p.foreground, Some(p.background)),
        CardVariant::Primary => (p.primary, Some(p.on_primary)),
    };
    Surface {
        fill,
        edge: if variant == CardVariant::Accent {
            p.primary
        } else {
            p.border
        },
        text,
    }
}

pub(super) fn frame(surface: &Surface, variant: CardVariant, padding: CardPadding) -> Frame {
    let mut margin = Margin::same(padding.points() as i8);
    if variant == CardVariant::Accent {
        margin.left += border::ACCENT as i8;
    }
    Frame::NONE.fill(surface.fill).inner_margin(margin).stroke(
        if variant == CardVariant::Bordered {
            Stroke::new(border::STANDARD, surface.edge)
        } else {
            Stroke::NONE
        },
    )
}

pub(super) fn content(ui: &mut Ui, surface: &Surface) {
    // Clear an enclosing inverted surface when this card introduces its own fill.
    let text = surface
        .text
        .unwrap_or(Theme::from_context(ui.ctx()).palette.foreground);
    let visuals = &mut ui.style_mut().visuals;
    visuals.override_text_color = surface.text;
    visuals.widgets.noninteractive.fg_stroke.color = text;
}
