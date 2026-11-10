use crate::tokens::split_pane as tokens;
use egui::{Pos2, Rect, pos2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Axis {
    Horizontal,
    Vertical,
}
impl Axis {
    pub fn coordinate(self, pos: Pos2) -> f32 {
        match self {
            Self::Horizontal => pos.x,
            Self::Vertical => pos.y,
        }
    }
    pub fn length(self, rect: Rect) -> f32 {
        match self {
            Self::Horizontal => rect.width(),
            Self::Vertical => rect.height(),
        }
    }
    pub fn rects(self, rect: Rect, first: f32) -> [Rect; 3] {
        let gap = tokens::DIVIDER.min(self.length(rect));
        let (end, next) = match self {
            Self::Horizontal => (
                pos2(rect.left() + first, rect.bottom()),
                pos2(rect.left() + first + gap, rect.top()),
            ),
            Self::Vertical => (
                pos2(rect.right(), rect.top() + first),
                pos2(rect.left(), rect.top() + first + gap),
            ),
        };
        let divider = match self {
            Self::Horizontal => {
                Rect::from_min_max(pos2(end.x, rect.top()), pos2(next.x, rect.bottom()))
            }
            Self::Vertical => {
                Rect::from_min_max(pos2(rect.left(), end.y), pos2(rect.right(), next.y))
            }
        };
        [
            Rect::from_min_max(rect.min, end),
            divider,
            Rect::from_min_max(next, rect.max),
        ]
    }
}

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub min: f32,
    pub max: f32,
    pub room: f32,
}
impl Bounds {
    pub fn new(extent: f32, first: f32, second: f32) -> Self {
        let room = (extent - tokens::DIVIDER).max(0.0);
        let first = dimension(first, 0.0);
        let second = dimension(second, 0.0);
        let sum = first + second;
        let scale = if sum > room { room / sum } else { 1.0 };
        let min = (first * scale).min(room);
        let max = (room - second * scale).max(min);
        Self { min, max, room }
    }
    pub fn clamp(self, value: f32) -> f32 {
        let value = if value.is_finite() {
            value
        } else {
            self.room / 2.0
        };
        value.clamp(self.min, self.max)
    }
}
pub(super) fn dimension(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, tokens::MAX_SIZE)
    } else {
        fallback
    }
}
