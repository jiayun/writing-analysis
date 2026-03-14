use unicode_segmentation::UnicodeSegmentation;

/// Raw text statistics used by readability formulas.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStatistics {
    pub sentence_count: usize,
    pub word_count: usize,
    pub syllable_count: usize,
    pub character_count: usize,
    pub polysyllable_count: usize,
}

static ABBREVIATIONS: &[&str] = &[
    "Mr.", "Mrs.", "Ms.", "Dr.", "Prof.", "Sr.", "Jr.", "St.", "Inc.", "Ltd.", "Corp.", "vs.",
    "etc.", "e.g.", "i.e.", "Vol.", "Dept.", "Est.", "Govt.", "No.",
];

/// Split text into sentences using punctuation heuristics.
pub fn split_sentences(text: &str) -> Vec<&str> {
    find_sentence_spans(text)
}

/// Find sentence spans directly in the source text.
fn find_sentence_spans(text: &str) -> Vec<&str> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let mut sentences = Vec::new();
    let mut start = 0;
    let bytes = text.as_bytes();
    let len = bytes.len();

    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if b == b'.' || b == b'!' || b == b'?' {
            // Check for ellipsis: consume all consecutive dots
            if b == b'.' {
                while i + 1 < len && bytes[i + 1] == b'.' {
                    i += 1;
                }
            }

            // Check if this is an abbreviation
            if b == b'.' && is_abbreviation(text, i) {
                i += 1;
                continue;
            }

            // Check if this is a decimal number (digit.digit)
            if b == b'.' && i > 0 && i + 1 < len && bytes[i - 1].is_ascii_digit() && bytes[i + 1].is_ascii_digit() {
                i += 1;
                continue;
            }

            // Look ahead: is there whitespace followed by an uppercase letter, or is this the end?
            let after = i + 1;
            if after >= len {
                // End of text — this is a sentence boundary
                let sentence = text[start..=i].trim();
                if !sentence.is_empty() {
                    sentences.push(sentence);
                }
                start = after;
            } else {
                // Check for whitespace then uppercase
                let mut j = after;
                while j < len && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < len && bytes[j].is_ascii_uppercase() && j > after {
                    // Sentence boundary
                    let sentence = text[start..=i].trim();
                    if !sentence.is_empty() {
                        sentences.push(sentence);
                    }
                    start = j;
                }
            }
        }
        i += 1;
    }

    // Remaining text
    let remaining = text[start..].trim();
    if !remaining.is_empty() {
        sentences.push(remaining);
    }

    sentences
}

/// Check if the period at position `dot_pos` is part of an abbreviation.
fn is_abbreviation(text: &str, dot_pos: usize) -> bool {
    for abbr in ABBREVIATIONS {
        let abbr_len = abbr.len();
        if dot_pos + 1 >= abbr_len {
            let candidate_start = dot_pos + 1 - abbr_len;
            let candidate = &text[candidate_start..=dot_pos];
            if candidate.eq_ignore_ascii_case(abbr) {
                return true;
            }
        }
    }
    false
}

/// Split text into words using Unicode word boundaries.
pub fn split_words(text: &str) -> Vec<&str> {
    text.unicode_words().collect()
}

/// Count syllables in an English word using vowel-group heuristics.
pub fn count_syllables(word: &str) -> usize {
    let word_lower = word.to_lowercase();
    let chars: Vec<char> = word_lower.chars().filter(|c| c.is_alphabetic()).collect();

    if chars.is_empty() {
        return 1;
    }

    let vowels = "aeiouy";
    let mut count: usize = 0;
    let mut prev_vowel = false;

    for &ch in &chars {
        let is_vowel = vowels.contains(ch);
        if is_vowel && !prev_vowel {
            count += 1;
        }
        prev_vowel = is_vowel;
    }

    // Silent-e adjustment: trailing 'e' after a consonant
    if chars.len() > 2 {
        if let Some(&last) = chars.last() {
            if last == 'e' {
                let second_last = chars[chars.len() - 2];
                if !vowels.contains(second_last) {
                    // But not if it's "-le" after a consonant (e.g., "table")
                    if chars.len() >= 3 {
                        let third_last = chars[chars.len() - 3];
                        if second_last == 'l' && !vowels.contains(third_last) {
                            // consonant + le: keep the syllable (e.g., "ta-ble")
                            // don't subtract
                        } else {
                            count = count.saturating_sub(1);
                        }
                    } else {
                        count = count.saturating_sub(1);
                    }
                }
            }
        }
    }

    count.max(1)
}

/// Count only alphabetic characters (letters).
pub fn count_characters(text: &str) -> usize {
    text.chars().filter(|c| c.is_alphabetic()).count()
}

/// Compute aggregate text statistics.
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
    fn split_sentences_no_final_punctuation() {
        let sentences = split_sentences("Hello world");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn split_sentences_ellipsis() {
        let sentences = split_sentences("Wait... What happened?");
        assert_eq!(sentences.len(), 2);
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
    fn count_syllables_table() {
        assert_eq!(count_syllables("table"), 2);
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
