//! Focus reveals once; wheel scrolling remains under the application's/user's control.
mod common;
use dutchy_egui::{Scribble, ScribbleBrush, ScribbleDrawing, ScribbleStroke, Theme};
use egui::{Event, Id, Key, Modifiers, MouseWheelUnit, Rect, accesskit::Role, pos2, vec2};
use egui_kittest::{Harness, kittest::Queryable};
struct State {
    drawing: ScribbleDrawing,
    offset: f32,
    viewport: Rect,
    editable: Rect,
    readonly: Rect,
    readonly_id: Option<Id>,
}
impl Default for State {
    fn default() -> Self {
        let mut drawing = ScribbleDrawing::default();
        drawing
            .push(ScribbleStroke::new(vec![pos2(10.0, 10.0)], ScribbleBrush::default()).unwrap())
            .unwrap();
        Self {
            drawing,
            offset: 0.0,
            viewport: Rect::NOTHING,
            editable: Rect::NOTHING,
            readonly: Rect::NOTHING,
            readonly_id: None,
        }
    }
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 420.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                let output = egui::ScrollArea::vertical()
                    .id_salt("drawing-page")
                    .animated(false)
                    .show(ui, |ui| {
                        let response = Scribble::new("edit", &mut state.drawing)
                            .label("Editable")
                            .width(600.0)
                            .instructions(false)
                            .show(ui);
                        state.editable = response.canvas.rect;
                        ui.add_space(600.0);
                        let response = Scribble::new("read", &mut state.drawing)
                            .label("Read-only")
                            .width(600.0)
                            .read_only(true)
                            .controls(false)
                            .instructions(false)
                            .show(ui);
                        state.readonly = response.canvas.rect;
                        state.readonly_id = Some(response.canvas.id);
                    });
                state.offset = output.state.clamped_offset().y;
                state.viewport = output.inner_rect;
            },
            State::default(),
        )
}
fn wheel(h: &mut Harness<'_, State>, delta: f32) {
    h.hover_at(h.state().viewport.center());
    h.run_steps(2);
    h.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: vec2(0.0, delta),
        modifiers: Modifiers::NONE,
    });
    h.run_steps(80);
}
#[test]
fn focused_editable_canvas_does_not_pull_the_page_back_from_the_readonly_view() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Canvas, "Editable").focus();
    h.run_steps(3);
    let snapshot = h.state().drawing.snapshot();
    wheel(&mut h, -1500.0);
    assert!(
        h.state().viewport.contains_rect(h.state().readonly),
        "{:?} {:?} offset={}",
        h.state().viewport,
        h.state().readonly,
        h.state().offset
    );
    assert_eq!(h.state().drawing.snapshot(), snapshot);
    assert!(
        h.get_by_role_and_label(Role::Canvas, "Editable")
            .is_focused()
    );
}
#[test]
fn focused_toolbar_buttons_do_not_block_scrolling_to_later_sections() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Export SVG Editable").focus();
    h.run_steps(3);
    wheel(&mut h, -1500.0);
    assert!(
        h.state().viewport.contains_rect(h.state().readonly),
        "offset={}",
        h.state().offset
    );
    assert!(h.get_by_label("Export SVG Editable").is_focused());
}
#[test]
fn readonly_focus_reveals_the_canvas_once_and_then_allows_scrolling_away() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Canvas, "Read-only").focus();
    h.run_steps(4);
    assert!(h.state().viewport.contains_rect(h.state().readonly));
    wheel(&mut h, 1500.0);
    assert!(h.state().offset < 1.0, "offset={}", h.state().offset);
    assert_eq!(h.ctx.memory(|m| m.focused()), h.state().readonly_id);
}
#[test]
fn keyboard_drawing_reveals_the_scrolled_away_canvas_and_retains_its_draft() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Canvas, "Editable").focus();
    h.run_steps(3);
    wheel(&mut h, -1500.0);
    assert!(h.state().offset > 400.0);
    for key in [Key::Space, Key::ArrowRight, Key::ArrowDown, Key::Enter] {
        h.key_press(key);
        h.run_steps(3);
    }
    assert!(h.state().viewport.contains_rect(h.state().editable));
    assert_eq!(h.state().drawing.strokes().len(), 2);
    assert_eq!(
        h.state().drawing.strokes()[1].points(),
        [pos2(300.0, 100.0), pos2(304.0, 100.0), pos2(304.0, 104.0)]
    );
}
