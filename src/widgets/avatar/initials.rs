//! Initials derived from a display name.

/// Up to two initials: the first letter or digit of the first and of the last
/// word, uppercased. Words without letters or digits are skipped. Characters
/// are Unicode scalar values, not grapheme clusters.
pub(super) fn initials(name: &str) -> String {
    let mut words = name
        .split_whitespace()
        .filter_map(|word| word.chars().find(|c| c.is_alphanumeric()));
    let first = words.next();
    let last = words.next_back();
    first
        .into_iter()
        .chain(last)
        .flat_map(char::to_uppercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::initials;

    #[test]
    fn first_and_last_words_give_two_initials() {
        assert_eq!(initials("Ada Lovelace"), "AL");
        assert_eq!(initials("Grace Brewster Murray Hopper"), "GH");
        assert_eq!(initials("ada"), "A");
        assert_eq!(initials("émile zola"), "ÉZ");
    }

    #[test]
    fn symbols_spacing_and_addresses_do_not_become_initials() {
        assert_eq!(initials("  (Ada) - Lovelace  "), "AL");
        assert_eq!(initials("ada@example.com"), "A");
        assert_eq!(initials("2nd shift"), "2S");
        assert_eq!(initials("— !"), "");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn uppercasing_follows_unicode_case_mappings() {
        assert_eq!(initials("ßeta"), "SS");
        assert_eq!(initials("ǆemal"), "Ǆ");
    }
}
