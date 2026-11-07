//! Public arithmetic boundaries and bounded calendar keyboard entry.
mod common;
use dutchy_egui::{
    DatePicker, Theme,
    calendar::{Date, Time, TimeSegment},
};
use egui::{Key, accesskit::Role};
use egui_kittest::{Harness, kittest::Queryable};

fn date(day: u8) -> Date {
    Date::new(2026, 11, day).unwrap()
}
fn calendar(value: Option<Date>) -> Harness<'static, Option<Date>> {
    Harness::builder()
        .with_size([600.0, 500.0])
        .with_context_setup(|ctx| common::install(ctx, Theme::default().with_reduced_motion(true)))
        .build_ui_state(
            |ui, value: &mut Option<Date>| {
                ui.add(
                    DatePicker::new("date", value)
                        .min(date(10))
                        .max(date(20))
                        .today(date(7))
                        .accessible_name("Date"),
                );
            },
            value,
        )
}
fn open(h: &mut Harness<'_, Option<Date>>) {
    h.run();
    h.get_by_role_and_label(Role::Button, "Choose Date").click();
    h.run_steps(3);
}
#[test]
fn empty_bounded_calendar_focuses_an_enabled_date() {
    let mut h = calendar(None);
    open(&mut h);
    assert!(
        h.get_by_role_and_label(Role::Button, "10 November 2026")
            .is_focused()
    );
    assert_eq!(
        *h.state(),
        None,
        "opening does not commit the fallback date"
    );
}
#[test]
fn keyboard_navigation_stays_inside_calendar_bounds() {
    let mut h = calendar(Some(date(10)));
    open(&mut h);
    h.key_press(Key::ArrowLeft);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "10 November 2026")
            .is_focused()
    );
    h.key_press(Key::PageDown);
    h.run_steps(2);
    assert!(
        h.get_by_role_and_label(Role::Button, "20 November 2026")
            .is_focused()
    );
    h.key_press(Key::Enter);
    h.run_steps(2);
    assert_eq!(*h.state(), Some(date(20)));
}
#[test]
fn epoch_conversion_rejects_extreme_days_without_overflow() {
    for days in [i64::MIN, i64::MIN + 719_468, i64::MAX - 719_468, i64::MAX] {
        assert_eq!(Date::from_days_since_epoch(days), None);
    }
}
#[test]
fn adding_extreme_days_returns_none() {
    assert_eq!(date(7).add_days(i64::MAX), None);
    assert_eq!(date(7).add_days(i64::MIN), None);
}
#[test]
fn clock_steps_handle_extreme_deltas() {
    let value = Time::new(23, 59, 59).unwrap();
    for delta in [i32::MIN, i32::MAX] {
        let expected = Time::new((23i64 + i64::from(delta)).rem_euclid(24) as u8, 59, 59).unwrap();
        assert_eq!(value.step(TimeSegment::Hour, delta), expected);
    }
}

#[test]
fn calendar_month_controls_paint_visible_chevrons() {
    fn visible_chevron(shape: &egui::Shape, rect: egui::Rect) -> bool {
        match shape {
            egui::Shape::Path(path) if path.points.len() == 3 => {
                let bounds = egui::Rect::from_points(&path.points);
                rect.contains_rect(bounds) && bounds.width() >= 8.0 && bounds.height() >= 8.0
            }
            egui::Shape::Vec(shapes) => shapes.iter().any(|shape| visible_chevron(shape, rect)),
            _ => false,
        }
    }
    let mut h = calendar(Some(date(10)));
    open(&mut h);
    for label in ["Previous month", "Next month"] {
        let rect = h.get_by_role_and_label(Role::Button, label).rect();
        assert!(
            h.output()
                .shapes
                .iter()
                .any(|clipped| visible_chevron(&clipped.shape, rect)),
            "{label} must paint a chevron with nonzero width and height"
        );
    }
}
