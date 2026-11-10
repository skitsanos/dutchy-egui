use super::support::*;
use dutchy_egui::ListSelectionMode;
use egui::{Event, Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::Queryable;

#[test]
fn pointer_modifiers_use_the_click_snapshot_for_disjoint_and_additive_ranges() {
    let mut page = Page::new(5);
    page.mode = ListSelectionMode::Multiple;
    let mut h = harness(page);
    for (label, modifiers, expected) in [
        ("Row 00000", Modifiers::NONE, vec![0]),
        ("Row 00002", Modifiers::COMMAND, vec![0, 2]),
        (
            "Row 00003",
            Modifiers {
                command: true,
                shift: true,
                ..Modifiers::NONE
            },
            vec![0, 2, 3],
        ),
        ("Row 00000", Modifiers::SHIFT, vec![0, 1, 2]),
    ] {
        h.get_by_role_and_label(Role::ListBoxOption, label)
            .click_modifiers(modifiers);
        h.run_steps(3);
        assert_eq!(selected(&h), expected);
    }
    h.check_accessibility();
}

#[test]
fn arrows_skip_disabled_rows_and_one_tab_stop_exits_in_both_directions() {
    let mut p = Page::new(8);
    p.items[1] = dutchy_egui::ListItem::new(1, "Row 00001").enabled(false);
    let mut h = harness(p);
    focus(&mut h, 0);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    assert_eq!(h.state().state.focused, Some(2));
    assert_eq!(selected(&h), [2]);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(h.get_by_label("After list").is_focused());
    key(&mut h, Key::Tab, Modifiers::SHIFT);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 00002")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::SHIFT);
    assert!(h.get_by_label("Before list").is_focused());
    h.check_accessibility();
}
#[test]
fn range_selection_uses_a_stable_anchor_and_control_preserves_selection() {
    let mut p = Page::new(30);
    p.mode = ListSelectionMode::Multiple;
    let mut h = harness(p);
    focus(&mut h, 0);
    key(&mut h, Key::Space, Modifiers::NONE);
    key(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    key(&mut h, Key::ArrowDown, Modifiers::SHIFT);
    assert_eq!(selected(&h), [0, 1, 2]);
    key(&mut h, Key::ArrowUp, Modifiers::SHIFT);
    assert_eq!(selected(&h), [0, 1]);
    key(&mut h, Key::End, Modifiers::COMMAND);
    assert_eq!(selected(&h), [0, 1]);
    assert_eq!(h.state().state.focused, Some(29));
    key(&mut h, Key::Space, Modifiers::COMMAND);
    assert_eq!(selected(&h), [0, 1, 29]);
    key(
        &mut h,
        Key::Home,
        Modifiers {
            command: true,
            shift: true,
            ..Modifiers::NONE
        },
    );
    assert_eq!(selected(&h).len(), 30);
    h.check_accessibility();
}
#[test]
fn command_select_all_excludes_disabled_rows_and_space_toggles() {
    let mut p = Page::new(12);
    p.mode = ListSelectionMode::Multiple;
    p.items[3] = dutchy_egui::ListItem::new(3, "Unavailable").enabled(false);
    let mut h = harness(p);
    focus(&mut h, 0);
    key(&mut h, Key::A, Modifiers::COMMAND);
    assert_eq!(selected(&h).len(), 11);
    assert!(!selected(&h).contains(&3));
    key(&mut h, Key::Space, Modifiers::NONE);
    assert!(!selected(&h).contains(&0));
}
#[test]
fn end_home_page_navigation_reveals_virtual_rows_and_activation_is_explicit() {
    let mut h = harness(Page::new(10_000));
    focus(&mut h, 0);
    key(&mut h, Key::End, Modifiers::NONE);
    assert_eq!(h.state().state.focused, Some(9999));
    assert!(h.state().range.contains(&9999));
    assert!(h.state().rendered < 10);
    assert!(
        h.get_by_role_and_label(Role::ListBoxOption, "Row 09999")
            .is_focused()
    );
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(h.state().activated, [9999]);
    key(&mut h, Key::PageUp, Modifiers::NONE);
    assert_eq!(h.state().state.focused, Some(9995));
    key(&mut h, Key::Home, Modifiers::NONE);
    assert_eq!(h.state().state.focused, Some(0));
    h.check_accessibility();
}
#[test]
fn type_ahead_finds_offscreen_prefixes_cycles_and_resets_after_timeout() {
    let mut p = Page::new(200);
    p.items[150] = dutchy_egui::ListItem::new(150, "Åland");
    p.items[151] = dutchy_egui::ListItem::new(151, "Ålesund");
    let mut h = harness(p);
    focus(&mut h, 0);
    h.event(Event::Text("Å".into()));
    h.run_steps(3);
    assert_eq!(h.state().state.focused, Some(150));
    h.event(Event::Text("å".into()));
    h.run_steps(3);
    assert_eq!(h.state().state.focused, Some(151));
    h.event(Event::Text("l".into()));
    h.run_steps(3);
    assert_eq!(h.state().state.focused, Some(151));
    h.run_steps(8);
    h.event(Event::Text("r".into()));
    h.run_steps(3);
    assert_eq!(h.state().state.focused, Some(152));
    h.check_accessibility();
}
#[test]
fn type_ahead_spaces_extend_an_active_prefix_without_toggling_selection() {
    let mut p = Page::new(0);
    p.mode = ListSelectionMode::Multiple;
    p.items = labeled(&["Alpha", "New Amsterdam", "New York", "Zulu"]);
    let mut h = harness(p);
    h.get_by_role_and_label(Role::ListBoxOption, "Alpha")
        .focus();
    h.run_steps(3);
    for c in "new y".chars() {
        type_char(&mut h, c);
    }
    assert_eq!(h.state().state.focused, Some(2));
    assert_eq!(selected(&h), [2]);
    // Once the prefix expires, Space operates the row again.
    h.run_steps(8);
    type_char(&mut h, ' ');
    assert!(selected(&h).is_empty());
    h.check_accessibility();
}
#[test]
fn only_command_is_additive_so_a_macos_control_click_replaces_selection() {
    let mut p = Page::new(4);
    p.mode = ListSelectionMode::Multiple;
    let mut h = harness(p);
    // macOS reports Control alone for its secondary click; Command (or Control
    // elsewhere, which egui reports as `command`) toggles.
    for (label, modifiers, expected) in [
        ("Row 00000", Modifiers::NONE, vec![0]),
        ("Row 00002", Modifiers::CTRL, vec![2]),
        ("Row 00003", Modifiers::COMMAND, vec![2, 3]),
    ] {
        h.get_by_role_and_label(Role::ListBoxOption, label)
            .click_modifiers(modifiers);
        h.run_steps(3);
        assert_eq!(selected(&h), expected);
    }
}
#[test]
fn none_mode_uses_list_semantics_and_never_selects() {
    let mut p = Page::new(3);
    p.mode = ListSelectionMode::None;
    let mut h = harness(p);
    h.get_by_role_and_label(Role::ListItem, "Row 00000").focus();
    h.run_steps(3);
    key(&mut h, Key::ArrowDown, Modifiers::NONE);
    key(&mut h, Key::Space, Modifiers::NONE);
    assert!(selected(&h).is_empty());
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(h.state().activated, [1]);
    h.get_by_role_and_label(Role::List, "Records");
    h.check_accessibility();
}
