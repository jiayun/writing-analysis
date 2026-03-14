# Complete Type Definitions

## Readability Types (`readability.rs`)

### ReadabilityScores

```rust
/// Readability scores computed from text statistics.
/// Each formula produces a grade level or ease score.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadabilityScores {
    /// Flesch-Kincaid Grade Level (US school grade, e.g., 8.0 = 8th grade)
    pub flesch_kincaid_grade: f64,

    /// Flesch Reading Ease (0-100, higher = easier to read)
    /// 90-100: very easy, 60-70: standard, 0-30: very difficult
    pub flesch_reading_ease: f64,

    /// SMOG Index (years of education needed to understand).
    /// Note: SMOG is designed for texts with 30+ sentences. Results for shorter
    /// texts are approximations and may be less accurate.
    pub smog_index: f64,

    /// Coleman-Liau Index (grade level, based on characters not syllables)
    pub coleman_liau_index: f64,

    /// Automated Readability Index (grade level, based on characters and words)
    pub automated_readability_index: f64,
}
```

### TextStatistics

```rust
/// Raw text statistics used by readability formulas.
/// Exposed for debugging and advanced use.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStatistics {
    /// Total number of sentences
    pub sentence_count: usize,

    /// Total number of words
    pub word_count: usize,

    /// Total number of syllables
    pub syllable_count: usize,

    /// Total number of characters (letters only, no spaces/punctuation)
    pub character_count: usize,

    /// Number of polysyllabic words (3+ syllables)
    pub polysyllable_count: usize,
}
```

## Passive Voice Types (`passive_voice.rs`)

### PassiveVoiceResult

```rust
/// Result of passive voice detection.
#[derive(Debug, Clone, PartialEq)]
pub struct PassiveVoiceResult {
    /// All detected passive voice instances
    pub instances: Vec<PassiveInstance>,

    /// Percentage of sentences containing passive voice (0.0-100.0)
    pub percentage: f64,
}
```

### PassiveInstance

```rust
/// A single passive voice occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct PassiveInstance {
    /// The matched passive phrase (e.g., "was written")
    pub phrase: String,

    /// Byte offset of the phrase start in the original text
    pub offset: usize,

    /// The full sentence containing the passive voice
    pub sentence: String,
}
```

## Cliché Types (`cliche.rs`)

### ClicheResult

```rust
/// Result of cliché detection.
#[derive(Debug, Clone, PartialEq)]
pub struct ClicheResult {
    /// All detected cliché instances
    pub instances: Vec<ClicheInstance>,

    /// Total number of clichés found
    pub count: usize,
}
```

### ClicheInstance

```rust
/// A single cliché occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct ClicheInstance {
    /// The matched cliché phrase (e.g., "at the end of the day")
    pub phrase: String,

    /// Byte offset of the phrase start in the original text
    pub offset: usize,

    /// The canonical form of the cliché (from the built-in list)
    pub canonical: String,
}
```

## Filter Word Types (`filter_words.rs`)

### FilterWordResult

```rust
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
```

### FilterWordInstance

```rust
/// A single filter word occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterWordInstance {
    /// The matched filter word (e.g., "just", "really")
    pub word: String,

    /// Byte offset of the word in the original text
    pub offset: usize,

    /// The sentence containing the filter word
    pub sentence: String,
}
```

## Sentiment Types (`sentiment.rs`)

### SentimentResult

```rust
/// Result of sentiment analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SentimentResult {
    /// Overall sentiment score, normalized to -1.0 (most negative) to 1.0 (most positive)
    pub score: f64,

    /// Comparative score: total sentiment / number of tokens
    /// Useful for comparing texts of different lengths
    pub comparative: f64,

    /// Per-token sentiment breakdown
    pub tokens: Vec<TokenSentiment>,
}
```

### TokenSentiment

```rust
/// Sentiment data for a single token.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSentiment {
    /// The word/token
    pub word: String,

    /// AFINN score for this word (-5 to +5), 0 if not in lexicon
    pub score: i32,
}
```

## Sentence Variety Types (`sentence_variety.rs`)

### SentenceVarietyResult

```rust
/// Result of sentence variety analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct SentenceVarietyResult {
    /// Average sentence length in words
    pub avg_length: f64,

    /// Variance of sentence lengths (higher = more varied)
    pub length_variance: f64,

    /// First word of each sentence (for detecting repetitive starters)
    pub starters: Vec<String>,

    /// Structure variety score (0.0-1.0, higher = more varied)
    /// Based on unique starters / total sentences ratio
    pub structure_variety: f64,
}
```

## Aggregated Analysis (`lib.rs`)

### AnalysisResult

```rust
/// Aggregated result of all analysis functions.
/// Returned by `analyze_all()`.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisResult {
    /// Readability scores
    pub readability: ReadabilityScores,

    /// Passive voice detection results
    pub passive_voice: PassiveVoiceResult,

    /// Cliché detection results
    pub cliches: ClicheResult,

    /// Filter word detection results
    pub filter_words: FilterWordResult,

    /// Sentiment analysis results
    pub sentiment: SentimentResult,

    /// Sentence variety analysis results
    pub sentence_variety: SentenceVarietyResult,
}
```

## Error Types (`error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WritingAnalysisError {
    #[error("Text is empty")]
    EmptyText,

    #[error("Text too short for analysis: need at least {min_words} words, found {found}")]
    TextTooShort {
        min_words: usize,
        found: usize,
    },

    #[error("No sentences detected in text")]
    NoSentences,

    #[error("Lexicon error: {0}")]
    LexiconError(String),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, WritingAnalysisError>;
```

### Error Notes

- `EmptyText` is returned when the input `&str` is empty or contains only whitespace
- `TextTooShort` is returned by analyzers that need a minimum word count (e.g., SMOG requires 30+ sentences)
- `NoSentences` is returned when the sentence splitter finds no sentence boundaries
- `LexiconError` covers issues with the AFINN lexicon data (should not occur with embedded data)
- Individual analyzers may return specific errors; `analyze_all` propagates the first error encountered

## Key Design Choices

1. **`f64` for all scores** — Readability formulas and sentiment scores use `f64` for precision. No rounding is applied; consumers can format as needed.

2. **Byte offsets** — All `offset` fields use byte offsets (not character offsets) to match Rust's `&str` indexing behavior. Use `text[offset..]` directly.

3. **Owned strings in results** — All result structs own their string data (`String`, not `&str`). This avoids lifetime complexity and allows results to outlive the input text.

4. **`PartialEq` on all types** — All result types derive `PartialEq` for easy testing with `assert_eq!`.
