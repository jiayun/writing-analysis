use regex::Regex;
use std::sync::LazyLock;

use crate::error::{Result, WritingAnalysisError};
use crate::utils::split_sentences;

/// Result of passive voice detection.
#[derive(Debug, Clone, PartialEq)]
pub struct PassiveVoiceResult {
    /// All detected passive voice instances
    pub instances: Vec<PassiveInstance>,
    /// Percentage of sentences containing passive voice (0.0-100.0)
    pub percentage: f64,
}

/// A single passive voice occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct PassiveInstance {
    /// The matched passive phrase (e.g., "was written")
    pub phrase: String,
    /// Byte offset in the original text
    pub offset: usize,
    /// The full sentence containing the passive voice
    pub sentence: String,
}

static IRREGULAR_PAST_PARTICIPLES: &[&str] = &[
    "awoken", "been", "born", "beaten", "become", "begun", "bent", "bitten", "blown", "broken",
    "brought", "built", "burnt", "bought", "caught", "chosen", "come", "cost", "cut", "done",
    "drawn", "driven", "drunk", "eaten", "fallen", "felt", "found", "flown", "forgotten",
    "forgiven", "frozen", "given", "gone", "grown", "had", "heard", "hidden", "hit", "held",
    "hurt", "kept", "known", "laid", "led", "left", "lent", "let", "lain", "lost", "made",
    "meant", "met", "paid", "put", "read", "ridden", "risen", "run", "said", "seen", "sent",
    "set", "shaken", "shown", "shut", "slept", "slid", "spoken", "spent", "split", "spread", "sung",
    "stood", "stolen", "stuck", "stung", "struck", "sworn", "swept", "swum", "taken", "taught",
    "thought", "thrown", "told", "torn", "understood", "woken", "worn", "wound", "written",
];

static ADJECTIVE_EXCLUSIONS: &[&str] = &[
    "advanced",
    "amazed",
    "associated",
    "attached",
    "bored",
    "complicated",
    "concerned",
    "confused",
    "connected",
    "convinced",
    "dedicated",
    "determined",
    "disappointed",
    "embarrassed",
    "excited",
    "experienced",
    "frustrated",
    "interested",
    "involved",
    "married",
    "organized",
    "overwhelmed",
    "pleased",
    "prepared",
    "related",
    "satisfied",
    "sophisticated",
    "supposed",
    "surprised",
    "tired",
    "used",
];

static PASSIVE_RE: LazyLock<Regex> = LazyLock::new(|| {
    let irregulars = IRREGULAR_PAST_PARTICIPLES.join("|");
    let pattern = format!(
        r"(?i)\b(am|is|are|was|were|be|been|being)\s+(\w+ed|{})\b",
        irregulars
    );
    Regex::new(&pattern).unwrap()
});

/// Detect passive voice in text.
pub fn detect_passive_voice(text: &str) -> Result<PassiveVoiceResult> {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return Err(WritingAnalysisError::NoSentences);
    }

    let mut instances = Vec::new();
    let mut sentences_with_passive = 0;
    let text_start = text.as_ptr() as usize;

    for sentence in &sentences {
        let mut found_in_sentence = false;

        for mat in PASSIVE_RE.find_iter(sentence) {
            let phrase = mat.as_str();

            // Check exclusion list: get the last word (the participle)
            let participle = phrase.split_whitespace().last().unwrap_or("");
            if ADJECTIVE_EXCLUSIONS
                .iter()
                .any(|&exc| participle.eq_ignore_ascii_case(exc))
            {
                continue;
            }

            let sentence_start = sentence.as_ptr() as usize - text_start;
            let offset = sentence_start + mat.start();

            instances.push(PassiveInstance {
                phrase: phrase.to_string(),
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
    }

    #[test]
    fn passive_percentage_calculation() {
        let text = "The ball was thrown. She ran quickly.";
        let result = detect_passive_voice(text).unwrap();
        assert_eq!(result.percentage, 50.0);
    }
}
