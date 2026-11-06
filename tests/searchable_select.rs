//! Query filtering, active descendants, commits, dismissal, and stable input IDs.
mod common;
use dutchy_egui::{Input, SearchState, SearchableSelect, SelectOption, Theme};
use egui::{Key, accesskit::Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    value: Option<u8>,
    search: SearchState,
    commits: usize,
    changes: usize,
    outside: String,
    disabled: bool,
    inserted: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([500.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.inserted {
                    ui.label("Inserted context");
                }
                let options = [
                    SelectOption::new(1, "React").id_salt(1),
                    SelectOption::new(2, "Reserved").enabled(false).id_salt(2),
                    SelectOption::new(3, "Vue").id_salt(3),
                    SelectOption::new(4, "Svelte").id_salt(4),
                ];
                let result = SearchableSelect::new(
                    "framework",
                    &mut state.value,
                    &mut state.search,
                    &options,
                )
                .accessible_name("Framework")
                .enabled(!state.disabled)
                .show(ui);
                if result.committed {
                    state.commits += 1;
                }
                if result.selection_changed {
                    state.changes += 1;
                }
                ui.add(Input::new("outside", &mut state.outside).accessible_name("Outside"));
            },
            State::default(),
        )
}
#[test]
fn arrow_navigation_retains_input_focus_and_commits_only_on_enter() {
    let mut h = harness();
    h.get_by_label("Framework").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowDown);
    h.run_steps(4);
    assert!(h.state().search.open && h.state().value.is_none());
    assert!(
        h.get_by_label("Framework")
            .accesskit_node()
            .is_focused_in_tree()
    );
    let first = h.get_by_role_and_label(Role::ListBoxOption, "React");
    assert_eq!(
        h.get_by_label("Framework")
            .accesskit_node()
            .active_descendant()
            .unwrap()
            .id(),
        first.accesskit_node().id()
    );
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    let active = h
        .get_by_label("Framework")
        .accesskit_node()
        .active_descendant()
        .unwrap()
        .id();
    assert_eq!(
        active,
        h.get_by_role_and_label(Role::ListBoxOption, "Vue")
            .accesskit_node()
            .id()
    );
    h.key_press(Key::Enter);
    h.run_steps(3);
    assert_eq!(h.state().value, Some(3));
    assert_eq!((h.state().commits, h.state().changes), (1, 1));
    assert_eq!(h.state().search.query, "Vue");
    assert!(!h.state().search.open);
    assert!(
        h.get_by_label("Framework")
            .accesskit_node()
            .is_focused_in_tree()
    );
    h.check_accessibility();
}
#[test]
fn filtering_is_case_insensitive_and_no_matches_does_not_erase_selection() {
    let mut h = harness();
    h.state_mut().value = Some(1);
    h.get_by_label("Framework").focus();
    h.run();
    h.get_by_label("Framework").type_text("vUe");
    h.run_steps(4);
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "Vue")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::ListBoxOption, "React")
            .is_none()
    );
    assert_eq!(h.state().value, Some(1));
    h.state_mut().search.query = "no-such-framework".into();
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Status, "No matches")
            .is_some()
    );
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().value, Some(1));
    assert_eq!(h.state().commits, 0);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert!(
        h.get_by_label("Framework")
            .accesskit_node()
            .is_focused_in_tree()
    );
}
#[test]
fn pointer_choices_and_tab_exit_preserve_native_focus_and_disabled_options() {
    let mut h = harness();
    h.get_by_label("Framework").click();
    h.run_steps(4);
    h.get_by_role_and_label(Role::ListBoxOption, "Reserved")
        .click();
    h.run();
    assert!(h.state().value.is_none());
    h.get_by_role_and_label(Role::ListBoxOption, "Svelte")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().value, Some(4));
    h.state_mut().search.query.clear();
    h.get_by_label("Framework").click();
    h.run_steps(4);
    h.key_press(Key::Tab);
    h.run_steps(3);
    assert!(!h.state().search.open);
    assert!(h.get_by_label("Outside").is_focused());
}
#[test]
fn inserting_siblings_keeps_query_focus_and_disabled_controls_cannot_edit() {
    let mut h = harness();
    h.get_by_label("Framework").focus();
    h.run();
    let id = h.get_by_label("Framework").accesskit_node().id();
    h.state_mut().inserted = true;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Framework").accesskit_node().id(), id);
    assert!(
        h.get_by_label("Framework")
            .accesskit_node()
            .is_focused_in_tree()
    );
    h.state_mut().disabled = true;
    h.run();
    assert!(h.get_by_label("Framework").accesskit_node().is_disabled());
    h.get_by_label("Framework").type_text("Cannot edit");
    h.run();
    assert!(h.state().search.query.is_empty());
    assert!(!h.state().search.open);
}
