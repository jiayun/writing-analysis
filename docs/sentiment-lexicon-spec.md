# Sentiment & Lexicon Specification

## Overview

This document specifies the sentiment analysis system (AFINN lexicon loading and scoring), sentence variety analysis, and the aggregated `analyze_all` API.

## AFINN Lexicon (`lexicon.rs`)

### Format

The AFINN-111 lexicon is a tab-separated file with ~2,477 entries:

```
abandon	-2
abandoned	-2
abandons	-2
abducted	-2
...
wonderful	4
worthless	-3
wow	4
zealous	2
```

Each line: `word\tscore` where score is an integer from -5 (most negative) to +5 (most positive).

### Embedding

The lexicon is embedded at compile time using `include_str!` and parsed into a `HashMap` on first access:

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static AFINN_DATA: &str = include_str!("../data/afinn-111.txt");

static AFINN: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    AFINN_DATA
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('\t');
            let word = parts.next()?;
            let score = parts.next()?.parse::<i32>().ok()?;
            Some((word, score))
        })
        .collect()
});

pub fn get_score(word: &str) -> Option<i32> {
    AFINN.get(word).copied()
}
```

### File Location

The AFINN data file is stored at `data/afinn-111.txt` in the crate root. This file is included in the published crate.

```
writing-analysis/
├── Cargo.toml
├── data/
│   └── afinn-111.txt     # AFINN-111 lexicon (tab-separated)
├── src/
│   └── ...
```

## Sentiment Analysis (`sentiment.rs`)

### Scoring Algorithm

1. **Tokenize**: Split text into lowercase words using `unicode_words()`
2. **Lookup**: For each word, look up AFINN score (default 0 if not found)
3. **Sum**: Calculate total raw score = sum of all AFINN scores
4. **Comparative**: comparative = raw_score / total_token_count
5. **Normalize**: score = clamp(comparative × 2.0, -1.0, 1.0) — approximate normalization

```rust
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
```

### Score Interpretation

| Score Range | Interpretation |
|------------|----------------|
| 0.5 to 1.0 | Strongly positive |
| 0.1 to 0.5 | Mildly positive |
| -0.1 to 0.1 | Neutral |
| -0.5 to -0.1 | Mildly negative |
| -1.0 to -0.5 | Strongly negative |

### Example Scoring

Text: `"I love this wonderful movie"`

| Token | AFINN Score |
|-------|------------|
| "i" | 0 |
| "love" | 3 |
| "this" | 0 |
| "wonderful" | 4 |
| "movie" | 0 |

- Total raw score: 3 + 4 = 7
- Token count: 5
- Comparative: 7 / 5 = 1.4
- Normalized score: clamp(1.4 × 2.0, -1.0, 1.0) = 1.0

Text: `"This is a terrible, awful day"`

| Token | AFINN Score |
|-------|------------|
| "this" | 0 |
| "is" | 0 |
| "a" | 0 |
| "terrible" | -3 |
| "awful" | -3 |
| "day" | 0 |

- Total raw score: -3 + -3 = -6
- Token count: 6
- Comparative: -6 / 6 = -1.0
- Normalized score: clamp(-1.0 × 2.0, -1.0, 1.0) = -1.0

## Sentence Variety Analysis (`sentence_variety.rs`)

### Algorithm

Analyzes the structural diversity of sentences in a text.

```rust
pub fn analyze_sentence_variety(text: &str) -> Result<SentenceVarietyResult> {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    // 1. Compute sentence lengths (in words)
    let lengths: Vec<usize> = sentences
        .iter()
        .map(|s| split_words(s).len())
        .collect();

    // 2. Average length
    let total_words: usize = lengths.iter().sum();
    let avg_length = total_words as f64 / lengths.len() as f64;

    // 3. Length variance
    let variance = lengths
        .iter()
        .map(|&len| {
            let diff = len as f64 - avg_length;
            diff * diff
        })
        .sum::<f64>()
        / lengths.len() as f64;

    // 4. Sentence starters (first word of each sentence)
    let starters: Vec<String> = sentences
        .iter()
        .filter_map(|s| split_words(s).first().map(|w| w.to_lowercase()))
        .collect();

    // 5. Structure variety: unique starters / total sentences
    let unique_starters: std::collections::HashSet<&str> = starters
        .iter()
        .map(|s| s.as_str())
        .collect();
    let structure_variety = unique_starters.len() as f64 / starters.len() as f64;

    Ok(SentenceVarietyResult {
        avg_length,
        length_variance: variance,
        starters,
        structure_variety,
    })
}
```

### Metrics Explained

**Average sentence length**: Target range for readable prose is 15-20 words per sentence. Consistently short sentences (< 10) feel choppy; consistently long sentences (> 30) are hard to follow.

**Length variance**: Higher variance indicates more varied sentence lengths, which is generally desirable for engaging writing. A variance of 0 means all sentences are the same length.

**Sentence starters**: Collects the first word of each sentence (lowercased). Repetitive starters (e.g., every sentence starting with "The" or "I") indicate monotonous writing.

**Structure variety**: Ratio of unique starters to total sentences. 1.0 means every sentence starts differently. Below 0.5 suggests repetitive sentence openings.

### Example Analysis

Text: `"The cat sat. The dog ran. The bird flew. A fish swam."`

| Metric | Value |
|--------|-------|
| Avg length | 3.0 |
| Variance | 0.0 |
| Starters | ["the", "the", "the", "a"] |
| Structure variety | 0.5 (2 unique / 4 total) |

Text: `"Rain fell heavily. Under the bridge, a cat sheltered. Nobody noticed. It was a cold, dreary afternoon in November."`

| Metric | Value |
|--------|-------|
| Avg length | 5.0 |
| Variance | 9.5 |
| Starters | ["rain", "under", "nobody", "it"] |
| Structure variety | 1.0 (4 unique / 4 total) |

## Aggregated Analysis API (`lib.rs`)

### `analyze_all`

Runs all 6 analyzers and returns a combined result.

```rust
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
```

### Error Propagation

`analyze_all` uses `?` to propagate errors from any individual analyzer. If the text is too short for one analyzer but valid for others, the function will return an error.

For consumers who want partial results, calling individual analyzers directly is recommended:

```rust
// Get what you can, ignore errors
let readability = analyze_readability(text).ok();
let sentiment = analyze_sentiment(text).ok();
```

### Performance Notes

`analyze_all` calls `split_sentences` and `split_words` multiple times (once per analyzer). For v0.1.0 this is acceptable — the text processing is fast relative to the analysis. A future optimization could share computed statistics across analyzers via a context struct.
