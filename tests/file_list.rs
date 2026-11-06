//! Keyed file actions, transfer semantics and focus after host removal.
mod common;
use dutchy_egui::{FileItem, FileList, FileStatus, FileUpload, Theme};
use egui::accesskit::Role;
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
#[derive(Default)]
struct State {
    items: Vec<(u8, String, FileStatus)>,
    events: Vec<(&'static str, u8)>,
    remove: bool,
    disabled: bool,
    parent_disabled: bool,
}
fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([520.0, 1000.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                let picker = FileUpload::new("picker").label("Choose files").show(ui);
                let files: Vec<_> = state
                    .items
                    .iter()
                    .map(|(key, name, status)| {
                        FileItem::new(*key, name.as_str())
                            .size(1200)
                            .status(status.clone())
                    })
                    .collect();
                let result = ui
                    .add_enabled_ui(!state.parent_disabled, |ui| {
                        FileList::new("files", &files)
                            .enabled(!state.disabled)
                            .empty_focus(picker.response.id)
                            .show(ui)
                    })
                    .inner;
                for (kind, key) in [
                    ("remove", result.remove_requested),
                    ("cancel", result.cancel_requested),
                    ("retry", result.retry_requested),
                ] {
                    if let Some(key) = key {
                        state.events.push((kind, key));
                        if state.remove && kind != "retry" {
                            state.items.retain(|(id, _, _)| *id != key);
                        }
                    }
                }
            },
            State {
                items: vec![
                    (0, "alpha.pdf".into(), FileStatus::Ready),
                    (1, "beta.png".into(), FileStatus::Uploading(Some(25.0))),
                    (2, "gamma.txt".into(), FileStatus::Uploading(None)),
                    (3, "delta.jpg".into(), FileStatus::Complete),
                    (
                        4,
                        "epsilon.pdf".into(),
                        FileStatus::Failed("Connection interrupted".into()),
                    ),
                ],
                ..Default::default()
            },
        )
}
#[test]
fn remove_cancel_and_retry_are_independent_named_host_requests() {
    let mut h = harness();
    h.run_steps(3);
    h.get_by_label("Remove file alpha.pdf").click();
    h.run_steps(2);
    h.get_by_label("Cancel upload beta.png").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
    h.get_by_label("Retry upload epsilon.pdf").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Space);
    h.run_steps(2);
    assert_eq!(
        h.state().events,
        [("remove", 0), ("cancel", 1), ("retry", 4)]
    );
    assert_eq!(h.state().items.len(), 5);
    let progress = h.get_by_role_and_label(Role::ProgressIndicator, "Uploading beta.png");
    assert_eq!(progress.accesskit_node().data().numeric_value(), Some(25.0));
    assert_eq!(
        h.get_by_role_and_label(Role::ProgressIndicator, "Uploading gamma.txt")
            .accesskit_node()
            .data()
            .numeric_value(),
        None
    );
    h.check_accessibility();
}
#[test]
fn stable_keys_preserve_control_identity_and_focus_across_reorder_and_renaming() {
    let mut h = harness();
    h.run_steps(3);
    let control = h
        .get_by_label("Remove file alpha.pdf")
        .accesskit_node()
        .id();
    h.get_by_label("Remove file alpha.pdf").focus();
    h.run_steps(2);
    h.state_mut().items.reverse();
    h.state_mut().items.last_mut().unwrap().1 = "renamed.pdf".into();
    h.run_steps(3);
    assert_eq!(
        h.get_by_label("Remove file renamed.pdf")
            .accesskit_node()
            .id(),
        control
    );
    assert!(h.get_by_label("Remove file renamed.pdf").is_focused());
}
#[test]
fn actual_host_removal_moves_focus_to_the_next_row_then_back_to_the_picker() {
    let mut h = harness();
    h.state_mut().remove = true;
    h.state_mut().items.truncate(2);
    h.run_steps(3);
    h.get_by_label("Remove file alpha.pdf").focus();
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(4);
    assert!(h.get_by_label("Cancel upload beta.png").is_focused());
    h.key_press(egui::Key::Space);
    h.run_steps(4);
    assert!(h.state().items.is_empty());
    assert!(h.get_by_label("Choose files").is_focused());
}
#[test]
fn disabled_lists_and_disabled_parents_keep_transfer_details_without_actions() {
    for parent in [false, true] {
        let mut h = harness();
        h.state_mut().disabled = !parent;
        h.state_mut().parent_disabled = parent;
        h.run_steps(3);
        for label in [
            "Remove file alpha.pdf",
            "Cancel upload beta.png",
            "Retry upload epsilon.pdf",
        ] {
            assert!(h.get_by_label(label).accesskit_node().is_disabled());
            h.get_by_label(label).click();
            h.run_steps(2);
        }
        assert!(h.state().events.is_empty());
        assert_eq!(
            h.get_by_role_and_label(Role::ProgressIndicator, "Uploading beta.png")
                .accesskit_node()
                .data()
                .numeric_value(),
            Some(25.0)
        );
    }
}
