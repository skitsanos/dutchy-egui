//! Native egui layout composition with shared spacing and stable child scopes.
//!
//! These helpers do not measure children twice, implement CSS, or own host state.

mod container;
mod flex;
mod grid;

pub use container::Container;
pub use flex::{CrossAlign, Flex, FlexDirection};
pub use grid::ResponsiveGrid;
