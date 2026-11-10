//! In-place rename geometry in logical points.
use super::space;

/// Gap between the editor and explicit actions.
pub const GAP: f32 = space::XS;
/// Editor width retained before actions move to a second row.
pub const MIN_EDITOR: f32 = space::MAJOR;
/// Fallback width in an unbounded parent.
pub const WIDTH: f32 = space::PAGE * 4.0;
/// Finite upper allocation bound.
pub const MAX_WIDTH: f32 = 32768.0;
/// Label/editor horizontal inset.
pub const PADDING: f32 = space::MD;
/// Rename affordance glyph extent.
pub const ICON: f32 = space::LG;
