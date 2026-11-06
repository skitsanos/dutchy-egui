use crate::{ColumnWidth, Spacing};
use egui::{Align, IdSalt, InnerResponse, Layout, Ui, UiBuilder};

/// Responsive equal-width columns with host-supplied, stable item keys.
///
/// Draws each item once, in row-major order; rows use their tallest child's
/// height. This is not a virtualized grid. Keys must be unique within this grid.
#[derive(Clone, Debug)]
#[must_use = "Call show to lay out content"]
pub struct ResponsiveGrid {
    id: IdSalt,
    columns: usize,
    minimum: ColumnWidth,
    gap: Spacing,
}

impl ResponsiveGrid {
    /// At most three columns, 280-point minimum, with 24-point gaps.
    pub fn new(id: impl egui::AsIdSalt) -> Self {
        Self {
            id: IdSalt::new(id),
            columns: 3,
            minimum: ColumnWidth::Card,
            gap: Spacing::Xxl,
        }
    }
    /// Maximum columns. Zero is treated as one.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }
    /// Minimum desired width; the final single column may be narrower.
    pub fn min_column_width(mut self, width: ColumnWidth) -> Self {
        self.minimum = width;
        self
    }
    /// Horizontal and vertical spacing.
    pub fn gap(mut self, gap: Spacing) -> Self {
        self.gap = gap;
        self
    }
    /// Render items with stable identities across reordering and column changes.
    /// The final partial row keeps the same column widths as preceding rows.
    pub fn show<'a, T, K: egui::AsIdSalt, R>(
        self,
        ui: &mut Ui,
        items: &'a [T],
        key: impl Fn(&'a T) -> K,
        mut contents: impl FnMut(&mut Ui, &'a T) -> R,
    ) -> InnerResponse<Vec<R>> {
        let id = ui.make_persistent_id(("dutchy-grid", self.id));
        let gap = self.gap.points();
        let fit = ((ui.available_width() + gap) / (self.minimum.points() + gap)).floor() as usize;
        let count = self.columns.min(fit.max(1)).min(items.len().max(1));
        ui.scope_builder(
            UiBuilder::new()
                .scope_id(id)
                .layout(Layout::top_down(Align::Min)),
            |ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::splat(gap);
                let mut results = Vec::with_capacity(items.len());
                for row in items.chunks(count) {
                    ui.columns(count, |columns| {
                        for (column, item) in columns.iter_mut().zip(row) {
                            let item_id = id.with(IdSalt::new(key(item)));
                            results.push(
                                column
                                    .scope_builder(UiBuilder::new().scope_id(item_id), |ui| {
                                        contents(ui, item)
                                    })
                                    .inner,
                            );
                        }
                    });
                }
                results
            },
        )
    }
}
