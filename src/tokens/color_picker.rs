//! Geometry for flat color editing and transparency previews.

use super::{control, space};

/// Preferred popup width, constrained to the viewport.
pub const POPUP_WIDTH: f32 = 320.0;
/// Maximum scrolling editor height.
pub const POPUP_MAX_HEIGHT: f32 = 480.0;
/// Preset square side.
pub const SWATCH: f32 = control::INDICATOR + space::SM;
/// Checkerboard tile side; two flat semantic fills indicate transparency.
pub const CHECKER: f32 = space::XS;
/// Minimum content width for a channel's slider and number to share a row.
pub const CHANNEL_ROW_MIN: f32 = 240.0;
/// Channel number editor width, including its two step buttons.
pub const CHANNEL_VALUE_WIDTH: f32 = 112.0;
/// Short channel label column width.
pub const CHANNEL_LABEL_WIDTH: f32 = space::XXL;
