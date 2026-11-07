use super::{ScribbleAction, keyboard};
use crate::{
    ScribbleBrush, ScribbleDrawing, ScribbleError, ScribbleSmoothing, ScribbleStroke,
    scribble::{MAX_POINTS, MAX_STROKE_POINTS, Stabilizer, fit, simplify},
    tokens::scribble as tokens,
};
use egui::{Event, Pos2, Rect, Response, Ui, emath::RectTransform};
use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Source {
    Pointer,
    Keyboard,
}
/// Brush and smoothing captured when a gesture starts.
#[derive(Clone, Copy)]
pub(super) struct Pen {
    pub brush: ScribbleBrush,
    pub smoothing: ScribbleSmoothing,
}
#[derive(Clone)]
pub(super) struct Draft {
    pub points: Vec<Pos2>,
    pub brush: ScribbleBrush,
    pub source: Source,
    pub revision: u64,
    pub generation: std::sync::Weak<()>,
    /// Present only for smoothed pointer gestures; keyboard input stays exact.
    stabilizer: Option<Stabilizer>,
}
impl Draft {
    /// Live geometry, matching the committed stroke up to simplification.
    pub fn preview(&self) -> Cow<'_, [Pos2]> {
        match &self.stabilizer {
            Some(stabilizer) => Cow::Owned(fit(&stabilizer.trail(&self.points))),
            None => Cow::Borrowed(&self.points),
        }
    }
    /// Smoothed points when they fit the remaining capacity, else the stored points.
    fn commit(self, drawing: &ScribbleDrawing) -> Vec<Pos2> {
        let Some(stabilizer) = self.stabilizer else {
            return self.points;
        };
        let smoothed = simplify(&fit(&stabilizer.trail(&self.points)));
        if smoothed.len() <= capacity(drawing) {
            smoothed
        } else {
            // Stored points were capped while capturing, so they always fit.
            self.points
        }
    }
    fn push(&mut self, point: Pos2, drawing: &ScribbleDrawing) -> Result<(), ScribbleError> {
        if self.points.len() >= capacity(drawing) {
            return Err(ScribbleError::Capacity("gesture points"));
        }
        self.points.push(point);
        Ok(())
    }
}
fn capacity(drawing: &ScribbleDrawing) -> usize {
    MAX_STROKE_POINTS.min(MAX_POINTS.saturating_sub(drawing.point_count()))
}
#[derive(Clone, Default)]
pub(super) struct State {
    pub cursor: Option<Pos2>,
    pub draft: Option<Draft>,
    pub rect: Option<Rect>,
    frame: Option<(egui::ViewportId, u64)>,
}
#[derive(Default)]
pub(super) struct Outcome {
    pub strokes: usize,
    pub actions: Vec<ScribbleAction>,
    pub error: Option<ScribbleError>,
    pub keyboard_used: bool,
}
impl State {
    pub fn start(&mut self, point: Pos2, pen: Pen, source: Source, drawing: &ScribbleDrawing) {
        self.cursor = Some(point);
        let stabilizer = match source {
            Source::Pointer => Stabilizer::new(pen.smoothing, point),
            Source::Keyboard => None,
        };
        self.draft = Some(Draft {
            points: vec![point],
            brush: pen.brush,
            source,
            revision: drawing.revision(),
            generation: drawing.generation(),
            stabilizer,
        });
    }
    pub fn append(&mut self, point: Pos2, drawing: &ScribbleDrawing) -> Result<(), ScribbleError> {
        self.cursor = Some(point);
        let Some(draft) = &mut self.draft else {
            return Ok(());
        };
        let Some(&last) = draft.points.last() else {
            return Ok(());
        };
        let next = match &mut draft.stabilizer {
            Some(stabilizer) => stabilizer.advance(point, last),
            None => (last.distance(point) >= tokens::SAMPLE_DISTANCE).then_some(point),
        };
        match next {
            Some(next) => draft.push(next, drawing),
            None => Ok(()),
        }
    }
    pub fn finish(&mut self, drawing: &mut ScribbleDrawing, outcome: &mut Outcome) {
        if let Some(draft) = self.draft.take() {
            let brush = draft.brush;
            match ScribbleStroke::new(draft.commit(drawing), brush)
                .and_then(|stroke| drawing.push(stroke))
            {
                Ok(()) => outcome.strokes += 1,
                Err(error) => outcome.error = Some(error),
            }
        }
    }
    pub fn is_pointer(&self) -> bool {
        self.draft
            .as_ref()
            .is_some_and(|draft| draft.source == Source::Pointer)
    }
}
pub(super) fn update(
    ui: &Ui,
    canvas: &Response,
    drawing: &mut ScribbleDrawing,
    state: &mut State,
    pen: Pen,
    editable: bool,
) -> Outcome {
    let mut outcome = Outcome::default();
    let invalid = state.draft.as_ref().is_some_and(|draft| {
        draft.revision != drawing.revision()
            || !std::sync::Weak::ptr_eq(&draft.generation, &drawing.generation())
    }) || (state.is_pointer() && state.rect.is_some_and(|rect| rect != canvas.rect));
    if !editable || invalid {
        state.draft = None;
    }
    state.rect = Some(canvas.rect);
    let bounds = Rect::from_min_size(Pos2::ZERO, drawing.size());
    if state.cursor.is_none_or(|point| !bounds.contains(point)) {
        state.cursor = Some((drawing.size() / 2.0).to_pos2());
    }
    let frame = (ui.ctx().viewport_id(), ui.ctx().cumulative_frame_nr());
    let fresh = state.frame != Some(frame);
    state.frame = Some(frame);
    if !editable || !fresh {
        return outcome;
    }
    let events = ui.input(|input| input.raw.events.clone());
    for event in events {
        process(ui, canvas, drawing, state, pen, &mut outcome, event);
    }
    if state
        .draft
        .as_ref()
        .is_some_and(|draft| draft.source == Source::Keyboard)
        && !canvas.has_focus()
    {
        state.draft = None;
    }
    outcome
}
fn process(
    ui: &Ui,
    canvas: &Response,
    drawing: &mut ScribbleDrawing,
    state: &mut State,
    pen: Pen,
    outcome: &mut Outcome,
    event: Event,
) {
    let transform =
        RectTransform::from_to(canvas.rect, Rect::from_min_size(Pos2::ZERO, drawing.size()));
    match event {
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            ..
        } => {
            if accepts(ui, canvas, pos) {
                canvas.request_focus();
                state.start(transform.transform_pos(pos), pen, Source::Pointer, drawing);
            }
        }
        Event::PointerMoved(pos) if state.is_pointer() => {
            if canvas.rect.contains(pos) && ui.clip_rect().contains(pos) {
                if let Err(error) = state.append(transform.transform_pos(pos), drawing) {
                    // The gesture ends at the cap; the captured ink is kept.
                    state.finish(drawing, outcome);
                    outcome.error.get_or_insert(error);
                }
            } else {
                state.finish(drawing, outcome);
            }
        }
        Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            ..
        } if state.is_pointer() => {
            if canvas.rect.contains(pos)
                && ui.clip_rect().contains(pos)
                && let Err(error) = state.append(transform.transform_pos(pos), drawing)
            {
                outcome.error = Some(error);
            }
            state.finish(drawing, outcome);
        }
        Event::PointerGone if state.is_pointer() => state.finish(drawing, outcome),
        Event::WindowFocused(false) => state.draft = None,
        Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } if canvas.has_focus() => {
            outcome.keyboard_used |=
                keyboard::key(ui, drawing, state, pen, outcome, key, modifiers);
        }
        _ => {}
    }
}
fn accepts(ui: &Ui, canvas: &Response, point: Pos2) -> bool {
    canvas.rect.contains(point)
        && ui.clip_rect().contains(point)
        && ui.ctx().layer_id_at(point) == Some(ui.layer_id())
        && ui.memory(|memory| {
            memory
                .top_modal_layer()
                .is_none_or(|layer| layer == ui.layer_id())
        })
}
