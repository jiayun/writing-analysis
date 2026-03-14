use crate::error::{Result, WritingAnalysisError};
use crate::filter_words::{FilterWordInstance, FilterWordResult};
use crate::zh::segmenter::segment;
use crate::zh::utils::split_sentences_zh;

/// Chinese filter/filler words (贅詞).
static FILTER_WORDS_ZH: &[&str] = &[
    "其實",
    "基本上",
    "就是",
    "然後",
    "所以",
    "的話",
    "應該",
    "可能",
    "大概",
    "反正",
    "就是說",
    "差不多",
    "總之",
    "老實說",
    "簡單來說",
    "坦白說",
    "說實話",
    "一般來說",
    // Simplified equivalents
    "其实",
    "简单来说",
    "说实话",
    "一般来说",
    "老实说",
    "坦白说",
    "总之",
    "应该",
];

/// Detect filter words in Chinese text using substring matching.
///
/// Uses substring matching (like cliché detection) rather than jieba tokenization,
/// because jieba's dictionary is primarily Simplified Chinese and may not correctly
/// segment Traditional Chinese filter words.
pub fn detect_filter_words_zh(text: &str) -> Result<FilterWordResult> {
    if text.trim().is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let sentences = split_sentences_zh(text);
    let word_count = segment(text).len();
    let mut instances = Vec::new();

    for &filter_word in FILTER_WORDS_ZH {
        let mut start = 0;
        while let Some(pos) = text[start..].find(filter_word) {
            let offset = start + pos;
            let sentence = sentences
                .iter()
                .find(|s| {
                    let s_start = s.as_ptr() as usize - text.as_ptr() as usize;
                    let s_end = s_start + s.len();
                    offset >= s_start && offset < s_end
                })
                .map(|s| s.to_string())
                .unwrap_or_default();

            instances.push(FilterWordInstance {
                word: filter_word.to_string(),
                offset,
                sentence,
            });
            start = offset + filter_word.len();
        }
    }

    instances.sort_by_key(|i| i.offset);
    instances.dedup_by_key(|i| i.offset);

    let count = instances.len();
    let percentage = if word_count > 0 {
        (count as f64 / word_count as f64) * 100.0
    } else {
        0.0
    };

    Ok(FilterWordResult {
        instances,
        count,
        percentage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_basic_filter_words() {
        let result = detect_filter_words_zh("其實這個問題基本上不難。").unwrap();
        assert!(result.count >= 2);
    }

    #[test]
    fn no_filter_words() {
        let result = detect_filter_words_zh("量子計算機的運算速度極快。").unwrap();
        assert_eq!(result.count, 0);
    }

    #[test]
    fn filter_word_percentage() {
        let result = detect_filter_words_zh("其實這個問題基本上不難。").unwrap();
        assert!(result.percentage > 0.0);
    }

    #[test]
    fn empty_text_error() {
        let result = detect_filter_words_zh("");
        assert!(result.is_err());
    }

    #[test]
    fn detect_simplified_filter_words() {
        let result = detect_filter_words_zh("其实这个问题不难。").unwrap();
        assert!(result.count >= 1);
    }
}
