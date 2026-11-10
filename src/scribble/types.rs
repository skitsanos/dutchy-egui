use crate::tokens::scribble as tokens;
use egui::{Color32, Pos2};
use std::sync::Arc;

/// Tokenized pen widths, expressed in drawing coordinates rather than screen pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScribbleWidth {
    /// One drawing unit.
    Fine,
    /// Two drawing units.
    #[default]
    Regular,
    /// Four drawing units.
    Bold,
    /// Eight drawing units.
    Heavy,
}
impl ScribbleWidth {
    /// Width in drawing units.
    pub const fn points(self) -> f32 {
        match self {
            Self::Fine => tokens::FINE,
            Self::Regular => tokens::REGULAR,
            Self::Bold => tokens::BOLD,
            Self::Heavy => tokens::HEAVY,
        }
    }
}
/// Immutable per-stroke color and width. New brush choices never recolor old ink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScribbleBrush {
    color: Color32,
    width: ScribbleWidth,
}
impl Default for ScribbleBrush {
    fn default() -> Self {
        Self {
            color: tokens::INK,
            width: Default::default(),
        }
    }
}
impl ScribbleBrush {
    /// Regular neutral ink on the fixed light paper.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set ordinary source-over ink; additive colors are normalized to RGBA.
    pub fn color(mut self, color: Color32) -> Self {
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        self.color = Color32::from_rgba_unmultiplied(r, g, b, a);
        self
    }
    /// Select a shared width token.
    pub fn width(mut self, width: ScribbleWidth) -> Self {
        self.width = width;
        self
    }
    /// Stored ink color.
    pub const fn ink(self) -> Color32 {
        self.color
    }
    /// Stored width token.
    pub const fn pen_width(self) -> ScribbleWidth {
        self.width
    }
}
/// One immutable polyline, or a single-point dot. Clone shares the point buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct ScribbleStroke {
    points: Arc<[Pos2]>,
    brush: ScribbleBrush,
}
impl ScribbleStroke {
    /// Validate a nonempty, finite stroke. The drawing checks its own bounds on add.
    pub fn new(points: Vec<Pos2>, brush: ScribbleBrush) -> Result<Self, ScribbleError> {
        if points.is_empty() {
            return Err(ScribbleError::EmptyStroke);
        }
        if points.len() > super::MAX_STROKE_POINTS {
            return Err(ScribbleError::Capacity("stroke points"));
        }
        if points.iter().any(|point| !point.is_finite()) {
            return Err(ScribbleError::InvalidPoint);
        }
        Ok(Self {
            points: points.into(),
            brush,
        })
    }
    /// Immutable drawing-coordinate points.
    pub fn points(&self) -> &[Pos2] {
        &self.points
    }
    /// Brush captured when this stroke began.
    pub const fn brush(&self) -> ScribbleBrush {
        self.brush
    }
}
/// File/export geometry. Exports contain ink/background only, never UI chrome.
///
/// Build with [`ScribbleExportOptions::new`] or `default()` and the
/// [`background`](Self::background) setter. New options may be added in minor
/// releases.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct ScribbleExportOptions {
    /// Output resolution multiplier; dimensions are rounded to whole pixels.
    pub scale: f32,
    /// None preserves transparent paper; Some composites this background.
    pub background: Option<Color32>,
}
impl Default for ScribbleExportOptions {
    fn default() -> Self {
        Self::new(1.0)
    }
}
impl ScribbleExportOptions {
    /// Transparent export at this resolution multiplier.
    pub const fn new(scale: f32) -> Self {
        Self {
            scale,
            background: None,
        }
    }

    /// Composite this background, or keep transparent paper with `None`.
    pub fn background(mut self, background: impl Into<Option<Color32>>) -> Self {
        self.background = background.into();
        self
    }
}
/// Invalid model input or an explicitly bounded export failure.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScribbleError {
    /// Drawing dimensions must be 1..=4096.
    InvalidDimensions,
    /// A stroke must contain a point.
    EmptyStroke,
    /// Coordinates must be finite.
    InvalidPoint,
    /// A point is outside the drawing-space rectangle.
    OutOfBounds,
    /// The named model capacity was exhausted.
    Capacity(&'static str),
    /// Scale must produce finite positive dimensions no larger than 8192 per side.
    InvalidExportSize,
    /// The raster would exceed 16,777,216 pixels.
    RasterTooLarge,
    /// The raster's estimated work exceeds the bounded sampling budget.
    RasterTooComplex,
    /// PNG encoding failed.
    Encoding(String),
}
impl std::fmt::Display for ScribbleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("Drawing dimensions must be between 1 and 4096"),
            Self::EmptyStroke => f.write_str("A stroke needs at least one point"),
            Self::InvalidPoint => f.write_str("Drawing points must be finite"),
            Self::OutOfBounds => f.write_str("A point is outside the drawing"),
            Self::Capacity(kind) => write!(f, "Drawing capacity reached: {kind}"),
            Self::InvalidExportSize => {
                f.write_str("Export dimensions must be between 1 and 8192 pixels")
            }
            Self::RasterTooLarge => f.write_str("The raster is too large; reduce the export scale"),
            Self::RasterTooComplex => {
                f.write_str("The raster is too complex; reduce the scale or export SVG")
            }
            Self::Encoding(error) => write!(f, "PNG encoding failed: {error}"),
        }
    }
}
impl std::error::Error for ScribbleError {}
