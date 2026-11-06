//! Data table geometry, widths and surfaces.
use super::space;
/// Default minimum width of a flexible data column.
pub const COLUMN_WIDTH: f32 = 160.0;
/// Minimum accepted data column width.
pub const MIN_COLUMN: f32 = 96.0;
/// Maximum accepted preferred/minimum column width.
pub const MAX_COLUMN: f32 = 4096.0;
/// Selection column, including its native control target.
pub const SELECTION_WIDTH: f32 = 64.0;
/// Default body viewport maximum height.
pub const BODY_HEIGHT: f32 = 384.0;
/// Minimum configurable body viewport maximum height.
pub const MIN_BODY_HEIGHT: f32 = 96.0;
/// Maximum configurable body viewport maximum height.
pub const MAX_BODY_HEIGHT: f32 = 4096.0;
/// Normal horizontal cell/header padding.
pub const PADDING_X: i8 = 24;
/// Normal data cell vertical padding.
pub const PADDING_Y: i8 = 16;
/// Normal header vertical padding.
pub const HEADER_Y: i8 = 12;
/// Compact horizontal padding.
pub const COMPACT_X: i8 = 16;
/// Compact vertical padding.
pub const COMPACT_Y: i8 = 8;
/// Sort arrow extent.
pub const SORT_ICON: f32 = space::MD;
/// Space between header text and sort arrow.
pub const SORT_GAP: f32 = space::XS;
/// Alternating muted surface blend.
pub const STRIPE_OPACITY: f32 = 0.3;
/// Row hover muted surface blend.
pub const HOVER_OPACITY: f32 = 0.5;
/// Selected row primary tint.
pub const SELECTED_OPACITY: f32 = 0.12;
/// Number of decorative loading rows.
pub const LOADING_ROWS: usize = 4;
/// Gap between loading rows.
pub const LOADING_GAP: f32 = space::LG;
