use crate::error::{Result, WritingAnalysisError};

/// Result of cliché detection.
#[derive(Debug, Clone, PartialEq)]
pub struct ClicheResult {
    /// All detected cliché instances
    pub instances: Vec<ClicheInstance>,
    /// Total number of clichés found
    pub count: usize,
}

/// A single cliché occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct ClicheInstance {
    /// The matched cliché phrase as it appears in the text
    pub phrase: String,
    /// Byte offset in the original text
    pub offset: usize,
    /// The canonical form from the built-in list
    pub canonical: String,
}

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
    "easier said than done",
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
    "read between the lines",
    "reinvent the wheel",
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
];

/// Detect clichés in text.
pub fn detect_cliches(text: &str) -> Result<ClicheResult> {
    if text.trim().is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

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

    // Sort by offset for consistent ordering
    instances.sort_by_key(|i| i.offset);

    let count = instances.len();
    Ok(ClicheResult { instances, count })
}

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
        assert_eq!(result.count, 2);
    }

    #[test]
    fn no_cliches() {
        let result =
            detect_cliches("The quantum processor achieved remarkable throughput.").unwrap();
        assert_eq!(result.count, 0);
    }

    #[test]
    fn case_insensitive_match() {
        let result = detect_cliches("Think Outside The Box to solve this.").unwrap();
        assert_eq!(result.count, 1);
    }

    #[test]
    fn empty_text_error() {
        let result = detect_cliches("");
        assert!(result.is_err());
    }
}
