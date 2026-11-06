//! Notification stack geometry and lifetime policy.
use super::space;
use std::time::Duration;
/// Preferred outer stack width, constrained by the viewport.
pub const WIDTH: f32 = 384.0;
/// Clearance from viewport edges.
pub const GUTTER: f32 = space::XXL;
/// Space between visible notifications.
pub const GAP: f32 = space::MD;
/// Horizontal content padding.
pub const PADDING_X: i8 = 24;
/// Vertical content padding.
pub const PADDING_Y: i8 = 16;
/// Insets for the documented outline treatment.
pub const OUTLINE_PADDING: i8 = 16;
/// Text/controls gap.
pub const CONTENT_GAP: f32 = space::MD;
/// Space above an optional action.
pub const ACTION_GAP: f32 = space::SM;
/// Maximum simultaneously rendered notifications.
pub const MAX_VISIBLE: usize = 4;
/// Default visible count; additional entries wait without expiring.
pub const DEFAULT_VISIBLE: usize = 3;
/// Default visible lifetime.
pub const DEFAULT_DURATION: Duration = Duration::from_secs(5);
/// Minimum requested visible lifetime.
pub const MIN_DURATION: Duration = Duration::from_secs(4);
/// Translucent accent edge on solid surfaces.
pub const EDGE_OPACITY: f32 = 0.2;
