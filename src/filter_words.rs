use crate::error::{Result, WritingAnalysisError};
use crate::utils::{split_sentences, split_words};

/// Result of filter word detection.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterWordResult {
    /// All detected filter word instances
    pub instances: Vec<FilterWordInstance>,
    /// Total number of filter words found
    pub count: usize,
    /// Percentage of total words that are filter words (0.0-100.0)
    pub percentage: f64,
}

/// A single filter word occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterWordInstance {
    /// The matched filter word
    pub word: String,
    /// Byte offset in the original text
    pub offset: usize,
    /// The sentence containing the filter word
    pub sentence: String,
}

static FILTER_WORDS: &[&str] = &[
    "just",
    "really",
    "very",
    "quite",
    "rather",
    "somewhat",
    "somehow",
    "perhaps",
    "basically",
    "actually",
    "literally",
    "definitely",
    "certainly",
    "probably",
    "simply",
    "extremely",
    "absolutely",
    "totally",
    "completely",
    "utterly",
];

/// Detect filter words in text.
pub fn detect_filter_words(text: &str) -> Result<FilterWordResult> {
    let words = split_words(text);
    if words.is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let sentences = split_sentences(text);
    let text_start = text.as_ptr() as usize;
    let mut instances = Vec::new();

    for &word_ref in &words {
        let lower = word_ref.to_lowercase();
        if FILTER_WORDS.contains(&lower.as_str()) {
            let word_offset = word_ref.as_ptr() as usize - text_start;
            let sentence = sentences
                .iter()
                .find(|s| {
                    let s_start = s.as_ptr() as usize - text_start;
                    let s_end = s_start + s.len();
                    word_offset >= s_start && word_offset < s_end
                })
                .map(|s| s.to_string())
                .unwrap_or_default();

            instances.push(FilterWordInstance {
                word: word_ref.to_string(),
                offset: word_offset,
                sentence,
            });
        }
    }

    let count = instances.len();
    let percentage = (count as f64 / words.len() as f64) * 100.0;

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
    fn detect_filter_words_basic() {
        let result = detect_filter_words("She just really wanted to go.").unwrap();
        assert_eq!(result.count, 2);
    }

    #[test]
    fn no_filter_words() {
        let result = detect_filter_words("The cat sat on the mat.").unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.percentage, 0.0);
    }

    #[test]
    fn filter_word_percentage() {
        // "I very simply want this" — 5 words, 2 filter words = 40%
        let result = detect_filter_words("I very simply want this.").unwrap();
        assert_eq!(result.count, 2);
        assert!((result.percentage - 40.0).abs() < 1.0);
    }

    #[test]
    fn case_insensitive_detection() {
        let result = detect_filter_words("JUST do it. Really.").unwrap();
        assert_eq!(result.count, 2);
    }

    #[test]
    fn empty_text_error() {
        let result = detect_filter_words("");
        assert!(result.is_err());
    }
}
