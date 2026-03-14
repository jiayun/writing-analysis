use std::collections::HashSet;

use crate::error::{Result, WritingAnalysisError};
use crate::sentence_variety::SentenceVarietyResult;
use crate::zh::segmenter::segment;
use crate::zh::utils::split_sentences_zh;

/// Analyze sentence variety in Chinese text.
pub fn analyze_sentence_variety_zh(text: &str) -> Result<SentenceVarietyResult> {
    let sentences = split_sentences_zh(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    // Sentence lengths in segmented words
    let lengths: Vec<usize> = sentences.iter().map(|s| segment(s).len()).collect();

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

    // Sentence starters (first segmented word)
    let starters: Vec<String> = sentences
        .iter()
        .filter_map(|s| segment(s).into_iter().next())
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
        let text = "他去學校。他去公園。他去圖書館。他去醫院。";
        let result = analyze_sentence_variety_zh(text).unwrap();
        assert!(result.structure_variety < 0.5);
    }

    #[test]
    fn varied_writing() {
        let text = "雨下得很大。橋下躲著一隻貓。沒有人注意到。這是個陰沉的下午。";
        let result = analyze_sentence_variety_zh(text).unwrap();
        assert!(result.structure_variety > 0.5);
    }

    #[test]
    fn average_length() {
        let text = "你好。世界。";
        let result = analyze_sentence_variety_zh(text).unwrap();
        assert!(result.avg_length > 0.0);
    }

    #[test]
    fn starters_collected() {
        let text = "今天天氣好。明天會下雨。";
        let result = analyze_sentence_variety_zh(text).unwrap();
        assert_eq!(result.starters.len(), 2);
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentence_variety_zh("");
        assert!(result.is_err());
    }
}
