//! File selection and transfer presentation geometry.
use super::{ControlSize, space};
/// Preferred drop-zone height; wrapped content may grow it.
pub const DROP_HEIGHT: f32 = 192.0;
/// Drop-zone inner clearance.
pub const PADDING: f32 = space::XXL;
/// File-row inner clearance.
pub const ROW_PADDING: i8 = space::MD as i8;
/// Upload glyph size.
pub const ICON_SIZE: f32 = space::XL;
/// File glyph size.
pub const FILE_ICON_SIZE: f32 = space::LG;
/// File action square target.
pub const ACTION_SIZE: f32 = ControlSize::Small.height();
/// Close glyph size.
pub const CLOSE_ICON_SIZE: f32 = space::MD;
/// Retry target width.
pub const RETRY_WIDTH: f32 = 72.0;
/// Minimum text width before the decorative file icon is omitted.
pub const MIN_TEXT_WIDTH: f32 = 64.0;
/// Dashed border segment length.
pub const DASH: f32 = 6.0;
/// Dashed border gap.
pub const DASH_GAP: f32 = space::XS;
/// Flat accent fill on hover and external drag-over.
pub const HOVER_TINT: f32 = 0.05;

/// Flat accent fill while pressing the selection control.
pub const PRESSED_TINT: f32 = 0.1;
