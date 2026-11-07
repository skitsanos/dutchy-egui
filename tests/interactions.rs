//! Pointer, keyboard, state-isolation, and geometry contracts.

mod common;

use dutchy_egui::{Accent, Badge, Button, ButtonVariant, ControlSize, Mode, Theme};
use egui::{Key, Rect};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

#[derive(Default)]
struct Actions {
    clicks: usize,
    dismissals: usize,
    rects: Vec<Rect>,
}

fn actions() -> Harness<'static, Actions> {
    Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_size([640.0, 480.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state| {
                state.rects.clear();
                for size in [ControlSize::Small, ControlSize::Medium, ControlSize::Large] {
                    let response = ui.add(Button::new(format!("{:?}", size)).size(size));
                    state.rects.push(response.rect);
                    if response.clicked() {
                        state.clicks += 1;
                    }
                }
                if ui.add(Button::new("Disabled").enabled(false)).clicked() {
                    state.clicks += 1;
                }
                if ui.add(Button::new("Busy").loading(true)).clicked() {
                    state.clicks += 1;
                }
                ui.add_enabled_ui(false, |ui| {
                    if ui.add(Button::new("Parent disabled")).clicked() {
                        state.clicks += 1;
                    }
                });
                if ui.add(Badge::new("Removable").closable(true)).clicked() {
                    state.dismissals += 1;
                }
            },
            Actions::default(),
        )
}

#[test]
fn pointer_and_keyboard_activate_once() {
    let mut harness = actions();
    harness.get_by_label("Small").click();
    harness.run();
    assert_eq!(harness.state().clicks, 1);
    harness.get_by_label("Medium").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().clicks, 2);
    harness.key_press(Key::Space);
    harness.run();
    assert_eq!(harness.state().clicks, 3);
}

#[test]
fn disabled_and_loading_controls_do_not_activate() {
    let mut harness = actions();
    for label in ["Disabled", "Busy", "Parent disabled"] {
        assert!(harness.get_by_label(label).accesskit_node().is_disabled());
        harness.get_by_label(label).click();
        harness.run();
    }
    assert_eq!(harness.state().clicks, 0);
    assert!(harness.get_by_label("Busy").accesskit_node().is_busy());
}

#[test]
fn control_height_is_independent_of_label_and_state() {
    let mut harness = actions();
    harness.run();
    for (rect, expected) in harness.state().rects.iter().zip([32.0, 48.0, 56.0]) {
        assert_eq!(rect.height(), expected);
    }
}

#[test]
fn badge_label_does_not_dismiss_but_close_supports_keyboard() {
    let mut harness = actions();
    harness.get_by_label("Removable").click();
    harness.run();
    assert_eq!(harness.state().dismissals, 0);
    harness.get_by_label("Dismiss Removable").click();
    harness.run();
    assert_eq!(harness.state().dismissals, 1);
    harness.get_by_label("Dismiss Removable").focus();
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().dismissals, 2);
}

#[test]
fn variants_do_not_leak_style_to_following_controls() {
    let mut harness = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let before = ui.style().clone();
            for variant in ButtonVariant::ALL {
                ui.add(Button::new("Variant").variant(variant));
            }
            assert_eq!(before.as_ref(), ui.style().as_ref());
        });
    harness.run();
}

#[test]
fn wrapped_buttons_keep_natural_width_and_stay_in_the_column() {
    let mut harness = Harness::builder()
        .with_size([320.0, 260.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default()))
        .build_ui(|ui| {
            let bounds = ui.max_rect();
            let mut rows = Vec::new();
            ui.horizontal_wrapped(|ui| {
                for variant in ButtonVariant::ALL {
                    let response = ui.add(
                        Button::new(variant.name())
                            .variant(variant)
                            .size(ControlSize::Small),
                    );
                    assert!(response.rect.right() <= bounds.right() + 1.0);
                    assert!(response.rect.width() > 70.0);
                    rows.push(response.rect.top());
                }
            });
            assert!(rows.iter().any(|y| *y > rows[0]));
        });
    harness.run();
}

#[test]
fn theme_changes_preserve_application_zoom_and_unrelated_state() {
    let mut harness = actions();
    harness.ctx.set_zoom_factor(1.5);
    let state = egui::Id::unique("application-value");
    harness.ctx.data_mut(|data| data.insert_temp(state, 42_u32));
    for accent in [Accent::Orange, Accent::Purple, Accent::Crimson] {
        Theme::new(accent, Mode::Dark)
            .with_reduced_motion(true)
            .apply(&harness.ctx);
        harness.run();
        assert_eq!(harness.ctx.zoom_factor(), 1.5);
        assert_eq!(
            harness.ctx.data(|data| data.get_temp::<u32>(state)),
            Some(42)
        );
        assert_eq!(
            harness.ctx.global_style().visuals.window_corner_radius,
            egui::CornerRadius::ZERO
        );
    }
}
