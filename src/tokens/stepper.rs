//! Step sequence geometry in logical points.
use super::{ControlSize, space};
/// Standard/vertical square indicator.
pub const INDICATOR: f32 = space::LARGE;
/// Compact square indicator.
pub const COMPACT_INDICATOR: f32 = ControlSize::Small.height();
/// Minimum horizontal step column before switching to vertical presentation.
pub const MIN_COLUMN: f32 = 96.0;
/// Minimum progress-label column, allowing longer words before reflow.
pub const PROGRESS_COLUMN: f32 = 120.0;
/// Gap between a standard indicator and its label.
pub const LABEL_GAP: f32 = space::SM;
/// Gap between vertical indicator and text.
pub const TEXT_GAP: f32 = space::LG;
/// Gap between vertical title and description.
pub const DESCRIPTION_GAP: f32 = space::XS;
/// Space after a vertical step.
pub const VERTICAL_GAP: f32 = space::SECTION;
/// Clear space at both ends of a vertical connector.
pub const CONNECTOR_INSET: f32 = space::SM;
/// Length of a compact connector.
pub const COMPACT_CONNECTOR: f32 = space::SECTION;
/// Clear space at both ends of a compact connector.
pub const COMPACT_GAP: f32 = space::XS;
/// Space reserved around the compact scroll viewport for focus outlines.
pub const SCROLL_PADDING: i8 = 4;
/// Label-column inset.
pub const COLUMN_PADDING: f32 = space::XS;
/// Checkmark extent.
pub const CHECK: f32 = space::LG;
