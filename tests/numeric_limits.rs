//! Large valid typed numbers never become non-finite through decimal rounding.
mod common;
use dutchy_egui::{NumberInput, Theme};
use egui_kittest::{Harness, kittest::Queryable};

#[test]
fn large_finite_numbers_remain_finite_when_rounded() {
    let mut h = Harness::builder()
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui_state(
            |ui, value: &mut f64| {
                ui.add(
                    NumberInput::new("value", value)
                        .decimals(1)
                        .accessible_name("Value"),
                );
            },
            0.0,
        );
    h.run();
    h.get_by_label("Value").focus();
    h.run();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run();
    h.get_by_label("Value").type_text(&f64::MAX.to_string());
    h.run();
    assert_eq!(*h.state(), f64::MAX);
}
