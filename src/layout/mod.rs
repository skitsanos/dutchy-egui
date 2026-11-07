//! Native egui layout composition with shared spacing and stable child scopes.
//!
//! These helpers do not measure children twice, implement CSS, or own host state.

mod container;
mod flex;
mod grid;

pub use container::Container;
pub use flex::{CrossAlign, Flex, FlexDirection};
pub use grid::ResponsiveGrid;

/// Advance so the next widget sits `total` points below the previous one,
/// absorbing the item spacing egui already adds after a widget.
pub(crate) fn gap_after(ui: &mut egui::Ui, total: f32) {
    ui.add_space((total - ui.spacing().item_spacing.y).max(0.0));
}
