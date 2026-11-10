use super::support::*;
use dutchy_egui::{Input, SplitPane, Theme};
use egui::{
    Event, Key, Modifiers,
    accesskit::{Action, ActionData, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[test]
fn batched_mouse_input_does_not_steal_gestures_from_panes_or_overlays() {
    let mut h = Harness::builder()
        .with_size([700.0, 500.0])
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut (Page, bool)| {
                super::support::draw(ui, &mut state.0);
                if state.1 {
                    egui::Area::new(egui::Id::unique("overlay"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(state.0.rects[1].center() - egui::vec2(20.0, 20.0))
                        .show(ui.ctx(), |ui| {
                            ui.add_sized([40.0, 40.0], egui::Button::new("Cover"));
                        });
                }
            },
            (Page::default(), false),
        );
    h.run_steps(3);
    for cover in [false, true] {
        h.state_mut().1 = cover;
        h.run_steps(3);
        let start = h.state().0.rects[usize::from(cover)].center();
        let end = start + egui::vec2(200.0, 0.0);
        h.input_mut().events = vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerMoved(end),
            Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ];
        h.run_steps(1);
        assert_eq!(h.state().0.first, 160.0);
    }
}
#[test]
fn a_complete_mouse_gesture_in_one_frame_resizes_once_on_both_axes() {
    for vertical in [false, true] {
        let mut h = harness(Page {
            vertical,
            first: 100.0,
            minima: [40.0, 40.0],
            discard: true,
            ..Default::default()
        });
        h.run_steps(3);
        let start = h.state().rects[1].center();
        let delta = if vertical {
            egui::vec2(0.0, 30.0)
        } else {
            egui::vec2(30.0, 0.0)
        };
        let end = start + delta;
        h.input_mut().events = vec![
            Event::PointerMoved(start),
            Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerMoved(end),
            Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ];
        h.run_steps(1);
        assert_eq!(h.state().first, 130.0);
    }
}
#[test]
fn motion_and_release_in_one_frame_still_apply_the_final_drag_position() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    let start = h.state().rects[1].center();
    let end = start + egui::vec2(100.0, 0.0);
    h.hover_at(start);
    h.run_steps(2);
    button(&mut h, start, true);
    // Harness::event separates events into frames; native motion and release
    // can arrive together before a click-and-drag threshold is crossed.
    h.input_mut().events = vec![
        Event::PointerMoved(end),
        Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ];
    h.run_steps(1);
    assert_eq!(h.state().first, 260.0);
}

#[test]
fn vertical_drag_ignores_horizontal_motion_and_keyboard_edits_are_idempotent_on_discard() {
    let mut h = harness(Page {
        vertical: true,
        first: 100.0,
        minima: [40.0, 40.0],
        discard: true,
        ..Default::default()
    });
    h.run_steps(3);
    let start = h.state().rects[1].center();
    h.hover_at(start);
    h.run_steps(2);
    button(&mut h, start, true);
    let next = start + egui::vec2(70.0, 50.0);
    h.event(Event::PointerMoved(next));
    h.run_steps(3);
    assert_eq!(h.state().first, 150.0);
    button(&mut h, next, false);
    focus(&mut h);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!(h.state().first, 158.0);
    h.check_accessibility();
}
#[test]
fn locking_or_changing_orientation_during_a_drag_cancels_the_pointer_anchor() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    let start = h.state().rects[1].center();
    h.hover_at(start);
    h.run_steps(2);
    button(&mut h, start, true);
    let next = start + egui::vec2(30.0, 0.0);
    h.event(Event::PointerMoved(next));
    h.run_steps(3);
    assert_eq!(h.state().first, 190.0);
    h.state_mut().read_only = true;
    h.event(Event::PointerMoved(next + egui::vec2(30.0, 0.0)));
    h.run_steps(3);
    assert_eq!(h.state().first, 190.0);
    h.state_mut().read_only = false;
    h.state_mut().vertical = true;
    h.state_mut().first = 100.0;
    h.event(Event::PointerMoved(next + egui::vec2(30.0, 50.0)));
    h.run_steps(3);
    assert_eq!(h.state().first, 100.0);
    button(&mut h, next, false);
    h.check_accessibility();
}
#[test]
fn assistive_values_are_bounded_invalid_values_are_ignored_and_readonly_advertises_no_edits() {
    let mut h = harness(Page::default());
    h.run_steps(3);
    let (target_node, target_tree) = h
        .get_by_role_and_label(Role::Splitter, "Folders")
        .accesskit_node()
        .locate();
    for (value, expected) in [(f64::MAX, 448.0), (f64::NAN, 448.0), (-10.0, 80.0)] {
        h.event(Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                target_node,
                target_tree,
                action: Action::SetValue,
                data: Some(ActionData::NumericValue(value)),
            },
        ));
        h.run_steps(3);
        assert_eq!(h.state().first, expected);
    }
    h.state_mut().read_only = true;
    h.run_steps(3);
    let split = h.get_by_role_and_label(Role::Splitter, "Folders");
    let node = split.accesskit_node().data();
    assert!(node.is_read_only());
    assert!(!node.supports_action(Action::SetValue));
    assert!(!node.supports_action(Action::Increment));
    h.check_accessibility();
}

#[derive(Default)]
struct Nested {
    left: f32,
    top: f32,
    text: String,
}
#[test]
fn nested_dividers_keep_distinct_state_and_child_identity() {
    let mut h = Harness::builder()
        .with_size([700.0, 450.0])
        .with_context_setup(|ctx| super::common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, p: &mut Nested| {
                SplitPane::horizontal("outer", &mut p.left)
                    .height(320.0)
                    .accessible_name("Folders")
                    .show(
                        ui,
                        |ui| {
                            ui.label("Folders");
                        },
                        |ui| {
                            SplitPane::vertical("inner", &mut p.top)
                                .height(ui.available_height())
                                .accessible_name("Preview")
                                .show(
                                    ui,
                                    |ui| {
                                        ui.add(
                                            Input::new("editor", &mut p.text)
                                                .accessible_name("Editor"),
                                        );
                                    },
                                    |ui| {
                                        ui.label("Source");
                                    },
                                );
                        },
                    );
            },
            Nested {
                left: 180.0,
                top: 140.0,
                ..Default::default()
            },
        );
    h.run_steps(3);
    h.get_by_label("Editor").focus();
    h.run_steps(3);
    let editor = h.get_by_label("Editor").accesskit_node().id();
    h.get_by_label("Editor").type_text("Nested draft");
    h.run_steps(3);
    h.state_mut().left = 240.0;
    h.run_steps(3);
    assert_eq!(h.get_by_label("Editor").accesskit_node().id(), editor);
    assert!(h.get_by_label("Editor").is_focused());
    assert_eq!(h.state().text, "Nested draft");
    h.get_by_role_and_label(Role::Splitter, "Preview").focus();
    h.run_steps(3);
    h.key_press(Key::ArrowDown);
    h.run_steps(3);
    assert_eq!(h.state().top, 148.0);
    assert_eq!(h.state().left, 240.0);
    h.check_accessibility();
}
