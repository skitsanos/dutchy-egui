//! Fixed-height collection geometry and type-ahead timing.
use super::{ControlSize, space};
/// Default row height.
pub const ROW_HEIGHT: f32 = ControlSize::Medium.height();
/// Smallest supported row height.
pub const MIN_ROW_HEIGHT: f32 = ControlSize::Small.height();
/// Largest supported row height.
pub const MAX_ROW_HEIGHT: f32 = space::MAJOR * 4.0;
/// Default viewport height cap.
pub const HEIGHT: f32 = super::table::BODY_HEIGHT;
/// Minimum viewport height cap.
pub const MIN_HEIGHT: f32 = super::table::MIN_BODY_HEIGHT;
/// Maximum viewport height cap.
pub const MAX_HEIGHT: f32 = super::table::MAX_BODY_HEIGHT;
/// Row content inset.
pub const PADDING: f32 = space::LG;
/// Selected surface tint.
pub const SELECTED_OPACITY: f32 = super::table::SELECTED_OPACITY;
/// Hover surface tint.
pub const HOVER_OPACITY: f32 = super::table::HOVER_OPACITY;
/// Idle time before a typed prefix starts a new search, in seconds.
pub const TYPE_AHEAD_TIMEOUT: f64 = 1.0;
/// Maximum Unicode characters retained in the typed prefix.
pub const TYPE_AHEAD_LIMIT: usize = 64;
