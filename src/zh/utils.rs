/// Split Chinese text into sentences by sentence-ending punctuation.
///
/// Splits on: `。！？；.!?` (full-width and ASCII).
pub fn split_sentences_zh(text: &str) -> Vec<&str> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

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

/// Count CJK Unified Ideographs (漢字) in text.
pub fn count_hanzi(text: &str) -> usize {
    text.chars()
        .filter(|&c| ('\u{4e00}'..='\u{9fff}').contains(&c))
        .count()
}

/// Check if a character is a CJK ideograph.
pub fn is_hanzi(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic_chinese() {
        let sentences = split_sentences_zh("你好。世界！");
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "你好。");
        assert_eq!(sentences[1], "世界！");
    }

    #[test]
    fn split_with_semicolon() {
        let sentences = split_sentences_zh("第一句；第二句。");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn split_mixed_punctuation() {
        let sentences = split_sentences_zh("Hello世界。This is test。");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn split_empty() {
        let sentences = split_sentences_zh("");
        assert!(sentences.is_empty());
    }

    #[test]
    fn split_no_punctuation() {
        let sentences = split_sentences_zh("沒有標點的句子");
        assert_eq!(sentences.len(), 1);
    }

    #[test]
    fn split_with_quotation() {
        let sentences = split_sentences_zh("他說：「你好。」她笑了。");
        assert_eq!(sentences.len(), 2);
    }

    #[test]
    fn count_hanzi_basic() {
        assert_eq!(count_hanzi("你好世界"), 4);
        assert_eq!(count_hanzi("Hello世界"), 2);
        assert_eq!(count_hanzi("Hello World"), 0);
    }

    #[test]
    fn count_hanzi_with_punctuation() {
        assert_eq!(count_hanzi("你好！世界。"), 4);
    }
}
