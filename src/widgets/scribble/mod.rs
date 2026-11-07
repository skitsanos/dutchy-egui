mod capture;
mod keyboard;
mod paint;
mod render;
mod toolbar;

use crate::{ScribbleBrush, ScribbleDrawing, ScribbleError, ScribbleSmoothing};
use egui::{IdSalt, Response, Ui};
use std::borrow::Cow;

/// History/export request. Applications execute these in order against their
/// drawing or snapshot; the widget never opens files or writes exported data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScribbleAction {
    /// Undo one completed host-model operation.
    Undo,
    /// Redo one completed host-model operation.
    Redo,
    /// Clear completed ink as one undoable host-model operation.
    Clear,
    /// Generate a retained raster preview from a completed snapshot.
    Preview,
    /// Export a completed snapshot as SVG.
    ExportSvg,
    /// Export a completed snapshot as PNG (`png-export` feature).
    #[cfg(feature = "png-export")]
    ExportPng,
}
/// Canvas editing and independently executable history/export requests.
pub struct ScribbleResponse {
    /// Named composite, including label, drawing surface and optional controls.
    pub response: Response,
    /// Focusable drawing surface, excluding its border and controls.
    pub canvas: Response,
    /// Number of strokes committed directly to the bound host drawing this pass.
    pub strokes_added: usize,
    /// Ordered requests, including focused-canvas keyboard commands.
    pub actions: Vec<ScribbleAction>,
    /// Activated toolbar control, for native dialog focus return.
    pub action_response: Option<Response>,
    /// Capacity/input failure; completed host data remains intact.
    pub error: Option<ScribbleError>,
}
/// Freehand surface bound to a host-owned vector drawing. Pointer/touch input
/// records ordinary strokes; focused arrow/Space/Enter input is an alternative.
#[must_use = "Call show to edit the drawing and receive action requests"]
pub struct Scribble<'a> {
    id: IdSalt,
    drawing: &'a mut ScribbleDrawing,
    label: Cow<'a, str>,
    placeholder: Option<Cow<'a, str>>,
    brush: ScribbleBrush,
    smoothing: ScribbleSmoothing,
    width: Option<f32>,
    enabled: bool,
    read_only: bool,
    controls: bool,
    instructions: bool,
    exports_enabled: bool,
}
impl<'a> Scribble<'a> {
    /// Editable canvas with regular neutral ink, instructions and wrapped controls.
    pub fn new(id: impl egui::AsIdSalt, drawing: &'a mut ScribbleDrawing) -> Self {
        Self {
            id: IdSalt::new(id),
            drawing,
            label: "Drawing".into(),
            placeholder: None,
            brush: Default::default(),
            smoothing: Default::default(),
            width: None,
            enabled: true,
            read_only: false,
            controls: true,
            instructions: true,
            exports_enabled: true,
        }
    }
    /// Visible and accessible drawing name.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }
    /// Override the empty hint (default: "Draw here", or "No drawing" when read-only).
    /// Shown only while both completed and transient drawing data are empty.
    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    /// Brush for newly started strokes; completed strokes retain their own brush.
    pub fn brush(mut self, brush: ScribbleBrush) -> Self {
        self.brush = brush;
        self
    }
    /// Smoothing for newly started pointer strokes (default: off). Applied once
    /// when a stroke is committed; keyboard strokes always keep exact positions.
    pub fn smoothing(mut self, smoothing: ScribbleSmoothing) -> Self {
        self.smoothing = smoothing;
        self
    }
    /// Preferred outer width, constrained to the available column and drawing aspect.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.is_finite().then_some(width.max(1.0));
        self
    }
    /// Disable drawing and all actions, honoring disabled parent UIs too.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    /// Prevent drawing/history edits while retaining inspection and export actions.
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
    /// Show/hide the native history/export toolbar. Focused editing shortcuts remain.
    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }
    /// Show/hide visible keyboard help; accessible instructions remain available.
    pub fn instructions(mut self, instructions: bool) -> Self {
        self.instructions = instructions;
        self
    }
    /// Disable preview/export requests, e.g. while the host runs an export job.
    pub fn exports_enabled(mut self, enabled: bool) -> Self {
        self.exports_enabled = enabled;
        self
    }
    /// Edit completed strokes through the bound model; return other host requests.
    pub fn show(self, ui: &mut Ui) -> ScribbleResponse {
        render::show(self, ui)
    }
}
