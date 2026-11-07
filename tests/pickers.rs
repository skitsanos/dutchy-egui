//! DatePicker and TimePicker typing, caret stepping, calendar navigation and bounds.

mod common;

use dutchy_egui::calendar::{Date, Time};
use dutchy_egui::{DatePicker, Theme, TimePicker};
use egui::Key;
use egui::accesskit::{Role, Toggled};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

struct DateState {
    value: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
}

fn date_harness(
    value: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
) -> Harness<'static, DateState> {
    let mut h = Harness::builder()
        .with_size([520.0, 480.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, state: &mut DateState| {
                let mut picker = DatePicker::new("issued", &mut state.value)
                    .accessible_name("Date of issue")
                    .today(date(2026, 11, 7));
                if let Some(min) = state.min {
                    picker = picker.min(min);
                }
                if let Some(max) = state.max {
                    picker = picker.max(max);
                }
                ui.add(picker);
                let _ = ui.button("Elsewhere");
            },
            DateState { value, min, max },
        );
    h.run();
    h
}

fn select_all(h: &mut Harness<'_, DateState>) {
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    h.run();
}

#[test]
fn date_picker_commits_typed_iso_dates_and_reverts_invalid_text_on_blur() {
    let mut h = date_harness(None, None, None);
    h.get_by_label("Date of issue").focus();
    h.run();
    h.get_by_label("Date of issue").type_text("2026-11-07");
    h.run();
    assert_eq!(h.state().value, Some(date(2026, 11, 7)));
    select_all(&mut h);
    h.get_by_label("Date of issue").type_text("2026-13-40");
    h.run();
    assert_eq!(
        h.state().value,
        Some(date(2026, 11, 7)),
        "invalid text keeps the value"
    );
    h.get_by_role_and_label(Role::Button, "Elsewhere").focus();
    h.run_steps(2);
    assert_eq!(
        h.get_by_label("Date of issue")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("2026-11-07"),
        "losing focus reformats the text"
    );
    h.get_by_label("Date of issue").focus();
    h.run();
    select_all(&mut h);
    h.key_press(Key::Backspace);
    h.run();
    assert_eq!(h.state().value, None, "clearing the text clears the value");
}

#[test]
fn date_picker_steps_the_segment_under_the_caret() {
    let mut h = date_harness(Some(date(2026, 11, 7)), None, None);
    h.get_by_label("Date of issue").focus();
    h.run();
    h.key_press(Key::End);
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(
        h.state().value,
        Some(date(2026, 11, 8)),
        "the caret ends in the day"
    );
    h.key_press(Key::Home);
    h.run();
    h.key_press(Key::ArrowDown);
    h.run();
    assert_eq!(
        h.state().value,
        Some(date(2025, 11, 8)),
        "Home reaches the year"
    );
    assert!(h.get_by_label("Date of issue").is_focused());
}

#[test]
fn calendar_selects_days_by_pointer_and_keyboard_and_returns_focus() {
    let mut h = date_harness(Some(date(2026, 11, 7)), None, None);
    h.get_by_role_and_label(Role::Button, "Choose Date of issue")
        .click();
    h.run_steps(2);
    h.get_by_label("November 2026");
    assert_eq!(
        h.get_by_role_and_label(Role::Button, "7 November 2026")
            .accesskit_node()
            .toggled(),
        Some(Toggled::True)
    );
    h.get_by_role_and_label(Role::Button, "12 November 2026")
        .click();
    h.run_steps(2);
    assert_eq!(h.state().value, Some(date(2026, 11, 12)));
    assert!(
        h.query_by_label("November 2026").is_none(),
        "selection closes the calendar"
    );

    h.get_by_role_and_label(Role::Button, "Choose Date of issue")
        .click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "12 November 2026")
        .focus();
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert!(
        h.get_by_role_and_label(Role::Button, "13 November 2026")
            .is_focused()
    );
    h.key_press(Key::ArrowDown);
    h.run();
    assert!(
        h.get_by_role_and_label(Role::Button, "20 November 2026")
            .is_focused()
    );
    h.key_press(Key::PageDown);
    h.run_steps(2);
    h.get_by_label("December 2026");
    assert!(
        h.get_by_role_and_label(Role::Button, "20 December 2026")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(h.state().value, Some(date(2026, 12, 20)));
    assert!(h.query_by_label("December 2026").is_none());

    h.get_by_role_and_label(Role::Button, "Choose Date of issue")
        .click();
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.query_by_label("December 2026").is_none(),
        "Escape closes the calendar"
    );
    assert!(
        h.get_by_role_and_label(Role::Button, "Choose Date of issue")
            .is_focused(),
        "focus returns to the calendar button"
    );
}

#[test]
fn calendar_bounds_disable_days_outside_the_range_and_typed_dates_clamp() {
    let mut h = date_harness(
        Some(date(2026, 11, 10)),
        Some(date(2026, 11, 5)),
        Some(date(2026, 11, 20)),
    );
    h.get_by_role_and_label(Role::Button, "Choose Date of issue")
        .click();
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "4 November 2026")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.get_by_role_and_label(Role::Button, "21 November 2026")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !h.get_by_role_and_label(Role::Button, "10 November 2026")
            .accesskit_node()
            .is_disabled()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    h.get_by_label("Date of issue").focus();
    h.run();
    select_all(&mut h);
    h.get_by_label("Date of issue").type_text("2026-11-30");
    h.run();
    assert_eq!(
        h.state().value,
        Some(date(2026, 11, 20)),
        "typed dates clamp to the bounds"
    );
}

#[test]
fn time_picker_parses_text_and_steps_the_caret_segment_with_wrap() {
    let mut h = Harness::builder()
        .with_size([520.0, 200.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut Option<Time>| {
                ui.add(TimePicker::new("starts", value).accessible_name("Start time"));
            },
            None,
        );
    h.run();
    h.get_by_label("Start time").focus();
    h.run();
    h.get_by_label("Start time").type_text("09:30");
    h.run();
    assert_eq!(*h.state(), Time::new(9, 30, 0));
    h.key_press(Key::End);
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(
        *h.state(),
        Time::new(9, 31, 0),
        "the caret ends in the minutes"
    );
    h.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    h.run();
    h.get_by_label("Start time").type_text("23:59");
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(
        *h.state(),
        Time::new(23, 0, 0),
        "minutes wrap without touching the hour"
    );
    h.key_press(Key::Home);
    h.run();
    h.key_press(Key::ArrowUp);
    h.run();
    assert_eq!(*h.state(), Time::new(0, 0, 0), "hours wrap at midnight");
    assert_eq!(
        h.get_by_label("Start time")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("00:00")
    );
}

#[test]
fn time_picker_with_seconds_formats_three_segments() {
    let mut h = Harness::builder()
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut Option<Time>| {
                ui.add(
                    TimePicker::new("precise", value)
                        .seconds(true)
                        .accessible_name("Precise time"),
                );
            },
            Time::new(7, 5, 9),
        );
    h.run();
    assert_eq!(
        h.get_by_label("Precise time")
            .accesskit_node()
            .value()
            .as_deref(),
        Some("07:05:09")
    );
}

#[test]
fn calendar_opens_below_the_field_aligned_with_its_start() {
    // Wide viewport: there is room to the right, so nothing pushes the popup back.
    let mut h = Harness::builder()
        .with_size([900.0, 480.0])
        .with_theme(egui::Theme::Light)
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut Option<Date>| {
                ui.add(
                    DatePicker::new("issued", value)
                        .width(480.0)
                        .today(date(2026, 11, 7))
                        .accessible_name("Date of issue"),
                );
            },
            Some(date(2026, 11, 7)),
        );
    h.run();
    let field = h.get_by_label("Date of issue").rect();
    let button = h
        .get_by_role_and_label(Role::Button, "Choose Date of issue")
        .rect();
    h.get_by_role_and_label(Role::Button, "Choose Date of issue")
        .click();
    h.run_steps(2);
    let monday = h
        .get_by_role_and_label(Role::Button, "2 November 2026")
        .rect();
    let sunday = h
        .get_by_role_and_label(Role::Button, "8 November 2026")
        .rect();
    assert!(
        monday.top() > field.bottom(),
        "the grid hangs below the field"
    );
    assert!(
        monday.left() < button.left(),
        "the grid starts under the field, not under the button: {monday:?} / {button:?}"
    );
    assert!(
        monday.left() >= field.left() - 1.0,
        "the grid does not start left of the field"
    );
    assert!(
        sunday.right() <= button.right() + 1.0,
        "the grid does not spill past the field"
    );
}
