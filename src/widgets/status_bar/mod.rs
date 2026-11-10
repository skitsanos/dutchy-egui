mod fit;
mod item;
mod segment;

pub use item::{StatusAlign, StatusItem, StatusTone};

use crate::{
    Theme,
    tokens::{border, status_bar as tokens},
};
use egui::{Id, IdSalt, Rect, Response, Ui, UiBuilder, WidgetInfo, accesskit::Role, pos2, vec2};
use fit::Slot;
use segment::Parts;
use std::{
    borrow::Cow,
    hash::{DefaultHasher, Hash, Hasher},
};

/// One-line strip of keyed status segments, usually at the bottom of a window.
///
/// The host owns every value and decides what activating a segment does.
/// Segments that do not fit are hidden by priority and reported back.
#[must_use = "Call show to lay out the status bar"]
pub struct StatusBar<'a, K> {
    id: IdSalt,
    items: &'a [StatusItem<'a, K>],
    name: Cow<'a, str>,
    enabled: bool,
    divider: bool,
}

/// The bar, its visible segments and this frame's activation.
#[non_exhaustive]
pub struct StatusBarResponse<K> {
    /// Allocated bar rectangle.
    pub response: Response,
    /// Key of a clickable segment activated this frame.
    pub clicked: Option<K>,
    /// Keys hidden for lack of space, in item order.
    pub hidden: Vec<K>,
    /// Visible segments in item order, for tooltips, menus or anchors.
    pub segments: Vec<(K, Response)>,
}

impl<'a, K: Clone + Eq + Hash> StatusBar<'a, K> {
    /// A bar over the host's items, in reading order.
    pub fn new(id: impl egui::AsIdSalt, items: &'a [StatusItem<'a, K>]) -> Self {
        Self {
            id: IdSalt::new(id),
            items,
            name: Cow::Borrowed("Status"),
            enabled: true,
            divider: true,
        }
    }
    /// Accessible name of the segment group; defaults to `Status`.
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }
    /// Disable every segment, respecting disabled parents.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Paint a thin divider along the top edge; on by default.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// Allocate the full available width at a fixed height and show the
    /// segments that fit.
    pub fn show(self, ui: &mut Ui) -> StatusBarResponse<K> {
        let width = ui.available_width();
        let width = if width.is_finite() {
            width.clamp(0.0, tokens::MAX_WIDTH)
        } else {
            tokens::WIDTH
        };
        let (_, rect) = ui.allocate_space(vec2(width, tokens::HEIGHT));
        let id = ui.make_persistent_id(("dutchy-status-bar", self.id));
        let mut root = ui.new_child(UiBuilder::new().scope_id(id).max_rect(rect));
        root.set_min_size(rect.size());
        root.set_clip_rect(ui.clip_rect().intersect(rect));
        if !self.enabled {
            root.disable();
        }
        self.paint_surface(&root, rect);
        let (placed, parts) = self.layout(&root, rect);
        let mut result = StatusBarResponse {
            response: root.response(),
            clicked: None,
            hidden: Vec::new(),
            segments: Vec::new(),
        };
        for ((item, placed), parts) in self.items.iter().zip(placed).zip(parts) {
            let Some(segment_rect) = placed else {
                result.hidden.push(item.key.clone());
                continue;
            };
            let response = segment::show(
                &mut root,
                segment_id(id, &item.key),
                item,
                parts,
                segment_rect,
            );
            if response.clicked() {
                result.clicked = Some(item.key.clone());
            }
            result.segments.push((item.key.clone(), response));
        }
        let name = self.name.to_string();
        result.response = root.response();
        result.response.rect = rect;
        result
            .response
            .widget_info(|| WidgetInfo::labeled(Role::Group, root.is_enabled(), &name));
        result
    }

    fn paint_surface(&self, ui: &Ui, rect: Rect) {
        if !ui.is_rect_visible(rect) {
            return;
        }
        let p = Theme::from_context(ui.ctx()).palette;
        ui.painter().rect_filled(rect, 0, p.background);
        if self.divider {
            let line = Rect::from_min_size(rect.min, vec2(rect.width(), border::THIN));
            ui.painter().rect_filled(line, 0, p.border);
        }
    }

    /// Measure every item, choose the visible ones and place them.
    fn layout(&self, ui: &Ui, bar: Rect) -> (Vec<Option<Rect>>, Vec<Parts>) {
        let mut parts: Vec<Parts> = self
            .items
            .iter()
            .map(|item| Parts::measure(ui, item, None))
            .collect();
        let slots: Vec<Slot> = self
            .items
            .iter()
            .zip(&parts)
            .map(|(item, parts)| Slot {
                width: parts.width(item),
                priority: item.priority,
                end: item.align == StatusAlign::End,
            })
            .collect();
        let fit = fit::fit(&slots, bar.width());
        if let Some((index, budget)) = fit.elide {
            parts[index] = Parts::measure(ui, &self.items[index], Some(budget));
        }
        let widths: Vec<f32> = self
            .items
            .iter()
            .zip(&parts)
            .map(|(item, parts)| parts.width(item))
            .collect();
        (place(bar, &slots, &widths, &fit.visible), parts)
    }
}

/// Pack start segments from the left and end segments against the right.
fn place(bar: Rect, slots: &[Slot], widths: &[f32], visible: &[bool]) -> Vec<Option<Rect>> {
    let end_width = (0..slots.len())
        .filter(|index| visible[*index] && slots[*index].end)
        .map(|index| widths[index] + tokens::GAP)
        .sum::<f32>()
        - tokens::GAP;
    let mut start_x = bar.left() + tokens::INSET;
    let mut end_x = bar.right() - tokens::INSET - end_width.max(0.0);
    (0..slots.len())
        .map(|index| {
            if !visible[index] {
                return None;
            }
            let x = if slots[index].end {
                &mut end_x
            } else {
                &mut start_x
            };
            let rect = Rect::from_min_size(pos2(*x, bar.top()), vec2(widths[index], bar.height()));
            *x += widths[index] + tokens::GAP;
            Some(rect)
        })
        .collect()
}

/// Stable identity of a keyed segment, independent of its position.
fn segment_id<K: Hash>(bar: Id, key: &K) -> Id {
    let mut hash = DefaultHasher::new();
    key.hash(&mut hash);
    bar.with(("segment", hash.finish()))
}
