use crate::error::{Result, WritingAnalysisError};
use crate::lexicon::get_score;
use crate::utils::split_words;

/// Result of sentiment analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SentimentResult {
    /// Overall sentiment score (-1.0 to 1.0)
    pub score: f64,
    /// Comparative score: total sentiment / token count
    pub comparative: f64,
    /// Per-token sentiment breakdown
    pub tokens: Vec<TokenSentiment>,
}

/// Sentiment data for a single token.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSentiment {
    /// The word/token
    pub word: String,
    /// AFINN score (-5 to +5), 0 if not in lexicon
    pub score: i32,
}

/// Analyze sentiment of text using AFINN lexicon.
pub fn analyze_sentiment(text: &str) -> Result<SentimentResult> {
    let words = split_words(text);
    if words.is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let mut tokens = Vec::new();
    let mut total_score: i32 = 0;

    for word in &words {
        let lower = word.to_lowercase();
        let score = get_score(&lower).unwrap_or(0);
        total_score += score;
        tokens.push(TokenSentiment {
            word: word.to_string(),
            score,
        });
    }

    let token_count = tokens.len() as f64;
    let comparative = total_score as f64 / token_count;
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
        let result = analyze_sentiment("I love this wonderful movie.").unwrap();
        assert!(result.score > 0.0);
        assert!(result.comparative > 0.0);
    }

    #[test]
    fn negative_sentiment() {
        let result = analyze_sentiment("This is a terrible awful disaster.").unwrap();
        assert!(result.score < 0.0);
        assert!(result.comparative < 0.0);
    }

    #[test]
    fn neutral_sentiment() {
        let result = analyze_sentiment("The table is in the room.").unwrap();
        assert!(result.score.abs() < 0.3);
    }

    #[test]
    fn token_level_scores() {
        let result = analyze_sentiment("I love hate things.").unwrap();
        let love_token = result.tokens.iter().find(|t| t.word == "love").unwrap();
        let hate_token = result.tokens.iter().find(|t| t.word == "hate").unwrap();
        assert!(love_token.score > 0);
        assert!(hate_token.score < 0);
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentiment("");
        assert!(result.is_err());
    }

    #[test]
    fn comparative_normalization() {
        let short = analyze_sentiment("I love this.").unwrap();
        let long = analyze_sentiment("I love this thing that is here today.").unwrap();
        assert!(short.comparative.abs() > long.comparative.abs());
    }
}
