//! Chinese language analysis module (中文分析).
//!
//! Provides Chinese text analysis including readability scoring,
//! passive voice detection, cliché detection, filter word detection,
//! sentiment analysis, and sentence variety analysis.
//!
//! This module is feature-gated behind the `chinese` feature flag.

mod cliche;
mod filter_words;
mod lexicon;
mod passive_voice;
mod readability;
pub(crate) mod segmenter;
mod sentence_variety;
mod sentiment;
pub(crate) mod utils;

pub use cliche::detect_cliches_zh;
pub use filter_words::detect_filter_words_zh;
pub use passive_voice::detect_passive_voice_zh;
pub use readability::{analyze_readability_zh, ChineseReadabilityScores};
pub use segmenter::{segment, segment_for_search};
pub use sentence_variety::analyze_sentence_variety_zh;
pub use sentiment::analyze_sentiment_zh;
pub use utils::{count_hanzi, split_sentences_zh};

use crate::cliche::ClicheResult;
use crate::error::Result;
use crate::filter_words::FilterWordResult;
use crate::passive_voice::PassiveVoiceResult;
use crate::sentence_variety::SentenceVarietyResult;
use crate::sentiment::SentimentResult;

/// Aggregated result of all Chinese analysis functions.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisResultZh {
    pub readability: ChineseReadabilityScores,
    pub passive_voice: PassiveVoiceResult,
    pub cliches: ClicheResult,
    pub filter_words: FilterWordResult,
    pub sentiment: SentimentResult,
    pub sentence_variety: SentenceVarietyResult,
}

/// Run all Chinese analysis functions on the given text.
pub fn analyze_all_zh(text: &str) -> Result<AnalysisResultZh> {
    Ok(AnalysisResultZh {
        readability: analyze_readability_zh(text)?,
        passive_voice: detect_passive_voice_zh(text)?,
        cliches: detect_cliches_zh(text)?,
        filter_words: detect_filter_words_zh(text)?,
        sentiment: analyze_sentiment_zh(text)?,
        sentence_variety: analyze_sentence_variety_zh(text)?,
    })
}
