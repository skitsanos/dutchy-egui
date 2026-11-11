//! Fuzzy scoring: matching rules, ranking order and highlight ranges.
use dutchy_egui::{FuzzyMatch, fuzzy_match};

fn score(query: &str, text: &str) -> i32 {
    fuzzy_match(query, text)
        .unwrap_or_else(|| panic!("{query:?} should match {text:?}"))
        .score
}

/// Matched byte ranges as `(start, end)` pairs.
fn ranges(query: &str, text: &str) -> Vec<(usize, usize)> {
    let found =
        fuzzy_match(query, text).unwrap_or_else(|| panic!("{query:?} should match {text:?}"));
    found.ranges.iter().map(|r| (r.start, r.end)).collect()
}

#[test]
fn blank_queries_match_everything_without_ranges() {
    for query in ["", "   ", "\t"] {
        let found: FuzzyMatch = fuzzy_match(query, "Open file").unwrap();
        assert_eq!((found.score, found.ranges.len()), (0, 0), "{query:?}");
    }
    assert!(fuzzy_match("", "").is_some());
}

#[test]
fn characters_must_appear_in_order() {
    assert!(fuzzy_match("opn", "Open file").is_some());
    assert!(fuzzy_match("npo", "Open file").is_none());
    assert!(fuzzy_match("openx", "Open file").is_none());
    assert!(fuzzy_match("a", "").is_none());
}

#[test]
fn matching_ignores_case_and_reports_byte_ranges() {
    assert_eq!(ranges("OPEN", "open file"), [(0, 4)]);
    // "É" is two bytes; the range still ends on a character boundary.
    assert_eq!(ranges("éc", "École"), [(0, 3)]);
    assert_eq!(ranges("ñ", "Año"), [(1, 3)]);
    let text = "Año nuevo";
    for (start, end) in ranges("ñ nv", text) {
        assert!(text.get(start..end).is_some());
    }
}

#[test]
fn a_contiguous_run_beats_scattered_characters() {
    assert!(score("open", "Open file") > score("open", "Option panel entry"));
    // The best alignment wins, not the first one found.
    assert_eq!(ranges("file", "fix line file"), [(9, 13)]);
}

#[test]
fn word_starts_beat_matches_inside_words() {
    assert!(score("fil", "Open File") > score("fil", "Profile"));
    assert!(score("fil", "A fairly long group name File") > score("fil", "Profile"));
    assert_eq!(ranges("of", "Open File"), [(0, 1), (5, 6)]);
    assert_eq!(ranges("fb", "FooBar"), [(0, 1), (3, 4)]);
    assert!(score("fb", "FooBar") > score("fb", "Foobar"));
    assert!(score("np", "new-project") > score("np", "snapshot"));
}

#[test]
fn earlier_matches_rank_higher() {
    assert!(score("da", "Dashboard") > score("da", "Sandbox data"));
    assert!(score("dash", "Dashboard") > score("dash", "My dashboard"));
}

#[test]
fn loose_matches_have_a_positive_score() {
    let text = format!("a{}b", "x".repeat(240));
    assert!(score("ab", &text) > 0);
    assert!(score("ab", &text) < score("ab", "ab"));
    assert_eq!(ranges("ab", &text), [(0, 1), (241, 242)]);
}

#[test]
fn every_word_must_match_in_any_order() {
    assert_eq!(ranges("file open", "Open File"), [(0, 4), (5, 9)]);
    assert!(fuzzy_match("open zip", "Open File").is_none());
    assert!(score("open file", "Open File") > score("open", "Open File"));
    // Overlapping and touching word matches merge into one range.
    assert_eq!(ranges("op open", "Open File"), [(0, 4)]);
}

#[test]
fn ranges_are_sorted_and_disjoint() {
    let found = ranges("r pt ex", "Export report as text");
    assert!(!found.is_empty());
    for pair in found.windows(2) {
        assert!(pair[0].1 < pair[1].0, "{found:?}");
    }
}

#[test]
fn only_the_first_256_characters_are_scored() {
    let long = format!("{}z", "a".repeat(300));
    assert!(fuzzy_match("z", &long).is_none());
    assert_eq!(ranges("a", &long), [(0, 1)]);
    let within = format!("{}z", "a".repeat(255));
    assert_eq!(ranges("z", &within), [(255, 256)]);
    // A query longer than the text cannot match and must not panic.
    assert!(fuzzy_match(&"a".repeat(400), &long).is_none());
}

#[test]
fn merging_adds_scores_and_joins_ranges() {
    let mut subject = FuzzyMatch::default();
    let file = fuzzy_match("file", "Open File").unwrap();
    let open = fuzzy_match("open", "Open File").unwrap();
    let total = file.score + open.score;
    subject.merge(file);
    subject.merge(open);
    assert_eq!(
        (subject.score, subject.ranges.as_slice()),
        (total, &[0..4, 5..9][..])
    );
    subject.merge(fuzzy_match("n f", "Open File").unwrap());
    assert_eq!(subject.ranges, [0..4, 5..9]);
}
