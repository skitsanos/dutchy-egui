use super::support::*;
use dutchy_egui::{ControlSize, InlineEditEvent};
use egui::{Key, Modifiers, accesskit::Role};
use egui_kittest::kittest::{NodeT, Queryable};

#[test]
fn externally_changed_labels_hidden_rows_and_insertions_keep_the_draft_and_identity() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.get_by_label(NAME).type_text("Local draft");
    h.run_steps(3);
    let id = h.get_by_label(NAME).accesskit_node().id();
    h.state_mut().prefix = true;
    h.state_mut().value = "Remote rename".into();
    h.run_steps(3);
    assert_eq!(h.get_by_label(NAME).accesskit_node().id(), id);
    assert!(h.get_by_label(NAME).is_focused());
    assert_eq!(h.state().edit.draft(), Some("Local draft"));
    h.state_mut().shown = false;
    h.run_steps(3);
    h.state_mut().shown = true;
    h.run_steps(3);
    assert_eq!(h.state().edit.draft(), Some("Local draft"));
    h.get_by_role_and_label(Role::Button, "Cancel Document name")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().value, "Remote rename");
}

#[test]
fn parent_disabled_and_read_only_controls_preserve_drafts_and_do_not_commit() {
    let mut h = harness(Page::default());
    start(&mut h);
    h.state_mut().read_only = true;
    h.run_steps(3);
    assert!(h.get_by_label(NAME).accesskit_node().is_read_only());
    h.get_by_label(NAME).type_text("No");
    h.run_steps(3);
    key(&mut h, Key::Enter, Modifiers::NONE);
    assert!(h.state().edit.is_editing());
    assert_eq!(h.state().edit.draft(), Some("Design notes.md"));
    assert!(
        h.get_by_role_and_label(Role::Button, "Save Document name")
            .accesskit_node()
            .is_disabled()
    );
    key(&mut h, Key::Escape, Modifiers::NONE);
    assert!(!h.state().edit.is_editing());
    h.get_by_label("Rename Document name").click();
    h.run_steps(3);
    assert!(!h.state().edit.is_editing());
    h.state_mut().read_only = false;
    start(&mut h);
    for parent in [true, false] {
        h.state_mut().parent_disabled = parent;
        h.state_mut().enabled = parent;
        h.run_steps(3);
        assert!(h.get_by_label(NAME).accesskit_node().is_disabled());
        assert!(
            h.get_by_role_and_label(Role::Button, "Cancel Document name")
                .accesskit_node()
                .is_disabled()
        );
        h.get_by_role_and_label(Role::Button, "Save Document name")
            .click();
        h.run_steps(3);
        assert_eq!(h.state().value, "Design notes.md");
        assert!(h.state().edit.is_editing());
    }
}

#[test]
fn compact_layout_keeps_actions_bounded_and_editor_identity_stable() {
    let mut h = harness(Page::default());
    start(&mut h);
    let id = h.get_by_label(NAME).accesskit_node().id();
    for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
        for width in [0.0, 40.0, 120.0, 220.0, 400.0] {
            h.state_mut().width = width;
            h.state_mut().size = size;
            h.run_steps(3);
            assert!(h.state().rect.is_finite());
            if width == 0.0 {
                continue;
            }
            assert_eq!(h.get_by_label(NAME).accesskit_node().id(), id);
            for name in ["Save Document name", "Cancel Document name"] {
                assert!(
                    h.state()
                        .rect
                        .contains_rect(h.get_by_role_and_label(Role::Button, name).rect()),
                    "{size:?} {width} {name}"
                );
            }
            assert!(h.state().below.top() >= h.state().rect.bottom());
            h.check_accessibility();
        }
    }
}

#[test]
fn empty_names_announce_the_placeholder_they_show() {
    let h = harness(Page {
        value: String::new(),
        ..Default::default()
    });
    let trigger = h.get_by_role_and_label(Role::Button, "Rename Document name");
    assert_eq!(
        trigger.accesskit_node().value().as_deref(),
        Some("Untitled")
    );
    let h = harness(Page::default());
    let trigger = h.get_by_role_and_label(Role::Button, "Rename Document name");
    assert_eq!(
        trigger.accesskit_node().value().as_deref(),
        Some("Design notes.md")
    );
}

#[test]
fn commit_is_a_verbatim_request_that_the_host_can_decline() {
    let mut h = harness(Page {
        apply: false,
        ..Default::default()
    });
    for value in ["", "  folder  ", "Design notes.md"] {
        h.state_mut().edit.begin(value);
        h.run_steps(3);
        key(&mut h, Key::Enter, Modifiers::NONE);
        assert_eq!(
            h.state().events.last(),
            Some(&InlineEditEvent::Commit(value.into()))
        );
        assert_eq!(h.state().value, "Design notes.md");
    }
    assert_eq!(h.state().changes, 0);
    h.get_by_role_and_label(Role::Button, "Rename Document name")
        .focus();
}
