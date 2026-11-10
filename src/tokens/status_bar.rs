//! Status-bar geometry in logical points.
use super::{ControlSize, space};

/// Bar height: one compact control.
pub const HEIGHT: f32 = ControlSize::Small.height();
/// Inset before the first and after the last segment.
pub const INSET: f32 = space::SM;
/// Horizontal padding inside a segment; also the hover area of a clickable one.
pub const PADDING: f32 = space::SM;
/// Space between neighboring segments of one group.
pub const GAP: f32 = space::XS;
/// Minimum space between the start and end groups.
pub const GROUP_GAP: f32 = space::LG;
/// Space between the parts of one segment.
pub const PART_GAP: f32 = space::TIGHT;
/// Icon side.
pub const ICON: f32 = space::LG;
/// Side of the square tone marker.
pub const MARKER: f32 = space::SM;
/// Width of the inline progress track.
pub const PROGRESS_WIDTH: f32 = space::MAJOR;
/// Width fallback when the host supplies unbounded horizontal space.
pub const WIDTH: f32 = space::MAJOR * 10.0;
/// Finite upper bound for the bar width.
pub const MAX_WIDTH: f32 = 32768.0;
