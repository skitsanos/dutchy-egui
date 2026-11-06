//! Progress and placeholder geometry.

use super::space;

/// Progress track height, independent of input/button heights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProgressSize {
    /// Four-point track.
    Small,
    /// Eight-point track.
    #[default]
    Medium,
    /// Sixteen-point track.
    Large,
}

impl ProgressSize {
    /// Track height in logical points.
    pub const fn height(self) -> f32 {
        match self {
            Self::Small => space::XS,
            Self::Medium => space::SM,
            Self::Large => space::LG,
        }
    }
}

/// Default progress maximum, including fallback for an invalid maximum.
pub const PROGRESS_MAX: f64 = 100.0;
/// Fraction of the track occupied by an unknown-progress segment.
pub const INDETERMINATE_FRACTION: f32 = 0.25;
/// Minimum pulse opacity.
pub const SKELETON_MIN_OPACITY: f32 = 0.4;
/// Text placeholder height.
pub const TEXT_HEIGHT: f32 = space::LG;
/// Title placeholder height.
pub const TITLE_HEIGHT: f32 = space::SECTION;
/// Square avatar placeholder side.
pub const AVATAR_SIDE: f32 = space::HERO;
/// Image/card placeholder height.
pub const CARD_HEIGHT: f32 = 192.0;
/// Default title width relative to its column.
pub const TITLE_FRACTION: f32 = 0.75;
/// Preferred width of fixed trailing table placeholder cells.
pub const TABLE_CELL_WIDTH: f32 = space::PAGE;
/// Action placeholder width in the card pattern.
pub const ACTION_WIDTH: f32 = 128.0;
