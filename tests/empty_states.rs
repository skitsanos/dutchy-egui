//! Empty-state action independence, announcements, identity, and responsiveness.

mod common;
use dutchy_egui::{EmptyState, EmptyStateVariant, Theme};
use egui::accesskit::{Live, Role};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct State {
    primary: u32,
    secondary: u32,
    expanded: bool,
    inserted: bool,
    width: Option<f32>,
}

fn harness() -> Harness<'static, State> {
    Harness::builder()
        .with_size([800.0, 1300.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                if let Some(width) = state.width {
                    ui.set_width(width);
                }
                let _ = ui.button("Keep focus");
                if state.inserted {
                    ui.label("Inserted context");
                }
                let mut empty = EmptyState::new("projects", "No projects yet")
                    .action("Create project")
                    .secondary_action("Browse templates")
                    .announce(true);
                if state.expanded {
                    empty = empty
                        .description("Create a project to bring your work together.")
                        .variant(EmptyStateVariant::Search);
                }
                let result = empty.show(ui);
                if result.action_clicked {
                    state.primary += 1;
                }
                if result.secondary_clicked {
                    state.secondary += 1;
                }
            },
            State::default(),
        )
}

#[test]
fn title_clicks_do_not_activate_actions_and_native_keyboard_actions_remain_independent() {
    let mut h = harness();
    h.get_by_role_and_label(Role::Heading, "No projects yet")
        .click();
    h.run();
    assert_eq!((h.state().primary, h.state().secondary), (0, 0));
    h.get_by_label("Create project").focus();
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!((h.state().primary, h.state().secondary), (1, 0));
    h.get_by_label("Browse templates").focus();
    h.run();
    h.key_press(egui::Key::Space);
    h.run();
    assert_eq!((h.state().primary, h.state().secondary), (1, 1));
    assert!(
        h.query_by_role_and_label(Role::Heading, "No projects yet")
            .is_some()
    );
}

#[test]
fn changing_guidance_preserves_action_focus_and_announcements_are_explicit() {
    let mut h = harness();
    let region = h.get_by_role_and_label(Role::Status, "No projects yet");
    assert_eq!(region.accesskit_node().live(), Live::Polite);
    h.get_by_label("Create project").focus();
    h.run();
    let id = h.get_by_label("Create project").accesskit_node().id();
    h.state_mut().expanded = true;
    h.state_mut().inserted = true;
    h.state_mut().width = Some(260.0);
    h.run_steps(2);
    assert_eq!(h.get_by_label("Create project").accesskit_node().id(), id);
    assert!(h.get_by_label("Create project").is_focused());
    assert_eq!(
        h.get_by_role_and_label(Role::Status, "No projects yet")
            .accesskit_node()
            .description()
            .as_deref(),
        Some("Create a project to bring your work together.")
    );
}

#[test]
fn disabled_actions_and_disabled_parents_cannot_activate() {
    let mut h = Harness::builder()
        .with_size([700.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, state: &mut State| {
                let result = EmptyState::new("disabled", "Unavailable")
                    .action("Disabled primary")
                    .secondary_action("Enabled secondary")
                    .action_enabled(false)
                    .show(ui);
                if result.action_clicked {
                    state.primary += 1;
                }
                if result.secondary_clicked {
                    state.secondary += 1;
                }
                ui.add_enabled_ui(false, |ui| {
                    let result = EmptyState::new("parent", "Parent disabled")
                        .enabled(true)
                        .action("Parent primary")
                        .secondary_action("Parent secondary")
                        .show(ui);
                    if result.action_clicked || result.secondary_clicked {
                        state.primary += 100;
                    }
                });
                let result = EmptyState::new("all", "All disabled")
                    .enabled(false)
                    .action("All primary")
                    .secondary_action("All secondary")
                    .show(ui);
                if result.action_clicked || result.secondary_clicked {
                    state.primary += 100;
                }
            },
            State::default(),
        );
    for label in [
        "Disabled primary",
        "Parent primary",
        "Parent secondary",
        "All primary",
        "All secondary",
    ] {
        assert!(h.get_by_label(label).accesskit_node().is_disabled());
        h.get_by_label(label).click();
        h.run();
    }
    h.get_by_label("Enabled secondary").click();
    h.run();
    assert_eq!((h.state().primary, h.state().secondary), (0, 1));
}

#[test]
fn narrow_guidance_wraps_and_actions_stack_inside_the_column_without_style_leaks() {
    let mut h = Harness::builder()
        .with_size([260.0, 1800.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            let bounds = ui.available_rect_before_wrap();
            let result = EmptyState::new("dashboard", "Set up your dashboard")
                .variant(EmptyStateVariant::Dashboard)
                .description("Choose a widget or a template to begin monitoring your workspace.")
                .action("Add widget")
                .secondary_action("Browse templates")
                .show(ui);
            assert!(result.response.rect.right() <= bounds.right() + 0.1);
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    h.run();
    let primary = h.get_by_label("Add widget").rect();
    let secondary = h.get_by_label("Browse templates").rect();
    assert!(secondary.top() > primary.top());
    assert!(primary.right() < 260.0 && secondary.right() < 260.0);
    let region = h.get_by_role_and_label(Role::Group, "Set up your dashboard");
    assert_eq!(region.accesskit_node().live(), Live::Off);
    assert_eq!(
        h.get_by_role_and_label(Role::Heading, "Set up your dashboard")
            .accesskit_node()
            .level(),
        Some(3)
    );
}

#[test]
fn the_description_sits_twenty_four_points_above_the_actions() {
    let mut h = harness();
    h.state_mut().expanded = true;
    h.run_steps(2);
    let description = h
        .get_by_label("Create a project to bring your work together.")
        .rect();
    let action = h.get_by_label("Create project").rect();
    let gap = action.top() - description.bottom();
    assert!((gap - 24.0).abs() < 0.5, "description to actions gap {gap}");
}
