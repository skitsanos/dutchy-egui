//! Split-pane geometry and keyboard increments in logical points.
use super::space;

/// Divider interaction width, including the grip.
pub const DIVIDER: f32 = space::MD;
/// Length of the centered grip.
pub const GRIP: f32 = space::XXL;
/// Default allocated height.
pub const HEIGHT: f32 = space::MAJOR * 5.0;
/// Width fallback when the host supplies unbounded horizontal space.
pub const WIDTH: f32 = space::MAJOR * 10.0;
/// Default minimum size of either pane.
pub const MIN_SIZE: f32 = space::HERO;
/// Normal arrow-key increment.
pub const STEP: f32 = space::SM;
/// Shift+arrow increment.
pub const LARGE_STEP: f32 = space::SECTION;
/// Finite upper bound for dimensions and supplied minima.
pub const MAX_SIZE: f32 = 32768.0;
