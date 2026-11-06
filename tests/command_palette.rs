//! Native modal input blocking, filtering, shortcut toggles, and focus return.
mod common;
use dutchy_egui::{CommandPalette, MenuItem, SearchState, Theme};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    search: SearchState,
    actions: Vec<u8>,
    outside: usize,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([800.0, 600.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if ui.button("Open commands").clicked() {
                    state.search.open = true;
                }
                if ui.button("Outside action").clicked() {
                    state.outside += 1;
                }
                let items = [
                    MenuItem::new(1, "Dashboard").group("Pages").shortcut("G D"),
                    MenuItem::new(2, "Create project")
                        .group("Actions")
                        .shortcut("Ctrl+N"),
                    MenuItem::new(3, "Disabled command").enabled(false),
                ];
                if let Some(result) =
                    CommandPalette::new("global-commands", &mut state.search, &items).show(ui.ctx())
                    && let Some(action) = result.action
                {
                    state.actions.push(action);
                }
            },
            State::default(),
        )
}
#[test]
fn shortcut_opens_named_modal_focuses_search_and_enter_returns_action_and_focus() {
    let mut h = harness();
    h.get_by_label("Open commands").focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    assert!(h.state().search.open);
    let dialog = h.get_by_role_and_label(Role::Dialog, "Command palette");
    assert!(dialog.accesskit_node().data().is_modal());
    assert!(dialog.query_by_label("Search commands").is_some());
    assert!(
        h.get_by_label("Search commands")
            .accesskit_node()
            .is_focused_in_tree()
    );
    h.get_by_label("Search commands").type_text("CREATE");
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Dashboard")
            .is_none()
    );
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().actions, [2]);
    assert!(!h.state().search.open);
    assert!(h.get_by_label("Open commands").is_focused());
    h.check_accessibility();
}
#[test]
fn tab_stays_in_modal_and_escape_returns_focus_without_action() {
    let mut h = harness();
    h.get_by_label("Open commands").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run_steps(4);
    for modifiers in [Modifiers::NONE, Modifiers::SHIFT, Modifiers::NONE] {
        h.key_press_modifiers(modifiers, Key::Tab);
        h.run_steps(3);
        assert!(
            h.get_by_label("Search commands")
                .accesskit_node()
                .is_focused_in_tree()
        );
    }
    h.key_press(Key::End);
    h.run_steps(2);
    assert_eq!(
        h.get_by_label("Search commands")
            .accesskit_node()
            .active_descendant()
            .unwrap()
            .id(),
        h.get_by_role_and_label(Role::ListBoxOption, "Create project")
            .accesskit_node()
            .id()
    );
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().search.open && h.state().actions.is_empty());
    assert!(h.get_by_label("Open commands").is_focused());
}
#[test]
fn backdrop_blocks_underlying_actions_and_shortcut_can_toggle_closed() {
    let mut h = harness();
    h.get_by_label("Open commands").click();
    h.run_steps(4);
    h.get_by_label("Outside action").click();
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert_eq!(h.state().outside, 0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(4);
    assert!(h.state().search.open);
    h.key_press_modifiers(Modifiers::COMMAND, Key::K);
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert!(h.state().actions.is_empty());
}
#[test]
fn no_matches_and_disabled_commands_do_not_dispatch() {
    let mut h = harness();
    h.get_by_label("Open commands").click();
    h.run_steps(4);
    h.get_by_label("Search commands").type_text("Disabled");
    h.run_steps(3);
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert!(h.state().search.open && h.state().actions.is_empty());
    h.state_mut().search.query = "nothing-matches".into();
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Status, "No matches")
            .is_some()
    );
    h.key_press(Key::Enter);
    h.run();
    assert!(h.state().actions.is_empty());
}
