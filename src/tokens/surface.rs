//! Shared card and notification geometry.

use super::space;

/// Insets around card content, matching the web padding tiers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardPadding {
    /// Edge-to-edge content.
    None,
    /// 24-point inset.
    Small,
    /// Default 32-point inset.
    #[default]
    Medium,
    /// 40-point inset.
    Large,
}

impl CardPadding {
    /// Logical-point inset on every side.
    pub const fn points(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Small => space::XXL,
            Self::Medium => space::SECTION,
            Self::Large => space::LARGE,
        }
    }
}

/// Notification inset.
pub const ALERT_PADDING: f32 = space::LG;
/// Status-symbol side length.
pub const STATUS_ICON: f32 = 20.0;
/// Opaque tint blended over the current background.
pub const STATUS_TINT: f32 = 0.1;
