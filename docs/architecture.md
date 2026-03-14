# Architecture & Module Design

## Overview

writing-analysis is a lightweight writing analysis and NLP tools crate for Rust. It provides rule-based text analysis including readability scoring, passive voice detection, cliché detection, filter word detection, sentiment analysis, and sentence variety analysis. All algorithms are pure rule-based — no ML models required.

## Data Flow

```
&str  ──▶  Text Utilities  ──▶  Sentences / Words  ──▶  Analyzers  ──▶  Results
           (tokenization)        (syllables, counts)      │   │   │
                                                          │   │   │
                                      ┌───────────────────┘   │   └──────────────────┐
                                      ▼                       ▼                      ▼
                               ReadabilityScores    PassiveVoiceResult    SentimentResult
                                                    ClicheResult          SentenceVarietyResult
                                                    FilterWordResult
                                                          │
                                                          ▼
                                                   AnalysisResult
                                                   (aggregated)
```

## Module Structure

```
src/
├── lib.rs              # Public API, re-exports, analyze_all()
├── error.rs            # Error types (thiserror)
├── utils.rs            # Text utilities: sentence splitting, syllable counting, word tokenization
├── readability.rs      # 5 readability formulas
├── passive_voice.rs    # be-verb + past participle pattern matching
├── cliche.rs           # Built-in cliché list + exact matching
├── filter_words.rs     # Common filter word detection
├── sentiment.rs        # AFINN-based lexicon sentiment scoring
├── sentence_variety.rs # Sentence length, starters, structure diversity
└── lexicon.rs          # AFINN lexicon data and loading

tests/
├── integration_tests.rs       # End-to-end tests for analyze_all()
└── fixtures/
    ├── simple_text.txt
    ├── passive_heavy.txt
    └── literary_sample.txt

# Unit tests are in-module via #[cfg(test)] in each src/*.rs file

benches/
└── analysis_benchmark.rs
```

### Module Responsibilities

| Module | Visibility | Responsibility |
|--------|-----------|----------------|
| `lib.rs` | public | Entry points (`analyze_readability`, `analyze_all`, etc.), re-exports |
| `error.rs` | public | `WritingAnalysisError` enum, `Result<T>` type alias |
| `utils.rs` | internal | Sentence splitting, word tokenization, syllable counting, word counting |
| `readability.rs` | public | 5 readability formulas (Flesch-Kincaid, Flesch RE, SMOG, Coleman-Liau, ARI) |
| `passive_voice.rs` | public | be-verb + past participle detection with position tracking |
| `cliche.rs` | public | Built-in cliché list, exact matching (case-insensitive) |
| `filter_words.rs` | public | Filter word detection (just, really, very, etc.) |
| `sentiment.rs` | public | AFINN lexicon scoring, token-level sentiment |
| `sentence_variety.rs` | public | Sentence length stats, starter analysis, structure diversity |
| `lexicon.rs` | internal | AFINN lexicon data (embedded), loading and lookup |

## Key Design Decisions

### 1. Pure Rule-based Analysis (no ML)

All analysis functions use deterministic, rule-based algorithms. No machine learning models are loaded or trained. This keeps the crate lightweight, fast, and dependency-minimal.

Sentiment analysis uses an AFINN word list (scored -5 to +5). Passive voice detection uses pattern matching on be-verb + past participle combinations. Readability uses established mathematical formulas.

### 2. AFINN Lexicon for Sentiment

The AFINN-111 lexicon is embedded directly in the binary via `include_str!` or a compile-time constant. This avoids runtime file I/O and makes the crate self-contained. The lexicon contains ~2,477 English words with integer sentiment scores from -5 (most negative) to +5 (most positive).

### 3. Pattern Matching for Passive Voice

Passive voice is detected by matching `be-verb + past participle` patterns, where:
- Be-verbs: am, is, are, was, were, be, been, being
- Past participles: words ending in `-ed` plus a curated list of irregular past participles (written, broken, taken, etc.)

This approach is not 100% accurate (no POS tagger), but is fast and sufficient for writing feedback.

### 4. Embedded Word Lists

Cliché lists and filter word lists are compiled into the binary as static data. This avoids any runtime I/O or configuration files. Lists can be extended via a builder pattern or custom list parameter.

### 5. `LazyLock` for Static Initialization

Static regex patterns and word lists use `std::sync::LazyLock` (stable since Rust 1.80) instead of `lazy_static!`. This eliminates the `lazy_static` dependency entirely.

### 6. Stateless Functions

All analysis functions take `&str` input and return owned result structs. No mutable state, no context objects. This makes the API simple and thread-safe by default.

## Public API

```rust
use writing_analysis::*;

// Individual analyzers
pub fn analyze_readability(text: &str) -> Result<ReadabilityScores>;
pub fn detect_passive_voice(text: &str) -> Result<PassiveVoiceResult>;
pub fn detect_cliches(text: &str) -> Result<ClicheResult>;
pub fn detect_filter_words(text: &str) -> Result<FilterWordResult>;
pub fn analyze_sentiment(text: &str) -> Result<SentimentResult>;
pub fn analyze_sentence_variety(text: &str) -> Result<SentenceVarietyResult>;

// Aggregated analysis
pub fn analyze_all(text: &str) -> Result<AnalysisResult>;
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `regex` | 1.11 | Pattern matching for passive voice, clichés |
| `unicode-segmentation` | 1.12 | Unicode-aware word boundary detection |
| `thiserror` | 2.0 | Error type derivation |
| `pretty_assertions` | 1.4 | (dev) Readable test diffs |
| `criterion` | 0.5 | (dev) Benchmarking |
