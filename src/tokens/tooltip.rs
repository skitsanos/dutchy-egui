//! Compact inverted tooltip geometry.

/// Horizontal padding (live stylesheet `6px 12px`).
pub const PADDING_X: i8 = 12;
/// Vertical padding.
pub const PADDING_Y: i8 = 6;
/// Distance from the anchor to the tooltip box.
pub const GAP: f32 = 8.0;
/// Arrow length and half-width.
pub const ARROW: f32 = 5.0;
/// Maximum text width (live `max-width: min(256px, calc(100vw - 2rem))`).
pub const MAX_WIDTH: f32 = 256.0;
/// Viewport clearance subtracted from the maximum width.
pub const VIEWPORT_INSET: f32 = 32.0;
/// Letter spacing in em (live `0.02em`), between the shared tracking steps.
pub const TRACKING_EM: f32 = 0.02;
/// Reference tooltip line height multiplier.
pub const LINE_HEIGHT: f32 = 1.4;
