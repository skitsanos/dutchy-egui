use crate::tokens::navigation::ALL_PAGES_LIMIT;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Entry {
    Page(usize),
    Ellipsis,
}

pub(super) fn entries(current: usize, total: usize) -> Vec<Entry> {
    if total <= ALL_PAGES_LIMIT {
        return (1..=total).map(Entry::Page).collect();
    }
    let mut candidates = vec![
        1,
        current.saturating_sub(1).max(1),
        current,
        current.saturating_add(1).min(total),
        total,
    ];
    candidates.sort_unstable();
    candidates.dedup();
    let mut result = Vec::with_capacity(ALL_PAGES_LIMIT);
    let mut previous = 0;
    for page in candidates {
        let gap = page - previous;
        if gap == 2 {
            result.push(Entry::Page(page - 1));
        } else if gap > 2 {
            result.push(Entry::Ellipsis);
        }
        result.push(Entry::Page(page));
        previous = page;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ranges_are_bounded_ordered_and_cover_current_and_endpoints() {
        for total in (1..=100).chain([usize::MAX]) {
            for current in [
                1,
                2.min(total),
                total / 2 + 1,
                total.saturating_sub(1).max(1),
                total,
            ] {
                let list = entries(current, total);
                assert!(list.len() <= ALL_PAGES_LIMIT);
                assert_eq!(list.first(), Some(&Entry::Page(1)));
                assert_eq!(list.last(), Some(&Entry::Page(total)));
                assert!(list.contains(&Entry::Page(current)));
                let pages: Vec<_> = list
                    .iter()
                    .filter_map(|entry| {
                        if let Entry::Page(page) = entry {
                            Some(*page)
                        } else {
                            None
                        }
                    })
                    .collect();
                assert!(pages.windows(2).all(|pair| pair[0] < pair[1]));
            }
        }
    }
    #[test]
    fn small_sets_and_single_page_gaps_do_not_hide_a_destination() {
        assert!(entries(1, 0).is_empty());
        assert_eq!(entries(3, 5), (1..=5).map(Entry::Page).collect::<Vec<_>>());
        assert_eq!(
            entries(4, 20),
            vec![
                Entry::Page(1),
                Entry::Page(2),
                Entry::Page(3),
                Entry::Page(4),
                Entry::Page(5),
                Entry::Ellipsis,
                Entry::Page(20)
            ]
        );
    }
}
