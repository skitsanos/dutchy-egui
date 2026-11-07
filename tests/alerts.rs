//! Alert dismissal/action contracts, announcements, and focus restoration.

mod common;
use dutchy_egui::{Alert, AlertVariant, Button, Theme};
use egui::accesskit::{Live, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    dismissed: usize,
    actions: usize,
    remove: bool,
    hidden: bool,
    inserted: bool,
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([640.0, 800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let trigger = ui.add(Button::new("Trigger"));
                if state.inserted {
                    ui.label("Inserted sibling");
                }
                if !state.hidden {
                    let result = Alert::new("retry", "Please reconnect and retry.")
                        .variant(AlertVariant::Error)
                        .title("Connection lost")
                        .action("Retry")
                        .dismissible(true)
                        .return_focus_to(trigger.id)
                        .show(ui);
                    if result.action_clicked {
                        state.actions += 1;
                    }
                    if result.dismissed {
                        state.dismissed += 1;
                        state.hidden = state.remove;
                    }
                }
                let result = Alert::new("disabled", "Unavailable")
                    .title("Disabled")
                    .enabled(false)
                    .dismissible(true)
                    .action("Disabled action")
                    .show(ui);
                if result.dismissed || result.action_clicked {
                    state.actions += 100;
                }
                ui.add_enabled_ui(false, |ui| {
                    let result = Alert::new("parent-disabled", "Unavailable")
                        .title("Parent disabled")
                        .dismissible(true)
                        .action("Parent disabled action")
                        .show(ui);
                    if result.dismissed || result.action_clicked {
                        state.actions += 100;
                    }
                });
            },
            State::default(),
        )
}

#[test]
fn message_action_and_dismiss_are_independent_and_visibility_is_host_owned() {
    let mut h = harness();
    h.get_by_label("Please reconnect and retry.").click();
    h.run();
    assert_eq!((h.state().actions, h.state().dismissed), (0, 0));
    h.get_by_label("Retry").focus();
    h.run();
    h.key_press(egui::Key::Space);
    h.run();
    assert_eq!((h.state().actions, h.state().dismissed), (1, 0));
    h.get_by_label("Dismiss Connection lost").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(h.state().dismissed, 1);
    assert!(h.get_by_label("Trigger").is_focused());
    assert!(h.query_by_label("Dismiss Connection lost").is_some());
    h.state_mut().remove = true;
    h.get_by_label("Dismiss Connection lost").click();
    h.run_steps(2);
    assert!(h.query_by_label("Dismiss Connection lost").is_none());
    assert!(h.get_by_label("Trigger").is_focused());
}

#[test]
fn disabled_alert_controls_cannot_activate_and_ids_survive_inserted_siblings() {
    let mut h = harness();
    for name in [
        "Dismiss Disabled",
        "Disabled action",
        "Dismiss Parent disabled",
        "Parent disabled action",
    ] {
        assert!(h.get_by_label(name).accesskit_node().is_disabled());
        h.get_by_label(name).click();
        h.run();
    }
    assert_eq!(h.state().actions, 0);
    h.get_by_label("Dismiss Connection lost").focus();
    h.run();
    let id = h
        .get_by_label("Dismiss Connection lost")
        .accesskit_node()
        .id();
    h.state_mut().inserted = true;
    h.run_steps(2);
    assert_eq!(
        h.get_by_label("Dismiss Connection lost")
            .accesskit_node()
            .id(),
        id
    );
    assert!(h.get_by_label("Dismiss Connection lost").is_focused());
}

#[test]
fn severity_metadata_preserves_focus_and_static_messages_can_disable_announcements() {
    let mut h = Harness::builder()
        .with_size([640.0, 800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            for variant in AlertVariant::ALL {
                Alert::new(variant.name(), "Status details")
                    .variant(variant)
                    .show(ui);
            }
            Alert::new("history", "Historical message")
                .title("History")
                .announce(false)
                .show(ui);
            let _ = ui.button("Keep focus");
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.get_by_label("Keep focus").focus();
    h.run();
    for variant in AlertVariant::ALL {
        let urgent = matches!(variant, AlertVariant::Warning | AlertVariant::Error);
        let node = h.get_by_role_and_label(
            if urgent { Role::Alert } else { Role::Status },
            variant.name(),
        );
        assert_eq!(
            node.accesskit_node().role(),
            if urgent { Role::Alert } else { Role::Status }
        );
        assert_eq!(
            node.accesskit_node().live(),
            if urgent {
                Live::Assertive
            } else {
                Live::Polite
            }
        );
        assert_eq!(
            node.accesskit_node().description().as_deref(),
            Some("Status details")
        );
    }
    assert_eq!(
        h.get_by_label("Information: History")
            .accesskit_node()
            .live(),
        Live::Off
    );
    assert!(h.get_by_label("Keep focus").is_focused());
}

#[test]
fn tinted_alerts_paint_foreground_titles_and_muted_messages() {
    let mut h = harness();
    h.run_steps(2);
    let p = Theme::default().palette;
    let color = |needle: &str| {
        common::galleys(&h.output().shapes)
            .into_iter()
            .find(|galley| galley.job.text == needle)
            .map(|galley| galley.job.sections[0].format.color)
            .expect("alert text painted")
    };
    assert_eq!(color("Connection lost"), p.foreground);
    assert_eq!(color("Please reconnect and retry."), p.muted_foreground);
}
#[test]
fn status_glyphs_are_circle_backed() {
    let mut h = harness();
    h.run_steps(2);
    let circles = h
        .output()
        .shapes
        .iter()
        .filter(|clipped| {
            matches!(&clipped.shape, egui::Shape::Circle(circle) if circle.stroke.width == 2.0)
        })
        .count();
    assert_eq!(circles, 3, "one circle per alert status glyph");
}

#[test]
fn resting_dismiss_glyphs_use_the_muted_foreground() {
    let mut h = harness();
    h.run_steps(2);
    let target = h.get_by_label("Dismiss Connection lost").rect();
    let colors: Vec<_> = h
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { points, stroke }
                if points.iter().all(|point| target.contains(*point)) =>
            {
                Some(stroke.color)
            }
            _ => None,
        })
        .collect();
    assert_eq!(colors.len(), 2, "two glyph strokes");
    assert!(
        colors
            .iter()
            .all(|color| *color == Theme::default().palette.muted_foreground)
    );
}
