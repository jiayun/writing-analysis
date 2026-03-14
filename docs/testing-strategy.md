# Testing Strategy

## Test Organization

```
tests/
├── integration_tests.rs       # analyze_all end-to-end tests
└── fixtures/
    ├── simple_text.txt        # Simple prose for baseline testing
    ├── passive_heavy.txt      # Text with many passive voice instances
    └── literary_sample.txt    # Literary excerpt for realistic testing

# Unit tests are in-module via #[cfg(test)] in each src/*.rs file

benches/
└── analysis_benchmark.rs      # Criterion performance benchmarks
```

## Unit Tests (in-module `#[cfg(test)]`)

### Text Utility Tests (`utils.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_sentences_basic() {
        let sentences = split_sentences("Hello world. How are you? I am fine!");
        assert_eq!(sentences.len(), 3);
    }

    #[test]
    fn split_sentences_abbreviation() {
        let sentences = split_sentences("Dr. Smith went to Washington. He arrived on time.");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn split_sentences_decimal() {
        let sentences = split_sentences("He scored 3.5 points. That was great.");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn split_sentences_empty() {
        let sentences = split_sentences("");
        assert_eq!(sentences.len(), 0);
    }

    #[test]
    fn split_sentences_single() {
        let sentences = split_sentences("Just one sentence.");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn count_syllables_monosyllabic() {
        assert_eq!(count_syllables("the"), 1);
        assert_eq!(count_syllables("cat"), 1);
        assert_eq!(count_syllables("fire"), 1);
    }

    #[test]
    fn count_syllables_multisyllabic() {
        assert_eq!(count_syllables("hello"), 2);
        assert_eq!(count_syllables("beautiful"), 3);
        assert_eq!(count_syllables("understanding"), 4);
    }

    #[test]
    fn split_words_basic() {
        let words = split_words("Hello world");
        assert_eq!(words, vec!["Hello", "world"]);
    }

    #[test]
    fn split_words_with_punctuation() {
        let words = split_words("Hello, world!");
        assert_eq!(words, vec!["Hello", "world"]);
    }

    #[test]
    fn count_characters_letters_only() {
        assert_eq!(count_characters("Hello, world! 123"), 10);
    }

    #[test]
    fn compute_statistics_basic() {
        let stats = compute_statistics("The cat sat on the mat. The dog ran fast.");
        assert_eq!(stats.sentence_count, 2);
        assert_eq!(stats.word_count, 10);
    }
}
```

### Readability Tests (`readability.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readability_simple_text() {
        // "The cat sat on the mat." — very simple text
        let scores = analyze_readability("The cat sat on the mat.").unwrap();
        // Should be low grade level (elementary school)
        assert!(scores.flesch_kincaid_grade < 5.0);
        assert!(scores.flesch_reading_ease > 80.0);
    }

    #[test]
    fn readability_complex_text() {
        let text = "The implementation of sophisticated algorithms \
                    necessitates comprehensive understanding of computational \
                    complexity and mathematical abstractions.";
        let scores = analyze_readability(text).unwrap();
        // Should be high grade level (college+)
        assert!(scores.flesch_kincaid_grade > 12.0);
        assert!(scores.flesch_reading_ease < 30.0);
    }

    #[test]
    fn readability_empty_text() {
        let result = analyze_readability("");
        assert!(result.is_err());
    }

    #[test]
    fn readability_single_sentence() {
        let scores = analyze_readability("Hello world.").unwrap();
        assert!(scores.flesch_kincaid_grade.is_finite());
        assert!(scores.flesch_reading_ease.is_finite());
        assert!(scores.smog_index.is_finite());
        assert!(scores.coleman_liau_index.is_finite());
        assert!(scores.automated_readability_index.is_finite());
    }
}
```

### Passive Voice Tests (`passive_voice.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_basic_passive() {
        let result = detect_passive_voice("The ball was thrown by the boy.").unwrap();
        assert_eq!(result.instances.len(), 1);
        assert_eq!(result.instances[0].phrase, "was thrown");
    }

    #[test]
    fn detect_irregular_passive() {
        let result = detect_passive_voice("The report was written by the team.").unwrap();
        assert_eq!(result.instances.len(), 1);
        assert_eq!(result.instances[0].phrase, "was written");
    }

    #[test]
    fn no_passive_active_voice() {
        let result = detect_passive_voice("The boy threw the ball.").unwrap();
        assert_eq!(result.instances.len(), 0);
        assert_eq!(result.percentage, 0.0);
    }

    #[test]
    fn exclude_adjectives() {
        let result = detect_passive_voice("She was excited about the project.").unwrap();
        assert_eq!(result.instances.len(), 0);
    }

    #[test]
    fn multiple_passive_instances() {
        let text = "The cake was eaten. The song was sung. He walked home.";
        let result = detect_passive_voice(text).unwrap();
        assert_eq!(result.instances.len(), 2);
        // 2 out of 3 sentences have passive voice
        assert!((result.percentage - 66.67).abs() < 1.0);
    }

    #[test]
    fn passive_percentage_calculation() {
        let text = "The ball was thrown. She ran quickly.";
        let result = detect_passive_voice(text).unwrap();
        assert_eq!(result.percentage, 50.0);
    }
}
```

### Cliché Tests (`cliche.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_single_cliche() {
        let result = detect_cliches("At the end of the day, we need results.").unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.instances[0].canonical, "at the end of the day");
    }

    #[test]
    fn detect_multiple_cliches() {
        let text = "It was easier said than done, but better late than never.";
        let result = detect_cliches(text).unwrap();
        assert!(result.count >= 1);
    }

    #[test]
    fn no_cliches() {
        let result = detect_cliches("The quantum processor achieved remarkable throughput.").unwrap();
        assert_eq!(result.count, 0);
    }

    #[test]
    fn case_insensitive_match() {
        let result = detect_cliches("Think Outside The Box to solve this.").unwrap();
        assert_eq!(result.count, 1);
    }

    #[test]
    fn cliche_offset_tracking() {
        let text = "Well, at the end of the day it matters.";
        let result = detect_cliches(text).unwrap();
        assert_eq!(result.instances[0].offset, 6); // "at" starts at position 6
    }
}
```

### Filter Word Tests (`filter_words.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_filter_words_basic() {
        let result = detect_filter_words("She just really wanted to go.").unwrap();
        assert_eq!(result.count, 2);
    }

    #[test]
    fn no_filter_words() {
        let result = detect_filter_words("The cat sat on the mat.").unwrap();
        assert_eq!(result.count, 0);
        assert_eq!(result.percentage, 0.0);
    }

    #[test]
    fn filter_word_percentage() {
        let result = detect_filter_words("I very simply want this.").unwrap();
        // 2 filter words out of 5 total = 40%
        assert!((result.percentage - 40.0).abs() < 1.0);
    }

    #[test]
    fn case_insensitive_detection() {
        let result = detect_filter_words("JUST do it. Really.").unwrap();
        assert_eq!(result.count, 2);
    }

    #[test]
    fn empty_text_error() {
        let result = detect_filter_words("");
        assert!(result.is_err());
    }
}
```

### Sentiment Tests (`sentiment.rs`)

```rust
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
        assert!((result.score).abs() < 0.3);
    }

    #[test]
    fn token_level_scores() {
        let result = analyze_sentiment("I love hate things.").unwrap();
        let scores: Vec<i32> = result.tokens.iter().map(|t| t.score).collect();
        // "love" = 3, "hate" = -3
        assert!(scores.contains(&3));
        assert!(scores.contains(&-3));
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentiment("");
        assert!(result.is_err());
    }

    #[test]
    fn comparative_normalization() {
        // Longer text with same sentiment words should have lower comparative
        let short = analyze_sentiment("I love this.").unwrap();
        let long = analyze_sentiment("I love this thing that is here today.").unwrap();
        assert!(short.comparative.abs() > long.comparative.abs());
    }
}
```

### Sentence Variety Tests (`sentence_variety.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monotonous_writing() {
        let text = "The cat sat. The dog ran. The bird flew. The fish swam.";
        let result = analyze_sentence_variety(text).unwrap();
        // All sentences start with "the" → low variety
        assert!(result.structure_variety < 0.5);
    }

    #[test]
    fn varied_writing() {
        let text = "Rain fell heavily. Under the bridge, a cat sheltered. \
                    Nobody noticed. It was a dreary afternoon.";
        let result = analyze_sentence_variety(text).unwrap();
        // All different starters → high variety
        assert!(result.structure_variety > 0.9);
    }

    #[test]
    fn average_length_calculation() {
        // Two sentences: 3 words + 3 words = avg 3.0
        let text = "The cat sat. The dog ran.";
        let result = analyze_sentence_variety(text).unwrap();
        assert!((result.avg_length - 3.0).abs() < 0.5);
    }

    #[test]
    fn length_variance_same_length() {
        let text = "The cat sat. The dog ran.";
        let result = analyze_sentence_variety(text).unwrap();
        // Same length sentences → low variance
        assert!(result.length_variance < 1.0);
    }

    #[test]
    fn starters_collected() {
        let text = "Hello world. Goodbye moon.";
        let result = analyze_sentence_variety(text).unwrap();
        assert_eq!(result.starters, vec!["hello", "goodbye"]);
    }

    #[test]
    fn empty_text_error() {
        let result = analyze_sentence_variety("");
        assert!(result.is_err());
    }
}
```

## Integration Tests (`tests/`)

### End-to-end Tests (`tests/integration_tests.rs`)

```rust
use writing_analysis::*;

#[test]
fn analyze_all_basic() {
    let text = "The quick brown fox jumped over the lazy dog. \
                It was a beautiful day in the neighborhood. \
                She was excited about the upcoming adventure.";
    let result = analyze_all(text).unwrap();

    // Readability should be computed
    assert!(result.readability.flesch_reading_ease.is_finite());

    // Should detect some analysis results
    assert!(result.sentiment.score.is_finite());
    assert!(result.sentence_variety.avg_length > 0.0);
}

#[test]
fn analyze_all_empty_text() {
    let result = analyze_all("");
    assert!(result.is_err());
}

#[test]
fn analyze_all_single_sentence() {
    let result = analyze_all("The cat sat on the mat.");
    assert!(result.is_ok());
}

#[test]
fn analyze_all_with_passive_voice() {
    let text = "The ball was thrown by the boy. The cake was eaten. \
                He walked home quickly.";
    let result = analyze_all(text).unwrap();
    assert!(result.passive_voice.instances.len() >= 2);
    assert!(result.passive_voice.percentage > 50.0);
}

#[test]
fn analyze_all_with_cliches() {
    let text = "At the end of the day, we need to think outside the box. \
                It's time to bite the bullet and make a decision.";
    let result = analyze_all(text).unwrap();
    assert!(result.cliches.count >= 2);
}

#[test]
fn analyze_all_with_filter_words() {
    let text = "She just really wanted to basically understand the situation. \
                It was actually quite simple.";
    let result = analyze_all(text).unwrap();
    assert!(result.filter_words.count >= 3);
}
```

### Fixture-based Tests

```rust
#[test]
fn analyze_fixture_simple_text() {
    let text = include_str!("fixtures/simple_text.txt");
    let result = analyze_all(text).unwrap();

    // Simple prose should have moderate readability
    assert!(result.readability.flesch_reading_ease > 50.0);
    assert!(result.readability.flesch_kincaid_grade < 12.0);
}

#[test]
fn analyze_fixture_passive_heavy() {
    let text = include_str!("fixtures/passive_heavy.txt");
    let result = analyze_all(text).unwrap();

    // Should detect significant passive voice usage
    assert!(result.passive_voice.percentage > 30.0);
}
```

### Test Fixture Files

#### `tests/fixtures/simple_text.txt`
```
The sun rose over the quiet village. Birds began to sing in the tall oak trees.
A gentle breeze carried the scent of fresh bread from the bakery. Children played
in the park while their parents watched from nearby benches. It was a peaceful
morning, the kind that makes you grateful to be alive. The old church bell rang
eight times, marking the start of a new day.
```

#### `tests/fixtures/passive_heavy.txt`
```
The report was written by the committee. The decision was made after careful
deliberation. The project was completed ahead of schedule. The results were
reviewed by the board. The proposal was rejected due to budget constraints.
The meeting was attended by all department heads. The new policy was implemented
last month. The error was discovered during testing.
```

## Performance Benchmarks (`benches/analysis_benchmark.rs`)

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_readability(c: &mut Criterion) {
    let text = "The quick brown fox jumped over the lazy dog. ".repeat(100);
    c.bench_function("readability_500_words", |b| {
        b.iter(|| writing_analysis::analyze_readability(&text).unwrap())
    });
}

fn bench_passive_voice(c: &mut Criterion) {
    let text = "The ball was thrown by the boy. She ran quickly. ".repeat(50);
    c.bench_function("passive_voice_100_sentences", |b| {
        b.iter(|| writing_analysis::detect_passive_voice(&text).unwrap())
    });
}

fn bench_sentiment(c: &mut Criterion) {
    let text = "I love this wonderful beautiful amazing great fantastic day. ".repeat(50);
    c.bench_function("sentiment_500_words", |b| {
        b.iter(|| writing_analysis::analyze_sentiment(&text).unwrap())
    });
}

fn bench_analyze_all(c: &mut Criterion) {
    let text = include_str!("../tests/fixtures/simple_text.txt");
    c.bench_function("analyze_all_fixture", |b| {
        b.iter(|| writing_analysis::analyze_all(text).unwrap())
    });
}

fn bench_analyze_all_large(c: &mut Criterion) {
    // ~10KB of text
    let text = "The quick brown fox jumped over the lazy dog. \
                She was very happy about the wonderful results. \
                At the end of the day, it was a great success. ".repeat(100);
    c.bench_function("analyze_all_10kb", |b| {
        b.iter(|| writing_analysis::analyze_all(&text).unwrap())
    });
}

criterion_group!(
    benches,
    bench_readability,
    bench_passive_voice,
    bench_sentiment,
    bench_analyze_all,
    bench_analyze_all_large,
);
criterion_main!(benches);
```

**Performance target**: `analyze_all` on 10KB text in < 5ms.

## Test Coverage Goals

| Module | Coverage Target | Key Scenarios |
|--------|----------------|---------------|
| `utils.rs` | 95%+ | Sentence splitting edge cases, syllable counting accuracy |
| `readability.rs` | 90%+ | All 5 formulas, edge cases (single sentence, very long text) |
| `passive_voice.rs` | 90%+ | Regular/irregular participles, exclusion list, false positives |
| `cliche.rs` | 90%+ | Exact match, case insensitivity, multiple occurrences |
| `filter_words.rs` | 95%+ | All default filter words, percentage calculation |
| `sentiment.rs` | 90%+ | Positive/negative/neutral texts, per-token scores, normalization |
| `sentence_variety.rs` | 90%+ | Monotonous vs varied text, variance calculation |
| `lexicon.rs` | 95%+ | Lexicon loading, lookup hit/miss |
| Integration | N/A | `analyze_all` with various text types, error propagation |
