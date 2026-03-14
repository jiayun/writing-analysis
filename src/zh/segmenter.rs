use opencc_jieba_rs::OpenCC;
use std::sync::LazyLock;

static OPENCC: LazyLock<OpenCC> = LazyLock::new(OpenCC::new);

/// Segment Chinese text into words (supports both Traditional and Simplified).
pub fn segment(text: &str) -> Vec<String> {
    OPENCC
        .jieba_cut(text, false)
        .into_iter()
        .filter(|w| !w.trim().is_empty())
        .collect()
}

/// Segment Chinese text for search (finer granularity).
pub fn segment_for_search(text: &str) -> Vec<String> {
    // opencc-jieba-rs doesn't have a separate search mode,
    // so we use the standard cut which already handles both scripts.
    OPENCC
        .jieba_cut(text, true)
        .into_iter()
        .filter(|w| !w.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_simplified() {
        let words = segment("我来到北京清华大学");
        assert!(words.contains(&"北京".to_string()));
        assert!(words.contains(&"清华大学".to_string()) || words.contains(&"清华".to_string()));
    }

    #[test]
    fn segment_traditional() {
        let words = segment("其實這個問題很簡單");
        // Should segment into words, not individual characters
        assert!(words.contains(&"其實".to_string()) || words.contains(&"其实".to_string()));
        assert!(words.contains(&"問題".to_string()) || words.contains(&"问题".to_string()));
    }

    #[test]
    fn segment_filters_whitespace() {
        let words = segment("  你好  世界  ");
        assert!(words.iter().all(|w| !w.trim().is_empty()));
    }
}
