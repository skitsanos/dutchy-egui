//! RangeSlider regressions from the DS-128 review.

mod common;

use dutchy_egui::{Field, Mode, RangeSlider, Theme, tokens::border};
use egui::{
    Event, Rect, Shape,
    accesskit::{Action, ActionData, ActionRequest, Role},
};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use std::ops::RangeInclusive;

const NAME: &str = "Price";
const HANDLES: [&str; 2] = ["Price minimum", "Price maximum"];
const HELP: &str = "Limits in dollars.";

struct State {
    values: [f64; 2],
    domain: RangeInclusive<f64>,
    disabled: bool,
    inherited: bool,
    width: f32,
    changes: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            values: [20.0, 80.0],
            domain: 0.0..=100.0,
            disabled: false,
            inherited: false,
            width: 400.0,
            changes: 0,
        }
    }
}

fn harness(state: State, theme: Theme, scale: f32) -> Harness<'static, State> {
    Harness::builder()
        .with_size([480.0, 200.0])
        .with_pixels_per_point(scale)
        .with_context_setup(move |ctx| common::install(ctx, theme.with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut State| {
                if state.inherited {
                    ui.disable();
                }
                let [low, high] = &mut state.values;
                let response = Field::new(NAME).help(HELP).show(ui, |ui| {
                    ui.add(
                        RangeSlider::new("price", low, high, state.domain.clone())
                            .accessible_name(NAME)
                            .width(state.width)
                            .value_label(|low, high| format!("{low:.0}–{high:.0}"))
                            .enabled(!state.disabled),
                    )
                });
                state.changes += usize::from(response.changed());
            },
            state,
        )
}

fn edit(h: &mut Harness<'_, State>, label: &str, action: Action, data: Option<ActionData>) {
    let (target_node, target_tree) = h
        .get_by_role_and_label(Role::Slider, label)
        .accesskit_node()
        .locate();
    h.event(Event::AccessKitActionRequest(ActionRequest {
        target_node,
        target_tree,
        action,
        data,
    }));
    h.run_steps(2);
}

#[test]
fn disabled_handles_reject_assistive_edits_and_drop_edit_actions() {
    for inherited in [false, true] {
        let mut h = harness(State::default(), Theme::default(), 1.0);
        h.get_by_role_and_label(Role::Slider, HANDLES[0]).focus();
        h.run();
        h.state_mut().disabled = !inherited;
        h.state_mut().inherited = inherited;
        // The request can arrive in the same frame the host disables the control.
        edit(
            &mut h,
            HANDLES[0],
            Action::SetValue,
            Some(ActionData::NumericValue(40.0)),
        );
        edit(&mut h, HANDLES[1], Action::Decrement, None);
        assert_eq!(h.state().values, [20.0, 80.0]);
        assert_eq!(h.state().changes, 0);
        for label in HANDLES {
            let handle = h.get_by_role_and_label(Role::Slider, label);
            let node = handle.accesskit_node().data();
            assert!(node.is_disabled());
            for action in [Action::SetValue, Action::Increment, Action::Decrement] {
                assert!(
                    !node.supports_action(action),
                    "{label} advertises {action:?}"
                );
            }
        }
    }
}

#[test]
fn invalid_assistive_values_are_ignored_and_valid_edits_still_work() {
    let mut h = harness(State::default(), Theme::default(), 1.0);
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 40.0] {
        edit(
            &mut h,
            HANDLES[0],
            Action::SetValue,
            Some(ActionData::NumericValue(value)),
        );
        let expected = if value.is_finite() { 40.0 } else { 20.0 };
        assert_eq!(h.state().values, [expected, 80.0]);
    }
    edit(&mut h, HANDLES[1], Action::Decrement, None);
    assert!(h.state().values[1] < 80.0);
    assert_eq!(h.state().values[0], 40.0);
    assert_eq!(h.state().changes, 2);
}

fn handles(h: &Harness<'_, State>) -> [Rect; 2] {
    HANDLES.map(|label| h.get_by_role_and_label(Role::Slider, label).rect())
}

#[test]
fn focus_and_fill_follow_native_handles_on_descending_and_single_value_domains() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let theme = Theme::new(dutchy_egui::Accent::Orange, mode);
        for (domain, descending) in [(100.0..=0.0, true), (50.0..=50.0, false)] {
            let mut h = harness(
                State {
                    domain,
                    ..State::default()
                },
                theme,
                scale,
            );
            h.get_by_role_and_label(Role::Slider, HANDLES[0]).focus();
            h.run_steps(2);
            let [low, high] = handles(&h);
            let shapes = &h.output().shapes;
            let ring = shapes.iter().find_map(|shape| match &shape.shape {
                Shape::Rect(rect)
                    if rect.stroke.color == theme.palette.primary
                        && (rect.stroke.width - border::FOCUS_WIDTH).abs() < 0.1 =>
                {
                    Some(rect.rect)
                }
                _ => None,
            });
            assert!((ring.unwrap().center().x - low.center().x).abs() < 0.5);
            if descending {
                assert!(high.center().x < low.center().x);
                assert!(shapes.iter().any(|shape| match &shape.shape {
                    Shape::Rect(rect) => {
                        rect.fill == theme.palette.primary
                            && (rect.rect.height() - border::ACCENT).abs() < 0.5
                            && (rect.rect.left() - high.center().x).abs() < 0.5
                            && (rect.rect.right() - low.center().x).abs() < 0.5
                    }
                    _ => false,
                }));
            } else {
                assert_eq!(low.center(), high.center());
            }
        }
    }
}

#[test]
fn narrow_labeled_range_keeps_both_handles_on_the_rail_and_retains_the_label() {
    for (mode, scale) in [(Mode::Light, 1.0), (Mode::Dark, 2.0)] {
        let mut h = harness(
            State {
                width: 120.0,
                ..State::default()
            },
            Theme::new(dutchy_egui::Accent::Orange, mode),
            scale,
        );
        h.run();
        let rail = h.get_by_role_and_label(Role::Group, NAME).rect();
        let [low, high] = handles(&h);
        assert!(
            low.center().x < high.center().x && rail.contains_rect(low) && rail.contains_rect(high)
        );
        assert!(rail.width() <= 120.0);
        let label = h.get_by_label("20–80").rect();
        assert!(label.top() >= rail.bottom());
    }
}

#[test]
fn normalization_reports_the_host_mutation_once() {
    let mut h = harness(
        State {
            values: [-10.0, 80.0],
            ..State::default()
        },
        Theme::default(),
        1.0,
    );
    h.run_steps(3);
    assert_eq!(h.state().values, [0.0, 80.0]);
    assert_eq!(h.state().changes, 1);
}

#[test]
fn field_metadata_is_on_the_group_that_contains_both_handles() {
    let mut h = harness(State::default(), Theme::default(), 1.0);
    h.run();
    let group = h.get_by_role_and_label(Role::Group, NAME);
    assert_eq!(group.accesskit_node().data().description(), Some(HELP));
    for label in HANDLES {
        group.get_by_role_and_label(Role::Slider, label);
    }
    assert_eq!(h.query_all_by_role(Role::Slider).count(), 2);
    h.check_accessibility();
}
