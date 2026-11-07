//! Drawing surface, pen and keyboard geometry.
use super::{Accent, Mode, Palette, space};
/// Default drawing-space width.
pub const WIDTH: u32 = 600;
/// Default drawing-space height.
pub const HEIGHT: u32 = 200;
/// Maximum drawing-space dimension.
pub const MAX_DIMENSION: u32 = 4096;
/// Fixed light paper, independent of the surrounding application's theme.
pub const PAPER: egui::Color32 = Palette::new(Accent::Orange, Mode::Light).background;
/// Default neutral ink, readable on the fixed paper.
pub const INK: egui::Color32 = Palette::new(Accent::Orange, Mode::Light).foreground;
/// Placeholder and keyboard cursor color.
pub const MUTED_INK: egui::Color32 = Palette::new(Accent::Orange, Mode::Light).muted_foreground;
/// Smallest stored movement, in drawing units.
pub const SAMPLE_DISTANCE: f32 = 0.5;
/// Light pointer-smoothing stabilizer string, in drawing units.
pub const SMOOTH_LIGHT: f32 = 2.0;
/// Regular pointer-smoothing stabilizer string, in drawing units.
pub const SMOOTH_REGULAR: f32 = 4.0;
/// Strong pointer-smoothing stabilizer string, in drawing units.
pub const SMOOTH_STRONG: f32 = 8.0;
/// Maximum curve-flattening and simplification error, in drawing units.
pub const SMOOTH_TOLERANCE: f32 = 0.25;
/// Ordinary arrow-key movement, in drawing units.
pub const KEY_STEP: f32 = space::XS;
/// Shift + arrow-key movement, in drawing units.
pub const KEY_STEP_LARGE: f32 = space::LG;
/// Keyboard pen cursor extent in UI points.
pub const CURSOR_SIZE: f32 = space::SM;
/// Fine pen width in drawing units.
pub const FINE: f32 = 1.0;
/// Regular pen width in drawing units.
pub const REGULAR: f32 = 2.0;
/// Bold pen width in drawing units.
pub const BOLD: f32 = 4.0;
/// Heavy pen width in drawing units.
pub const HEAVY: f32 = 8.0;
