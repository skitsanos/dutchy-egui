//! Fuzzy text scoring for search fields, command lists and host filters.
//!
//! A query matches a text when the characters of every query word appear in
//! the text in order, ignoring case. The score rewards contiguous runs, word
//! starts and early matches, so sorting by it puts the likeliest text first.
//!
//! ```
//! use dutchy_egui::fuzzy_match;
//!
//! let found = fuzzy_match("of", "Open File").unwrap();
//! assert_eq!(found.ranges, [0..1, 5..6]);
//! assert!(found.score > fuzzy_match("of", "Profile").unwrap().score);
//! assert!(fuzzy_match("xy", "Open File").is_none());
//! ```

use std::ops::Range;

/// Characters of a text that are scored; later characters never match.
const MAX_TEXT_CHARS: usize = 256;
/// Every matched character.
const MATCH: i32 = 16;
/// A match directly after the previous one.
const CONSECUTIVE: i32 = 12;
/// A match on the first character of a word or of a camel-case hump.
const BOUNDARY: i32 = 12;
/// A match on the first character of the text.
const START: i32 = 16;
/// The first skipped character between two matches.
const GAP_START: i32 = 3;
/// Each further skipped character.
const GAP_EXTEND: i32 = 1;
/// Most that characters before the first match can cost.
const LEADING_CAP: usize = 8;
/// "No alignment"; halved so penalties cannot overflow.
const NONE: i32 = i32::MIN / 2;

/// How a query matched one text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct FuzzyMatch {
    /// Rank within one query: higher is better. Scores of different queries
    /// are not comparable, and the scale may change between releases.
    pub score: i32,
    /// Matched byte ranges of the text, sorted, merged and on character
    /// boundaries. Pass them to [`Text::highlight`](crate::Text::highlight).
    pub ranges: Vec<Range<usize>>,
}

/// Score `text` against `query`, or return `None` when it does not match.
///
/// The query splits on whitespace into words. Every word must match, in any
/// order, and their scores add up. A blank query matches everything with a
/// score of zero and no ranges. Case is ignored one character at a time, and
/// only the first 256 characters of the text are scored.
pub fn fuzzy_match(query: &str, text: &str) -> Option<FuzzyMatch> {
    let mut found = FuzzyMatch::default();
    let mut hay = None;
    for word in query.split_whitespace() {
        let needle: Vec<char> = word.chars().map(fold).collect();
        if !is_subsequence(&needle, text) {
            return None;
        }
        let hay = hay.get_or_insert_with(|| glyphs(text));
        let (score, positions) = align(&needle, hay)?;
        // Keep the weakest match positive so callers can discount or compare it by ratio.
        found.score += score.max(1);
        found
            .ranges
            .extend(positions.into_iter().map(|at| hay[at].bytes.clone()));
    }
    found.ranges = merge_ranges(found.ranges);
    Some(found)
}

impl FuzzyMatch {
    /// Add another match on the same text, such as the match of a further
    /// query word: the scores add up and the ranges merge.
    ///
    /// ```
    /// use dutchy_egui::{FuzzyMatch, fuzzy_match};
    ///
    /// let mut subject = FuzzyMatch::default();
    /// for word in ["open", "pen", "file"] {
    ///     subject.merge(fuzzy_match(word, "Open File").unwrap());
    /// }
    /// assert_eq!(subject.ranges, [0..4, 5..9]);
    /// ```
    pub fn merge(&mut self, other: Self) {
        self.score += other.score;
        let mut ranges = std::mem::take(&mut self.ranges);
        ranges.extend(other.ranges);
        self.ranges = merge_ranges(ranges);
    }
}

/// Sort ranges and join the ones that overlap or touch.
fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
            _ => merged.push(range),
        }
    }
    merged
}

struct Glyph {
    folded: char,
    bytes: Range<usize>,
    bonus: i32,
}

/// Lowercase when that is a single character; otherwise keep the character.
fn fold(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(single), None) => single,
        _ => c,
    }
}

fn scored(text: &str) -> impl Iterator<Item = (usize, char)> + '_ {
    text.char_indices().take(MAX_TEXT_CHARS)
}

/// Cheap rejection before any allocation for the text.
fn is_subsequence(needle: &[char], text: &str) -> bool {
    let mut wanted = needle.iter();
    let mut next = wanted.next();
    for (_, c) in scored(text) {
        match next {
            Some(&n) if n == fold(c) => next = wanted.next(),
            Some(_) => {}
            None => break,
        }
    }
    next.is_none()
}

fn glyphs(text: &str) -> Vec<Glyph> {
    let mut previous = None;
    scored(text)
        .map(|(offset, c)| {
            let bonus = bonus(previous, c);
            previous = Some(c);
            Glyph {
                folded: fold(c),
                bytes: offset..offset + c.len_utf8(),
                bonus,
            }
        })
        .collect()
}

fn bonus(previous: Option<char>, current: char) -> i32 {
    match previous {
        None => START,
        Some(p) if !p.is_alphanumeric() && current.is_alphanumeric() => BOUNDARY,
        Some(p) if p.is_lowercase() && current.is_uppercase() => BOUNDARY,
        _ => 0,
    }
}

/// Best score reachable through a gap, and the match it resumes from.
struct Gap {
    score: i32,
    from: usize,
}
impl Gap {
    /// Extend the running gap by one character, or open one after `at`.
    fn advance(&mut self, score_at: i32, at: usize) {
        let opened = score_at - GAP_START;
        let extended = self.score - GAP_EXTEND;
        if opened >= extended {
            *self = Self {
                score: opened,
                from: at,
            };
        } else {
            self.score = extended;
        }
    }
}

/// The best-scoring placement of `needle` in `hay`, as glyph positions.
fn align(needle: &[char], hay: &[Glyph]) -> Option<(i32, Vec<usize>)> {
    let (rows, columns) = (needle.len(), hay.len());
    if rows == 0 || rows > columns {
        return None;
    }
    // `back[row][column]` is where the previous needle character matched.
    let mut back = vec![0u16; rows * columns];
    let mut previous = vec![NONE; columns];
    let mut current = vec![NONE; columns];
    for (row, &wanted) in needle.iter().enumerate() {
        let back = &mut back[row * columns..(row + 1) * columns];
        fill_row(row == 0, wanted, hay, &previous, &mut current, back);
        std::mem::swap(&mut previous, &mut current);
    }
    let (mut at, &score) = previous
        .iter()
        .enumerate()
        .max_by_key(|&(column, &score)| (score, std::cmp::Reverse(column)))?;
    if score <= NONE / 2 {
        return None;
    }
    let mut positions = vec![0; rows];
    for row in (0..rows).rev() {
        positions[row] = at;
        at = usize::from(back[row * columns + at]);
    }
    Some((score, positions))
}

fn fill_row(
    first: bool,
    wanted: char,
    hay: &[Glyph],
    previous: &[i32],
    current: &mut [i32],
    back: &mut [u16],
) {
    let mut gap = Gap {
        score: NONE,
        from: 0,
    };
    for (column, glyph) in hay.iter().enumerate() {
        if !first && column >= 2 {
            gap.advance(previous[column - 2], column - 2);
        }
        current[column] = NONE;
        if glyph.folded != wanted {
            continue;
        }
        let (before, from) = if first {
            (-(column.min(LEADING_CAP) as i32), 0)
        } else {
            predecessor(previous, column, &gap)
        };
        if before > NONE / 2 {
            current[column] = MATCH + glyph.bonus + before;
            // Columns stay below `MAX_TEXT_CHARS`, so they fit.
            back[column] = from as u16;
        }
    }
}

/// The better way to reach `column`: straight after the previous match, or
/// across a gap.
fn predecessor(previous: &[i32], column: usize, gap: &Gap) -> (i32, usize) {
    if column == 0 {
        return (NONE, 0);
    }
    let run = previous[column - 1] + CONSECUTIVE;
    if run >= gap.score {
        (run, column - 1)
    } else {
        (gap.score, gap.from)
    }
}
