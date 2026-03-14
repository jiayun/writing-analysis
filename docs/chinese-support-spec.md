# Chinese Language Support Specification

## Overview

Phase 2 extension to add Chinese (Traditional + Simplified) text analysis capabilities. The design goal is to mirror the English API surface — same result types, same function signatures — with language-appropriate algorithms underneath.

## Architecture

### Language Detection Strategy

```
&str  ──▶  detect_language()  ──▶  Language::English | Language::Chinese
                                          │
                              ┌───────────┴───────────┐
                              ▼                       ▼
                     English pipeline          Chinese pipeline
                     (existing code)           (new module)
```

Two approaches, in order of preference:

1. **Explicit API**: `analyze_all_zh(text)` / `analyze_readability_zh(text)` — separate functions, zero ambiguity
2. **Unified API**: `analyze_all(text)` auto-detects language — convenient but adds complexity

Recommendation: **Option 1** for v0.2.0. Language detection can be added later as sugar.

### Module Structure

```
src/
├── zh/                        # Chinese analysis module
│   ├── mod.rs                 # Re-exports
│   ├── segmenter.rs           # Word segmentation (jieba wrapper)
│   ├── readability.rs         # Chinese readability formulas
│   ├── passive_voice.rs       # 被動語態偵測
│   ├── cliche.rs              # 中文慣用語 / 陳腔濫調
│   ├── filter_words.rs        # 中文贅詞
│   ├── sentiment.rs           # 中文情感分析
│   ├── sentence_variety.rs    # 句子變化分析
│   └── lexicon.rs             # 中文情感詞典
├── zh_data/
│   ├── sentiment_ntusd.txt    # NTUSD 情感詞典 (or DUTIR)
│   ├── cliches_zh.txt         # 中文陳腔濫調清單
│   └── filter_words_zh.txt   # 中文贅詞清單
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `jieba-rs` | 0.7 | Chinese word segmentation (分詞) |
| `regex` | 1.11 | (existing) Pattern matching |
| `unicode-segmentation` | 1.12 | (existing) Sentence/character boundaries |

`jieba-rs` embeds a dictionary (~5MB) at compile time. This is acceptable for a text analysis crate but should be behind a **feature flag** to avoid bloating English-only users.

### Feature Flag

```toml
[features]
default = []
chinese = ["jieba-rs"]

[dependencies]
jieba-rs = { version = "0.7", optional = true }
```

Users opt in with:

```toml
writing-analysis = { version = "0.2", features = ["chinese"] }
```

All `zh/` modules are gated behind `#[cfg(feature = "chinese")]`.

## Component Specifications

### 1. Sentence Splitting

Chinese uses full-width punctuation for sentence boundaries.

**Sentence-ending punctuation**: `。！？；` (period, exclamation, question, semicolon)

**Clause-ending punctuation** (not sentence boundaries): `，、：「」『』（）——……`

**Algorithm**:
1. Split on `。！？` (and optionally `；`)
2. Handle mixed Chinese/English text: also recognize `.!?` for embedded English
3. Handle quotation marks: `「...。」` — the `。` inside quotes may or may not be a sentence boundary (treat as boundary for simplicity)

```rust
pub fn split_sentences_zh(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        if matches!(ch, '。' | '！' | '？' | '；' | '.' | '!' | '?') {
            let end = i + ch.len_utf8();
            let sentence = text[start..end].trim();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            start = end;
        }
    }

    let remaining = text[start..].trim();
    if !remaining.is_empty() {
        sentences.push(remaining);
    }

    sentences
}
```

### 2. Word Segmentation (分詞)

Chinese text has no spaces between words. Word segmentation is required for all downstream analyses.

```rust
use jieba_rs::Jieba;
use std::sync::LazyLock;

static JIEBA: LazyLock<Jieba> = LazyLock::new(Jieba::new);

pub fn segment(text: &str) -> Vec<String> {
    JIEBA.cut(text, false)
        .into_iter()
        .filter(|w| !w.trim().is_empty())
        .map(|w| w.to_string())
        .collect()
}

pub fn segment_for_search(text: &str) -> Vec<String> {
    JIEBA.cut_for_search(text, false)
        .into_iter()
        .filter(|w| !w.trim().is_empty())
        .map(|w| w.to_string())
        .collect()
}
```

### 3. Chinese Readability

English readability formulas (Flesch-Kincaid etc.) are not applicable to Chinese. Chinese readability is typically measured by:

- **Character count per sentence** (字數/句)
- **Stroke complexity** (筆劃數 — optional, complex)
- **Word frequency** (常用字比例)

#### Proposed Formulas

**Simple Chinese Readability Index (SCRI)**:

A custom formula based on published Chinese readability research:

```
SCRI = 0.5 × avg_sentence_length + 0.5 × avg_word_length_chars - 3.0
```

Where:
- `avg_sentence_length` = total words / total sentences (after segmentation)
- `avg_word_length_chars` = total characters / total words

Higher SCRI = harder to read.

**Character Frequency Score**:

Percentage of characters that are in the most common 1000/2000/3000 Chinese characters. Higher percentage = easier to read.

```rust
pub struct ChineseReadabilityScores {
    /// Simple Chinese Readability Index (higher = harder)
    pub scri: f64,

    /// Average sentence length in words (after segmentation)
    pub avg_sentence_length: f64,

    /// Average word length in characters
    pub avg_word_length: f64,

    /// Percentage of characters in top-1000 most common (0.0-100.0)
    pub common_char_ratio: f64,

    /// Total character count (漢字 only)
    pub character_count: usize,

    /// Total word count (after segmentation)
    pub word_count: usize,

    /// Total sentence count
    pub sentence_count: usize,
}
```

### 4. Chinese Passive Voice (被動語態)

Chinese passive voice patterns:

| Pattern | Example | Notes |
|---------|---------|-------|
| 被 + verb | 被打、被罵 | Most common |
| 受 + verb | 受傷、受騙 | Formal |
| 遭 + verb | 遭遇、遭受 | Negative connotation |
| 給 + verb | 給騙了 | Colloquial |
| 為...所 + verb | 為人所知 | Literary/formal |
| 受到 + noun | 受到批評 | With 到 complement |
| 遭到 + noun | 遭到反對 | With 到 complement |

**Algorithm**:

```rust
use regex::Regex;
use std::sync::LazyLock;

static PASSIVE_ZH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(被|受|遭|給)\S{1,4}(了|過|著)?").unwrap()
});

static PASSIVE_WEI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"為\S{1,6}所\S{1,4}").unwrap()
});
```

**Exclusions**: Some 「被」 usages are not passive voice:
- 被子 (quilt)
- 被告 (defendant) — when used as a noun
- 被動 (passive) — when used as an adjective

Maintain an exclusion list of common non-passive 「被」 compounds.

### 5. Chinese Cliches (陳腔濫調)

Categories:

**四字成語 used as cliches** (overused idioms):
```
眾所周知、不言而喻、一目了然、理所當然、
與日俱增、息息相關、不可或缺、有目共睹、
顯而易見、毋庸置疑、不可思議、迫不及待
```

**套話 / 官話** (bureaucratic cliches):
```
高度重視、深入貫徹、全面落實、積極推進、
不斷加強、進一步提高、認真學習、堅定不移
```

**寫作陳腔** (writing cliches):
```
在...的過程中、從...的角度來看、值得一提的是、
不得不說、眾所周知、毫無疑問、歸根結底
```

Format: one phrase per line in `zh_data/cliches_zh.txt`, ~100 entries.

### 6. Chinese Filter Words (贅詞)

Common Chinese filler/filter words:

```
其實、基本上、就是、然後、所以、
的話、應該、可能、大概、反正、
就是說、差不多、總之、老實說、
簡單來說、坦白說、說實話、一般來說、
某種程度上、在某種意義上
```

### 7. Chinese Sentiment Analysis

#### Lexicon Options

| Lexicon | Size | License | Notes |
|---------|------|---------|-------|
| **NTUSD** (台大情感詞典) | ~11,000 words | Academic | Positive/negative binary |
| **DUTIR** (大連理工) | ~27,000 words | Academic | 7 emotion categories |
| **HowNet** | ~8,000 words | Research | Structured sentiment |
| **Custom AFINN-style** | ~3,000 words | MIT | Scored -5 to +5, curated |

Recommendation: Start with a **custom scored lexicon** (AFINN-style, -5 to +5) for consistency with the English API. Can extend with NTUSD/DUTIR later.

#### Scoring Algorithm

Same as English: tokenize → lookup → sum → normalize.

```rust
pub fn analyze_sentiment_zh(text: &str) -> Result<SentimentResult> {
    let words = segment(text);
    // ... same scoring logic as English, different lexicon
}
```

Key difference: Chinese sentiment often depends on **negation words** (不、沒、別、非、未) which invert the sentiment of the following word. The algorithm should handle:
- 不好 = negative (not + good)
- 不壞 = mildly positive (not + bad)
- 非常好 = intensified positive (very + good)

**Negation handling**:
1. If a negation word precedes a sentiment word, multiply the score by -0.5
2. If an intensifier (非常、很、特別、極) precedes a sentiment word, multiply by 1.5

### 8. Sentence Variety (Chinese)

Same algorithm as English, but using segmented words instead of space-split words:

- Average sentence length: words per sentence (after jieba segmentation)
- Starter variety: first word/phrase of each sentence
- Length variance: same calculation

## Public API

```rust
// Feature-gated under "chinese"
#[cfg(feature = "chinese")]
pub mod zh;

// Individual analyzers
pub fn analyze_readability_zh(text: &str) -> Result<ChineseReadabilityScores>;
pub fn detect_passive_voice_zh(text: &str) -> Result<PassiveVoiceResult>;
pub fn detect_cliches_zh(text: &str) -> Result<ClicheResult>;
pub fn detect_filter_words_zh(text: &str) -> Result<FilterWordResult>;
pub fn analyze_sentiment_zh(text: &str) -> Result<SentimentResult>;
pub fn analyze_sentence_variety_zh(text: &str) -> Result<SentenceVarietyResult>;

// Aggregated
pub fn analyze_all_zh(text: &str) -> Result<AnalysisResultZh>;
```

Result types reuse existing structs where possible (`PassiveVoiceResult`, `ClicheResult`, `FilterWordResult`, `SentimentResult`, `SentenceVarietyResult`). Only `ChineseReadabilityScores` is new (replaces English-specific `ReadabilityScores`).

## Implementation Order

| Step | Task | Estimate |
|------|------|----------|
| 1 | Feature flag setup + `zh/mod.rs` scaffolding | 0.5 day |
| 2 | Sentence splitting (`。！？`) | 0.5 day |
| 3 | jieba-rs integration + word segmentation | 1 day |
| 4 | Chinese readability formulas | 1 day |
| 5 | 被動語態偵測 | 1 day |
| 6 | 中文陳腔濫調清單 + 匹配 | 1 day |
| 7 | 中文贅詞清單 + 匹配 | 0.5 day |
| 8 | 中文情感詞典建立 + 評分 (含否定詞處理) | 2 days |
| 9 | 句子變化分析 (Chinese) | 0.5 day |
| 10 | `analyze_all_zh` 整合 | 0.5 day |
| 11 | Tests | 2 days |
| **Total** | | **~10 days** |

## Testing Strategy

### Test Fixtures

```
tests/fixtures/
├── zh_simple.txt         # 簡單中文散文
├── zh_passive_heavy.txt  # 大量被動語態的文本
├── zh_literary.txt       # 文學作品節選
└── zh_mixed.txt          # 中英混合文本
```

### Key Test Cases

**Sentence splitting**:
- `"你好。世界！"` → `["你好。", "世界！"]`
- `"他說：「你好。」她笑了。"` → `["他說：「你好。」", "她笑了。"]`
- Mixed: `"Hello世界。This is test。"` → `["Hello世界。", "This is test。"]`

**Word segmentation**:
- `"我來到北京清華大學"` → `["我", "來到", "北京", "清華大學"]`

**Passive voice**:
- `"他被老師罵了。"` → detected: `"被老師罵了"`
- `"這件事為人所知。"` → detected: `"為人所知"`
- `"被子很暖和。"` → NOT detected (被子 = quilt)

**Sentiment**:
- `"這部電影非常好看"` → positive
- `"今天天氣不好"` → negative (negation: 不 + 好)
- `"這個不壞"` → mildly positive (negation: 不 + 壞)

## Mixed Language Support

For texts containing both Chinese and English:

1. **Detection**: If > 30% of characters are CJK (Unicode range `\u{4e00}-\u{9fff}`), use Chinese pipeline
2. **Hybrid analysis**: Run both pipelines and merge results (future enhancement)
3. **v0.2.0 scope**: Require explicit API call (`analyze_all` vs `analyze_all_zh`), no auto-detection

## Known Challenges

### 1. Segmentation Accuracy

**Problem**: jieba-rs is not perfect. Domain-specific terms, new words, and names may be mis-segmented.

**Solution**: jieba supports custom dictionaries. Allow users to pass additional dictionary entries:
```rust
pub fn analyze_all_zh_with_dict(text: &str, custom_words: &[&str]) -> Result<AnalysisResultZh>;
```

### 2. Traditional vs Simplified

**Problem**: The crate should support both 繁體 and 簡體.

**Solution**: jieba-rs handles both. Sentiment lexicon and cliché lists should include both forms. Use a mapping table or maintain dual lists.

### 3. Binary Size

**Problem**: jieba-rs embeds a ~5MB dictionary, significantly increasing binary size.

**Solution**: Feature flag (`chinese`) keeps it opt-in. Users who only need English analysis pay no cost.

### 4. Negation Scope

**Problem**: In `"我不覺得這很糟糕"` (I don't think this is terrible), the negation `不` applies to `覺得`, not `糟糕`. Determining negation scope requires syntactic understanding.

**Solution**: For v0.2.0, use a simple window approach: negation affects the next sentiment word within 2 tokens. This is imperfect but practical.
