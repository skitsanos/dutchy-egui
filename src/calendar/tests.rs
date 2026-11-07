use super::*;

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::new(year, month, day).unwrap()
}

#[test]
fn epoch_days_and_weekdays_match_reference_values() {
    for (text, days, weekday) in [
        ("1970-01-01", 0, Weekday::Thursday),
        ("2000-01-01", 10957, Weekday::Saturday),
        ("2024-02-29", 19782, Weekday::Thursday),
        ("2026-11-07", 20764, Weekday::Saturday),
        ("2026-11-01", 20758, Weekday::Sunday),
        ("2026-12-01", 20788, Weekday::Tuesday),
        ("1999-12-31", 10956, Weekday::Friday),
        ("2100-03-01", 47541, Weekday::Monday),
    ] {
        let parsed = Date::parse(text).unwrap();
        assert_eq!(parsed.days_since_epoch(), days, "{text}");
        assert_eq!(parsed.weekday(), weekday, "{text}");
        assert_eq!(Date::from_days_since_epoch(days), Some(parsed), "{text}");
        assert_eq!(parsed.to_string(), text);
    }
}

#[test]
fn leap_years_and_month_lengths() {
    assert!(Date::is_leap_year(2024) && Date::is_leap_year(2000));
    assert!(!Date::is_leap_year(2100) && !Date::is_leap_year(1900));
    assert_eq!(Date::days_in_month(2024, 2), 29);
    assert_eq!(Date::days_in_month(2100, 2), 28);
    assert_eq!(Date::days_in_month(2026, 11), 30);
    assert_eq!(Date::new(2026, 2, 29), None);
    assert_eq!(Date::new(2026, 13, 1), None);
    assert_eq!(Date::new(0, 1, 1), None);
}

#[test]
fn month_and_day_arithmetic_clamp_and_roll_over() {
    assert_eq!(date(2026, 1, 31).add_months(1), Some(date(2026, 2, 28)));
    assert_eq!(date(2024, 1, 31).add_months(1), Some(date(2024, 2, 29)));
    assert_eq!(date(2026, 11, 7).add_months(-11), Some(date(2025, 12, 7)));
    assert_eq!(date(2026, 12, 31).add_days(1), Some(date(2027, 1, 1)));
    assert_eq!(date(2026, 1, 1).add_days(-1), Some(date(2025, 12, 31)));
    assert_eq!(date(9999, 12, 31).add_days(1), None);
    assert_eq!(
        date(2026, 11, 7).step(DateSegment::Year, 1),
        Some(date(2027, 11, 7))
    );
    assert_eq!(date(2026, 11, 7).first_of_month(), date(2026, 11, 1));
    assert_eq!(
        date(2026, 11, 30).clamp_to(Some(date(2026, 11, 5)), Some(date(2026, 11, 20))),
        date(2026, 11, 20)
    );
}

#[test]
fn date_parsing_is_strict() {
    assert_eq!(Date::parse("2026-11-07"), Some(date(2026, 11, 7)));
    for text in [
        "2026-13-01",
        "2026-2-1",
        "2026/11/07",
        "",
        "2026-11-07 ",
        "+026-11-07",
    ] {
        assert_eq!(Date::parse(text), None, "{text:?}");
    }
}

#[test]
fn times_parse_format_and_wrap_within_a_segment() {
    let time = Time::parse("09:30").unwrap();
    assert_eq!(time, Time::new(9, 30, 0).unwrap());
    assert_eq!(Time::parse("09:30:15").unwrap().second(), 15);
    for text in ["24:00", "9:30", "09:60", "09:30:60", "0930", ""] {
        assert_eq!(Time::parse(text), None, "{text:?}");
    }
    assert_eq!(time.format(false), "09:30");
    assert_eq!(time.format(true), "09:30:00");
    assert_eq!(time.to_string(), "09:30:00");
    let late = Time::new(23, 59, 59).unwrap();
    assert_eq!(
        late.step(TimeSegment::Minute, 1),
        Time::new(23, 0, 59).unwrap()
    );
    assert_eq!(
        late.step(TimeSegment::Hour, 1),
        Time::new(0, 59, 59).unwrap()
    );
    assert_eq!(
        Time::new(0, 0, 0).unwrap().step(TimeSegment::Second, -1),
        Time::new(0, 0, 59).unwrap()
    );
}
