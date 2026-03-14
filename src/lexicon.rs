use std::collections::HashMap;
use std::sync::LazyLock;

static AFINN_DATA: &str = include_str!("../data/afinn-111.txt");

static AFINN: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    AFINN_DATA
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let word = parts.next()?.trim();
            let score = parts.next()?.trim().parse::<i32>().ok()?;
            if word.is_empty() {
                return None;
            }
            Some((word, score))
        })
        .collect()
});

/// Look up the AFINN sentiment score for a word.
/// Returns None if the word is not in the lexicon.
pub fn get_score(word: &str) -> Option<i32> {
    AFINN.get(word).copied()
}

/// Returns the number of words in the AFINN lexicon.
#[cfg(test)]
pub fn lexicon_size() -> usize {
    AFINN.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicon_loads() {
        assert!(lexicon_size() > 2000);
    }

    #[test]
    fn positive_word() {
        assert!(get_score("love").unwrap() > 0);
    }

    #[test]
    fn negative_word() {
        assert!(get_score("hate").unwrap() < 0);
    }

    #[test]
    fn unknown_word() {
        assert_eq!(get_score("asdfghjkl"), None);
    }
}
