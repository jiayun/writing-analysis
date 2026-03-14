# Implementation Guide

## Cargo.toml

```toml
[package]
name = "writing-analysis"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Lightweight writing analysis and NLP tools for Rust"
repository = "https://github.com/jiayun/writing-analysis"
keywords = ["nlp", "writing", "readability", "sentiment", "analysis"]
categories = ["text-processing"]

[dependencies]
regex = "1.11"
unicode-segmentation = "1.10"
thiserror = "2.0"

[dev-dependencies]
pretty_assertions = "1.4"
criterion = "0.5"

[[bench]]
name = "analysis_benchmark"
harness = false
```

## Implementation Order

Each step builds on the previous. Run `cargo check` after each step.

### Step 1: Project Scaffolding (~30 min)

- Create `Cargo.toml` as above
- Create `src/lib.rs` with module declarations (all modules as empty stubs)
- Create all module files: `error.rs`, `utils.rs`, `readability.rs`, `passive_voice.rs`, `cliche.rs`, `filter_words.rs`, `sentiment.rs`, `sentence_variety.rs`, `lexicon.rs`
- Run `cargo check` to verify compilation

**Expected output**: Project compiles with empty modules.

### Step 2: Error Types (~15 min)

- Implement `error.rs` with all `WritingAnalysisError` variants
- Define `pub type Result<T>`
- No dependencies on other modules

**Expected output**: `cargo check` passes.

### Step 3: Text Utilities / Tokenizer (~1 day)

- Implement `utils.rs` with core text processing functions:
  - `split_sentences(text: &str) -> Vec<&str>` — split on `.!?` followed by whitespace or EOF
  - `split_words(text: &str) -> Vec<&str>` — Unicode-aware word boundaries via `unicode-segmentation`
  - `count_syllables(word: &str) -> usize` — English syllable estimation algorithm
  - `count_characters(text: &str) -> usize` — letter-only character count
  - `is_polysyllabic(word: &str) -> bool` — 3+ syllables
  - `compute_statistics(text: &str) -> TextStatistics` — aggregate stats
- Write unit tests for each function, especially edge cases:
  - Abbreviations: "Dr. Smith went to Washington."
  - Ellipsis: "Wait... what?"
  - Numbers: "He scored 3.5 points."

**Expected output**: All text utility functions work correctly with comprehensive tests.

### Step 4: Readability Module (~1-2 days)

- Implement `readability.rs` with all 5 formulas
- Each formula is a private function taking `&TextStatistics` → `f64`
- Public function `analyze_readability(text: &str) -> Result<ReadabilityScores>`:
  1. Call `compute_statistics(text)`
  2. Validate minimum requirements (at least 1 sentence, 1 word)
  3. Compute all 5 scores
  4. Return `ReadabilityScores`
- Write tests against known reference texts with expected score ranges

**Expected output**: Readability scores match reference implementations within ±0.5.

### Step 5: Passive Voice Module (~1-2 days)

- Implement `passive_voice.rs`
- Build regex pattern: `\b(am|is|are|was|were|be|been|being)\s+(\w+ed|IRREGULAR_LIST)\b`
- Compile the regex once using `LazyLock<Regex>`
- Public function `detect_passive_voice(text: &str) -> Result<PassiveVoiceResult>`:
  1. Split text into sentences
  2. For each sentence, find all regex matches
  3. Record `PassiveInstance` with phrase, offset, and sentence
  4. Calculate percentage: sentences with passive / total sentences × 100
- Handle false positives: skip known exceptions (e.g., "excited about", "used to")

**Expected output**: Correctly detects common passive voice patterns. Tests pass.

### Step 6: Cliché Module (~0.5 day)

- Implement `cliche.rs`
- Embed a list of ~100 common English clichés as a static `&[&str]`
- Public function `detect_cliches(text: &str) -> Result<ClicheResult>`:
  1. Lowercase the input text
  2. For each cliché in the list, search for exact match (case-insensitive)
  3. Optionally: fuzzy match by allowing minor word variations
  4. Record `ClicheInstance` with offset and match type
- Use `LazyLock` for compiled regex patterns if fuzzy matching is enabled

**Expected output**: Detects clichés in sample text. Tests pass.

### Step 7: Filter Words Module (~0.5 day)

- Implement `filter_words.rs`
- Default filter word list: just, really, very, quite, rather, somewhat, somehow, perhaps, basically, actually, literally, definitely, certainly, probably, simply, extremely, absolutely, totally, completely, utterly
- Public function `detect_filter_words(text: &str) -> Result<FilterWordResult>`:
  1. Split into words
  2. Check each word against the filter list (case-insensitive)
  3. Record instances with offset and containing sentence
  4. Calculate percentage: filter words / total words × 100

**Expected output**: Correctly identifies filter words with positions. Tests pass.

### Step 8: Sentiment Module (~1-2 days)

- Implement `lexicon.rs`:
  - Embed AFINN-111 word list as a static string (or build a `HashMap<&str, i32>` at compile time)
  - `fn get_score(word: &str) -> Option<i32>` — lookup function
  - Use `LazyLock<HashMap<&str, i32>>` for the lexicon
- Implement `sentiment.rs`:
  - Public function `analyze_sentiment(text: &str) -> Result<SentimentResult>`:
    1. Tokenize text into lowercase words
    2. Look up each word in AFINN lexicon
    3. Sum scores, compute comparative (sum / total tokens)
    4. Normalize to -1.0..1.0 range for the `score` field
    5. Return `SentimentResult` with per-token breakdown

**Expected output**: Sentiment scores match expected values for test phrases. Tests pass.

### Step 9: Sentence Variety Module (~1 day)

- Implement `sentence_variety.rs`
- Public function `analyze_sentence_variety(text: &str) -> Result<SentenceVarietyResult>`:
  1. Split into sentences
  2. For each sentence, count words → lengths vector
  3. Compute average length and variance
  4. Extract first word of each sentence → starters
  5. Compute structure variety: unique starters / total sentences
- Return `SentenceVarietyResult`

**Expected output**: Variety metrics computed correctly. Tests pass.

### Step 10: Public API Integration (~0.5 day)

- Wire everything in `lib.rs`
- Implement `analyze_all(text: &str) -> Result<AnalysisResult>`:
  - Call all 6 analyzers
  - Aggregate into `AnalysisResult`
- Re-export all public types and functions
- Add doc comments on all public items

**Expected output**: Full pipeline works: `analyze_all("some text")` returns complete results.

### Step 11: Tests (~1-2 days)

- Create test fixtures in `tests/fixtures/`
- Write integration tests covering:
  - Known readability scores for reference texts
  - Passive voice detection accuracy
  - Cliché detection with various texts
  - Sentiment scoring for positive/negative/neutral texts
  - Sentence variety for monotonous vs. varied writing
  - `analyze_all` end-to-end
- Edge cases: empty text, single word, single sentence, very long text

**Expected output**: All tests pass.

### Step 12: Benchmarks & Polish (~0.5 day)

- Create `benches/analysis_benchmark.rs` with Criterion
- Benchmark: analyze a ~10KB text, target < 5ms for `analyze_all`
- Add `#![doc = ...]` and module-level docs
- Update README with usage examples

**Expected output**: Benchmarks run, performance is acceptable.

## Known Challenges & Solutions

### 1. Sentence Splitting Accuracy

**Problem**: Naive splitting on `.!?` breaks on abbreviations ("Dr. Smith"), decimal numbers ("3.5"), ellipsis ("wait..."), and URLs.

**Solution**: Use a rule-based heuristic:
1. Split on `.!?` followed by whitespace + uppercase letter, or end of string
2. Don't split after known abbreviations (Mr., Mrs., Dr., etc.)
3. Don't split within numbers (digit + `.` + digit)

This is not perfect but handles common cases. A regex like `(?<=[.!?])\s+(?=[A-Z])` provides a reasonable baseline.

### 2. Syllable Counting

**Problem**: English syllable counting is notoriously irregular. No simple algorithm is 100% accurate.

**Solution**: Use a heuristic algorithm:
1. Count vowel groups (consecutive vowels = 1 syllable)
2. Subtract silent-e at end of word
3. Handle special suffixes: `-le`, `-es`, `-ed` (sometimes silent)
4. Minimum 1 syllable per word
5. Known exceptions list for common words (e.g., "fire" = 1, not 2)

Target: 90%+ accuracy on common English words. This is sufficient for readability scoring where aggregate statistics smooth out individual errors.

### 3. Passive Voice False Positives

**Problem**: "was excited about the project" — "excited" ends in `-ed` but this is an adjective, not passive voice.

**Solution**: Maintain an exclusion list of common `-ed` adjectives that are not past participles in passive constructions: excited, interested, bored, tired, surprised, amazed, concerned, dedicated, etc. These are skipped during detection.

### 4. AFINN Lexicon Coverage

**Problem**: The AFINN-111 lexicon contains ~2,477 words, which misses many domain-specific or modern words.

**Solution**: For v0.1.0, the AFINN-111 list is sufficient. The lexicon module is designed to allow future extension (custom word lists can be added). Words not in the lexicon are scored as 0 (neutral) and still included in the token list.

### 5. Unicode Word Boundaries

**Problem**: Simple whitespace splitting fails for languages with non-space word boundaries, contractions ("don't"), and hyphenated words.

**Solution**: Use `unicode-segmentation` crate's `UnicodeSegmentation::unicode_words()` for proper Unicode-aware word boundary detection. This handles contractions and most edge cases correctly for English text.

### 6. Thread Safety of Static Data

**Problem**: Static word lists and compiled regex patterns must be safely accessible from multiple threads.

**Solution**: Use `std::sync::LazyLock` (stable since Rust 1.80) for all lazy-initialized static data. This provides thread-safe initialization without the `lazy_static` dependency.
