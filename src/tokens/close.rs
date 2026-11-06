//! Shared close-button emphasis and icon dimensions.
use super::space;

/// Foreground tint for the flat hover surface.
pub const HOVER_TINT: f32 = 0.12;
/// Stronger foreground tint while pressed.
pub const PRESSED_TINT: f32 = 0.2;
/// Resting icon opacity on an inherited surface.
pub const IDLE_OPACITY: f32 = 0.7;
/// Alert close glyph extent inside the compact button target.
pub const ALERT_ICON: f32 = space::LG;
/// Compact removable-tag close glyph extent.
pub const BADGE_ICON: f32 = space::SM;
