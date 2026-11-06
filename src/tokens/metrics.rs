//! Metric and empty-state layout geometry.

use super::space;

/// Minimum regular statistic column width before the grid reduces its columns.
pub const STAT_COLUMN: f32 = 200.0;
/// Minimum compact statistic column width.
pub const COMPACT_COLUMN: f32 = 160.0;
/// Width at which a regular stat uses the largest display value.
pub const LARGE_VALUE_WIDTH: f32 = 240.0;
/// Decorative trend arrow side.
pub const TREND_ICON: f32 = space::LG;
/// Maximum empty-state body width.
pub const EMPTY_TEXT_WIDTH: f32 = 384.0;
/// Maximum dashboard description width.
pub const DASHBOARD_TEXT_WIDTH: f32 = 448.0;
/// Standard decorative icon tile.
pub const EMPTY_ICON: f32 = space::MAJOR;
/// Table-body icon tile.
pub const TABLE_ICON: f32 = space::HERO;
/// Dashboard icon tile.
pub const DASHBOARD_ICON: f32 = space::PAGE;
