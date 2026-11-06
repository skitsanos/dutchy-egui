//! Anchoring, keyboard dismissal/exit, context activation, and host-owned actions.
mod common;
use dutchy_egui::{Button, Dropdown, DropdownAlign, Input, MenuItem, MenuWidth, Theme};
use egui::{Key, Modifiers, Rect, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    open: bool,
    context: bool,
    disabled: bool,
    actions: Vec<u8>,
    outside: String,
    popup: Option<Rect>,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([600.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                let _ = ui.button("Before actions");
                let items = [
                    MenuItem::new(1, "Edit"),
                    MenuItem::new(2, "Disabled").enabled(false),
                    MenuItem::new(3, "Delete"),
                ];
                let style = ui.style().clone();
                let dropdown = Dropdown::new("actions", &mut state.open, &items)
                    .enabled(!state.disabled)
                    .accessible_name("Document actions");
                let result = if state.context {
                    let target = ui.add(Button::new("Context target"));
                    dropdown.show_context(ui, &target)
                } else {
                    dropdown.show(ui, |ui| ui.add(Button::new("Actions")))
                };
                state.popup = result.popup.map(|response| response.rect);
                if let Some(action) = result.action {
                    state.actions.push(action);
                }
                ui.add(Input::new("outside", &mut state.outside).accessible_name("Outside input"));
                assert_eq!(style.as_ref(), ui.style().as_ref());
            },
            State::default(),
        )
}

#[test]
fn arrows_open_at_either_end_and_selection_or_escape_returns_focus() {
    let mut h = harness();
    h.get_by_label("ACTIONS").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    assert!(h.state().open);
    assert!(h.get_by_role_and_label(Role::MenuItem, "Edit").is_focused());
    assert_eq!(
        h.get_by_label("ACTIONS")
            .accesskit_node()
            .data()
            .is_expanded(),
        Some(true)
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Delete")
            .is_focused()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(!h.state().open && h.get_by_label("ACTIONS").is_focused());
    h.key_press(Key::ArrowUp);
    h.run_steps(4);
    assert!(
        h.get_by_role_and_label(Role::MenuItem, "Delete")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().actions, [3]);
    assert!(!h.state().open && h.get_by_label("ACTIONS").is_focused());
    h.check_accessibility();
}

#[test]
fn tab_exits_and_outside_input_click_does_not_lose_focus() {
    let mut h = harness();
    h.get_by_label("ACTIONS").click();
    h.run_steps(4);
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(!h.state().open);
    assert!(h.get_by_label("Outside input").is_focused());
    h.get_by_label("ACTIONS").click();
    h.run_steps(4);
    h.key_press_modifiers(Modifiers::SHIFT, Key::Tab);
    h.run_steps(2);
    assert!(!h.state().open);
    assert!(h.get_by_label("Before actions").is_focused());
    h.get_by_label("ACTIONS").click();
    h.run_steps(4);
    h.get_by_label("Outside input").click();
    h.run_steps(2);
    assert!(!h.state().open);
    assert!(h.get_by_label("Outside input").is_focused());
    assert!(h.state().actions.is_empty());
}

#[test]
fn secondary_click_and_shift_f10_open_context_actions() {
    let mut h = harness();
    h.state_mut().context = true;
    h.run();
    h.get_by_label("CONTEXT TARGET").click_secondary();
    h.run_steps(4);
    assert!(h.state().open);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.get_by_label("CONTEXT TARGET").is_focused());
    h.key_press_modifiers(Modifiers::SHIFT, Key::F10);
    h.run_steps(4);
    assert!(h.state().open);
    h.get_by_role_and_label(Role::MenuItem, "Edit").click();
    h.run_steps(2);
    assert_eq!(h.state().actions, [1]);
    assert!(!h.state().open);
    for modifiers in [Modifiers::NONE, Modifiers::SHIFT] {
        h.key_press_modifiers(Modifiers::SHIFT, Key::F10);
        h.run_steps(4);
        h.key_press_modifiers(modifiers, Key::Tab);
        h.run_steps(3);
        assert!(!h.state().open);
        assert!(h.get_by_label("CONTEXT TARGET").is_focused());
    }
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(h.get_by_label("Outside input").is_focused());
}

#[test]
fn disabled_parent_closes_and_prevents_activation() {
    let mut h = harness();
    h.get_by_label("ACTIONS").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::MenuItem, "Disabled").click();
    h.run();
    assert!(h.state().open && h.state().actions.is_empty());
    h.state_mut().disabled = true;
    h.run_steps(2);
    assert!(!h.state().open);
    assert!(h.get_by_label("ACTIONS").accesskit_node().is_disabled());
    h.get_by_label("ACTIONS").click();
    h.run();
    assert!(!h.state().open);
}

#[test]
fn popup_flips_and_scrolls_within_a_narrow_viewport() {
    let mut h = Harness::builder()
        .with_size([240.0, 180.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, rect: &mut Option<Rect>| {
                ui.add_space(100.0);
                let items: Vec<_> = (0..12)
                    .map(|id| MenuItem::new(id, format!("Action {id}")))
                    .collect();
                let mut open = true;
                let result = Dropdown::new("bounded", &mut open, &items)
                    .width(MenuWidth::Large)
                    .align(DropdownAlign::End)
                    .show(ui, |ui| ui.add(Button::new("More")));
                *rect = result.popup.map(|response| response.rect);
            },
            None,
        );
    h.run_steps(5);
    let rect = h.state().unwrap();
    assert!(
        rect.left() >= -1.0
            && rect.right() <= 241.0
            && rect.top() >= -1.0
            && rect.bottom() <= 181.0,
        "{rect:?}"
    );
}
