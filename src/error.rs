use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritingAnalysisError {
    #[error("Text is empty")]
    EmptyText,

    #[error("Text too short for analysis: need at least {min_words} words, found {found}")]
    TextTooShort { min_words: usize, found: usize },

    #[error("No sentences detected in text")]
    NoSentences,

    #[error("Lexicon error: {0}")]
    LexiconError(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, WritingAnalysisError>;
