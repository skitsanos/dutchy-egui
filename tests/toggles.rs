//! Switch, Kbd and Segmented interaction, accessibility and paint checks.

mod common;

use dutchy_egui::{Switch, Theme, tokens::control};
use egui::accesskit::{Role, Toggled};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn switch_harness(on: bool) -> Harness<'static, bool> {
    let mut h = Harness::builder()
        .with_size([320.0, 120.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, on: &mut bool| {
                ui.add(Switch::new("dark", on, "Dark mode"));
            },
            on,
        );
    h.run();
    h
}

#[test]
fn switch_toggles_by_click_and_keyboard_and_reports_the_switch_role() {
    let mut h = switch_harness(false);
    let node = h.get_by_role_and_label(Role::Switch, "Dark mode");
    assert_eq!(node.accesskit_node().toggled(), Some(Toggled::False));
    node.click();
    h.run();
    assert!(*h.state());
    assert_eq!(
        h.get_by_role_and_label(Role::Switch, "Dark mode")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
    h.get_by_role_and_label(Role::Switch, "Dark mode").focus();
    h.run();
    h.key_press(egui::Key::Space);
    h.run();
    assert!(!*h.state(), "Space toggles the focused switch");
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(*h.state(), "Enter toggles the focused switch");
}

#[test]
fn switch_paints_a_primary_track_with_an_on_primary_thumb_when_on() {
    let h = switch_harness(true);
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
    let track = rects
        .iter()
        .find(|(rect, _)| {
            (rect.width() - control::SWITCH_WIDTH).abs() < 0.5
                && (rect.height() - control::INDICATOR).abs() < 0.5
        })
        .expect("a 36 x 20 track is painted");
    assert_eq!(track.1, p.primary);
    let thumb = rects
        .iter()
        .find(|(rect, _)| (rect.width() - control::SWITCH_THUMB).abs() < 0.5)
        .expect("a square thumb is painted");
    assert_eq!(thumb.1, p.on_primary);
    assert!(
        thumb.0.center().x > track.0.center().x,
        "under reduced motion the thumb rests at the on side immediately"
    );
}

#[test]
fn switch_off_paints_the_thumb_at_the_start_in_muted_foreground() {
    let h = switch_harness(false);
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
    let track = rects
        .iter()
        .find(|(rect, _)| (rect.width() - control::SWITCH_WIDTH).abs() < 0.5)
        .expect("track");
    assert_eq!(track.1, p.background);
    let thumb = rects
        .iter()
        .find(|(rect, _)| (rect.width() - control::SWITCH_THUMB).abs() < 0.5)
        .expect("thumb");
    assert_eq!(thumb.1, p.muted_foreground);
    assert!(thumb.0.center().x < track.0.center().x);
}

#[test]
fn disabled_switch_ignores_activation() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, on: &mut bool| {
                ui.add(Switch::new("locked", on, "Locked").enabled(false));
            },
            false,
        );
    h.run();
    let node = h.get_by_role_and_label(Role::Switch, "Locked");
    assert!(node.accesskit_node().is_disabled());
    node.click();
    h.run();
    assert!(!*h.state());
}

#[test]
fn kbd_paints_an_uppercase_mono_chip_on_the_muted_surface() {
    let mut h = Harness::builder()
        .with_size([320.0, 120.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui(|ui| {
            ui.add(dutchy_egui::Kbd::new("Ctrl+K"));
        });
    h.run();
    let label = h.get_by_label("Ctrl+K");
    assert_eq!(label.accesskit_node().role(), Role::Label);
    assert!((label.rect().height() - control::INDICATOR).abs() < 0.5);
    let p = Theme::default().palette;
    let chip = h
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == p.muted => Some(rect.clone()),
            _ => None,
        })
        .expect("the chip fills with the muted surface");
    assert_eq!(chip.stroke.color, p.border);
    let galley = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|galley| galley.job.text == "CTRL+K")
        .expect("the key text is uppercased");
    assert!(galley.job.sections[0].format.font_id.family != egui::FontFamily::Proportional);
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum View {
    Day,
    Week,
    Month,
}

fn segmented_harness(value: Option<View>) -> Harness<'static, Option<View>> {
    let mut h = Harness::builder()
        .with_size([480.0, 120.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut Option<View>| {
                let options = [
                    dutchy_egui::SelectOption::new(View::Day, "Day"),
                    dutchy_egui::SelectOption::new(View::Week, "Week").enabled(false),
                    dutchy_egui::SelectOption::new(View::Month, "Month"),
                ];
                ui.add(
                    dutchy_egui::Segmented::new("view", value, &options)
                        .accessible_name("Calendar view"),
                );
            },
            value,
        );
    h.run();
    h
}

#[test]
fn segmented_is_a_radio_group_that_selects_by_click_and_arrow_keys() {
    let mut h = segmented_harness(Some(View::Day));
    let _group = h.get_by_role_and_label(Role::RadioGroup, "Calendar view");
    assert_eq!(
        h.get_by_role_and_label(Role::RadioButton, "Day")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
    h.get_by_role_and_label(Role::RadioButton, "Month").click();
    h.run();
    assert_eq!(*h.state(), Some(View::Month));
    h.get_by_role_and_label(Role::RadioButton, "Month").focus();
    h.run();
    h.key_press(egui::Key::ArrowRight);
    h.run();
    assert_eq!(
        *h.state(),
        Some(View::Day),
        "Right wraps past the disabled segment"
    );
    h.key_press(egui::Key::ArrowLeft);
    h.run();
    assert_eq!(
        *h.state(),
        Some(View::Month),
        "Left skips the disabled segment"
    );
    assert!(
        h.get_by_role_and_label(Role::RadioButton, "Month")
            .is_focused(),
        "roving focus follows the selection"
    );
    h.key_press(egui::Key::Home);
    h.run();
    assert_eq!(*h.state(), Some(View::Day));
    assert!(
        h.get_by_role_and_label(Role::RadioButton, "Week")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn segmented_paints_equal_segments_with_a_foreground_selection() {
    let h = segmented_harness(Some(View::Month));
    let p = Theme::default().palette;
    let day = h.get_by_role_and_label(Role::RadioButton, "Day").rect();
    let month = h.get_by_role_and_label(Role::RadioButton, "Month").rect();
    assert!(
        (day.width() - month.width()).abs() < 0.5,
        "segments share one width"
    );
    let group = h
        .get_by_role_and_label(Role::RadioGroup, "Calendar view")
        .rect();
    assert!((group.height() - dutchy_egui::ControlSize::Medium.height()).abs() < 0.5);
    let inner =
        dutchy_egui::ControlSize::Medium.height() - 2.0 * dutchy_egui::tokens::border::STANDARD;
    assert!(
        (day.height() - inner).abs() < 0.5,
        "segments sit inside the group border"
    );
    let selected = h
        .output()
        .shapes
        .iter()
        .find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill == p.foreground => Some(rect.rect),
            _ => None,
        })
        .expect("the selected segment fills with the foreground");
    assert!((selected.center().x - month.center().x).abs() < 1.0);
    let galley = common::galleys(&h.output().shapes)
        .into_iter()
        .find(|galley| galley.job.text == "Month")
        .expect("segment label");
    assert_eq!(galley.job.sections[0].format.color, p.background);
}
