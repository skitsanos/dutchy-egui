use super::support::*;
use dutchy_egui::InlineEditEvent;
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::Queryable;

#[test]
fn pointer_start_selects_the_seed_and_enter_requests_one_host_commit() {
    let mut h = harness(Page::default());
    let below = h.state().below;
    start(&mut h);
    assert!(h.get_by_label(NAME).is_focused());
    assert_eq!(h.state().events, [InlineEditEvent::Started]);
    h.get_by_label(NAME).type_text("Canal notebook.md");
    h.run_steps(3);
    assert_eq!(h.state().value, "Design notes.md");
    assert_eq!(h.state().edit.draft(), Some("Canal notebook.md"));
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert_eq!(h.state().value, "Canal notebook.md");
    assert!(!h.state().edit.is_editing());
    assert!(h.get_by_label("Rename Document name").is_focused());
    assert_eq!(h.state().below, below);
    assert_eq!(h.state().events.len(), 2);
    assert!(!h.state().keys.contains(&Key::Enter));
    h.run_steps(5);
    assert_eq!(h.state().events.len(), 2);
    h.check_accessibility();
}

#[test]
fn escape_and_cancel_discard_drafts_and_restore_the_trigger() {
    for button in [false, true] {
        let mut h = harness(Page::default());
        start(&mut h);
        h.get_by_label(NAME).type_text("Discard me");
        h.run_steps(3);
        if button {
            h.get_by_role_and_label(Role::Button, "Cancel Document name")
                .click();
            h.run_steps(3);
        } else {
            key(&mut h, Key::Escape, Modifiers::NONE);
        }
        assert_eq!(h.state().value, "Design notes.md");
        assert!(!h.state().edit.is_editing());
        assert_eq!(h.state().events.last(), Some(&InlineEditEvent::Cancelled));
        assert!(h.get_by_label("Rename Document name").is_focused());
        assert!(!h.state().keys.contains(&Key::Escape));
    }
}

#[test]
fn save_and_cancel_are_keyboard_reachable_and_blur_does_not_finish() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.get_by_label(NAME).type_text("A retained draft");
    h.run_steps(3);
    h.get_by_label("Other field").click();
    h.run_steps(3);
    assert_eq!(h.state().edit.draft(), Some("A retained draft"));
    assert_eq!(h.state().events.len(), 1);
    h.get_by_label(NAME).focus();
    h.run_steps(3);
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(
        h.get_by_role_and_label(Role::Button, "Save Document name")
            .is_focused()
    );
    key(&mut h, Key::Tab, Modifiers::NONE);
    assert!(
        h.get_by_role_and_label(Role::Button, "Cancel Document name")
            .is_focused()
    );
    key(&mut h, Key::Escape, Modifiers::NONE);
    assert!(!h.state().edit.is_editing());
    start(&mut h);
    h.get_by_label(NAME).type_text("Saved by action");
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Save Document name")
        .focus();
    h.run_steps(3);
    key(&mut h, Key::Space, Modifiers::NONE);
    assert_eq!(h.state().value, "Saved by action");
}

fn pointer_click(h: &mut egui_kittest::Harness<'_, Page>, pos: egui::Pos2) {
    h.event(egui::Event::PointerMoved(pos));
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
}

#[test]
fn double_clicking_to_rename_keeps_the_whole_name_selected() {
    // Frames at display rate, so both clicks fall within egui's 0.3 s delay.
    let frame = 1.0 / 60.0;
    let mut h = harness_stepped(Page::default(), frame);
    let label = h.get_by_label("Rename Document name").rect();
    // Past the name's end, and on its first word, where a double-click
    // would otherwise select only that word.
    for pos in [label.center(), label.left_center() + egui::vec2(24.0, 0.0)] {
        let mut h = harness_stepped(Page::default(), frame);
        pointer_click(&mut h, pos);
        h.run_steps(2);
        pointer_click(&mut h, pos);
        h.run_steps(3);
        h.event(egui::Event::Text("X".into()));
        h.run_steps(3);
        assert_eq!(h.state().edit.draft(), Some("X"), "{pos:?}");
        assert_eq!(h.state().events, [InlineEditEvent::Started]);
    }
    // A deliberate click after the double-click delay still places the caret.
    pointer_click(&mut h, label.center());
    h.run_steps(30);
    pointer_click(&mut h, label.center());
    h.run_steps(3);
    h.event(egui::Event::Text("X".into()));
    h.run_steps(3);
    assert_eq!(h.state().edit.draft(), Some("Design notes.mdX"));
}

#[test]
fn focus_returns_after_a_held_save_only_if_nothing_else_took_it() {
    for moved in [false, true] {
        let mut h = harness(Page {
            hold_on_commit: true,
            ..Default::default()
        });
        start(&mut h);
        h.get_by_label(NAME).type_text("Renamed");
        h.run_steps(3);
        key(&mut h, Key::Enter, Modifiers::NONE);
        assert_eq!(h.state().value, "Renamed");
        assert!(
            !h.state().enabled,
            "the host holds the control while saving"
        );
        if moved {
            h.get_by_label("Other field").click();
            h.run_steps(2);
            h.get_by_label("Other field").type_text("elsewhere");
            h.run_steps(2);
        }
        h.state_mut().enabled = true;
        h.run_steps(3);
        assert_eq!(h.get_by_label("Other field").is_focused(), moved);
        assert_eq!(h.get_by_label("Rename Document name").is_focused(), !moved);
        h.run_steps(3);
        assert_eq!(h.get_by_label("Other field").is_focused(), moved);
    }
}

#[test]
fn native_activation_keys_start_and_host_chords_do_not_commit() {
    for activation in [Key::Enter, Key::Space, Key::F2] {
        let mut h = harness(Page::default());
        h.get_by_label("Rename Document name").focus();
        h.run_steps(3);
        key(&mut h, activation, Modifiers::NONE);
        assert!(h.get_by_label(NAME).is_focused(), "{activation:?}");
        assert_eq!(h.state().edit.draft(), Some("Design notes.md"));
        key(&mut h, Key::Enter, Modifiers::COMMAND);
        assert!(h.state().edit.is_editing());
        assert!(h.state().keys.contains(&Key::Enter));
    }
}
