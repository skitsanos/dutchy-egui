//! Slider and NumberInput interaction, accessibility and paint checks.

mod common;

use dutchy_egui::{ControlSize, NumberInput, Slider, Theme, tokens::border, tokens::control};
use egui::accesskit::Role;
use egui::{Event, Key, Modifiers, PointerButton};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn slider_harness(value: f64) -> Harness<'static, f64> {
    let mut h = Harness::builder()
        .with_size([480.0, 120.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut f64| {
                ui.add(
                    Slider::new("volume", value, 0.0..=100.0)
                        .step(10.0)
                        .accessible_name("Volume")
                        .value_label(|value| format!("{value:.0} dB")),
                );
            },
            value,
        );
    h.run();
    h
}

#[test]
fn slider_exposes_slider_semantics_and_steps_with_arrow_keys() {
    let mut h = slider_harness(20.0);
    let node = h.get_by_role_and_label(Role::Slider, "Volume");
    let data = node.accesskit_node();
    assert_eq!(data.numeric_value(), Some(20.0));
    assert_eq!(data.min_numeric_value(), Some(0.0));
    assert_eq!(data.max_numeric_value(), Some(100.0));
    node.focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert_eq!(*h.state(), 30.0);
    h.key_press(Key::ArrowLeft);
    h.run();
    h.key_press(Key::ArrowLeft);
    h.run();
    assert_eq!(*h.state(), 10.0);
    assert!(
        h.get_by_role_and_label(Role::Slider, "Volume").is_focused(),
        "arrow keys stay inside the focused slider"
    );
}

#[test]
fn slider_click_on_the_rail_moves_to_a_stepped_value() {
    let mut h = slider_harness(0.0);
    let rect = h.get_by_role_and_label(Role::Slider, "Volume").rect();
    let pos = egui::pos2(rect.left() + rect.width() * 0.75, rect.center().y);
    h.hover_at(pos);
    h.run();
    for pressed in [true, false] {
        h.input_mut().events.push(Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        h.run();
    }
    let value = *h.state();
    assert!((60.0..=90.0).contains(&value), "{value}");
    assert_eq!(value % 10.0, 0.0, "values snap to the step");
}

#[test]
fn slider_paints_a_four_point_rail_with_primary_fill_and_a_square_primary_handle() {
    let h = slider_harness(50.0);
    let p = Theme::default().palette;
    let rects: Vec<(egui::Rect, egui::Color32)> = h
        .output()
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) => Some((rect.rect, rect.fill)),
            _ => None,
        })
        .collect();
    let rail = rects
        .iter()
        .find(|(rect, fill)| (rect.height() - border::ACCENT).abs() < 0.5 && *fill == p.border)
        .expect("a four-point rail in the border color");
    let fill = rects
        .iter()
        .find(|(rect, fill)| (rect.height() - border::ACCENT).abs() < 0.5 && *fill == p.primary)
        .expect("a primary trailing fill");
    let handle = rects
        .iter()
        .find(|(rect, fill)| {
            (rect.width() - control::INDICATOR).abs() < 0.5
                && (rect.height() - control::INDICATOR).abs() < 0.5
                && *fill == p.primary
        })
        .expect("a 20-point primary handle");
    assert!((fill.0.left() - rail.0.left()).abs() < 0.5);
    assert!((fill.0.right() - handle.0.center().x).abs() < 1.0);
    assert!(
        (handle.0.center().x - rail.0.center().x).abs() < 1.0,
        "half way at 50"
    );
    let label_x = h
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "50 dB" => Some(text.pos.x),
            _ => None,
        })
        .expect("the host-formatted value label");
    assert!(
        label_x > rail.0.right(),
        "the label sits in its column after the rail"
    );
}

#[test]
fn disabled_slider_ignores_keys() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut f64| {
                ui.add(
                    Slider::new("locked", value, 0.0..=10.0)
                        .accessible_name("Locked")
                        .enabled(false),
                );
            },
            4.0,
        );
    h.run();
    let node = h.get_by_role_and_label(Role::Slider, "Locked");
    assert!(node.accesskit_node().is_disabled());
    node.focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert_eq!(*h.state(), 4.0);
}

fn number_harness(value: f64) -> Harness<'static, f64> {
    let mut h = Harness::builder()
        .with_size([480.0, 160.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut f64| {
                ui.add(
                    NumberInput::new("count", value)
                        .range(0.0..=10.0)
                        .step(1.0)
                        .decimals(1)
                        .accessible_name("Count"),
                );
                let _ = ui.button("Elsewhere");
            },
            value,
        );
    h.run();
    h
}

#[test]
fn number_input_steps_with_buttons_and_arrow_keys_within_its_bounds() {
    let mut h = number_harness(5.0);
    h.get_by_role_and_label(Role::Button, "Increase Count")
        .click();
    h.run();
    assert_eq!(*h.state(), 6.0);
    h.get_by_role_and_label(Role::SpinButton, "Count").focus();
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(*h.state(), 7.0);
    h.key_press(Key::ArrowDown);
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(*h.state(), 5.0);
    assert!(
        h.get_by_role_and_label(Role::SpinButton, "Count")
            .is_focused()
    );
    *h.state_mut() = 0.0;
    h.run_steps(2);
    let decrease = h.get_by_role_and_label(Role::Button, "Decrease Count");
    assert!(decrease.accesskit_node().is_disabled(), "at the minimum");
    decrease.click();
    h.run();
    assert_eq!(*h.state(), 0.0);
    h.get_by_role_and_label(Role::SpinButton, "Count").focus();
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(*h.state(), 0.0, "keys clamp to the range");
}

#[test]
fn number_input_parses_typed_text_and_reverts_invalid_text_on_blur() {
    let mut h = number_harness(5.0);
    let editor = h.get_by_role_and_label(Role::SpinButton, "Count");
    assert_eq!(editor.accesskit_node().numeric_value(), Some(5.0));
    assert_eq!(editor.accesskit_node().min_numeric_value(), Some(0.0));
    assert_eq!(editor.accesskit_node().max_numeric_value(), Some(10.0));
    editor.focus();
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Count")
        .type_text("7.5");
    h.run();
    assert_eq!(*h.state(), 7.5);
    h.key_press_modifiers(Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_role_and_label(Role::SpinButton, "Count")
        .type_text("abc");
    h.run();
    assert_eq!(*h.state(), 7.5, "unparsable text keeps the last value");
    h.get_by_role_and_label(Role::Button, "Elsewhere").focus();
    h.run_steps(2);
    assert_eq!(
        h.get_by_role_and_label(Role::SpinButton, "Count")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("7.5"),
        "losing focus reformats the text from the value"
    );
}

#[test]
fn number_input_step_buttons_are_square_and_sit_inside_the_control() {
    let h = number_harness(5.0);
    let editor = h.get_by_role_and_label(Role::SpinButton, "Count").rect();
    let increase = h
        .get_by_role_and_label(Role::Button, "Increase Count")
        .rect();
    let decrease = h
        .get_by_role_and_label(Role::Button, "Decrease Count")
        .rect();
    assert!((increase.width() - increase.height()).abs() < 0.5);
    assert!((decrease.width() - decrease.height()).abs() < 0.5);
    assert!(decrease.left() > editor.left() && increase.left() > decrease.right() - 0.5);
    let slot = ControlSize::Medium.height() - 2.0 * border::STANDARD;
    assert!(
        (editor.height() - slot).abs() < 0.5,
        "the editor fills the slot inside the border"
    );
}
