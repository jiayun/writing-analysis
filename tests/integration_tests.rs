use writing_analysis::*;

#[test]
fn analyze_all_basic() {
    let text = "The quick brown fox jumped over the lazy dog. \
                It was a beautiful day in the neighborhood. \
                She walked briskly through the morning air.";
    let result = analyze_all(text).unwrap();

    assert!(result.readability.flesch_reading_ease.is_finite());
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
                It is time to bite the bullet and make a decision.";
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

#[test]
fn analyze_fixture_simple_text() {
    let text = include_str!("fixtures/simple_text.txt");
    let result = analyze_all(text).unwrap();

    assert!(result.readability.flesch_reading_ease > 50.0);
    assert!(result.readability.flesch_kincaid_grade < 12.0);
}

#[test]
fn analyze_fixture_passive_heavy() {
    let text = include_str!("fixtures/passive_heavy.txt");
    let result = analyze_all(text).unwrap();

    assert!(result.passive_voice.percentage > 30.0);
}

#[test]
fn analyze_fixture_literary_sample() {
    let text = include_str!("fixtures/literary_sample.txt");
    let result = analyze_all(text).unwrap();

    assert!(result.readability.flesch_reading_ease.is_finite());
    assert!(result.sentence_variety.avg_length > 5.0);
}
