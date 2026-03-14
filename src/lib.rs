//! Lightweight writing analysis and NLP tools for Rust.
//!
//! Provides rule-based text analysis including readability scoring,
//! passive voice detection, cliché detection, filter word detection,
//! sentiment analysis, and sentence variety analysis.

pub mod error;
mod lexicon;
mod utils;

mod cliche;
mod filter_words;
mod passive_voice;
mod readability;
mod sentence_variety;
mod sentiment;

#[cfg(feature = "chinese")]
pub mod zh;

pub use cliche::{detect_cliches, ClicheInstance, ClicheResult};
pub use error::{Result, WritingAnalysisError};
pub use filter_words::{detect_filter_words, FilterWordInstance, FilterWordResult};
pub use passive_voice::{detect_passive_voice, PassiveInstance, PassiveVoiceResult};
pub use readability::{analyze_readability, ReadabilityScores};
pub use sentence_variety::{analyze_sentence_variety, SentenceVarietyResult};
pub use sentiment::{analyze_sentiment, SentimentResult, TokenSentiment};
pub use utils::TextStatistics;

#[cfg(feature = "chinese")]
pub use zh::{
    analyze_all_zh, analyze_readability_zh, analyze_sentence_variety_zh, analyze_sentiment_zh,
    detect_cliches_zh, detect_filter_words_zh, detect_passive_voice_zh, AnalysisResultZh,
    ChineseReadabilityScores,
};

/// Aggregated result of all analysis functions.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisResult {
    pub readability: ReadabilityScores,
    pub passive_voice: PassiveVoiceResult,
    pub cliches: ClicheResult,
    pub filter_words: FilterWordResult,
    pub sentiment: SentimentResult,
    pub sentence_variety: SentenceVarietyResult,
}

/// Run all analysis functions on the given text.
pub fn analyze_all(text: &str) -> Result<AnalysisResult> {
    Ok(AnalysisResult {
        readability: analyze_readability(text)?,
        passive_voice: detect_passive_voice(text)?,
        cliches: detect_cliches(text)?,
        filter_words: detect_filter_words(text)?,
        sentiment: analyze_sentiment(text)?,
        sentence_variety: analyze_sentence_variety(text)?,
    })
}
