use crate::error::{Result, WritingAnalysisError};
use crate::sentiment::{SentimentResult, TokenSentiment};
use crate::zh::lexicon::get_score_zh;
use crate::zh::segmenter::segment;

/// Negation words that invert the sentiment of the next word.
static NEGATION_WORDS: &[&str] = &["不", "沒", "沒有", "別", "非", "未", "無", "无"];

/// Intensifier words that amplify the sentiment of the next word.
static INTENSIFIERS: &[&str] = &[
    "非常", "很", "特別", "極", "極其", "十分", "相當",
    "特别", "极", "极其", "相当",
];

/// Try to decompose a compound token that starts with a negation/intensifier prefix.
/// Returns (multiplier, base_word_score) if decomposition succeeds.
fn try_decompose_compound(word: &str) -> Option<(f64, i32)> {
    // Try negation prefixes (longest first to match "沒有" before "沒")
    let mut sorted_negations: Vec<&str> = NEGATION_WORDS.to_vec();
    sorted_negations.sort_by_key(|b| std::cmp::Reverse(b.len()));
    for neg in &sorted_negations {
        if let Some(rest) = word.strip_prefix(neg) {
            if !rest.is_empty() {
                if let Some(score) = get_score_zh(rest) {
                    return Some((-0.5, score));
                }
            }
        }
    }

    // Try intensifier prefixes
    let mut sorted_intensifiers: Vec<&str> = INTENSIFIERS.to_vec();
    sorted_intensifiers.sort_by_key(|b| std::cmp::Reverse(b.len()));
    for int in &sorted_intensifiers {
        if let Some(rest) = word.strip_prefix(int) {
            if !rest.is_empty() {
                if let Some(score) = get_score_zh(rest) {
                    return Some((1.5, score));
                }
            }
        }
    }

    None
}

/// Analyze sentiment of Chinese text with negation and intensifier handling.
pub fn analyze_sentiment_zh(text: &str) -> Result<SentimentResult> {
    let words = segment(text);
    if words.is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let mut tokens = Vec::new();
    let mut total_score: f64 = 0.0;

    // Track modifier state: (multiplier, tokens_since_modifier)
    let mut modifier: Option<(f64, usize)> = None;

    for word in &words {
        let base_score = get_score_zh(word).unwrap_or(0);

        let effective_score = if base_score != 0 {
            // Direct lexicon hit
            if let Some((multiplier, distance)) = modifier {
                modifier = None;
                if distance <= 2 {
                    (base_score as f64 * multiplier).round() as i32
                } else {
                    base_score
                }
            } else {
                base_score
            }
        } else if let Some((multiplier, compound_score)) = try_decompose_compound(word) {
            // Compound token like "不好" → negation(-0.5) × 好(3)
            modifier = None;
            (compound_score as f64 * multiplier).round() as i32
        } else {
            // No sentiment score — check if it's a modifier
            if NEGATION_WORDS.contains(&word.as_str()) {
                modifier = Some((-0.5, 0));
            } else if INTENSIFIERS.contains(&word.as_str()) {
                modifier = Some((1.5, 0));
            } else if let Some((m, d)) = modifier {
                modifier = Some((m, d + 1));
            }
            0
        };

        total_score += effective_score as f64;
        tokens.push(TokenSentiment {
            word: word.to_string(),
            score: effective_score,
        });
    }

    let token_count = tokens.len() as f64;
    let comparative = total_score / token_count;
    let score = (comparative * 2.0).clamp(-1.0, 1.0);

    Ok(SentimentResult {
        score,
        comparative,
        tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_sentiment() {
        let result = analyze_sentiment_zh("這部電影非常好看").unwrap();
        assert!(result.score > 0.0);
    }

    #[test]
    fn negative_sentiment() {
        let result = analyze_sentiment_zh("今天天氣不好").unwrap();
        assert!(result.score < 0.0);
    }

    #[test]
    fn negation_inverts_negative() {
        let result = analyze_sentiment_zh("這個不壞").unwrap();
        // 不 + 壞 should yield mildly positive
        assert!(result.score > 0.0 || result.score.abs() < 0.3);
    }

    #[test]
    fn intensifier_amplifies() {
        let normal = analyze_sentiment_zh("好").unwrap();
        let intensified = analyze_sentiment_zh("非常好").unwrap();
        assert!(intensified.score >= normal.score);
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentiment_zh("");
        assert!(result.is_err());
    }

    #[test]
    fn neutral_text() {
        let result = analyze_sentiment_zh("桌子在房間裡。").unwrap();
        assert!(result.score.abs() < 0.5);
    }

    #[test]
    fn compound_negation() {
        // jieba may produce "不好" as a single token
        let result = analyze_sentiment_zh("不好").unwrap();
        assert!(result.score < 0.0);
    }
}
