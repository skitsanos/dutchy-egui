# Fuzzy search

`fuzzy_match(query, text)` scores one text against a typed query and reports
which characters matched. CommandPalette and SearchableSelect rank their
results with it, and hosts can use it for their own lists. The scorer lives in
the repository; no search crate is required.

```rust
use dutchy_egui::{Text, fuzzy_match};
use std::cmp::Reverse;

let mut found: Vec<_> = projects
    .iter()
    .filter_map(|project| Some((fuzzy_match(&query, &project.name)?, project)))
    .collect();
// The sort is stable, so equal scores keep the host order.
found.sort_by_key(|(matched, _)| Reverse(matched.score));
for (matched, project) in &found {
    ui.add(Text::new(project.name.as_str()).highlight(&matched.ranges));
}
```

## Matching

- The query splits on whitespace into words. Every word must match, in any
  order, so "file open" finds "Open File".
- A word matches when its characters appear in the text in order: "opn" and
  "of" both find "Open File". Case is ignored.
- A blank query matches every text with a score of zero and no ranges.
- `None` means no match. The host decides what an empty result shows.

## Ranking

A higher `score` is a better match. The scorer finds the best placement of
each word, not the first one, and rewards:

- characters that follow each other, so "open" prefers "Open file" to
  "Option panel entry";
- the start of the text, the start of a word and a camel-case hump, so "fb"
  finds the F and B of "FooBar" and "fil" prefers "Open File" to "Profile";
- matches near the beginning of the text.

Skipped characters between matches cost a little. Compare scores only for one
query; the scale may change between releases.

## Highlighting

`FuzzyMatch::ranges` holds the matched byte ranges, sorted and merged.
`Text::highlight(&ranges)` draws them in the bold face and leaves the
accessible name unchanged. Ranges that are out of order, out of bounds or
inside a character are skipped. Bold text shows no difference, and neither
does a host font without the `bundled-fonts` feature, which has one weight.

## CommandPalette and SearchableSelect

Both match a query word against the label first and mark it there. A word
found only in the group, such as a submenu path in the palette, still matches
at half its score. With a query, groups follow their best result and results
follow their score; host separators are left out because they describe the
host order. A blank query lists the host order unchanged. Enter activates the
first enabled result, which is now the best match. While a query marks
matches, the selected option keeps its fill but not its bold label.

## Several fields and long lists

To search a record with several fields, match each word against each field
and keep the better one. `FuzzyMatch::merge` adds a further word's match on
the same field:

```rust
let (mut sender, mut subject) = (FuzzyMatch::default(), FuzzyMatch::default());
for word in query.split_whitespace() {
    match (fuzzy_match(word, &mail.sender), fuzzy_match(word, &mail.subject)) {
        (Some(a), Some(b)) if b.score > a.score => subject.merge(b),
        (Some(a), _) => sender.merge(a),
        (None, Some(b)) => subject.merge(b),
        (None, None) => return None,
    }
}
```

Short queries match loosely: in a list of thousands, two or three letters
appear in order in most rows. Sorting puts the good matches first, but the
tail is noise. DutchyMail drops a message when one of its words scores under
three quarters of that word's best score in the folder; see `Mailbox::search`
in `examples/dutchy_mail/mailbox.rs`.

## Limits

- Only the first 256 characters of a text are scored; later ones never match.
- Case folding is per character. There is no accent folding, locale rule or
  typo tolerance: a transposed or missing letter does not match.
- [List](lists.md) type-ahead stays a prefix search, as list boxes
  conventionally behave. Filter the items you pass to a List instead.
- Scoring a list runs on every frame it is shown. A few thousand short texts
  are fine; cache the result per query for larger sets.
