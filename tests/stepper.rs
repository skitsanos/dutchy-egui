//! Explicit workflow state, step navigation, accessibility and adaptive geometry.
mod common;
use dutchy_egui::{Input, Step, Stepper, StepperState, StepperVariant, Theme};
use egui::{
    Key, Rect,
    accesskit::{AriaCurrent, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
struct State {
    workflow: StepperState<u8>,
    variant: StepperVariant,
    navigable: bool,
    disabled: bool,
    inserted: bool,
    renamed: bool,
    requests: Vec<u8>,
    current: Option<usize>,
    progress: f64,
    rendered: StepperVariant,
    rects: Vec<Rect>,
    outside: String,
}
impl Default for State {
    fn default() -> Self {
        Self {
            workflow: StepperState::Current(2),
            variant: StepperVariant::Horizontal,
            navigable: false,
            disabled: false,
            inserted: false,
            renamed: false,
            requests: vec![],
            current: None,
            progress: 0.0,
            rendered: StepperVariant::Horizontal,
            rects: vec![],
            outside: String::new(),
        }
    }
}
fn harness(width: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([width, 800.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.inserted {
                    ui.label("Inserted context");
                }
                let steps = [
                    Step::new(
                        1,
                        if state.renamed {
                            "Renamed account"
                        } else {
                            "Account"
                        },
                    )
                    .description("Create your account.")
                    .navigable(state.navigable),
                    Step::new(2, "Details")
                        .description("Add the workspace details.")
                        .navigable(state.navigable),
                    Step::new(3, "Review")
                        .description("Review and confirm your choices.")
                        .navigable(state.navigable),
                    Step::new(4, "Confirmation")
                        .navigable(state.navigable)
                        .enabled(false),
                ];
                let result = Stepper::new("setup", &state.workflow, &steps)
                    .accessible_name("Setup")
                    .variant(state.variant)
                    .enabled(!state.disabled)
                    .show(ui);
                state.current = result.current_step;
                state.progress = result.progress;
                state.rendered = result.rendered_variant;
                state.rects = result.steps.iter().map(|r| r.rect).collect();
                if let Some(request) = result.requested_step {
                    state.requests.push(request);
                }
                ui.add(Input::new("outside", &mut state.outside).accessible_name("Outside"));
            },
            State::default(),
        )
}
#[test]
fn status_is_named_and_current_is_explicit_without_rewriting_host_state() {
    let mut h = harness(760.0);
    assert_eq!(h.state().current, Some(2));
    assert_eq!(
        h.get_by_label("Details")
            .accesskit_node()
            .data()
            .aria_current(),
        Some(AriaCurrent::Step)
    );
    for (name, status) in [
        ("Account", "Completed"),
        ("Details", "Current"),
        ("Review", "Upcoming"),
    ] {
        let node = h.get_by_label(name);
        assert!(
            node.accesskit_node()
                .description()
                .unwrap()
                .contains(status)
        );
        assert_eq!(node.accesskit_node().role(), Role::Label);
        node.click();
        h.run();
    }
    assert!(h.state().requests.is_empty());
    assert_eq!(h.state().workflow, StepperState::Current(2));
    let list = h.get_by_role_and_label(Role::List, "Setup steps");
    assert_eq!(list.accesskit_node().data().size_of_set(), Some(4));
    h.get_by_label("Outside").focus();
    h.run();
    h.state_mut().workflow = StepperState::Current(3);
    h.run_steps(2);
    assert!(h.get_by_label("Outside").is_focused());
    h.check_accessibility();
}
#[test]
fn optional_navigation_uses_native_keyboard_and_only_returns_requests() {
    let mut h = harness(760.0);
    h.state_mut().navigable = true;
    h.run_steps(2);
    h.get_by_label("Account").focus();
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(h.state().requests, [1]);
    assert_eq!(h.state().workflow, StepperState::Current(2));
    h.key_press(Key::Space);
    h.run();
    assert_eq!(h.state().requests, [1, 1]);
    h.get_by_label("Details").click();
    h.run();
    assert_eq!(h.state().requests, [1, 1]);
    h.get_by_label("Review").focus();
    h.run();
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert!(h.get_by_label("Outside").is_focused());
    h.get_by_label("Confirmation").click();
    h.run();
    assert_eq!(h.state().requests, [1, 1]);
    h.state_mut().disabled = true;
    h.run();
    h.get_by_label("Account").click();
    h.run();
    assert_eq!(h.state().requests, [1, 1]);
    h.check_accessibility();
}
#[test]
fn not_started_complete_unknown_and_progress_positions_are_finite_and_read_only() {
    let mut h = harness(760.0);
    h.state_mut().variant = StepperVariant::Progress;
    for (workflow, current, progress) in [
        (StepperState::NotStarted, None, 0.0),
        (StepperState::Current(1), Some(1), 0.0),
        (StepperState::Current(4), Some(4), 100.0),
        (StepperState::Complete, None, 100.0),
        (StepperState::Current(99), None, 0.0),
    ] {
        h.state_mut().workflow = workflow.clone();
        h.run_steps(2);
        assert_eq!(h.state().current, current);
        assert_eq!(h.state().progress, progress);
        assert!(progress.is_finite());
        assert_eq!(h.state().workflow, workflow);
        let indicator = h.get_by_role_and_label(Role::ProgressIndicator, "Setup position");
        assert_eq!(
            indicator.accesskit_node().data().numeric_value(),
            Some(progress)
        );
    }
    assert!(h.query_by_label("Current step unavailable").is_some());
    h.check_accessibility();
}
#[test]
fn focus_and_ids_survive_adaptation_renaming_and_sibling_insertion() {
    let mut h = harness(280.0);
    h.state_mut().navigable = true;
    h.run_steps(2);
    assert_eq!(h.state().rendered, StepperVariant::Vertical);
    h.get_by_label("Account").focus();
    h.run();
    let id = h.get_by_label("Account").accesskit_node().id();
    h.state_mut().inserted = true;
    h.state_mut().renamed = true;
    h.state_mut().variant = StepperVariant::Progress;
    h.run_steps(2);
    assert_eq!(h.get_by_label("Renamed account").accesskit_node().id(), id);
    assert!(h.get_by_label("Renamed account").is_focused());
    assert!(
        h.state()
            .rects
            .iter()
            .all(|rect| rect.left() >= 8.0 && rect.right() <= 272.5),
        "{:?}",
        h.state().rects
    );
    h.check_accessibility();
}
#[test]
fn compact_keyboard_focus_reveals_overflow_without_changing_the_workflow() {
    let mut h = harness(180.0);
    h.state_mut().navigable = true;
    h.state_mut().variant = StepperVariant::Compact;
    h.run_steps(3);
    let before = h.get_by_label("Review").rect();
    h.get_by_label("Review").focus();
    h.run_steps(3);
    assert!(h.get_by_label("Review").is_focused());
    assert!(h.get_by_label("Review").rect().right() < before.right());
    assert!(h.get_by_label("Review").rect().right() <= 172.5);
    assert_eq!(h.state().workflow, StepperState::Current(2));
    assert!(h.state().requests.is_empty());
}
#[test]
fn horizontal_markers_align_when_labels_wrap() {
    let mut h = harness(420.0);
    h.run_steps(2);
    assert_eq!(h.state().rendered, StepperVariant::Horizontal);
    let top = h.state().rects[0].top();
    assert!(
        h.state().rects.iter().all(|rect| rect.top() == top),
        "{:?}",
        h.state().rects
    );
}
#[test]
fn empty_and_single_step_states_are_safe() {
    let mut h = Harness::builder()
        .with_size([400.0, 400.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let steps: [Step<'_, u8>; 0] = [];
            let result = Stepper::new("empty", &StepperState::Complete, &steps).show(ui);
            assert!(result.steps.is_empty());
            assert_eq!(result.progress, 0.0);
            let steps = [Step::new(1, "Only step")];
            let result = Stepper::new("single", &StepperState::Current(1), &steps).show(ui);
            assert_eq!(result.current_step, Some(1));
            assert_eq!(result.progress, 0.0);
            let result = Stepper::new("finished", &StepperState::Complete, &steps).show(ui);
            assert_eq!(result.progress, 100.0);
        });
    h.run_steps(2);
    h.check_accessibility();
}

#[test]
fn settled_geometry_does_not_drift_on_repaints_or_focus_changes() {
    let mut h = Harness::builder()
        .with_size([760.0, 1100.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, rects: &mut Vec<Rect>| {
                rects.clear();
                let items = [
                    Step::new(1, "Account").navigable(true),
                    Step::new(2, "Workspace details"),
                    Step::new(3, "Review"),
                    Step::new(4, "Confirmation"),
                ];
                for variant in [
                    StepperVariant::Horizontal,
                    StepperVariant::Vertical,
                    StepperVariant::Compact,
                    StepperVariant::Progress,
                ] {
                    let result =
                        Stepper::new(format!("{variant:?}"), &StepperState::Current(2), &items)
                            .accessible_name(format!("{variant:?}"))
                            .variant(variant)
                            .show(ui);
                    rects.push(result.response.rect);
                    ui.add_space(24.0);
                }
            },
            Vec::new(),
        );
    h.run_steps(3);
    let before = h.state().clone();
    h.run_steps(3);
    assert_eq!(*h.state(), before);
    h.get_by_role_and_label(Role::List, "Horizontal steps")
        .get_by_label("Account")
        .focus();
    h.run_steps(3);
    assert_eq!(*h.state(), before);
}

#[test]
fn fractional_widths_keep_horizontal_sequences_on_one_row() {
    for width in [409.1, 752.7, 973.0, 1200.3] {
        let mut h = Harness::builder()
            .with_size([width, 500.0])
            .with_context_setup(|ctx| common::install(ctx, Theme::default()))
            .build_ui_state(
                |ui, rects: &mut Vec<Rect>| {
                    let items = [
                        Step::new(1, "Account"),
                        Step::new(2, "Workspace"),
                        Step::new(3, "Review"),
                    ];
                    *rects = Stepper::new("fractional", &StepperState::Current(1), &items)
                        .show(ui)
                        .steps
                        .into_iter()
                        .map(|response| response.rect)
                        .collect();
                },
                Vec::new(),
            );
        h.run_steps(3);
        let top = h.state()[0].top();
        assert!(
            h.state().iter().all(|rect| rect.top() == top),
            "width {width}: {:?}",
            h.state()
        );
        assert!(h.state().iter().all(|rect| rect.right() <= width - 7.5));
    }
}
