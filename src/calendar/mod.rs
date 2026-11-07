//! Calendar dates and clock times with ISO 8601 text for the date and time
//! pickers. Proleptic Gregorian calendar, years 1 to 9999, no time zones:
//! hosts convert to and from their own instants.

use std::fmt;

/// Day of the week; the pickers start weeks on Monday.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// First column of the calendar grid.
    Monday,
    /// Second column.
    Tuesday,
    /// Third column.
    Wednesday,
    /// Fourth column.
    Thursday,
    /// Fifth column.
    Friday,
    /// Sixth column.
    Saturday,
    /// Last column.
    Sunday,
}

impl Weekday {
    /// Monday first.
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];

    /// Column index, Monday is 0.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Full English name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Monday => "Monday",
            Self::Tuesday => "Tuesday",
            Self::Wednesday => "Wednesday",
            Self::Thursday => "Thursday",
            Self::Friday => "Friday",
            Self::Saturday => "Saturday",
            Self::Sunday => "Sunday",
        }
    }

    /// Three-letter header label.
    pub const fn short(self) -> &'static str {
        match self {
            Self::Monday => "Mon",
            Self::Tuesday => "Tue",
            Self::Wednesday => "Wed",
            Self::Thursday => "Thu",
            Self::Friday => "Fri",
            Self::Saturday => "Sat",
            Self::Sunday => "Sun",
        }
    }
}

/// Segment of an ISO date, for caret-based stepping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSegment {
    /// `YYYY`.
    Year,
    /// `MM`.
    Month,
    /// `DD`.
    Day,
}

/// A calendar date. Construct through [`Date::new`] or [`Date::parse`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

const EPOCH_OFFSET: i64 = 719_468;
const DAYS_PER_ERA: i64 = 146_097;

impl Date {
    /// A validated date; None for an impossible day or a year outside 1 to 9999.
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if year < 1 || year > 9999 || month < 1 || month > 12 || day < 1 {
            return None;
        }
        if day > Self::days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }
    /// Four-digit year.
    pub const fn year(self) -> i32 {
        self.year
    }
    /// Month, 1 to 12.
    pub const fn month(self) -> u8 {
        self.month
    }
    /// Day of the month, 1 to 31.
    pub const fn day(self) -> u8 {
        self.day
    }
    /// Gregorian leap-year rule.
    pub const fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }
    /// Length of a month; 0 for an invalid month.
    pub const fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }
    /// English month name.
    pub const fn month_name(month: u8) -> &'static str {
        match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        }
    }
    /// Days since 1970-01-01, negative before it.
    pub const fn days_since_epoch(self) -> i64 {
        let year = if self.month <= 2 {
            self.year as i64 - 1
        } else {
            self.year as i64
        };
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let shifted_month = (self.month as i64 + 9) % 12;
        let day_of_year = (153 * shifted_month + 2) / 5 + self.day as i64 - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * DAYS_PER_ERA + day_of_era - EPOCH_OFFSET
    }
    /// The date `days` after 1970-01-01; None outside the supported years.
    pub const fn from_days_since_epoch(days: i64) -> Option<Self> {
        let min = Self {
            year: 1,
            month: 1,
            day: 1,
        }
        .days_since_epoch();
        let max = Self {
            year: 9999,
            month: 12,
            day: 31,
        }
        .days_since_epoch();
        if days < min || days > max {
            return None;
        }
        let shifted = days + EPOCH_OFFSET;
        let era = if shifted >= 0 {
            shifted
        } else {
            shifted - (DAYS_PER_ERA - 1)
        } / DAYS_PER_ERA;
        let day_of_era = shifted - era * DAYS_PER_ERA;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146_096) / 365;
        let year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u8;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        } as u8;
        let year = if month <= 2 { year + 1 } else { year };
        if year < 1 || year > 9999 {
            return None;
        }
        Self::new(year as i32, month, day)
    }
    /// Monday-first weekday.
    pub const fn weekday(self) -> Weekday {
        // 1970-01-01 was a Thursday, index 3.
        match (self.days_since_epoch() + 3).rem_euclid(7) {
            0 => Weekday::Monday,
            1 => Weekday::Tuesday,
            2 => Weekday::Wednesday,
            3 => Weekday::Thursday,
            4 => Weekday::Friday,
            5 => Weekday::Saturday,
            _ => Weekday::Sunday,
        }
    }
    /// Offset by whole days; None outside the supported years.
    pub const fn add_days(self, days: i64) -> Option<Self> {
        match self.days_since_epoch().checked_add(days) {
            Some(days) => Self::from_days_since_epoch(days),
            None => None,
        }
    }
    /// Offset by whole months, clamping the day to the target month's length.
    pub const fn add_months(self, months: i32) -> Option<Self> {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months as i64;
        let year = total.div_euclid(12);
        let month = (total.rem_euclid(12) + 1) as u8;
        if year < 1 || year > 9999 {
            return None;
        }
        let year = year as i32;
        let length = Self::days_in_month(year, month);
        let day = if self.day > length { length } else { self.day };
        Self::new(year, month, day)
    }
    /// The first day of this date's month.
    pub const fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }
    /// Step one segment by `delta` units.
    pub const fn step(self, segment: DateSegment, delta: i32) -> Option<Self> {
        match segment {
            DateSegment::Year => self.add_months(delta.saturating_mul(12)),
            DateSegment::Month => self.add_months(delta),
            DateSegment::Day => self.add_days(delta as i64),
        }
    }
    /// Parse strict `YYYY-MM-DD`.
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year = digits(&text[0..4])?;
        let month = digits(&text[5..7])?;
        let day = digits(&text[8..10])?;
        Self::new(year, u8::try_from(month).ok()?, u8::try_from(day).ok()?)
    }
    /// Clamp into optional inclusive bounds.
    pub fn clamp_to(self, min: Option<Self>, max: Option<Self>) -> Self {
        let mut date = self;
        if let Some(min) = min
            && date < min
        {
            date = min;
        }
        if let Some(max) = max
            && date > max
        {
            date = max;
        }
        date
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Parse a run of ASCII digits only; signs, spaces and other characters fail.
fn digits(text: &str) -> Option<i32> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

#[cfg(test)]
mod tests;
mod time;

pub use time::{Time, TimeSegment};
