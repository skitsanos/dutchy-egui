//! Nonblocking notification actions, names, identity, overflow and focus.
mod common;
use dutchy_egui::{
    Theme, Toast, ToastDismissal, ToastEvent, ToastId, ToastPosition, ToastQueue, ToastVariant,
    Toasts,
};
use egui::{
    Key, Rect,
    accesskit::{Live, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Debug, PartialEq, Eq)]
struct Action(String);
struct State {
    queue: ToastQueue<Action>,
    events: Vec<ToastEvent<Action>>,
    outside: usize,
    position: ToastPosition,
    limit: usize,
    rects: Vec<Rect>,
    pending: usize,
    return_focus: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            queue: ToastQueue::new(),
            events: Vec::new(),
            outside: 0,
            position: ToastPosition::BottomRight,
            limit: 3,
            rects: Vec::new(),
            pending: 0,
            return_focus: false,
        }
    }
}
fn harness(size: [f32; 2]) -> Harness<'static, State> {
    Harness::builder()
        .with_size(size)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if ui.button("Outside action").clicked() {
                    state.outside += 1;
                }
                let fallback = ui.button("Return target");
                let mut stack = Toasts::new("messages", &mut state.queue)
                    .position(state.position)
                    .max_visible(state.limit);
                if state.return_focus {
                    stack = stack.return_focus_to(fallback.id);
                }
                let result = stack.show(ui.ctx());
                state.events.extend(result.events);
                state.pending = result.pending;
                state.rects = result.visible.iter().map(|row| row.response.rect).collect();
            },
            State::default(),
        )
}
fn push(h: &mut Harness<'_, State>, title: &str, variant: ToastVariant) -> ToastId {
    h.state_mut().queue.push(
        Toast::new("Changes saved.")
            .title(title)
            .variant(variant)
            .timeout(None)
            .action("Undo", Action(title.into())),
    )
}
#[test]
fn arrivals_do_not_steal_focus_or_block_the_rest_of_the_host() {
    let mut h = harness([800.0, 700.0]);
    h.get_by_label("Outside action").focus();
    h.run();
    push(&mut h, "Saved", ToastVariant::Success);
    h.run_steps(4);
    assert!(h.get_by_label("Outside action").is_focused());
    h.get_by_label("Outside action").click();
    h.run_steps(3);
    assert_eq!(h.state().outside, 1);
    assert_eq!(h.state().queue.len(), 1);
    let status = h.get_by_role_and_label(Role::Status, "Success: Saved");
    assert_eq!(status.accesskit_node().data().live(), Some(Live::Polite));
    assert!(status.query_by_label("Dismiss Saved").is_some());
    h.check_accessibility();
}
#[test]
fn close_and_action_keyboard_results_remove_entries_once_without_cloning_payloads() {
    let mut h = harness([800.0, 700.0]);
    let first = push(&mut h, "First", ToastVariant::Info);
    let second = push(&mut h, "Second", ToastVariant::Success);
    h.run_steps(4);
    h.get_by_label("Dismiss First").focus();
    h.run();
    h.key_press(Key::Space);
    h.run_steps(3);
    assert_eq!(
        h.state().events,
        [ToastEvent::Dismissed {
            id: first,
            reason: ToastDismissal::CloseButton
        }]
    );
    assert!(h.get_by_label("Dismiss Second").is_focused());
    h.get_by_role_and_label(Role::Status, "Success: Second")
        .get_by_label("Undo")
        .focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(h.state().queue.is_empty());
    assert_eq!(
        h.state().events[1],
        ToastEvent::Action {
            id: second,
            action: Action("Second".into())
        }
    );
    h.run_steps(3);
    assert_eq!(h.state().events.len(), 2);
}
#[test]
fn pending_entries_use_fifo_slots_and_limit_is_clamped() {
    let mut h = harness([800.0, 1000.0]);
    let first = push(&mut h, "One", ToastVariant::Info);
    for title in ["Two", "Three", "Four"] {
        push(&mut h, title, ToastVariant::Info);
    }
    h.run_steps(4);
    assert_eq!(h.state().rects.len(), 3);
    assert_eq!(h.state().pending, 1);
    assert!(h.query_by_label("Dismiss Four").is_none());
    h.state_mut().queue.dismiss(first);
    h.run_steps(4);
    assert!(h.query_by_label("Dismiss Four").is_some());
    h.state_mut().limit = 0;
    h.run_steps(3);
    assert_eq!(h.state().rects.len(), 1);
    assert_eq!(h.state().pending, 2);
}
#[test]
fn error_is_assertive_warning_is_polite_and_announcement_can_be_disabled() {
    let mut h = harness([800.0, 1000.0]);
    push(&mut h, "Failed", ToastVariant::Error);
    push(&mut h, "Warning", ToastVariant::Warning);
    h.state_mut().queue.push(
        Toast::new("History")
            .title("History")
            .announce(false)
            .timeout(None),
    );
    h.run_steps(4);
    assert_eq!(
        h.get_by_role_and_label(Role::Alert, "Error: Failed")
            .accesskit_node()
            .data()
            .live(),
        Some(Live::Assertive)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::Status, "Warning: Warning")
            .accesskit_node()
            .data()
            .live(),
        Some(Live::Polite)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::Group, "Information: History")
            .accesskit_node()
            .data()
            .live(),
        Some(Live::Off)
    );
    h.check_accessibility();
}
#[test]
fn keyed_focus_survives_replacement_and_corner_changes() {
    let mut h = harness([800.0, 700.0]);
    let id = push(&mut h, "Original", ToastVariant::Info);
    h.run_steps(4);
    h.get_by_label("Dismiss Original").focus();
    h.run();
    let before = h.get_by_label("Dismiss Original").accesskit_node().id();
    h.state_mut().queue.replace(
        id,
        Toast::new("Updated message").title("Renamed").timeout(None),
    );
    h.state_mut().position = ToastPosition::TopLeft;
    h.run_steps(4);
    assert_eq!(
        h.get_by_label("Dismiss Renamed").accesskit_node().id(),
        before
    );
    assert!(h.get_by_label("Dismiss Renamed").is_focused());
    push(&mut h, "Inserted", ToastVariant::Info);
    h.run_steps(3);
    assert!(h.get_by_label("Dismiss Renamed").is_focused());
}
#[test]
fn removing_a_focused_toast_returns_to_the_host_and_clear_honors_explicit_target() {
    let mut h = harness([800.0, 700.0]);
    h.get_by_label("Outside action").focus();
    h.run();
    push(&mut h, "Saved", ToastVariant::Success);
    h.run_steps(4);
    h.get_by_label("Dismiss Saved").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    assert!(h.get_by_label("Outside action").is_focused());
    push(&mut h, "New", ToastVariant::Info);
    h.run_steps(4);
    h.get_by_label("Dismiss New").focus();
    h.run();
    h.state_mut().return_focus = true;
    h.state_mut().queue.clear();
    h.run_steps(4);
    assert!(h.get_by_label("Return target").is_focused());
}
#[test]
fn four_corners_stay_inside_viewport_and_keep_oldest_first_order() {
    for position in [
        ToastPosition::TopLeft,
        ToastPosition::TopRight,
        ToastPosition::BottomLeft,
        ToastPosition::BottomRight,
    ] {
        let mut h = harness([800.0, 700.0]);
        h.state_mut().position = position;
        push(&mut h, "First", ToastVariant::Info);
        push(&mut h, "Second", ToastVariant::Success);
        h.run_steps(5);
        let a = h.state().rects[0];
        let b = h.state().rects[1];
        assert!(a.top() < b.top());
        assert_gutter(position, a, b);
        assert!(
            [a, b].into_iter().all(|rect| rect.left() >= 24.0
                && rect.right() <= 776.5
                && rect.top() >= 24.0
                && rect.bottom() <= 676.5),
            "{position:?}: {a:?} {b:?}"
        );
    }
}
#[test]
fn narrow_overflow_reveals_keyboard_controls_and_disabled_controls_do_not_fire() {
    let mut h = harness([280.0, 380.0]);
    h.state_mut().limit = 4;
    for title in ["First", "Second", "Third", "Fourth"] {
        push(&mut h, title, ToastVariant::Info);
    }
    h.run_steps(5);
    h.get_by_label("Dismiss Fourth").focus();
    h.run_steps(5);
    let rect = h.get_by_label("Dismiss Fourth").rect();
    assert!(rect.top() >= 24.0 && rect.bottom() <= 356.5, "{rect:?}");
    h.state_mut().queue.clear();
    h.state_mut().queue.push(
        Toast::new("Unavailable")
            .title("Disabled")
            .action("Retry", Action("disabled".into()))
            .enabled(false)
            .timeout(None),
    );
    h.run_steps(5);
    h.get_by_label("Dismiss Disabled").click();
    h.run_steps(3);
    assert!(h.state().events.is_empty());
    assert_eq!(h.state().queue.len(), 1);
}

fn assert_gutter(position: ToastPosition, first: Rect, last: Rect) {
    let (x, expected_x) = match position {
        ToastPosition::TopLeft | ToastPosition::BottomLeft => (first.left(), 24.0),
        ToastPosition::TopRight | ToastPosition::BottomRight => (first.right(), 776.0),
        _ => unreachable!("only the four corners are tested"),
    };
    let (y, expected_y) = match position {
        ToastPosition::TopLeft | ToastPosition::TopRight => (first.top(), 24.0),
        ToastPosition::BottomLeft | ToastPosition::BottomRight => (last.bottom(), 676.0),
        _ => unreachable!("only the four corners are tested"),
    };
    assert!(
        (x - expected_x).abs() < 0.5,
        "{position:?}: {first:?}, {last:?}"
    );
    assert!(
        (y - expected_y).abs() < 0.5,
        "{position:?}: {first:?}, {last:?}"
    );
}
