//! Stepper variants, responsive geometry and a host-validated workflow.
#![cfg(feature = "bundled-fonts")]
mod common;
#[path = "../examples/showcase/gallery/mod.rs"]
mod gallery;
use dutchy_egui::{Accent, Mode, Step, Stepper, StepperState, StepperVariant, Text, Theme};
use egui::accesskit::Role;
use egui_kittest::{Harness, SnapshotResults, kittest::Queryable};
fn matrix(ui: &mut egui::Ui) {
    ui.painter().rect_filled(
        ui.max_rect(),
        0,
        Theme::from_context(ui.ctx()).palette.background,
    );
    ui.add(Text::heading("Know the next step."));
    let steps = [
        Step::new(1, "Account")
            .description("Create an account.")
            .navigable(true),
        Step::new(2, "Workspace details")
            .description("Describe the workspace and retain your input in the host."),
        Step::new(3, "Review").description("Review the values before submitting."),
        Step::new(4, "Confirmation").description("The host marks completion explicitly."),
    ];
    for (id, variant) in [
        ("horizontal", StepperVariant::Horizontal),
        ("vertical", StepperVariant::Vertical),
        ("compact", StepperVariant::Compact),
        ("progress", StepperVariant::Progress),
    ] {
        ui.add_space(24.0);
        Stepper::new(id, &StepperState::Current(2), &steps)
            .variant(variant)
            .accessible_name(id)
            .show(ui);
    }
    ui.add_space(24.0);
    Stepper::new("complete", &StepperState::Complete, &steps)
        .accessible_name("Finished sequence")
        .show(ui);
}
#[test]
fn variants_render_across_themes_scales_and_keyboard_focus() {
    let mut results = SnapshotResults::new();
    for (accent, mode, scale, name) in [
        (Accent::Orange, Mode::Light, 1.0, "stepper-light-1x"),
        (Accent::Orange, Mode::Dark, 1.5, "stepper-dark-1_5x"),
        (Accent::Purple, Mode::Light, 2.0, "stepper-purple-2x"),
    ] {
        let theme = Theme::new(accent, mode).with_reduced_motion(true);
        let mut h = Harness::builder()
            .with_size([760.0, 1050.0])
            .with_pixels_per_point(scale)
            .with_render_cursor(false)
            .with_theme(if mode == Mode::Dark {
                egui::Theme::Dark
            } else {
                egui::Theme::Light
            })
            .with_context_setup(move |ctx| common::install(ctx, theme))
            .build_ui(matrix);
        h.get_by_role_and_label(Role::List, "horizontal steps")
            .get_by_label("Account")
            .focus();
        h.run_steps(3);
        h.check_accessibility();
        h.snapshot(name);
        results.extend_harness(&mut h);
    }
    results.unwrap();
}
#[test]
fn adaptive_vertical_and_progress_labels_stay_inside_a_narrow_column() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    let mut h = Harness::builder()
        .with_size([260.0, 2300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(matrix);
    h.run_steps(3);
    h.check_accessibility();
    for name in ["horizontal steps", "vertical steps", "progress steps"] {
        let list = h.get_by_role_and_label(Role::List, name);
        assert!(
            list.rect().left() >= 8.0 && list.rect().right() <= 252.5,
            "{name}: {:?}",
            list.rect()
        );
    }
    h.render()
        .unwrap()
        .save("target/visual-proof/stepper-matrix-narrow.png")
        .unwrap();
}
#[test]
fn navigable_hover_has_visible_feedback_without_a_workflow_request() {
    let mut h = Harness::builder()
        .with_size([760.0, 1050.0])
        .with_render_cursor(false)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(matrix);
    h.run_steps(2);
    let idle = h.render().unwrap();
    h.get_by_role_and_label(Role::List, "horizontal steps")
        .get_by_label("Account")
        .hover();
    h.run_steps(2);
    assert_ne!(idle, h.render().unwrap());
    h.snapshot("stepper-navigation-hover");
}
fn example(size: [f32; 2]) -> Harness<'static, gallery::Showcase> {
    Harness::builder()
        .with_size(size)
        .with_theme(egui::Theme::Light)
        .build_eframe(|cc| {
            common::install(&cc.egui_ctx, Theme::default().with_reduced_motion(true));
            gallery::Showcase {
                page: gallery::Page::Stepper,
                reduced_motion: true,
                ..Default::default()
            }
        })
}
#[test]
fn showcase_renders_at_desktop_and_narrow_widths() {
    std::fs::create_dir_all("target/visual-proof").unwrap();
    for (size, name) in [
        ([1200.0, 2300.0], "stepper-desktop"),
        ([480.0, 3400.0], "stepper-narrow"),
    ] {
        let mut h = example(size);
        h.run_steps(3);
        h.check_accessibility();
        h.render()
            .unwrap()
            .save(format!("target/visual-proof/{name}.png"))
            .unwrap();
    }
}
#[test]
fn host_validation_gates_advancement_and_data_survives_back_revisit_and_completion() {
    let mut h = example([1200.0, 2300.0]);
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.position(), 1);
    assert!(
        h.query_by_label("Enter an email before continuing.")
            .is_some()
    );
    assert!(h.get_by_label("Setup email").is_focused());
    h.get_by_label("Setup email").type_text("ada@example.test");
    h.run();
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.position(), 2);
    assert!(h.get_by_label("Setup workspace name").is_focused());
    h.get_by_label("Setup workspace name")
        .type_text("Design studio");
    h.run();
    h.get_by_label("BACK").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.email, "ada@example.test");
    assert_eq!(h.state().stepping.workspace, "Design studio");
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.position(), 3);
    h.get_by_role_and_label(Role::List, "Workspace setup steps")
        .get_by_label("Account")
        .click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.position(), 1);
    assert!(h.get_by_label("Setup email").is_focused());
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    h.get_by_label("CONTINUE SETUP").click();
    h.run_steps(3);
    complete_and_restart(&mut h);
}
fn complete_and_restart(h: &mut Harness<'static, gallery::Showcase>) {
    h.get_by_label("CREATE WORKSPACE").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.workflow, StepperState::Complete);
    assert_eq!(h.state().stepping.submissions, 1);
    assert!(h.get_by_label("START SETUP AGAIN").is_focused());
    h.get_by_label("START SETUP AGAIN").click();
    h.run_steps(3);
    assert_eq!(h.state().stepping.position(), 1);
    assert_eq!(h.state().stepping.email, "ada@example.test");
    assert_eq!(h.state().stepping.workspace, "Design studio");
}
