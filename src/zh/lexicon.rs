use std::collections::HashMap;
use std::sync::LazyLock;

static SENTIMENT_ZH_DATA: &str = include_str!("../../data/zh/sentiment_zh.txt");

static SENTIMENT_ZH: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    SENTIMENT_ZH_DATA
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
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

/// Look up the sentiment score for a Chinese word.
pub fn get_score_zh(word: &str) -> Option<i32> {
    SENTIMENT_ZH.get(word).copied()
}

/// Returns the number of words in the Chinese sentiment lexicon.
#[cfg(test)]
pub fn lexicon_size_zh() -> usize {
    SENTIMENT_ZH.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexicon_loads() {
        assert!(lexicon_size_zh() > 100);
    }

    #[test]
    fn positive_word() {
        assert!(get_score_zh("好").unwrap() > 0);
    }

    #[test]
    fn negative_word() {
        assert!(get_score_zh("壞").unwrap() < 0);
    }

    #[test]
    fn unknown_word() {
        assert_eq!(get_score_zh("asdfghjkl"), None);
    }
}
