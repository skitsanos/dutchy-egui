use super::{Stat, StatSize};
use crate::{
    Theme,
    tokens::{border, metrics, space},
};
use egui::{Frame, IdSalt, Margin, Response, Stroke, Ui, UiBuilder, Widget};
use std::borrow::Cow;

/// Statistic arrangement. Both preserve tile surfaces, trends, and context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum StatsLayout {
    /// Tiles separated by two-point dividers.
    #[default]
    Grid,
    /// Smaller values/insets and an enclosing two-point border.
    Compact,
}

/// Responsive, named grid of host-supplied statistics.
#[must_use = "Add this grid to a Ui"]
pub struct StatsGrid<'a> {
    id: IdSalt,
    stats: &'a [Stat<'a>],
    columns: usize,
    size: StatSize,
    layout: StatsLayout,
    name: Cow<'a, str>,
}

impl<'a> StatsGrid<'a> {
    /// Up to four columns, reducing to fit the available width.
    pub fn new(id: impl egui::AsIdSalt, stats: &'a [Stat<'a>]) -> Self {
        Self {
            id: IdSalt::new(id),
            stats,
            columns: 4,
            size: StatSize::Default,
            layout: StatsLayout::Grid,
            name: Cow::Borrowed("Statistics"),
        }
    }
    /// Preferred maximum columns, clamped to 1..=4.
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.clamp(1, 4);
        self
    }
    /// Shared value type scale for regular tiles.
    pub fn size(mut self, size: StatSize) -> Self {
        self.size = size;
        self
    }
    /// Regular or compact layout.
    pub fn layout(mut self, layout: StatsLayout) -> Self {
        self.layout = layout;
        self
    }
    /// Group context such as "Monthly performance".
    pub fn accessible_name(mut self, name: impl Into<Cow<'a, str>>) -> Self {
        self.name = name.into();
        self
    }

    fn show(self, ui: &mut Ui) -> Response {
        let p = Theme::from_context(ui.ctx()).palette;
        let compact = self.layout == StatsLayout::Compact;
        let id = ui.make_persistent_id("cells");
        let result = Frame::NONE
            .fill(p.border)
            .inner_margin(Margin::ZERO)
            .stroke(if compact && !self.stats.is_empty() {
                Stroke::new(border::STANDARD, p.border)
            } else {
                Stroke::NONE
            })
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(space::MICRO, space::MICRO);
                let minimum = if compact {
                    metrics::COMPACT_COLUMN
                } else {
                    metrics::STAT_COLUMN
                };
                let columns = ((ui.available_width() + space::MICRO) / (minimum + space::MICRO))
                    .floor()
                    .max(1.0) as usize;
                let columns = columns.min(self.columns);
                for row in self.stats.chunks(columns) {
                    let tiles = ui.columns(columns, |columns| {
                        columns
                            .iter_mut()
                            .zip(row)
                            .map(|(ui, stat)| {
                                stat.tile(ui, id.with(stat.id), self.size, compact, false)
                            })
                            .collect::<Vec<_>>()
                    });
                    let bottom = tiles
                        .iter()
                        .map(|tile| tile.rect.bottom())
                        .fold(0.0_f32, f32::max);
                    for tile in tiles {
                        tile.finish(bottom);
                    }
                }
            });
        result.response
    }
}

impl Widget for StatsGrid<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let name = self.name.to_string();
        let result = ui.scope_builder(
            UiBuilder::new().scope_id(ui.make_persistent_id(("dutchy-stats-grid", self.id))),
            |ui| self.show(ui),
        );
        result
            .response
            .ctx
            .accesskit_node_builder(result.response.id, |node| {
                node.set_role(egui::accesskit::Role::Group);
                node.set_label(name);
            });
        result.response
    }
}
