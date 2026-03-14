use std::collections::HashSet;
use std::sync::LazyLock;

use crate::error::{Result, WritingAnalysisError};
use crate::zh::segmenter::segment;
use crate::zh::utils::{count_hanzi, is_hanzi, split_sentences_zh};

static COMMON_CHARS_DATA: &str = include_str!("../../data/zh/common_chars_3000.txt");

static COMMON_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    COMMON_CHARS_DATA.chars().filter(|c| is_hanzi(*c)).collect()
});

/// Chinese readability scores based on the Simple Chinese Readability Index.
#[derive(Debug, Clone, PartialEq)]
pub struct ChineseReadabilityScores {
    /// Simple Chinese Readability Index (higher = harder to read)
    pub scri: f64,
    /// Average sentence length in words (after segmentation)
    pub avg_sentence_length: f64,
    /// Average word length in characters
    pub avg_word_length: f64,
    /// Percentage of characters in top-3000 most common (0.0-100.0)
    pub common_char_ratio: f64,
    /// Total character count (漢字 only)
    pub character_count: usize,
    /// Total word count (after segmentation)
    pub word_count: usize,
    /// Total sentence count
    pub sentence_count: usize,
}

/// Analyze Chinese text readability.
pub fn analyze_readability_zh(text: &str) -> Result<ChineseReadabilityScores> {
    let sentences = split_sentences_zh(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    let character_count = count_hanzi(text);
    if character_count == 0 {
        return Err(WritingAnalysisError::EmptyText);
    }

    let words = segment(text);
    let word_count = words.len();
    if word_count == 0 {
        return Err(WritingAnalysisError::EmptyText);
    }

    let sentence_count = sentences.len();
    let avg_sentence_length = word_count as f64 / sentence_count as f64;
    let avg_word_length = character_count as f64 / word_count as f64;

    let scri = 0.5 * avg_sentence_length + 0.5 * avg_word_length - 3.0;

    // Common character ratio
    let hanzi_chars: Vec<char> = text.chars().filter(|c| is_hanzi(*c)).collect();
    let common_count = hanzi_chars.iter().filter(|c| COMMON_CHARS.contains(c)).count();
    let common_char_ratio = if hanzi_chars.is_empty() {
        0.0
    } else {
        (common_count as f64 / hanzi_chars.len() as f64) * 100.0
    };

    Ok(ChineseReadabilityScores {
        scri,
        avg_sentence_length,
        avg_word_length,
        common_char_ratio,
        character_count,
        word_count,
        sentence_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readability_simple_text() {
        let scores = analyze_readability_zh("今天天氣很好。我們去公園玩。").unwrap();
        assert!(scores.scri.is_finite());
        assert!(scores.sentence_count == 2);
        assert!(scores.character_count > 0);
    }

    #[test]
    fn readability_common_char_ratio() {
        let scores = analyze_readability_zh("我是一個學生。他是老師。").unwrap();
        // Common characters should have a high ratio
        assert!(scores.common_char_ratio > 50.0);
    }

    #[test]
    fn readability_empty_error() {
        let result = analyze_readability_zh("");
        assert!(result.is_err());
    }

    #[test]
    fn readability_english_only_error() {
        let result = analyze_readability_zh("Hello world.");
        assert!(result.is_err());
    }

    #[test]
    fn readability_complex_text_higher_scri() {
        let simple = analyze_readability_zh("我好。你好。").unwrap();
        let complex = analyze_readability_zh(
            "在當今全球化的時代背景下，國際間的經濟合作與文化交流日益頻繁。\
             各國政府積極推動多邊貿易協定的簽署與實施。",
        )
        .unwrap();
        assert!(complex.scri > simple.scri);
    }
}
