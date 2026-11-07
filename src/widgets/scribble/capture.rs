use super::{ScribbleAction, keyboard};
use crate::{
    ScribbleBrush, ScribbleDrawing, ScribbleError, ScribbleStroke,
    scribble::{MAX_POINTS, MAX_STROKE_POINTS},
    tokens::scribble as tokens,
};
use egui::{Event, Pos2, Rect, Response, Ui, emath::RectTransform};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Source {
    Pointer,
    Keyboard,
}
#[derive(Clone)]
pub(super) struct Draft {
    pub points: Vec<Pos2>,
    pub brush: ScribbleBrush,
    pub source: Source,
    pub revision: u64,
    pub generation: std::sync::Weak<()>,
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
    pub fn start(
        &mut self,
        point: Pos2,
        brush: ScribbleBrush,
        source: Source,
        drawing: &ScribbleDrawing,
    ) {
        self.cursor = Some(point);
        self.draft = Some(Draft {
            points: vec![point],
            brush,
            source,
            revision: drawing.revision(),
            generation: drawing.generation(),
        });
    }
    pub fn append(&mut self, point: Pos2, drawing: &ScribbleDrawing) -> Result<(), ScribbleError> {
        self.cursor = Some(point);
        let Some(draft) = &mut self.draft else {
            return Ok(());
        };
        if draft
            .points
            .last()
            .is_some_and(|last| last.distance(point) < tokens::SAMPLE_DISTANCE)
        {
            return Ok(());
        }
        let remaining = MAX_POINTS.saturating_sub(drawing.point_count());
        if draft.points.len() >= MAX_STROKE_POINTS.min(remaining) {
            return Err(ScribbleError::Capacity("gesture points"));
        }
        draft.points.push(point);
        Ok(())
    }
    pub fn finish(&mut self, drawing: &mut ScribbleDrawing, outcome: &mut Outcome) {
        if let Some(draft) = self.draft.take() {
            match ScribbleStroke::new(draft.points, draft.brush)
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
    brush: ScribbleBrush,
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
        process(ui, canvas, drawing, state, brush, &mut outcome, event);
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
    brush: ScribbleBrush,
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
                state.start(
                    transform.transform_pos(pos),
                    brush,
                    Source::Pointer,
                    drawing,
                );
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
                keyboard::key(ui, drawing, state, brush, outcome, key, modifiers);
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
