# Analysis Specification

## Overview

This document specifies the algorithms for text analysis: text utilities (sentence splitting, syllable counting), readability formulas, passive voice detection, cliché detection, and filter word detection.

## Text Utilities (`utils.rs`)

### Sentence Splitting

Splits text into sentences using punctuation and capitalization heuristics.

**Algorithm**:
1. Find positions of sentence-ending punctuation: `.`, `!`, `?`
2. A split occurs when the punctuation is followed by:
   - One or more whitespace characters, then an uppercase letter
   - End of string
3. Do NOT split after known abbreviations: Mr., Mrs., Ms., Dr., Prof., Sr., Jr., St., etc.
4. Do NOT split when `.` is between two digits (decimal numbers)
5. Ellipsis (`...`) counts as one sentence boundary, not three

```rust
use std::sync::LazyLock;
use regex::Regex;

static SENTENCE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?<=[.!?])\s+(?=[A-Z])").unwrap()
});

static ABBREVIATIONS: &[&str] = &[
    "Mr.", "Mrs.", "Ms.", "Dr.", "Prof.", "Sr.", "Jr.", "St.",
    "Inc.", "Ltd.", "Corp.", "vs.", "etc.", "e.g.", "i.e.",
];

pub fn split_sentences(text: &str) -> Vec<&str> {
    // Pre-process: protect abbreviations by checking context
    // Split on regex matches
    // Filter empty results
    // Trim each sentence
    let parts: Vec<&str> = SENTENCE_RE.split(text)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    parts
}
```

**Edge cases**:

| Input | Expected Sentences |
|-------|-------------------|
| `"Hello world."` | `["Hello world."]` |
| `"First. Second."` | `["First.", "Second."]` |
| `"Dr. Smith went home."` | `["Dr. Smith went home."]` |
| `"Score: 3.5 points."` | `["Score: 3.5 points."]` |
| `"Wait... What?"` | `["Wait...", "What?"]` |
| `""` | `[]` |

### Word Tokenization

Uses `unicode-segmentation` for Unicode-aware word boundary detection.

```rust
use unicode_segmentation::UnicodeSegmentation;

pub fn split_words(text: &str) -> Vec<&str> {
    text.unicode_words().collect()
}
```

`unicode_words()` returns only "word-like" segments — it automatically skips punctuation and whitespace. Contractions like "don't" are handled as a single word.

### Syllable Counting

Estimates syllable count for an English word using vowel-group heuristics.

**Algorithm**:
1. Convert word to lowercase, strip non-alphabetic characters
2. Count vowel groups (consecutive `a, e, i, o, u, y` characters)
3. Apply adjustments:
   - Subtract 1 if word ends in silent `e` (but not `le` after consonant)
   - Subtract 1 if word ends in `es` or `ed` and the result would be > 1
   - Add 1 for `-le` endings preceded by a consonant (e.g., "table")
4. Minimum return value: 1

```rust
pub fn count_syllables(word: &str) -> usize {
    let word = word.to_lowercase();
    let chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.is_empty() {
        return 1;
    }

    let vowels = "aeiouy";
    let mut count = 0;
    let mut prev_vowel = false;

    for &ch in &chars {
        let is_vowel = vowels.contains(ch);
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Silent-e adjustment
    if chars.len() > 2 && chars.last() == Some(&'e') {
        let second_last = chars[chars.len() - 2];
        if !vowels.contains(second_last) {
            count = count.saturating_sub(1);
        }
    }

    // Ensure minimum of 1
    count.max(1)
}
```

**Reference values**:

| Word | Syllables | Notes |
|------|-----------|-------|
| "the" | 1 | |
| "hello" | 2 | hel-lo |
| "beautiful" | 3 | beau-ti-ful |
| "understanding" | 4 | un-der-stand-ing |
| "fire" | 1 | silent-e |
| "table" | 2 | ta-ble (consonant + le) |

### Character Counting

Counts only alphabetic characters (letters), excluding spaces, digits, and punctuation.

```rust
pub fn count_characters(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}
```

### Text Statistics

Aggregates all text metrics into a single struct.

```rust
pub fn compute_statistics(text: &str) -> TextStatistics {
    let sentences = split_sentences(text);
    let words = split_words(text);
    let syllable_count: usize = words.iter().map(|w| count_syllables(w)).sum();
    let character_count = count_characters(text);
    let polysyllable_count = words.iter().filter(|w| count_syllables(w) >= 3).count();

    TextStatistics {
        sentence_count: sentences.len(),
        word_count: words.len(),
        syllable_count,
        character_count,
        polysyllable_count,
    }
}
```

## Readability Formulas (`readability.rs`)

All formulas operate on `TextStatistics`. The public function computes all 5 scores at once.

### Flesch-Kincaid Grade Level

Estimates the US school grade level needed to understand the text.

```
FKGL = 0.39 × (words / sentences) + 11.8 × (syllables / words) − 15.59
```

```rust
fn flesch_kincaid_grade(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let syllables = stats.syllable_count as f64;

    0.39 * (words / sentences) + 11.8 * (syllables / words) - 15.59
}
```

### Flesch Reading Ease

Scores text readability from 0 (very difficult) to 100 (very easy).

```
FRE = 206.835 − 1.015 × (words / sentences) − 84.6 × (syllables / words)
```

```rust
fn flesch_reading_ease(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let syllables = stats.syllable_count as f64;

    206.835 - 1.015 * (words / sentences) - 84.6 * (syllables / words)
}
```

**Score interpretation**:

| Score Range | Difficulty | Grade Level |
|------------|------------|-------------|
| 90–100 | Very easy | 5th grade |
| 80–90 | Easy | 6th grade |
| 70–80 | Fairly easy | 7th grade |
| 60–70 | Standard | 8th–9th grade |
| 50–60 | Fairly difficult | 10th–12th grade |
| 30–50 | Difficult | College |
| 0–30 | Very difficult | Graduate |

### SMOG Index

Simple Measure of Gobbledygook. Estimates years of education needed.

```
SMOG = 3 + √(polysyllable_count × 30 / sentence_count)
```

```rust
fn smog_index(stats: &TextStatistics) -> f64 {
    let polysyllables = stats.polysyllable_count as f64;
    let sentences = stats.sentence_count as f64;

    3.0 + (polysyllables * 30.0 / sentences).sqrt()
}
```

Note: SMOG is designed for texts with 30+ sentences. For shorter texts, the result is an approximation.

### Coleman-Liau Index

Uses character counts instead of syllable counts, making it faster to compute.

```
CLI = 0.0588 × L − 0.296 × S − 15.8
```

Where `L` = average number of letters per 100 words, `S` = average number of sentences per 100 words.

```rust
fn coleman_liau_index(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let l = (stats.character_count as f64 / words) * 100.0;
    let s = (stats.sentence_count as f64 / words) * 100.0;

    0.0588 * l - 0.296 * s - 15.8
}
```

### Automated Readability Index

Uses character and word counts for grade level estimation.

```
ARI = 4.71 × (characters / words) + 0.5 × (words / sentences) − 21.43
```

```rust
fn automated_readability_index(stats: &TextStatistics) -> f64 {
    let words = stats.word_count as f64;
    let sentences = stats.sentence_count as f64;
    let characters = stats.character_count as f64;

    4.71 * (characters / words) + 0.5 * (words / sentences) - 21.43
}
```

### Public Function

```rust
pub fn analyze_readability(text: &str) -> Result<ReadabilityScores> {
    let stats = compute_statistics(text);

    if stats.sentence_count == 0 {
        return Err(WritingAnalysisError::NoSentences);
    }
    if stats.word_count == 0 {
        return Err(WritingAnalysisError::EmptyText);
    }

    Ok(ReadabilityScores {
        flesch_kincaid_grade: flesch_kincaid_grade(&stats),
        flesch_reading_ease: flesch_reading_ease(&stats),
        smog_index: smog_index(&stats),
        coleman_liau_index: coleman_liau_index(&stats),
        automated_readability_index: automated_readability_index(&stats),
    })
}
```

## Passive Voice Detection (`passive_voice.rs`)

### Algorithm

Detects passive voice by matching `be-verb + past participle` patterns.

**Be-verbs**: am, is, are, was, were, be, been, being

**Past participle identification**:
1. Words ending in `-ed` (regular verbs: "walked", "written" doesn't match this, "played")
2. Common irregular past participles (curated list)

**Irregular past participle list** (~60 common words):

```rust
static IRREGULAR_PAST_PARTICIPLES: &[&str] = &[
    "awoken", "been", "born", "beaten", "become", "begun", "bent",
    "bitten", "blown", "broken", "brought", "built", "burnt", "bought",
    "caught", "chosen", "come", "cost", "cut", "done", "drawn", "driven",
    "drunk", "eaten", "fallen", "felt", "found", "flown", "forgotten",
    "forgiven", "frozen", "given", "gone", "grown", "had", "heard",
    "hidden", "hit", "held", "hurt", "kept", "known", "laid", "led",
    "left", "lent", "let", "lain", "lost", "made", "meant", "met",
    "paid", "put", "read", "ridden", "risen", "run", "said", "seen",
    "sent", "set", "shaken", "shown", "shut", "slept", "slid",
    "spoken", "spent", "split", "spread", "stood", "stolen", "stuck",
    "stung", "struck", "sworn", "swept", "swum", "taken", "taught",
    "thought", "thrown", "told", "torn", "understood", "woken",
    "worn", "wound", "written",
];
```

**Exclusion list** (common `-ed` adjectives that are not passive voice):

```rust
static ADJECTIVE_EXCLUSIONS: &[&str] = &[
    "advanced", "amazed", "associated", "attached", "bored",
    "complicated", "concerned", "confused", "connected", "convinced",
    "dedicated", "determined", "disappointed", "embarrassed", "excited",
    "experienced", "frustrated", "interested", "involved", "married",
    "organized", "overwhelmed", "pleased", "prepared", "related",
    "satisfied", "sophisticated", "supposed", "surprised", "tired",
    "used",
];
```

### Regex Pattern

```rust
use std::sync::LazyLock;
use regex::Regex;

static PASSIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Build the irregular list into alternation
    let irregulars = IRREGULAR_PAST_PARTICIPLES.join("|");
    let pattern = format!(
        r"\b(am|is|are|was|were|be|been|being)\s+((\w+ed)|({}))\b",
        irregulars
    );
    Regex::new(&pattern).unwrap()
});
```

### Detection Logic

```rust
pub fn detect_passive_voice(text: &str) -> Result<PassiveVoiceResult> {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    let mut instances = Vec::new();
    let mut sentences_with_passive = 0;

    for sentence in &sentences {
        let mut found_in_sentence = false;
        for mat in PASSIVE_RE.find_iter(sentence) {
            let phrase = mat.as_str().to_string();

            // Check exclusion list
            let words: Vec<&str> = phrase.split_whitespace().collect();
            if let Some(participle) = words.last() {
                if ADJECTIVE_EXCLUSIONS.contains(&participle.to_lowercase().as_str()) {
                    continue;
                }
            }

            // Calculate offset in original text
            let sentence_start = sentence.as_ptr() as usize - text.as_ptr() as usize;
            let offset = sentence_start + mat.start();

            instances.push(PassiveInstance {
                phrase,
                offset,
                sentence: sentence.to_string(),
            });
            found_in_sentence = true;
        }
        if found_in_sentence {
            sentences_with_passive += 1;
        }
    }

    let percentage = (sentences_with_passive as f64 / sentences.len() as f64) * 100.0;

    Ok(PassiveVoiceResult {
        instances,
        percentage,
    })
}
```

**Example matches**:

| Input | Detected | Phrase |
|-------|----------|--------|
| "The ball was thrown." | ✓ | "was thrown" |
| "She was running." | ✗ | "running" is not past participle |
| "The report was written by John." | ✓ | "was written" |
| "He is excited about it." | ✗ | "excited" in exclusion list |
| "The window was broken." | ✓ | "was broken" (irregular) |

## Cliché Detection (`cliche.rs`)

### Built-in Cliché List

A curated list of ~100 common English clichés:

```rust
static CLICHES: &[&str] = &[
    "a chip on your shoulder",
    "a dime a dozen",
    "a picture is worth a thousand words",
    "absence makes the heart grow fonder",
    "actions speak louder than words",
    "add insult to injury",
    "against all odds",
    "all in a day's work",
    "all that glitters is not gold",
    "at the drop of a hat",
    "at the end of the day",
    "back to the drawing board",
    "barking up the wrong tree",
    "beat a dead horse",
    "beat around the bush",
    "better late than never",
    "better safe than sorry",
    "between a rock and a hard place",
    "bite the bullet",
    "bite the dust",
    "blood is thicker than water",
    "break the ice",
    "burning the midnight oil",
    "by the skin of your teeth",
    "calm before the storm",
    "cost an arm and a leg",
    "cut to the chase",
    "don't cry over spilled milk",
    "don't put all your eggs in one basket",
    "drastic times call for drastic measures",
    "easy as pie",
    "every cloud has a silver lining",
    "fall head over heels",
    "few and far between",
    "fit as a fiddle",
    "go the extra mile",
    "good things come to those who wait",
    "hit the nail on the head",
    "ignorance is bliss",
    "in the heat of the moment",
    "it takes two to tango",
    "keep your chin up",
    "kill two birds with one stone",
    "last but not least",
    "leave no stone unturned",
    "let the cat out of the bag",
    "like riding a bicycle",
    "live and learn",
    "look before you leap",
    "more than meets the eye",
    "needle in a haystack",
    "no pain no gain",
    "once in a blue moon",
    "only time will tell",
    "out of the frying pan into the fire",
    "par for the course",
    "play it by ear",
    "pulling someone's leg",
    "put all your eggs in one basket",
    "read between the lines",
    "reinvent the wheel",
    "Rome wasn't built in a day",
    "see eye to eye",
    "shoot for the moon",
    "sleep on it",
    "speak of the devil",
    "stand the test of time",
    "take it with a grain of salt",
    "the apple doesn't fall far from the tree",
    "the ball is in your court",
    "the best of both worlds",
    "the best thing since sliced bread",
    "the bigger they are the harder they fall",
    "the early bird catches the worm",
    "the elephant in the room",
    "the last straw",
    "the tip of the iceberg",
    "the whole nine yards",
    "think outside the box",
    "time flies when you're having fun",
    "time is money",
    "to each his own",
    "under the weather",
    "when it rains it pours",
    "you can't judge a book by its cover",
    // ... extendable
];
```

### Matching Strategy

**Exact match** (case-insensitive):
1. Lowercase the input text
2. For each cliché, check if `text.contains(cliché)`
3. Find all occurrences and record byte offsets

```rust
pub fn detect_cliches(text: &str) -> Result<ClicheResult> {
    let lower = text.to_lowercase();
    let mut instances = Vec::new();

    for &cliche in CLICHES {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(cliche) {
            let offset = start + pos;
            instances.push(ClicheInstance {
                phrase: text[offset..offset + cliche.len()].to_string(),
                offset,
                canonical: cliche.to_string(),
            });
            start = offset + cliche.len();
        }
    }

    let count = instances.len();
    Ok(ClicheResult { instances, count })
}
```

## Filter Word Detection (`filter_words.rs`)

### Default Filter Word List

```rust
static FILTER_WORDS: &[&str] = &[
    "just", "really", "very", "quite", "rather",
    "somewhat", "somehow", "perhaps", "basically",
    "actually", "literally", "definitely", "certainly",
    "probably", "simply", "extremely", "absolutely",
    "totally", "completely", "utterly",
];
```

### Detection Logic

```rust
pub fn detect_filter_words(text: &str) -> Result<FilterWordResult> {
    let words = split_words(text);
    let sentences = split_sentences(text);

    if words.is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let mut instances = Vec::new();

    for &word_ref in &words {
        let lower = word_ref.to_lowercase();
        if FILTER_WORDS.contains(&lower.as_str()) {
            // Find containing sentence
            let word_offset = word_ref.as_ptr() as usize - text.as_ptr() as usize;
            let sentence = sentences
                .iter()
                .find(|s| {
                    let s_start = s.as_ptr() as usize - text.as_ptr() as usize;
                    let s_end = s_start + s.len();
                    word_offset >= s_start && word_offset < s_end
                })
                .map(|s| s.to_string())
                .unwrap_or_default();

            instances.push(FilterWordInstance {
                word: word_ref.to_string(),
                offset: word_offset,
                sentence,
            });
        }
    }

    let count = instances.len();
    let percentage = (count as f64 / words.len() as f64) * 100.0;

    Ok(FilterWordResult {
        instances,
        count,
        percentage,
    })
}
```

**Example**:

| Input | Filter Words Found |
|-------|--------------------|
| "She just really wanted to go." | "just", "really" |
| "The results were very clear." | "very" |
| "He ran quickly." | (none) |
