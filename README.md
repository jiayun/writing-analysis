# writing-analysis

[![Crates.io](https://img.shields.io/crates/v/writing-analysis.svg)](https://crates.io/crates/writing-analysis)
[![docs.rs](https://img.shields.io/docsrs/writing-analysis)](https://docs.rs/writing-analysis)
[![CI](https://github.com/jiayun/writing-analysis/actions/workflows/ci.yml/badge.svg)](https://github.com/jiayun/writing-analysis/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Lightweight writing analysis and NLP tools for Rust. Pure rule-based — no ML models required.

## Features

### English (default)

- **Readability scoring** — Flesch-Kincaid, Flesch Reading Ease, SMOG, Coleman-Liau, ARI
- **Passive voice detection** — be-verb + past participle pattern matching with 90+ irregular verbs
- **Cliche detection** — 85 built-in English cliches, case-insensitive matching
- **Filter word detection** — spot filler words (just, really, very, basically, ...)
- **Sentiment analysis** — AFINN-111 lexicon (2400+ words), token-level scoring
- **Sentence variety** — length stats, starter diversity, structure variety score

### Chinese (feature flag: `chinese`)

- **Chinese readability** — SCRI (Simple Chinese Readability Index), common character ratio (top 3000)
- **Passive voice detection** — 被/受/遭/給 + 為...所 patterns with exclusion list
- **Cliche detection** — ~100 entries: overused idioms, bureaucratic phrases, writing cliches (Traditional + Simplified)
- **Filter word detection** — ~26 common filler words (其實、基本上、就是、然後...)
- **Sentiment analysis** — 500+ word lexicon with negation/intensifier handling (不/沒/非常/很)
- **Sentence variety** — segmentation-based analysis
- **Word segmentation** — opencc-jieba-rs, supports both Traditional and Simplified Chinese

### General

- **Zero unsafe code** — memory safe by design
- **No runtime I/O** — all data embedded at compile time

## Quick Start

```sh
cargo add writing-analysis

# With Chinese support
cargo add writing-analysis --features chinese
```

```rust
use writing_analysis::{analyze_all, analyze_readability, detect_passive_voice};

fn main() -> writing_analysis::Result<()> {
    let text = "The ball was thrown by the boy. \
                She ran quickly through the park. \
                It was a beautiful day.";

    // Run all analyses at once
    let result = analyze_all(text)?;

    println!("Flesch Reading Ease: {:.1}", result.readability.flesch_reading_ease);
    println!("Passive voice: {:.0}%", result.passive_voice.percentage);
    println!("Cliches found: {}", result.cliches.count);
    println!("Filter words: {}", result.filter_words.count);
    println!("Sentiment: {:.2}", result.sentiment.score);
    println!("Sentence variety: {:.2}", result.sentence_variety.structure_variety);

    Ok(())
}
```

## Usage

### Readability

```rust
let scores = writing_analysis::analyze_readability(
    "The implementation of sophisticated algorithms necessitates \
     comprehensive understanding of computational complexity."
)?;

assert!(scores.flesch_kincaid_grade > 12.0);  // college level
assert!(scores.flesch_reading_ease < 30.0);   // very difficult
```

### Passive Voice

```rust
let result = writing_analysis::detect_passive_voice(
    "The report was written by the team. She presented the findings."
)?;

assert_eq!(result.instances.len(), 1);
assert_eq!(result.instances[0].phrase, "was written");
assert_eq!(result.percentage, 50.0);  // 1 of 2 sentences
```

### Cliches

```rust
let result = writing_analysis::detect_cliches(
    "At the end of the day, we need to think outside the box."
)?;

assert_eq!(result.count, 2);
for cliche in &result.instances {
    println!("'{}' at offset {}", cliche.canonical, cliche.offset);
}
```

### Filter Words

```rust
let result = writing_analysis::detect_filter_words(
    "She just really wanted to basically understand."
)?;

assert_eq!(result.count, 3);  // just, really, basically
println!("{:.1}% filter words", result.percentage);
```

### Sentiment

```rust
let result = writing_analysis::analyze_sentiment(
    "I love this wonderful movie."
)?;

assert!(result.score > 0.0);  // positive
for token in &result.tokens {
    if token.score != 0 {
        println!("{}: {}", token.word, token.score);
    }
}
```

### Sentence Variety

```rust
let result = writing_analysis::analyze_sentence_variety(
    "The cat sat. The dog ran. The bird flew. A fish swam."
)?;

println!("Avg length: {:.1} words", result.avg_length);
println!("Variety: {:.2}", result.structure_variety);  // 0.5 — repetitive starters
```

### Chinese Analysis

```rust
use writing_analysis::{analyze_all_zh, analyze_readability_zh, detect_passive_voice_zh};

fn main() -> writing_analysis::Result<()> {
    let text = "今天天氣很好。我們去公園散步。孩子們在草地上玩耍。";

    let result = analyze_all_zh(text)?;

    println!("SCRI: {:.1}", result.readability.scri);
    println!("Common char ratio: {:.1}%", result.readability.common_char_ratio);
    println!("Passive voice: {:.0}%", result.passive_voice.percentage);
    println!("Cliches: {}", result.cliches.count);
    println!("Filter words: {}", result.filter_words.count);
    println!("Sentiment: {:.2}", result.sentiment.score);

    Ok(())
}
```

## API Overview

### English Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `analyze_all` | `AnalysisResult` | Run all 6 analyzers |
| `analyze_readability` | `ReadabilityScores` | 5 readability formulas |
| `detect_passive_voice` | `PassiveVoiceResult` | Passive voice instances + percentage |
| `detect_cliches` | `ClicheResult` | Cliche instances + count |
| `detect_filter_words` | `FilterWordResult` | Filter word instances + percentage |
| `analyze_sentiment` | `SentimentResult` | Score + per-token breakdown |
| `analyze_sentence_variety` | `SentenceVarietyResult` | Length stats + starter diversity |

### Chinese Functions (feature: `chinese`)

| Function | Returns | Description |
|----------|---------|-------------|
| `analyze_all_zh` | `AnalysisResultZh` | Run all 6 Chinese analyzers |
| `analyze_readability_zh` | `ChineseReadabilityScores` | SCRI + common char ratio |
| `detect_passive_voice_zh` | `PassiveVoiceResult` | 被/受/遭/給/為...所 patterns |
| `detect_cliches_zh` | `ClicheResult` | Chinese cliche detection |
| `detect_filter_words_zh` | `FilterWordResult` | Chinese filler word detection |
| `analyze_sentiment_zh` | `SentimentResult` | Sentiment with negation handling |
| `analyze_sentence_variety_zh` | `SentenceVarietyResult` | Segmentation-based variety analysis |
| `segment` | `Vec<String>` | Word segmentation (Traditional + Simplified) |
| `split_sentences_zh` | `Vec<&str>` | Chinese sentence splitting |

### Result Types

| Type | Key Fields |
|------|------------|
| `ReadabilityScores` | `flesch_kincaid_grade`, `flesch_reading_ease`, `smog_index`, `coleman_liau_index`, `automated_readability_index` |
| `ChineseReadabilityScores` | `scri`, `avg_sentence_length`, `avg_word_length`, `common_char_ratio`, `character_count`, `word_count`, `sentence_count` |
| `PassiveVoiceResult` | `instances: Vec<PassiveInstance>`, `percentage: f64` |
| `ClicheResult` | `instances: Vec<ClicheInstance>`, `count: usize` |
| `FilterWordResult` | `instances: Vec<FilterWordInstance>`, `count: usize`, `percentage: f64` |
| `SentimentResult` | `score: f64`, `comparative: f64`, `tokens: Vec<TokenSentiment>` |
| `SentenceVarietyResult` | `avg_length: f64`, `length_variance: f64`, `starters: Vec<String>`, `structure_variety: f64` |

## Part of the Scrivener Rust Ecosystem

| Crate | Description |
|-------|-------------|
| [`scrivener-rtf`](https://github.com/jiayun/scrivener-rtf) | RTF parser and generator |
| [`scrivener`](https://github.com/jiayun/scrivener-rs) | Scrivener `.scriv` project reader/writer |
| **writing-analysis** | Writing analysis and NLP tools |

## License

MIT
