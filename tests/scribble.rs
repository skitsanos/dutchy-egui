//! Ordered pointer capture, keyboard alternatives, host actions and lifecycle guards.
mod common;
use dutchy_egui::{Scribble, ScribbleAction, ScribbleBrush, ScribbleDrawing, ScribbleError, Theme};
use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, accesskit::Role, pos2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
struct State {
    drawing: ScribbleDrawing,
    disabled: bool,
    inserted: bool,
    parent_disabled: bool,
    read_only: bool,
    clipped: bool,
    modal: bool,
    discard: bool,
    brush: ScribbleBrush,
    rect: Rect,
    actions: Vec<ScribbleAction>,
    errors: Vec<ScribbleError>,
    added: usize,
}
impl Default for State {
    fn default() -> Self {
        Self {
            drawing: ScribbleDrawing::default(),
            disabled: false,
            inserted: false,
            parent_disabled: false,
            read_only: false,
            clipped: false,
            modal: false,
            discard: false,
            brush: ScribbleBrush::default(),
            rect: Rect::NOTHING,
            actions: vec![],
            errors: vec![],
            added: 0,
        }
    }
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([740.0, 650.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                if s.inserted {
                    ui.label("Inserted sibling");
                }
                ui.add_enabled_ui(!s.parent_disabled, |ui| {
                    if s.clipped {
                        ui.add_space(1000.0);
                    }
                    let result = Scribble::new("pad", &mut s.drawing)
                        .label("Drawing")
                        .enabled(!s.disabled)
                        .read_only(s.read_only)
                        .brush(s.brush)
                        .show(ui);
                    s.rect = result.canvas.rect;
                    s.actions.extend(result.actions);
                    s.errors.extend(result.error);
                    s.added += result.strokes_added;
                });
                if s.discard && ui.ctx().current_pass_index() == 0 {
                    ui.ctx()
                        .request_discard("check ordered capture once per frame");
                }
                if s.modal {
                    egui::Modal::new(egui::Id::unique("block")).show(ui.ctx(), |ui| {
                        ui.label("Blocking");
                    });
                }
            },
            State::default(),
        )
}
fn button(point: Pos2, pressed: bool, button: PointerButton) -> Event {
    Event::PointerButton {
        pos: point,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn events(h: &mut Harness<'_, State>, events: impl IntoIterator<Item = Event>) {
    h.input_mut().events.extend(events);
    h.run_steps(1);
}
fn stroke(h: &mut Harness<'_, State>, button_type: PointerButton) {
    let r = h.state().rect;
    let a = r.min + egui::vec2(r.width() / 4.0, r.height() / 4.0);
    let b = r.center();
    let c = r.max - egui::vec2(r.width() / 4.0, r.height() / 4.0);
    events(
        h,
        [
            Event::PointerMoved(a),
            button(a, true, button_type),
            Event::PointerMoved(b),
            button(c, false, button_type),
        ],
    );
}
fn focus(h: &mut Harness<'_, State>) {
    h.get_by_role_and_label(Role::Canvas, "Drawing").focus();
    h.run_steps(2);
}
#[test]
fn ordered_pointer_events_keep_all_samples_and_release_endpoint_once() {
    let mut h = harness();
    h.run_steps(3);
    stroke(&mut h, PointerButton::Secondary);
    assert!(h.state().drawing.is_empty());
    stroke(&mut h, PointerButton::Primary);
    assert_eq!(h.state().added, 1);
    assert_eq!(h.state().drawing.strokes().len(), 1);
    let points = h.state().drawing.strokes()[0].points();
    for (actual, expected) in
        points
            .iter()
            .zip([pos2(150.0, 50.0), pos2(300.0, 100.0), pos2(450.0, 150.0)])
    {
        assert!(actual.distance(expected) < 0.001, "{points:?}");
    }
    assert_eq!(points.len(), 3);
    h.run_steps(3);
    assert_eq!(h.state().drawing.strokes().len(), 1);
    h.check_accessibility();
}
#[test]
fn leaving_commits_without_bridging_reentry_and_clicks_make_dots() {
    let mut h = harness();
    h.run_steps(3);
    let r = h.state().rect;
    events(&mut h, [button(r.center(), true, PointerButton::Primary)]);
    events(
        &mut h,
        [
            Event::PointerMoved(r.center() + egui::vec2(20.0, 10.0)),
            Event::PointerMoved(r.max + egui::vec2(20.0, 0.0)),
        ],
    );
    assert_eq!(h.state().drawing.strokes().len(), 1);
    events(
        &mut h,
        [
            Event::PointerMoved(r.center()),
            button(r.center(), false, PointerButton::Primary),
        ],
    );
    assert_eq!(h.state().drawing.strokes().len(), 1);
    events(
        &mut h,
        [
            button(r.center(), true, PointerButton::Primary),
            button(r.center(), false, PointerButton::Primary),
        ],
    );
    assert_eq!(
        h.state().drawing.strokes()[1].points(),
        [pos2(300.0, 100.0)]
    );
}
#[test]
fn keyboard_moves_draws_dots_cancels_and_returns_history_requests_to_host() {
    let mut h = harness();
    focus(&mut h);
    for key in [Key::Space, Key::ArrowRight, Key::ArrowDown, Key::Enter] {
        h.key_press(key);
        // Native backends render between key down/up and the next command.
        h.run_steps(3);
        assert!(
            h.get_by_role_and_label(Role::Canvas, "Drawing")
                .is_focused()
        );
    }
    assert_eq!(
        h.state().drawing.strokes()[0].points(),
        [pos2(300.0, 100.0), pos2(304.0, 100.0), pos2(304.0, 104.0)]
    );
    h.key_press(Key::Enter);
    h.run_steps(1);
    assert_eq!(h.state().drawing.strokes()[1].points().len(), 1);
    for key in [Key::Space, Key::ArrowRight, Key::Escape] {
        h.key_press(key);
        h.run_steps(1);
    }
    focus(&mut h);
    h.key_press(Key::Delete);
    h.run_steps(1);
    assert_eq!(h.state().drawing.strokes().len(), 2);
    events(
        &mut h,
        [Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::CTRL,
        }],
    );
    assert_eq!(
        h.state().actions,
        [ScribbleAction::Clear, ScribbleAction::Undo]
    );
    assert_eq!(h.state().drawing.strokes().len(), 2);
    h.state_mut().drawing.undo();
    h.run_steps(2);
    assert_eq!(h.state().drawing.strokes().len(), 1);
    h.get_by_label("Redo Drawing").click();
    h.run_steps(2);
    h.get_by_label("Preview Drawing").click();
    h.run_steps(2);
    h.get_by_label("Export SVG Drawing").click();
    h.run_steps(2);
    assert_eq!(
        &h.state().actions[2..],
        [
            ScribbleAction::Redo,
            ScribbleAction::Preview,
            ScribbleAction::ExportSvg
        ]
    );
    assert_eq!(h.state().drawing.strokes().len(), 1);
}
#[test]
fn disabled_read_only_clipped_and_modal_obscured_canvases_ignore_pointer_edits() {
    for mode in 0..5 {
        let mut h = harness();
        h.run_steps(3);
        match mode {
            0 => h.state_mut().disabled = true,
            1 => h.state_mut().parent_disabled = true,
            2 => h.state_mut().read_only = true,
            3 => h.state_mut().clipped = true,
            _ => h.state_mut().modal = true,
        }
        h.run_steps(4);
        stroke(&mut h, PointerButton::Primary);
        assert!(h.state().drawing.is_empty(), "mode {mode}");
        assert!(h.state().actions.is_empty());
    }
}
#[test]
fn read_only_canvas_is_named_and_focusable_but_only_exports_are_actionable() {
    let mut h = harness();
    h.run_steps(3);
    stroke(&mut h, PointerButton::Primary);
    h.state_mut().read_only = true;
    h.run_steps(2);
    focus(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Canvas, "Drawing")
            .accesskit_node()
            .data()
            .is_read_only()
    );
    for key in [Key::Space, Key::Enter, Key::Delete] {
        h.key_press(key);
        h.run_steps(1);
    }
    h.get_by_label("Clear Drawing").click();
    h.run_steps(2);
    assert_eq!(h.state().drawing.strokes().len(), 1);
    assert!(h.state().actions.is_empty());
    h.get_by_label("Export SVG Drawing").click();
    h.run_steps(2);
    assert_eq!(h.state().actions, [ScribbleAction::ExportSvg]);
    h.check_accessibility();
}
#[test]
fn replacing_the_model_or_losing_window_focus_cancels_the_incomplete_gesture() {
    for replacement in [true, false] {
        let mut h = harness();
        h.run_steps(3);
        let r = h.state().rect;
        events(&mut h, [button(r.center(), true, PointerButton::Primary)]);
        if replacement {
            h.state_mut().drawing = ScribbleDrawing::default();
        } else {
            events(&mut h, [Event::WindowFocused(false)]);
        }
        events(&mut h, [button(r.center(), false, PointerButton::Primary)]);
        assert!(h.state().drawing.is_empty());
    }
}
#[test]
fn discarded_layout_passes_do_not_duplicate_strokes_or_action_requests() {
    let mut h = harness();
    h.run_steps(3);
    h.state_mut().discard = true;
    stroke(&mut h, PointerButton::Primary);
    assert_eq!(h.state().added, 1);
    assert_eq!(h.state().drawing.strokes().len(), 1);
    h.get_by_label("Clear Drawing").click();
    h.run_steps(2);
    assert_eq!(h.state().actions, [ScribbleAction::Clear]);
}
#[test]
fn resizing_preserves_logical_data_and_a_gesture_retains_its_starting_brush() {
    let mut h = harness();
    h.run_steps(3);
    let r = h.state().rect;
    events(&mut h, [button(r.center(), true, PointerButton::Primary)]);
    h.state_mut().brush = ScribbleBrush::new().color(egui::Color32::RED);
    events(&mut h, [button(r.center(), false, PointerButton::Primary)]);
    assert_eq!(
        h.state().drawing.strokes()[0].brush(),
        ScribbleBrush::default()
    );
    let before = h.state().drawing.snapshot();
    h.set_size(egui::vec2(360.0, 650.0));
    h.run_steps(3);
    assert_eq!(h.state().drawing.snapshot(), before);
    stroke(&mut h, PointerButton::Primary);
    assert!(h.state().drawing.strokes()[1].points()[0].distance(pos2(150.0, 50.0)) < 0.001);
    assert_eq!(
        h.state().drawing.strokes()[1].brush().ink(),
        egui::Color32::RED
    );
}

#[test]
fn capacity_errors_preserve_completed_ink_and_small_replacements_reset_the_pen() {
    let mut h = harness();
    h.run_steps(3);
    for _ in 0..512 {
        h.state_mut()
            .drawing
            .push(
                dutchy_egui::ScribbleStroke::new(vec![pos2(1.0, 1.0)], ScribbleBrush::default())
                    .unwrap(),
            )
            .unwrap();
    }
    let before = h.state().drawing.snapshot();
    stroke(&mut h, PointerButton::Primary);
    assert_eq!(h.state().drawing.snapshot(), before);
    assert_eq!(h.state().errors, [ScribbleError::Capacity("strokes")]);
    h.state_mut().drawing = ScribbleDrawing::new(1, 1).unwrap();
    h.run_steps(2);
    focus(&mut h);
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().drawing.strokes()[0].points(), [pos2(0.5, 0.5)]);
}

#[test]
fn toolbar_identity_and_focus_survive_sibling_insertion_and_reflow() {
    let mut h = harness();
    h.run_steps(3);
    stroke(&mut h, PointerButton::Primary);
    h.get_by_label("Export SVG Drawing").focus();
    h.run_steps(2);
    let id = h.get_by_label("Export SVG Drawing").accesskit_node().id();
    h.state_mut().inserted = true;
    h.set_size(egui::vec2(340.0, 650.0));
    h.run_steps(3);
    assert_eq!(
        h.get_by_label("Export SVG Drawing").accesskit_node().id(),
        id
    );
    assert!(h.get_by_label("Export SVG Drawing").is_focused());
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().actions, [ScribbleAction::ExportSvg]);
}
