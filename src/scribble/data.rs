use super::{HISTORY_LIMIT, MAX_POINTS, MAX_STROKES, ScribbleError, ScribbleStroke};
use crate::tokens::scribble as tokens;
use egui::{Rect, Vec2};
use std::sync::{Arc, Weak};

/// Immutable, cheap-to-clone completed drawing for background export jobs.
#[derive(Clone, Debug, PartialEq)]
pub struct ScribbleSnapshot {
    pub(super) size: [u32; 2],
    pub(super) strokes: Vec<ScribbleStroke>,
    revision: u64,
}
impl ScribbleSnapshot {
    /// Logical drawing dimensions.
    pub const fn dimensions(&self) -> [u32; 2] {
        self.size
    }
    /// Completed strokes in paint order.
    pub fn strokes(&self) -> &[ScribbleStroke] {
        &self.strokes
    }
    /// Host drawing revision captured by this snapshot.
    pub const fn revision(&self) -> u64 {
        self.revision
    }
}
/// Host-owned drawing and bounded undo/redo history. Finished data is never kept
/// in egui's temporary view memory. Clear is undoable too.
#[derive(Debug)]
pub struct ScribbleDrawing {
    size: [u32; 2],
    strokes: Vec<ScribbleStroke>,
    past: Vec<Vec<ScribbleStroke>>,
    future: Vec<Vec<ScribbleStroke>>,
    revision: u64,
    generation: Arc<()>,
}
impl Clone for ScribbleDrawing {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            strokes: self.strokes.clone(),
            past: self.past.clone(),
            future: self.future.clone(),
            revision: self.revision,
            generation: Arc::new(()),
        }
    }
}
impl Default for ScribbleDrawing {
    fn default() -> Self {
        Self::new(tokens::WIDTH, tokens::HEIGHT).expect("valid drawing tokens")
    }
}
impl ScribbleDrawing {
    /// Empty drawing with immutable logical dimensions, independent of UI/DPI size.
    pub fn new(width: u32, height: u32) -> Result<Self, ScribbleError> {
        if width == 0 || height == 0 || width.max(height) > tokens::MAX_DIMENSION {
            return Err(ScribbleError::InvalidDimensions);
        }
        Ok(Self {
            size: [width, height],
            strokes: Vec::new(),
            past: Vec::new(),
            future: Vec::new(),
            revision: 0,
            generation: Arc::new(()),
        })
    }
    /// Logical drawing dimensions.
    pub const fn dimensions(&self) -> [u32; 2] {
        self.size
    }
    /// Logical size as egui coordinates.
    pub fn size(&self) -> Vec2 {
        egui::vec2(self.size[0] as f32, self.size[1] as f32)
    }
    /// Completed strokes in paint order.
    pub fn strokes(&self) -> &[ScribbleStroke] {
        &self.strokes
    }
    /// Number of completed points, including single-point dots.
    pub fn point_count(&self) -> usize {
        self.strokes
            .iter()
            .map(|stroke| stroke.points().len())
            .sum()
    }
    /// Whether no completed strokes exist.
    pub fn is_empty(&self) -> bool {
        self.strokes.is_empty()
    }
    /// Changes whenever the host model changes, including undo/redo.
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    /// Capture completed data without history for a worker or retained preview.
    pub fn snapshot(&self) -> ScribbleSnapshot {
        ScribbleSnapshot {
            size: self.size,
            strokes: self.strokes.clone(),
            revision: self.revision,
        }
    }
    /// Add a validated stroke, clearing redo history. Failure leaves all data intact.
    pub fn push(&mut self, stroke: ScribbleStroke) -> Result<(), ScribbleError> {
        if self.strokes.len() >= MAX_STROKES {
            return Err(ScribbleError::Capacity("strokes"));
        }
        if stroke.points().len() > MAX_POINTS.saturating_sub(self.point_count()) {
            return Err(ScribbleError::Capacity("drawing points"));
        }
        let bounds = Rect::from_min_size(egui::Pos2::ZERO, self.size());
        if stroke.points().iter().any(|point| !bounds.contains(*point)) {
            return Err(ScribbleError::OutOfBounds);
        }
        self.remember();
        self.strokes.push(stroke);
        self.bump();
        Ok(())
    }
    /// Whether undo is available.
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }
    /// Whether redo is available.
    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }
    /// Restore the previous completed state. Returns whether anything changed.
    pub fn undo(&mut self) -> bool {
        let Some(previous) = self.past.pop() else {
            return false;
        };
        self.future
            .push(std::mem::replace(&mut self.strokes, previous));
        self.bump();
        true
    }
    /// Reapply an undone completed state.
    pub fn redo(&mut self) -> bool {
        let Some(next) = self.future.pop() else {
            return false;
        };
        self.past.push(std::mem::replace(&mut self.strokes, next));
        self.bump();
        true
    }
    /// Clear completed strokes as one undoable operation. Empty clears are no-ops.
    pub fn clear(&mut self) -> bool {
        if self.is_empty() {
            return false;
        }
        self.remember();
        self.strokes.clear();
        self.bump();
        true
    }
    pub(crate) fn generation(&self) -> Weak<()> {
        Arc::downgrade(&self.generation)
    }
    fn remember(&mut self) {
        if self.past.len() == HISTORY_LIMIT {
            self.past.remove(0);
        }
        self.past.push(self.strokes.clone());
        self.future.clear();
    }
    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}
