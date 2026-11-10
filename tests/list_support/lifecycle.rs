use super::support::*;
use dutchy_egui::{ListItem, ListSelectionMode};
use egui::{Event, Key, Modifiers, MouseWheelUnit, accesskit::Role, vec2};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn single_selection_metadata_is_consistent_in_the_click_frame() {
    let mut page = Page::new(5);
    page.state.selected.insert(0);
    let mut h = harness(page);
    h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
        .click();
    h.run_steps(1);
    assert_eq!(selected(&h), [2]);
    assert_eq!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00000")
            .accesskit_node()
            .is_selected(),
        Some(false)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .accesskit_node()
            .is_selected(),
        Some(true)
    );
}

#[test]
fn mode_normalization_marks_selection_changes_without_marking_focus_moves() {
    let mut page = Page::new(8);
    page.mode = ListSelectionMode::Multiple;
    page.state.selected.extend([1, 3]);
    let mut h = harness(page);
    assert_eq!(h.state().changes, 0);
    h.state_mut().mode = ListSelectionMode::Single;
    h.run_steps(3);
    assert_eq!(selected(&h), [1]);
    assert_eq!(h.state().changes, 1);
    focus(&mut h, 1);
    key(&mut h, Key::End, Modifiers::COMMAND);
    assert_eq!(h.state().changes, 1);
    h.state_mut().mode = ListSelectionMode::None;
    h.run_steps(3);
    assert!(selected(&h).is_empty());
    assert_eq!(h.state().changes, 2);
}

#[test]
fn response_changes_only_when_an_edit_alters_the_selection() {
    let mut page = Page::new(6);
    page.mode = ListSelectionMode::Multiple;
    let mut h = harness(page);
    focus(&mut h, 0);
    assert_eq!(h.state().changes, 0);
    key(&mut h, Key::Space, Modifiers::NONE);
    assert_eq!((selected(&h), h.state().changes), (vec![0], 1));
    // Replacing a selection with itself is not a change.
    h.get_by_role_and_label(Role::ListBoxOption, "Row 00000")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().changes, 1);
    key(&mut h, Key::A, Modifiers::COMMAND);
    key(&mut h, Key::A, Modifiers::COMMAND);
    assert_eq!((selected(&h).len(), h.state().changes), (6, 2));
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!((selected(&h), h.state().changes), (vec![1], 3));
}

#[test]
fn assistive_click_selects_the_named_row_without_a_pointer() {
    let mut h = harness(Page::new(10));
    h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
        .click_accesskit();
    h.run_steps(3);
    assert_eq!(selected(&h), [2]);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .is_focused()
    );
    h.check_accessibility();
}

#[test]
fn assistive_scroll_into_view_does_not_steal_keyboard_focus() {
    let mut h = harness(Page::new(10000));
    focus(&mut h, 0);
    h.hover_at(h.state().viewport.center());
    h.run_steps(2);
    h.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: vec2(0.0, -2000.0),
        modifiers: Modifiers::NONE,
    });
    h.run_steps(80);
    let visible = h.state().range.start + 1;
    h.get_by_role_and_label(Role::ListBoxOption, &format!("Row {visible:05}"))
        .scroll_to_me();
    h.run_steps(4);
    assert_eq!(h.state().state.focused, Some(0));
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00000")
            .is_focused()
    );
    h.check_accessibility();
}

#[test]
fn removing_the_last_focused_row_keeps_a_tab_stop_in_the_empty_collection() {
    let mut h = harness(Page::new(1));
    focus(&mut h, 0);
    h.state_mut().items.clear();
    h.run_steps(4);
    assert!(h.state().state.focused.is_none());
    assert!(
        h.get_by_role_and_label(Role::ListBox, "Records")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(h.get_by_label("After list").is_focused());
    h.check_accessibility();
}

#[test]
fn keyed_focus_and_selection_survive_reorder_and_removed_keys_are_pruned() {
    let mut h = harness(Page::new(30));
    focus(&mut h, 0);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    let id = h
        .get_by_role_and_label(Role::ListBoxOption, "Row 00001")
        .accesskit_node()
        .id();
    h.state_mut().items.reverse();
    h.run_steps(4);
    assert_eq!(selected(&h), [1]);
    assert_eq!(h.state().state.focused, Some(1));
    assert_eq!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00001")
            .accesskit_node()
            .id(),
        id
    );
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00001")
            .is_focused()
    );
    h.state_mut().items.retain(|item| item.key != 1);
    h.run_steps(4);
    assert!(selected(&h).is_empty());
    assert_eq!(h.state().state.focused, Some(29));
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00029")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn scrolling_away_preserves_focus_without_rendering_it_or_snapping_back() {
    let mut h = harness(Page::new(10000));
    focus(&mut h, 0);
    h.hover_at(h.state().viewport.center());
    h.run_steps(2);
    h.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
        delta: vec2(0.0, -2000.0),
        modifiers: Modifiers::NONE,
    });
    h.run_steps(80);
    assert!(h.state().range.start > 20, "{:?}", h.state().range);
    assert_eq!(h.state().state.focused, Some(0));
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00000")
            .is_focused()
    );
    assert!(h.state().rendered < 10);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert!(h.state().range.contains(&1));
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00001")
            .is_focused()
    );
    h.check_accessibility();
}
#[test]
fn external_focus_key_reveals_without_stealing_focus_from_another_control() {
    let mut h = harness(Page::new(10000));
    h.get_by_label("Before list").focus();
    h.run_steps(3);
    h.state_mut().state.focused = Some(5000);
    h.run_steps(4);
    assert!(h.state().range.contains(&5000));
    assert!(h.get_by_label("Before list").is_focused());
}
#[test]
fn empty_and_disabled_lists_are_safe_and_read_only_keeps_selection() {
    let mut h = harness(Page::new(0));
    h.check_accessibility();
    h.state_mut().items = (0..4)
        .map(|i| ListItem::new(i, format!("Row {i:05}")))
        .collect();
    h.run_steps(4);
    focus(&mut h, 0);
    key(&mut h, Key::Space, Modifiers::NONE);
    h.state_mut().read_only = true;
    h.run_steps(3);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    key(&mut h, Key::Space, Modifiers::NONE);
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(selected(&h), [0]);
    assert!(h.state().activated.is_empty());
    assert_eq!(h.state().state.focused, Some(1));
    h.state_mut().enabled = false;
    h.run_steps(3);
    key(&mut h, Key::End, Modifiers::NONE);
    assert_eq!(h.state().state.focused, Some(1));
    h.check_accessibility();
}
#[test]
fn semantics_include_total_size_positions_selection_and_multiselectability() {
    let mut p = Page::new(10000);
    p.mode = ListSelectionMode::Multiple;
    p.state.selected.insert(2);
    let h = harness(p);
    let list = h.get_by_role_and_label(Role::ListBox, "Records");
    assert_eq!(list.accesskit_node().size_of_set(), Some(10000));
    assert!(list.accesskit_node().is_multiselectable());
    let option = h.get_by_role_and_label(Role::ListBoxOption, "Row 00002");
    assert_eq!(option.accesskit_node().position_in_set(), Some(2));
    assert_eq!(option.accesskit_node().is_selected(), Some(true));
    assert!(h.state().rendered < 10);
    h.check_accessibility();
}
#[test]
fn single_mode_normalizes_multiple_keys_and_duplicate_keys_are_not_interactive() {
    let mut p = Page::new(4);
    p.state.selected.extend([0, 2, 99]);
    p.items.push(ListItem::new(0, "Duplicate"));
    let mut h = harness(p);
    assert_eq!(selected(&h), [0]);
    key(&mut h, Key::End, Modifiers::NONE);
    h.get_by_role_and_label(Role::ListBoxOption, "Duplicate")
        .click();
    h.run_steps(3);
    assert_eq!(selected(&h), [0]);
    h.check_accessibility();
}
