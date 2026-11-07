//! Clock times with per-segment wrapping steps and `HH:MM[:SS]` text.

use super::digits;
use std::fmt;

/// Segment of a clock time, for caret-based stepping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeSegment {
    /// `HH`.
    Hour,
    /// `MM`.
    Minute,
    /// `SS`.
    Second,
}

/// A 24-hour clock time. Construct through [`Time::new`] or [`Time::parse`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

impl Time {
    /// A validated time; None past 23:59:59.
    pub const fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        Some(Self {
            hour,
            minute,
            second,
        })
    }
    /// Hour, 0 to 23.
    pub const fn hour(self) -> u8 {
        self.hour
    }
    /// Minute, 0 to 59.
    pub const fn minute(self) -> u8 {
        self.minute
    }
    /// Second, 0 to 59.
    pub const fn second(self) -> u8 {
        self.second
    }
    /// Step one segment by `delta`, wrapping inside that segment only.
    pub const fn step(self, segment: TimeSegment, delta: i32) -> Self {
        match segment {
            TimeSegment::Hour => Self {
                hour: wrap(self.hour, delta, 24),
                ..self
            },
            TimeSegment::Minute => Self {
                minute: wrap(self.minute, delta, 60),
                ..self
            },
            TimeSegment::Second => Self {
                second: wrap(self.second, delta, 60),
                ..self
            },
        }
    }
    /// Parse strict `HH:MM` or `HH:MM:SS`.
    pub fn parse(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        let seconds = match bytes.len() {
            5 => false,
            8 if bytes[5] == b':' => true,
            _ => return None,
        };
        if bytes[2] != b':' {
            return None;
        }
        let hour = digits(&text[0..2])?;
        let minute = digits(&text[3..5])?;
        let second = if seconds { digits(&text[6..8])? } else { 0 };
        Self::new(
            u8::try_from(hour).ok()?,
            u8::try_from(minute).ok()?,
            u8::try_from(second).ok()?,
        )
    }
    /// `HH:MM`, or `HH:MM:SS` when `seconds` is set.
    pub fn format(self, seconds: bool) -> String {
        if seconds {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        } else {
            format!("{:02}:{:02}", self.hour, self.minute)
        }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.format(true))
    }
}

/// Offset a clock segment by `delta`, wrapping inside `modulus`.
const fn wrap(value: u8, delta: i32, modulus: i32) -> u8 {
    (value as i64 + delta as i64).rem_euclid(modulus as i64) as u8
}
