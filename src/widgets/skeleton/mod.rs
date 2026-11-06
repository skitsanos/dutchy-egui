mod group;
mod patterns;

use crate::{
    Theme,
    tokens::{loading, space},
};
use egui::{Rect, Response, Sense, Ui, Widget, pos2, vec2};
pub use group::SkeletonGroup;

/// Placeholder shapes matching the web component.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonVariant {
    /// Full-width 16-point line.
    #[default]
    Text,
    /// 32-point line occupying three quarters of its column.
    Title,
    /// Square 48-point avatar.
    Avatar,
    /// Full-width 192-point media block.
    Card,
    /// Four sharp cells, with flexible leading columns.
    TableRow,
}

/// Decorative placeholder blocks. Use SkeletonGroup for one loading announcement.
#[derive(Clone, Copy, Debug)]
#[must_use = "Add this skeleton to a Ui"]
pub struct Skeleton {
    variant: SkeletonVariant,
    count: usize,
    width: Option<f32>,
    height: Option<f32>,
    animate: bool,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            variant: SkeletonVariant::Text,
            count: 1,
            width: None,
            height: None,
            animate: true,
        }
    }
}

impl Skeleton {
    /// One full-width text line.
    pub fn new() -> Self {
        Self::default()
    }
    /// Shape preset.
    pub fn variant(mut self, variant: SkeletonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Number of repeated shapes, separated by the shared 12-point gap.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
    /// Preferred width, constrained to the column; overrides preset width.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(super::loading_motion::dimension(width));
        self
    }
    /// Height per shape; non-finite/negative dimensions become zero.
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(super::loading_motion::dimension(height));
        self
    }
    /// Pause pulse. Reduced motion also produces a static full-opacity block.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    fn size(self, available: f32) -> egui::Vec2 {
        let (width, height) = match self.variant {
            SkeletonVariant::Text | SkeletonVariant::TableRow => (available, loading::TEXT_HEIGHT),
            SkeletonVariant::Title => (available * loading::TITLE_FRACTION, loading::TITLE_HEIGHT),
            SkeletonVariant::Avatar => (loading::AVATAR_SIDE, loading::AVATAR_SIDE),
            SkeletonVariant::Card => (available, loading::CARD_HEIGHT),
        };
        vec2(
            self.width.unwrap_or(width).min(available).max(0.0),
            self.height.unwrap_or(height),
        )
    }

    fn paint(self, ui: &Ui, rect: Rect) {
        if !ui.is_rect_visible(rect) || rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let theme = Theme::from_context(ui.ctx());
        let opacity = super::loading_motion::opacity(ui, self.animate && !theme.reduced_motion);
        let color = theme.palette.muted.gamma_multiply(opacity);
        if self.variant != SkeletonVariant::TableRow {
            ui.painter().rect_filled(rect, 0, color);
            return;
        }
        let gap = space::LG.min(rect.width() / 12.0);
        let usable = (rect.width() - 3.0 * gap).max(0.0);
        let fixed = loading::TABLE_CELL_WIDTH.min(usable / 4.0);
        let flexible = (usable - 2.0 * fixed) / 2.0;
        let mut left = rect.left();
        for width in [flexible, flexible, fixed, fixed] {
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(left, rect.top()), vec2(width, rect.height())),
                0,
                color,
            );
            left += width + gap;
        }
    }
}

impl Widget for Skeleton {
    fn ui(self, ui: &mut Ui) -> Response {
        let result = ui.scope_builder(
            egui::UiBuilder::new().layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.spacing_mut().item_spacing.y = space::MD;
                let size = self.size(ui.available_width());
                for _ in 0..self.count {
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    self.paint(ui, rect);
                }
            },
        );
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| node.set_hidden());
        result.response
    }
}
