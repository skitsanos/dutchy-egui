//! RangeSlider interaction, accessibility and paint checks.

mod common;

use dutchy_egui::{
    RangeSlider, Theme,
    tokens::{border, control, space},
};
use egui::accesskit::Role;
use egui::{Color32, Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

struct Range {
    low: f64,
    high: f64,
    separation: f64,
    enabled: bool,
    prefix: bool,
    covered: bool,
    changes: usize,
}

fn range(low: f64, high: f64) -> Range {
    Range {
        low,
        high,
        separation: 0.0,
        enabled: true,
        prefix: false,
        covered: false,
        changes: 0,
    }
}

fn harness(state: Range) -> Harness<'static, Range> {
    let mut h = Harness::builder()
        .with_size([480.0, 120.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut Range| {
                if state.prefix {
                    ui.label("A newly inserted sibling");
                }
                let response = ui.add(
                    RangeSlider::new("price", &mut state.low, &mut state.high, 0.0..=100.0)
                        .step(10.0)
                        .min_separation(state.separation)
                        .accessible_name("Price")
                        .value_label(|low, high| format!("{low:.0}–{high:.0}"))
                        .enabled(state.enabled),
                );
                state.changes += usize::from(response.changed());
                let _ = ui.button("After");
                if state.covered {
                    egui::Modal::new(egui::Id::unique("cover")).show(ui.ctx(), |ui| {
                        ui.label("Covering modal");
                    });
                }
            },
            state,
        );
    h.run();
    h
}

fn values(h: &Harness<'_, Range>) -> (f64, f64) {
    (h.state().low, h.state().high)
}

fn press_key(h: &mut Harness<'_, Range>, key: Key, times: usize) {
    for _ in 0..times {
        h.key_press(key);
        h.run();
    }
}

fn pointer(h: &mut Harness<'_, Range>, pos: Pos2, pressed: bool) {
    h.input_mut().events.push(Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
    h.run();
}

/// Where a value sits on the rail: handles stay inside it by half their size.
fn at(rail: Rect, value: f64) -> Pos2 {
    let half = control::INDICATOR / 2.0;
    let travel = rail.width() - control::INDICATOR;
    pos2(
        rail.left() + half + travel * (value / 100.0) as f32,
        rail.center().y,
    )
}

fn rail(h: &Harness<'_, Range>) -> Rect {
    h.get_by_role_and_label(Role::Group, "Price").rect()
}

fn filled_rects(h: &Harness<'_, Range>) -> Vec<(Rect, Color32)> {
    h.output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) => Some((rect.rect, rect.fill)),
            _ => None,
        })
        .collect()
}

#[test]
fn each_handle_is_a_slider_bounded_by_the_other() {
    let h = harness(range(20.0, 80.0));
    let low = h.get_by_role_and_label(Role::Slider, "Price minimum");
    let low = low.accesskit_node();
    assert_eq!(low.numeric_value(), Some(20.0));
    assert_eq!(
        (low.min_numeric_value(), low.max_numeric_value()),
        (Some(0.0), Some(80.0))
    );
    let high = h.get_by_role_and_label(Role::Slider, "Price maximum");
    let high = high.accesskit_node();
    assert_eq!(high.numeric_value(), Some(80.0));
    assert_eq!(
        (high.min_numeric_value(), high.max_numeric_value()),
        (Some(20.0), Some(100.0))
    );
    assert_eq!(
        h.query_all_by_role(Role::Slider).count(),
        2,
        "the rail itself is a named group, not a third slider"
    );
}

#[test]
fn arrow_keys_step_the_focused_handle_and_stop_at_the_other() {
    let mut h = harness(range(20.0, 50.0));
    h.get_by_role_and_label(Role::Slider, "Price minimum")
        .focus();
    h.run();
    press_key(&mut h, Key::ArrowRight, 1);
    assert_eq!(values(&h), (30.0, 50.0));
    assert_eq!(h.state().changes, 1);
    press_key(&mut h, Key::ArrowRight, 5);
    assert_eq!(values(&h), (50.0, 50.0), "the handles meet but never cross");
    assert!(
        h.get_by_role_and_label(Role::Slider, "Price minimum")
            .is_focused(),
        "arrow keys stay inside the focused handle"
    );
    press_key(&mut h, Key::Tab, 1);
    assert!(
        h.get_by_role_and_label(Role::Slider, "Price maximum")
            .is_focused(),
        "Tab moves from the low handle to the high one"
    );
    press_key(&mut h, Key::ArrowLeft, 2);
    assert_eq!(values(&h), (50.0, 50.0));
    press_key(&mut h, Key::ArrowRight, 2);
    assert_eq!(values(&h), (50.0, 70.0));
    press_key(&mut h, Key::Tab, 1);
    assert!(h.get_by_role_and_label(Role::Button, "After").is_focused());
}

#[test]
fn min_separation_keeps_the_handles_apart() {
    let mut h = harness(Range {
        separation: 20.0,
        ..range(20.0, 50.0)
    });
    h.get_by_role_and_label(Role::Slider, "Price minimum")
        .focus();
    h.run();
    press_key(&mut h, Key::ArrowRight, 4);
    assert_eq!(values(&h), (30.0, 50.0));
    let low = h.get_by_role_and_label(Role::Slider, "Price minimum");
    assert_eq!(low.accesskit_node().max_numeric_value(), Some(30.0));
}

#[test]
fn host_values_are_ordered_and_kept_inside_the_range() {
    let h = harness(range(140.0, -20.0));
    let (low, high) = values(&h);
    assert!(
        (0.0..=100.0).contains(&low) && low <= high && high <= 100.0,
        "{low} {high}"
    );
}

#[test]
fn a_press_on_the_rail_moves_the_nearer_handle() {
    let mut h = harness(range(40.0, 60.0));
    let rail = rail(&h);
    for (target, expected) in [(10.0, (10.0, 60.0)), (90.0, (10.0, 90.0))] {
        let pos = at(rail, target);
        h.hover_at(pos);
        h.run();
        pointer(&mut h, pos, true);
        pointer(&mut h, pos, false);
        assert_eq!(values(&h), expected);
    }
    assert_eq!(h.state().changes, 2);
}

#[test]
fn a_dragged_handle_stops_at_the_other_and_keeps_the_pointer() {
    let mut h = harness(range(20.0, 60.0));
    let rail = rail(&h);
    let start = at(rail, 20.0);
    h.hover_at(start);
    h.run();
    pointer(&mut h, start, true);
    for target in [50.0, 70.0, 100.0] {
        h.hover_at(at(rail, target));
        h.run();
    }
    assert_eq!(values(&h), (60.0, 60.0), "the low handle waits at the high");
    h.hover_at(at(rail, 30.0));
    h.run();
    assert_eq!(
        values(&h),
        (30.0, 60.0),
        "one gesture keeps the handle it grabbed"
    );
    pointer(&mut h, at(rail, 30.0), false);
    assert_eq!(values(&h), (30.0, 60.0));
}

#[test]
fn the_rail_fills_between_two_square_handles_beside_a_value_label() {
    let h = harness(range(20.0, 80.0));
    let p = Theme::default().palette;
    let rects = filled_rects(&h);
    let thin = |fill: Color32| {
        rects
            .iter()
            .find(|(rect, color)| (rect.height() - border::ACCENT).abs() < 0.5 && *color == fill)
            .map(|(rect, _)| *rect)
    };
    let rail = thin(p.border).expect("a four-point rail in the border color");
    let fill = thin(p.primary).expect("a primary fill between the handles");
    let mut handles: Vec<Rect> = rects
        .iter()
        .filter(|(rect, color)| {
            (rect.width() - control::INDICATOR).abs() < 0.5
                && (rect.height() - control::INDICATOR).abs() < 0.5
                && *color == p.primary
        })
        .map(|(rect, _)| *rect)
        .collect();
    handles.sort_by(|a, b| a.left().total_cmp(&b.left()));
    assert_eq!(handles.len(), 2, "two 20-point primary handles");
    assert!((handles[0].center().x - at(rail, 20.0).x).abs() < 1.0);
    assert!((handles[1].center().x - at(rail, 80.0).x).abs() < 1.0);
    assert!((fill.left() - handles[0].center().x).abs() < 1.0);
    assert!((fill.right() - handles[1].center().x).abs() < 1.0);
    let reserved = control::RANGE_SLIDER_VALUE_WIDTH + space::MD;
    assert!(
        (rail.width() - (h.ctx.content_rect().width() - 2.0 * rail.left() - reserved)).abs() < 1.0,
        "the label column is reserved: {rail:?}"
    );
    let label_x = h
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "20–80" => Some(text.pos.x),
            _ => None,
        })
        .expect("the host-formatted label is painted");
    assert!(
        (label_x - (rail.right() + space::MD)).abs() < 1.0,
        "{label_x} {rail:?}"
    );
}

#[test]
fn keyboard_focus_rings_only_the_focused_handle() {
    let mut h = harness(range(20.0, 80.0));
    h.get_by_role_and_label(Role::Slider, "Price maximum")
        .focus();
    h.run();
    press_key(&mut h, Key::ArrowLeft, 1);
    let p = Theme::default().palette;
    let rail = rail(&h);
    let rings: Vec<Rect> = h
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect)
                if rect.fill == Color32::TRANSPARENT
                    && rect.stroke.color == p.primary
                    && (rect.stroke.width - border::FOCUS_WIDTH).abs() < 0.1 =>
            {
                Some(rect.rect)
            }
            _ => None,
        })
        .collect();
    assert_eq!(rings.len(), 1, "{rings:?}");
    assert!((rings[0].center().x - at(rail, 70.0).x).abs() < 1.0);
    let outset = 2.0 * (border::FOCUS_OFFSET + border::FOCUS_WIDTH);
    assert!((rings[0].width() - (control::INDICATOR + outset)).abs() < 0.5);
}

#[test]
fn a_disabled_range_ignores_keys_and_presses() {
    let mut h = harness(Range {
        enabled: false,
        ..range(20.0, 80.0)
    });
    let low = h.get_by_role_and_label(Role::Slider, "Price minimum");
    assert!(low.accesskit_node().is_disabled());
    low.focus();
    h.run();
    press_key(&mut h, Key::ArrowRight, 2);
    let pos = at(rail(&h), 50.0);
    h.hover_at(pos);
    h.run();
    pointer(&mut h, pos, true);
    pointer(&mut h, pos, false);
    assert_eq!(values(&h), (20.0, 80.0));
    assert_eq!(h.state().changes, 0);
}

fn focus_low(h: &mut Harness<'_, Range>) {
    h.get_by_role_and_label(Role::Slider, "Price minimum")
        .focus();
    h.run();
}

#[test]
fn disabling_a_focused_handle_stops_arrow_keys() {
    let mut h = harness(range(20.0, 80.0));
    focus_low(&mut h);
    h.state_mut().enabled = false;
    h.run_steps(2);
    press_key(&mut h, Key::ArrowRight, 2);
    assert_eq!(values(&h), (20.0, 80.0));
}

#[test]
fn inserted_siblings_keep_the_focused_handle() {
    let mut h = harness(range(20.0, 80.0));
    focus_low(&mut h);
    let focused = h.ctx.memory(|memory| memory.focused());
    h.state_mut().prefix = true;
    h.run_steps(2);
    assert_eq!(h.ctx.memory(|memory| memory.focused()), focused);
    press_key(&mut h, Key::ArrowRight, 1);
    assert_eq!(values(&h), (30.0, 80.0));
}

#[test]
fn a_covering_modal_blocks_arrow_keys() {
    let mut h = harness(range(20.0, 80.0));
    focus_low(&mut h);
    h.state_mut().covered = true;
    h.run_steps(3);
    press_key(&mut h, Key::ArrowRight, 2);
    assert_eq!(values(&h), (20.0, 80.0));
}

#[test]
fn a_field_names_the_rail_and_hosts_may_name_the_handles() {
    let mut h = Harness::builder()
        .with_size([480.0, 160.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, (low, high): &mut (f64, f64)| {
                dutchy_egui::Field::new("Budget").show(ui, |ui| {
                    ui.add(RangeSlider::new("budget", low, high, 0.0..=10.0))
                });
                ui.add(
                    RangeSlider::new("years", low, high, 0.0..=10.0)
                        .accessible_name("Jaren")
                        .handle_names("Vanaf jaar", "Tot en met jaar"),
                );
            },
            (2.0, 6.0),
        );
    h.run();
    h.get_by_role_and_label(Role::Group, "Budget");
    h.get_by_role_and_label(Role::Slider, "Minimum");
    h.get_by_role_and_label(Role::Slider, "Maximum");
    h.get_by_role_and_label(Role::Group, "Jaren");
    h.get_by_role_and_label(Role::Slider, "Vanaf jaar");
    let until = h.get_by_role_and_label(Role::Slider, "Tot en met jaar");
    assert_eq!(until.accesskit_node().numeric_value(), Some(6.0));
}
