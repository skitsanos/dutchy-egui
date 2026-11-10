use crate::{ControlSize, tokens::inline_edit as tokens};
use egui::{Rect, Vec2, pos2, vec2};

pub(super) fn width(requested: Option<f32>, available: f32) -> f32 {
    let finite = |value: f32| {
        if value.is_finite() {
            value.clamp(0.0, tokens::MAX_WIDTH)
        } else {
            tokens::WIDTH
        }
    };
    finite(requested.unwrap_or(available)).min(finite(available))
}

pub(super) fn extent(width: f32, size: ControlSize, editing: bool) -> Vec2 {
    let narrow = width < tokens::MIN_EDITOR + 2.0 * (size.height() + tokens::GAP);
    vec2(
        width,
        size.height()
            + if editing && narrow {
                size.height() + tokens::GAP
            } else {
                0.0
            },
    )
}

pub(super) struct Slots {
    pub editor: Rect,
    pub save: Rect,
    pub cancel: Rect,
}
impl Slots {
    pub fn new(outer: Rect, size: ControlSize) -> Self {
        let narrow = outer.height() > size.height();
        let gap = tokens::GAP.min(outer.width() / 4.0);
        let action = size.height().min((outer.width() - gap).max(0.0) / 2.0);
        let top = if narrow {
            outer.top() + size.height() + tokens::GAP
        } else {
            outer.top()
        };
        let cancel = Rect::from_min_size(
            pos2(outer.right() - action, top),
            vec2(action, size.height()),
        );
        let save = cancel.translate(vec2(-action - gap, 0.0));
        let right = if narrow {
            outer.right()
        } else {
            (save.left() - gap).max(outer.left())
        };
        Self {
            editor: Rect::from_min_max(outer.min, pos2(right, outer.top() + size.height())),
            save,
            cancel,
        }
    }
}
