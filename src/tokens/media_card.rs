//! Flat artwork-card geometry.
use super::{ControlSize, space};

/// Metadata inset on all four edges.
pub const PADDING: f32 = space::LG;
/// Gap between metadata rows and title arrow.
pub const GAP: f32 = space::SM;
/// Independent bookmark target.
pub const SAVE_TARGET: f32 = ControlSize::Small.height();
/// Bookmark inset from the image edge.
pub const SAVE_INSET: f32 = space::MD;
/// Bookmark and directional glyph extent.
pub const ICON: f32 = space::LG;
/// Maximum wrapped title rows.
pub const TITLE_ROWS: usize = 2;
/// Maximum supporting description rows.
pub const DESCRIPTION_ROWS: usize = 3;
