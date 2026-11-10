use super::support::*;
use dutchy_egui::{Icon, StatusItem, StatusTone, Theme};
use egui::accesskit::{Live, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn items() -> Vec<StatusItem<'static, u32>> {
    vec![
        StatusItem::labeled(1, "Unread", "42"),
        StatusItem::hint(2, "Ctrl+K", "Commands"),
        StatusItem::text(3, "Synced")
            .icon(Icon::Check)
            .tone(StatusTone::Success)
            .live(true),
        StatusItem::labeled(4, "Upload", "3 of 8").progress(37.5),
        StatusItem::text(5, "Indexing").busy(),
        StatusItem::labeled(6, "Sync", "Up to date")
            .clickable(true)
            .tooltip("Synchronise now"),
        StatusItem::text(7, "Offline").accessible_name("Connection: offline"),
    ]
}

fn wide() -> Harness<'static, Page> {
    let mut h = Harness::builder()
        .with_size([1400.0, 200.0])
        .with_context_setup(|ctx| {
            super::common::install(ctx, Theme::default().with_reduced_motion(true))
        })
        .build_ui_state(
            draw,
            Page {
                width: 1380.0,
                items: items(),
                ..Default::default()
            },
        );
    h.run_steps(3);
    assert!(h.state().hidden.is_empty());
    h
}

#[test]
fn the_bar_is_a_named_group_of_labelled_segments() {
    let h = wide();
    h.get_by_role_and_label(Role::Group, "Status");
    // AccessKit reads a Label's text from its value.
    let unread = h.get_by_role_and_label(Role::Label, "Unread: 42");
    assert_eq!(unread.accesskit_node().data().label(), Some("Unread: 42"));
    let hint = h.get_by_role_and_label(Role::Label, "Commands");
    assert_eq!(
        hint.accesskit_node().data().keyboard_shortcut(),
        Some("Ctrl+K")
    );
    let sync = h.get_by_role_and_label(Role::Button, "Sync");
    assert_eq!(sync.value().as_deref(), Some("Up to date"));
    assert_eq!(
        sync.accesskit_node().description().as_deref(),
        Some("Synchronise now")
    );
    h.get_by_role_and_label(Role::Label, "Connection: offline");
    h.check_accessibility();
}
#[test]
fn live_segments_announce_politely_and_progress_reports_its_value() {
    let h = wide();
    let synced = h.get_by_role_and_label(Role::Status, "Synced");
    assert_eq!(synced.accesskit_node().live(), Live::Polite);
    assert_eq!(
        h.get_by_role_and_label(Role::Label, "Unread: 42")
            .accesskit_node()
            .live(),
        Live::Off
    );
    let upload = h.get_by_role_and_label(Role::ProgressIndicator, "Upload");
    assert_eq!(upload.accesskit_node().numeric_value(), Some(37.5));
    assert_eq!(
        upload.accesskit_node().data().max_numeric_value(),
        Some(100.0)
    );
    assert_eq!(upload.value().as_deref(), Some("3 of 8"));
    let indexing = h.get_by_role_and_label(Role::ProgressIndicator, "Indexing");
    assert!(indexing.accesskit_node().is_busy());
    assert_eq!(indexing.accesskit_node().numeric_value(), None);
}
