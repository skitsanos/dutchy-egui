//! Deterministic fictional data for the examples.
//!
//! Everything here is invented: the people do not exist and the addresses use
//! the reserved `example.com` domain. The generator is a SplitMix64 sequence
//! from a fixed seed, so every run and every test sees the same records.

use dutchy_egui::calendar::Date;
use std::time::{SystemTime, UNIX_EPOCH};

/// Seed shared by the example data sets.
pub const SEED: u64 = 0x4455_5443_4859_3031;

/// Fixed anchor date for seeded records and tests.
pub const FIXED_TODAY: Date = match Date::new(2026, 11, 8) {
    Some(date) => date,
    None => panic!("2026-11-08 is a valid date"),
};

/// Fictional people for assignees, senders and avatars.
pub const PEOPLE: [&str; 10] = [
    "Mara Lindqvist",
    "Tobias Achterberg",
    "Priya Venkataraman",
    "Jonas Kettering",
    "Amara Okonkwo-Reyes",
    "Felix Brandvold",
    "Noor El-Amin",
    "Sanne Vredenburg",
    "Kenji Watanabe-Holm",
    "Ilse Marchetti",
];

/// A reserved-domain address derived from a fictional name.
pub fn email(name: &str) -> String {
    let local: String = name
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(".");
    format!("{local}@example.com")
}

/// The host's calendar date in UTC, or the fixed anchor when the clock is
/// unavailable. The library reads no clock; examples pass this in.
pub fn today() -> Date {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs() / 86_400).ok())
        .and_then(Date::from_days_since_epoch)
        .unwrap_or(FIXED_TODAY)
}

/// SplitMix64 generator: small, deterministic and dependency free.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    /// Start a sequence from a seed.
    pub const fn seeded(seed: u64) -> Self {
        Self(seed)
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..bound`; a zero bound yields zero.
    pub fn below(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            return 0;
        }
        ((self.next_u64() >> 33) % u64::from(bound)) as u32
    }

    /// True with the given percentage probability.
    pub fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent
    }

    /// One element of a non-empty slice.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let index = self.below(items.len() as u32) as usize;
        &items[index]
    }
}
