//! Fuzzy filtering and ordering of flat command and option lists.

use crate::{FuzzyMatch, MenuItem, fuzzy_match};
use std::{cmp::Reverse, ops::Range};

pub(super) struct Hit<'a, T> {
    pub item: &'a MenuItem<'a, T>,
    /// Matched byte ranges of the label.
    pub marks: Vec<Range<usize>>,
    score: i32,
}

/// Leaves whose label or group matches every query word.
///
/// A blank query keeps the host order. Otherwise groups follow their best
/// command and commands follow their score, with the host order breaking ties.
pub(super) fn rank<'a, T>(items: &'a [MenuItem<'a, T>], query: &str) -> Vec<Hit<'a, T>> {
    let mut hits: Vec<_> = items
        .iter()
        .filter(|item| item.children.is_none())
        .filter_map(|item| hit(item, query))
        .collect();
    if query.trim().is_empty() {
        return hits;
    }
    let mut groups: Vec<(Option<&'a str>, i32)> = Vec::new();
    for hit in &hits {
        let group = hit.item.group.as_deref();
        match groups.iter_mut().find(|(name, _)| *name == group) {
            Some((_, best)) => *best = (*best).max(hit.score),
            None => groups.push((group, hit.score)),
        }
    }
    groups.sort_by_key(|&(_, best)| Reverse(best));
    hits.sort_by_key(|hit| {
        let group = hit.item.group.as_deref();
        let place = groups.iter().position(|(name, _)| *name == group);
        (place, Reverse(hit.score))
    });
    hits
}

/// A word found only in the group counts half and marks nothing.
fn hit<'a, T>(item: &'a MenuItem<'a, T>, query: &str) -> Option<Hit<'a, T>> {
    let mut label = FuzzyMatch::default();
    let mut group = 0;
    for word in query.split_whitespace() {
        if let Some(found) = fuzzy_match(word, &item.label) {
            label.merge(found);
        } else {
            group += fuzzy_match(word, item.group.as_deref()?)?.score / 2;
        }
    }
    Some(Hit {
        item,
        score: label.score + group,
        marks: label.ranges,
    })
}
