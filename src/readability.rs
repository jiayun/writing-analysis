use crate::error::{Result, WritingAnalysisError};
use crate::utils::{compute_statistics, TextStatistics};

/// Readability scores computed from text statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadabilityScores {
    /// Flesch-Kincaid Grade Level (US school grade)
    pub flesch_kincaid_grade: f64,
    /// Flesch Reading Ease (0-100, higher = easier)
    pub flesch_reading_ease: f64,
    /// SMOG Index (years of education needed).
    /// Note: SMOG is designed for texts with 30+ sentences. Results for shorter
    /// texts are approximations and may be less accurate.
    pub smog_index: f64,
    /// Coleman-Liau Index (grade level)
    pub coleman_liau_index: f64,
    /// Automated Readability Index (grade level)
    pub automated_readability_index: f64,
}

fn flesch_kincaid_grade(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let syllables = stats.syllable_count as f64;
    0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
}

fn flesch_reading_ease(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let syllables = stats.syllable_count as f64;
    206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
}

fn smog_index(stats: &TextStatistics) -> f64 {
    let polysyllables = stats.polysyllable_count as f64;
    let sentences = stats.sentence_count as f64;
    3.0 + (polysyllables * 30.0 / sentences).sqrt()
}

fn coleman_liau_index(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let l = (stats.character_count as f64 / words) * 100.0;
    let s = (stats.sentence_count as f64 / words) * 100.0;
    0.0588 * l - 0.296 * s - 15.8
}

fn automated_readability_index(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let characters = stats.character_count as f64;
    4.71 * (characters / words) + 0.5 * (words / sentences) - 21.43
}

/// Analyze text readability using 5 standard formulas.
pub fn analyze_readability(text: &str) -> Result<ReadabilityScores> {
    let stats = compute_statistics(text);

    if stats.word_count == 0 {
        return Err(WritingAnalysisError::EmptyText);
    }
    if stats.word_count < 2 {
        return Err(WritingAnalysisError::TextTooShort {
            min_words: 2,
            found: stats.word_count,
        });
    }
    if stats.sentence_count == 0 {
        return Err(WritingAnalysisError::NoSentences);
    }

    Ok(ReadabilityScores {
        flesch_kincaid_grade: flesch_kincaid_grade(&stats),
        flesch_reading_ease: flesch_reading_ease(&stats),
        smog_index: smog_index(&stats),
        coleman_liau_index: coleman_liau_index(&stats),
        automated_readability_index: automated_readability_index(&stats),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readability_simple_text() {
        let scores = analyze_readability("The cat sat on the mat.").unwrap();
        assert!(scores.flesch_kincaid_grade < 5.0);
        assert!(scores.flesch_reading_ease > 80.0);
    }

    #[test]
    fn readability_complex_text() {
        let text = "The implementation of sophisticated algorithms \
                    necessitates comprehensive understanding of computational \
                    complexity and mathematical abstractions.";
        let scores = analyze_readability(text).unwrap();
        assert!(scores.flesch_kincaid_grade > 12.0);
        assert!(scores.flesch_reading_ease < 30.0);
    }

    #[test]
    fn readability_empty_text() {
        let result = analyze_readability("");
        assert!(result.is_err());
    }

    #[test]
    fn readability_all_scores_finite() {
        let scores = analyze_readability("Hello world. This is a test.").unwrap();
        assert!(scores.flesch_kincaid_grade.is_finite());
        assert!(scores.flesch_reading_ease.is_finite());
        assert!(scores.smog_index.is_finite());
        assert!(scores.coleman_liau_index.is_finite());
        assert!(scores.automated_readability_index.is_finite());
    }
}
