use super::ProgressVariant;
use crate::{
    Theme,
    tokens::{loading, motion},
};
use egui::{Rect, Ui, pos2};

pub(in crate::widgets) fn track(
    ui: &Ui,
    rect: Rect,
    fraction: Option<f32>,
    variant: ProgressVariant,
    animate: bool,
) {
    if !ui.is_rect_visible(rect) || rect.width() <= 0.0 {
        return;
    }
    let theme = Theme::from_context(ui.ctx());
    ui.painter().rect_filled(rect, 0, theme.palette.muted);
    let (start, end) = if let Some(fraction) = fraction {
        (0.0, fraction)
    } else {
        let travel = if animate && !theme.reduced_motion {
            super::super::loading_motion::wave(super::super::loading_motion::phase(
                ui,
                motion::PROGRESS_TRAVEL,
            ))
        } else {
            0.5
        };
        let start = travel * (1.0 - loading::INDETERMINATE_FRACTION);
        (start, start + loading::INDETERMINATE_FRACTION)
    };
    if end > start {
        let fill = Rect::from_min_max(
            pos2(rect.left() + start * rect.width(), rect.top()),
            pos2(rect.left() + end * rect.width(), rect.bottom()),
        );
        ui.painter().rect_filled(fill, 0, variant.color(theme));
    }
}
