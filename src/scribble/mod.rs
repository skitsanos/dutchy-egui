//! Host-owned vector drawing data and bounded, I/O-free exports.
//! The [`crate::Scribble`] widget edits this model; applications execute its
//! history/export requests and retain any saved bytes or native paths.
mod data;
mod export;
mod raster;
mod types;

pub use data::{ScribbleDrawing, ScribbleSnapshot};
pub use types::{
    ScribbleBrush, ScribbleError, ScribbleExportOptions, ScribbleStroke, ScribbleWidth,
};

/// Maximum completed strokes in a drawing.
pub const MAX_STROKES: usize = 512;
/// Maximum points across completed strokes.
pub const MAX_POINTS: usize = 65_536;
/// Maximum points in one stroke.
pub const MAX_STROKE_POINTS: usize = 8192;
/// Bounded history depth; snapshots share immutable stroke point buffers.
pub const HISTORY_LIMIT: usize = 32;
