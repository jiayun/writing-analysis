#![cfg(feature = "chinese")]

use writing_analysis::*;

#[test]
fn analyze_all_zh_basic() {
    let text = "今天天氣很好。我們去公園散步。孩子們在草地上玩耍。";
    let result = analyze_all_zh(text).unwrap();

    assert!(result.readability.scri.is_finite());
    assert!(result.sentiment.score.is_finite());
    assert!(result.sentence_variety.avg_length > 0.0);
}

#[test]
fn analyze_all_zh_empty_text() {
    let result = analyze_all_zh("");
    assert!(result.is_err());
}

#[test]
fn analyze_all_zh_with_passive_voice() {
    let text = "他被老師罵了。蛋糕被吃掉了。她走回家了。";
    let result = analyze_all_zh(text).unwrap();
    assert!(result.passive_voice.instances.len() >= 2);
    assert!(result.passive_voice.percentage > 50.0);
}

#[test]
fn analyze_all_zh_with_cliches() {
    let text = "眾所周知，這件事毋庸置疑。我們要高度重視。";
    let result = analyze_all_zh(text).unwrap();
    assert!(result.cliches.count >= 2);
}

#[test]
fn analyze_all_zh_with_filter_words() {
    let text = "其實這個問題基本上不難。可能大概就是這樣。";
    let result = analyze_all_zh(text).unwrap();
    assert!(result.filter_words.count >= 2);
}

#[test]
fn analyze_fixture_zh_simple() {
    let text = include_str!("fixtures/zh_simple.txt");
    let result = analyze_all_zh(text).unwrap();

    assert!(result.readability.common_char_ratio > 50.0);
    assert!(result.readability.sentence_count >= 4);
}

#[test]
fn analyze_fixture_zh_passive_heavy() {
    let text = include_str!("fixtures/zh_passive_heavy.txt");
    let result = analyze_all_zh(text).unwrap();

    assert!(result.passive_voice.percentage > 30.0);
}

#[test]
fn analyze_fixture_zh_literary() {
    let text = include_str!("fixtures/zh_literary.txt");
    let result = analyze_all_zh(text).unwrap();

    assert!(result.readability.scri.is_finite());
    assert!(result.sentence_variety.avg_length > 3.0);
}

#[test]
fn passive_voice_exclusions() {
    // 被子 should not be detected as passive voice
    let result = detect_passive_voice_zh("被子很暖和。天氣很冷。").unwrap();
    assert_eq!(result.instances.len(), 0);
}

#[test]
fn passive_voice_wei_suo() {
    let result = detect_passive_voice_zh("這件事為人所知。他很開心。").unwrap();
    assert_eq!(result.instances.len(), 1);
}

#[test]
fn sentiment_positive() {
    let result = analyze_sentiment_zh("這部電影非常好看，真的很精彩。").unwrap();
    assert!(result.score > 0.0);
}

#[test]
fn sentiment_negative_with_negation() {
    let result = analyze_sentiment_zh("今天天氣不好，心情很差。").unwrap();
    assert!(result.score < 0.0);
}

#[test]
fn readability_scores_finite() {
    let text = "科學家們發現了一種新的材料。這種材料具有優異的性能。它可以應用在很多領域。";
    let scores = analyze_readability_zh(text).unwrap();
    assert!(scores.scri.is_finite());
    assert!(scores.avg_sentence_length.is_finite());
    assert!(scores.avg_word_length.is_finite());
    assert!(scores.common_char_ratio.is_finite());
}

#[test]
fn sentence_variety_monotonous() {
    let text = "他去學校。他去公園。他去圖書館。他去醫院。";
    let result = analyze_sentence_variety_zh(text).unwrap();
    assert!(result.structure_variety < 0.5);
}

#[test]
fn sentence_variety_varied() {
    let text = "雨下得很大。橋下躲著一隻貓。沒有人注意到。這是個陰沉的下午。";
    let result = analyze_sentence_variety_zh(text).unwrap();
    assert!(result.structure_variety > 0.5);
}

#[test]
fn cliches_simplified_chinese() {
    let result = detect_cliches_zh("众所周知，这件事不难。").unwrap();
    assert!(result.count >= 1);
}
