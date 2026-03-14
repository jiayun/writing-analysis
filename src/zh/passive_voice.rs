use regex::Regex;
use std::sync::LazyLock;

use crate::error::{Result, WritingAnalysisError};
use crate::passive_voice::{PassiveInstance, PassiveVoiceResult};
use crate::zh::utils::split_sentences_zh;

static PASSIVE_ZH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(被|受|遭|給)\S{1,4}(了|過|著)?").unwrap()
});

static PASSIVE_WEI_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"為\S{1,6}所\S{1,4}").unwrap()
});

/// Non-passive usages of 被/受 etc.
static EXCLUSIONS: &[&str] = &[
    "被子", "被告", "被動", "被褥", "被套", "受益", "受眾",
];

/// Detect passive voice in Chinese text.
pub fn detect_passive_voice_zh(text: &str) -> Result<PassiveVoiceResult> {
    let sentences = split_sentences_zh(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    let mut instances = Vec::new();
    let mut sentences_with_passive = 0;
    let text_start = text.as_ptr() as usize;

    for sentence in &sentences {
        let mut found_in_sentence = false;
        let sentence_start = sentence.as_ptr() as usize - text_start;

        // Check 被/受/遭/給 patterns
        for mat in PASSIVE_ZH_RE.find_iter(sentence) {
            let phrase = mat.as_str();
            if EXCLUSIONS.iter().any(|&exc| phrase.contains(exc)) {
                continue;
            }
            let offset = sentence_start + mat.start();
            instances.push(PassiveInstance {
                phrase: phrase.to_string(),
                offset,
                sentence: sentence.to_string(),
            });
            found_in_sentence = true;
        }

        // Check 為...所 pattern
        for mat in PASSIVE_WEI_RE.find_iter(sentence) {
            let offset = sentence_start + mat.start();
            instances.push(PassiveInstance {
                phrase: mat.as_str().to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_bei_passive() {
        let result = detect_passive_voice_zh("他被老師罵了。").unwrap();
        assert_eq!(result.instances.len(), 1);
        assert!(result.instances[0].phrase.contains("被"));
    }

    #[test]
    fn detect_wei_suo_passive() {
        let result = detect_passive_voice_zh("這件事為人所知。").unwrap();
        assert_eq!(result.instances.len(), 1);
        assert!(result.instances[0].phrase.contains("為"));
    }

    #[test]
    fn exclude_beizi() {
        let result = detect_passive_voice_zh("被子很暖和。").unwrap();
        assert_eq!(result.instances.len(), 0);
    }

    #[test]
    fn exclude_beidong() {
        let result = detect_passive_voice_zh("他很被動。").unwrap();
        assert_eq!(result.instances.len(), 0);
    }

    #[test]
    fn no_passive() {
        let result = detect_passive_voice_zh("我今天去學校。").unwrap();
        assert_eq!(result.instances.len(), 0);
        assert_eq!(result.percentage, 0.0);
    }

    #[test]
    fn multiple_passive() {
        let text = "他被罵了。她被打了。我去上學了。";
        let result = detect_passive_voice_zh(text).unwrap();
        assert_eq!(result.instances.len(), 2);
        assert!((result.percentage - 66.666).abs() < 1.0);
    }

    #[test]
    fn passive_zao() {
        let result = detect_passive_voice_zh("他遭人陷害了。").unwrap();
        assert!(!result.instances.is_empty());
    }
}
