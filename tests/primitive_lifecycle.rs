//! Focused form primitives respect host lifecycle and stable identities.
mod common;
use dutchy_egui::calendar::{Date, Time};
use dutchy_egui::{DatePicker, NumberInput, Segmented, SelectOption, Slider, Theme, TimePicker};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

#[derive(Clone, Copy, Debug)]
enum Kind {
    Number,
    Date,
    Time,
    Slider,
    Segmented,
}
struct State {
    kind: Kind,
    enabled: bool,
    parent_enabled: bool,
    prefix: bool,
    covered: bool,
    number: f64,
    date: Option<Date>,
    time: Option<Time>,
    choice: Option<u8>,
}
fn harness(kind: Kind) -> Harness<'static, State> {
    Harness::builder()
        .with_size([500.0, 300.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, s: &mut State| {
                if s.prefix {
                    ui.label("A newly inserted sibling");
                }
                ui.add_enabled_ui(s.parent_enabled, |ui| match s.kind {
                    Kind::Number => {
                        ui.add(
                            NumberInput::new("value", &mut s.number)
                                .enabled(s.enabled)
                                .accessible_name("Value"),
                        );
                    }
                    Kind::Date => {
                        ui.add(
                            DatePicker::new("value", &mut s.date)
                                .enabled(s.enabled)
                                .accessible_name("Value"),
                        );
                    }
                    Kind::Time => {
                        ui.add(
                            TimePicker::new("value", &mut s.time)
                                .enabled(s.enabled)
                                .accessible_name("Value"),
                        );
                    }
                    Kind::Slider => {
                        ui.add(
                            Slider::new("value", &mut s.number, 0.0..=10.0)
                                .enabled(s.enabled)
                                .accessible_name("Value"),
                        );
                    }
                    Kind::Segmented => {
                        let options = [SelectOption::new(0, "Alpha"), SelectOption::new(1, "Beta")];
                        ui.add(Segmented::new("value", &mut s.choice, &options).enabled(s.enabled));
                    }
                });
                if s.covered {
                    egui::Modal::new(egui::Id::unique("cover")).show(ui.ctx(), |ui| {
                        ui.label("Covering modal");
                    });
                }
            },
            State {
                kind,
                enabled: true,
                parent_enabled: true,
                prefix: false,
                covered: false,
                number: 5.0,
                date: Date::new(2026, 11, 7),
                time: Time::new(9, 30, 0),
                choice: Some(0),
            },
        )
}

fn disable_focused(kind: Kind) {
    for parent in [false, true] {
        let mut h = harness(kind);
        h.run();
        h.get_by_label("Value").focus();
        h.run();
        if parent {
            h.state_mut().parent_enabled = false;
        } else {
            h.state_mut().enabled = false;
        }
        h.key_press(Key::ArrowUp);
        h.run_steps(2);
        assert_eq!(h.state().number, 5.0, "{kind:?}, parent={parent}");
        assert_eq!(
            h.state().date,
            Date::new(2026, 11, 7),
            "{kind:?}, parent={parent}"
        );
        assert_eq!(
            h.state().time,
            Time::new(9, 30, 0),
            "{kind:?}, parent={parent}"
        );
    }
}
#[test]
fn disabled_number_does_not_step_while_losing_focus() {
    disable_focused(Kind::Number);
}
#[test]
fn disabled_date_does_not_step_while_losing_focus() {
    disable_focused(Kind::Date);
}
#[test]
fn disabled_time_does_not_step_while_losing_focus() {
    disable_focused(Kind::Time);
}

#[test]
fn disabling_a_date_picker_closes_its_popup() {
    let mut h = harness(Kind::Date);
    h.run();
    h.get_by_role_and_label(Role::Button, "Choose Value")
        .click();
    h.run_steps(3);
    h.get_by_label("November 2026");
    h.state_mut().enabled = false;
    h.run_steps(3);
    assert!(h.query_by_label("November 2026").is_none());
    assert_eq!(h.state().date, Date::new(2026, 11, 7));
}

fn preserve_focus(kind: Kind) {
    let mut h = harness(kind);
    h.run();
    let label = if matches!(kind, Kind::Segmented) {
        "Alpha"
    } else {
        "Value"
    };
    h.get_by_label(label).focus();
    h.run();
    let focused = h.ctx.memory(|m| m.focused());
    h.state_mut().prefix = true;
    h.run_steps(2);
    assert_eq!(h.ctx.memory(|m| m.focused()), focused, "{kind:?}");
    assert!(h.get_by_label(label).is_focused(), "{kind:?}");
    h.key_press(Key::ArrowRight);
    h.run();
    match kind {
        Kind::Segmented => assert!(
            h.get_by_role_and_label(Role::RadioButton, "Beta")
                .is_focused()
        ),
        Kind::Slider => assert!(h.state().number > 5.0),
        _ => unreachable!(),
    }
}
#[test]
fn inserted_siblings_preserve_slider_focus() {
    preserve_focus(Kind::Slider);
}
#[test]
fn inserted_siblings_preserve_segment_focus() {
    preserve_focus(Kind::Segmented);
}

#[test]
fn covering_modal_blocks_background_stepping() {
    for kind in [Kind::Number, Kind::Date, Kind::Time, Kind::Segmented] {
        let mut h = harness(kind);
        h.run();
        h.get_by_label(if matches!(kind, Kind::Segmented) {
            "Alpha"
        } else {
            "Value"
        })
        .focus();
        h.run();
        h.state_mut().covered = true;
        h.run_steps(3);
        h.key_press(if matches!(kind, Kind::Segmented) {
            Key::ArrowRight
        } else {
            Key::ArrowUp
        });
        h.run_steps(2);
        assert_eq!(h.state().number, 5.0, "{kind:?}");
        assert_eq!(h.state().date, Date::new(2026, 11, 7), "{kind:?}");
        assert_eq!(h.state().time, Time::new(9, 30, 0), "{kind:?}");
        assert_eq!(h.state().choice, Some(0), "{kind:?}");
    }
}
