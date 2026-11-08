use crate::{
    ControlSize,
    tokens::{border, search_input as tokens},
};
use egui::{Rect, Vec2, pos2};

pub(super) struct Geometry {
    pub editor: Rect,
    pub icon: Option<Rect>,
    pub clear: Option<Rect>,
    pub hint: Option<Rect>,
}

impl Geometry {
    pub fn new(outer: Rect, clear: bool, hint: Option<Vec2>) -> Self {
        let inset = border::STANDARD.min(outer.width().min(outer.height()) / 2.0);
        let inner = outer.shrink(inset);
        let padding = tokens::PADDING.min(inner.width() / 8.0);
        let left = inner.left() + padding;
        let mut right = inner.right() - padding;
        let target = ControlSize::Small
            .height()
            .min(inner.height())
            .min((right - left - tokens::PADDING).max(0.0));
        let clear = clear && target > 0.0;
        let clear_space = if clear { target + tokens::GAP } else { 0.0 };
        let icon = (right - left - clear_space >= tokens::MIN_TEXT + tokens::ICON + tokens::GAP)
            .then(|| {
                Rect::from_center_size(
                    pos2(left + tokens::ICON / 2.0, inner.center().y),
                    Vec2::splat(tokens::ICON),
                )
            });
        let left = icon.map_or(left, |rect| rect.right() + tokens::GAP);
        let hint = hint
            .filter(|size| right - left - clear_space >= tokens::MIN_TEXT + tokens::GAP + size.x)
            .map(|size| {
                let rect =
                    Rect::from_center_size(pos2(right - size.x / 2.0, inner.center().y), size);
                right = rect.left() - tokens::GAP;
                rect
            });
        let clear = clear.then(|| {
            let rect = Rect::from_center_size(
                pos2(right - target / 2.0, inner.center().y),
                Vec2::splat(target),
            );
            right = (rect.left() - tokens::GAP.min((rect.left() - left) / 4.0)).max(left);
            rect
        });
        Self {
            editor: Rect::from_min_max(
                pos2(left, inner.top()),
                pos2(right.max(left), inner.bottom()),
            ),
            icon,
            clear,
            hint,
        }
    }
}
