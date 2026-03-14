use std::collections::HashSet;

use crate::error::{Result, WritingAnalysisError};
use crate::utils::{split_sentences, split_words};

/// Result of sentence variety analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SentenceVarietyResult {
    /// Average sentence length in words
    pub avg_length: f64,
    /// Variance of sentence lengths
    pub length_variance: f64,
    /// First word of each sentence (lowercased)
    pub starters: Vec<String>,
    /// Structure variety score (0.0-1.0, higher = more varied)
    pub structure_variety: f64,
}

/// Analyze sentence variety in text.
pub fn analyze_sentence_variety(text: &str) -> Result<SentenceVarietyResult> {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    // Sentence lengths in words
    let lengths: Vec<usize> = sentences.iter().map(|s| split_words(s).len()).collect();

    let total_words: usize = lengths.iter().sum();
    let avg_length = total_words as f64 / lengths.len() as f64;

    // Variance
    let variance = lengths
        .iter()
        .map(|&len| {
            let diff = len as f64 - avg_length;
            diff * diff
        })
        .sum::<f64>()
        / lengths.len() as f64;

    // Sentence starters
    let starters: Vec<String> = sentences
        .iter()
        .filter_map(|s| split_words(s).first().map(|w| w.to_lowercase()))
        .collect();

    // Structure variety: unique starters / total
    let unique_starters: HashSet<&str> = starters.iter().map(|s| s.as_str()).collect();
    let structure_variety = if starters.is_empty() {
        0.0
    } else {
        unique_starters.len() as f64 / starters.len() as f64
    };

    Ok(SentenceVarietyResult {
        avg_length,
        length_variance: variance,
        starters,
        structure_variety,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonous_writing() {
        let text = "The cat sat. The dog ran. The bird flew. The fish swam.";
        let result = analyze_sentence_variety(text).unwrap();
        assert!(result.structure_variety < 0.5);
    }

    #[test]
    fn varied_writing() {
        let text = "Rain fell heavily. Under the bridge, a cat sheltered. \
                    Nobody noticed. It was a dreary afternoon.";
        let result = analyze_sentence_variety(text).unwrap();
        assert!(result.structure_variety > 0.9);
    }

    #[test]
    fn average_length_calculation() {
        let text = "The cat sat. The dog ran.";
        let result = analyze_sentence_variety(text).unwrap();
        assert!((result.avg_length - 3.0).abs() < 0.5);
    }

    #[test]
    fn length_variance_same_length() {
        let text = "The cat sat. The dog ran.";
        let result = analyze_sentence_variety(text).unwrap();
        assert!(result.length_variance < 1.0);
    }

    #[test]
    fn starters_collected() {
        let text = "Hello world. Goodbye moon.";
        let result = analyze_sentence_variety(text).unwrap();
        assert_eq!(result.starters, vec!["hello", "goodbye"]);
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentence_variety("");
        assert!(result.is_err());
    }
}
