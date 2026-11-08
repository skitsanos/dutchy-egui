//! Code block geometry and copy feedback timing.
use super::space;

/// Padding inside the code surface.
pub const PADDING: f32 = space::LG;
/// Gap between the header row (language, copy) and the code.
pub const HEADER_GAP: f32 = space::MD;
/// Gap between line numbers and code.
pub const GUTTER_GAP: f32 = space::LG;
/// Seconds the copy control reads "Copied".
pub const COPY_FEEDBACK: f64 = 1.5;
/// Default maximum body height before the block scrolls vertically.
pub const MAX_HEIGHT: f32 = 480.0;
