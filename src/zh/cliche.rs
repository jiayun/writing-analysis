use crate::cliche::{ClicheInstance, ClicheResult};
use crate::error::{Result, WritingAnalysisError};

/// Chinese clichés: overused idioms, bureaucratic phrases, and writing clichés.
static CLICHES_ZH: &[&str] = &[
    // 四字成語（overused idioms）
    "眾所周知",
    "不言而喻",
    "一目了然",
    "理所當然",
    "與日俱增",
    "息息相關",
    "不可或缺",
    "有目共睹",
    "顯而易見",
    "毋庸置疑",
    "不可思議",
    "迫不及待",
    "博大精深",
    "源遠流長",
    "任重道遠",
    "刻不容緩",
    "首當其衝",
    "當務之急",
    "責無旁貸",
    "勢在必行",
    "義不容辭",
    "大勢所趨",
    "有口皆碑",
    "舉足輕重",
    "至關重要",
    "密不可分",
    "歸根結底",
    "行之有效",
    "與時俱進",
    "不遺餘力",
    // 簡體對應
    "众所周知",
    "不可或缺",
    "显而易见",
    "毋庸置疑",
    "迫不及待",
    "博大精深",
    "源远流长",
    "任重道远",
    "刻不容缓",
    "首当其冲",
    "当务之急",
    "责无旁贷",
    "势在必行",
    "义不容辞",
    "大势所趋",
    "有口皆碑",
    "举足轻重",
    "至关重要",
    "归根结底",
    "与时俱进",
    "不遗余力",
    // 套話 / 官話（bureaucratic clichés）
    "高度重視",
    "深入貫徹",
    "全面落實",
    "積極推進",
    "不斷加強",
    "進一步提高",
    "認真學習",
    "堅定不移",
    "統籌兼顧",
    "開拓創新",
    "求真務實",
    "與時俱進",
    // 簡體對應
    "高度重视",
    "深入贯彻",
    "全面落实",
    "积极推进",
    "不断加强",
    "进一步提高",
    "认真学习",
    "坚定不移",
    "统筹兼顾",
    "开拓创新",
    // 寫作陳腔（writing clichés）
    "在這個過程中",
    "值得一提的是",
    "不得不說",
    "毫無疑問",
    "換言之",
    "總而言之",
    "由此可見",
    "綜上所述",
    "不難發現",
    "不難看出",
    "言歸正傳",
    "長話短說",
    // 簡體對應
    "在这个过程中",
    "值得一提的是",
    "毫无疑问",
    "换言之",
    "总而言之",
    "综上所述",
    "不难发现",
    "不难看出",
    "言归正传",
    "长话短说",
];

/// Detect clichés in Chinese text.
pub fn detect_cliches_zh(text: &str) -> Result<ClicheResult> {
    if text.trim().is_empty() {
        return Err(WritingAnalysisError::EmptyText);
    }

    let mut instances = Vec::new();

    for &cliche in CLICHES_ZH {
        let mut start = 0;
        while let Some(pos) = text[start..].find(cliche) {
            let offset = start + pos;
            instances.push(ClicheInstance {
                phrase: text[offset..offset + cliche.len()].to_string(),
                offset,
                canonical: cliche.to_string(),
            });
            start = offset + cliche.len();
        }
    }

    instances.sort_by_key(|i| i.offset);
    // Deduplicate by offset (traditional/simplified may overlap)
    instances.dedup_by_key(|i| i.offset);

    let count = instances.len();
    Ok(ClicheResult { instances, count })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_idiom_cliche() {
        let result = detect_cliches_zh("這件事眾所周知，不必多說。").unwrap();
        assert!(result.count >= 1);
        assert_eq!(result.instances[0].canonical, "眾所周知");
    }

    #[test]
    fn detect_bureaucratic_cliche() {
        let result = detect_cliches_zh("我們要高度重視這個問題。").unwrap();
        assert!(result.count >= 1);
    }

    #[test]
    fn detect_writing_cliche() {
        let result = detect_cliches_zh("值得一提的是，這個方案很有效。").unwrap();
        assert!(result.count >= 1);
    }

    #[test]
    fn detect_simplified_cliche() {
        let result = detect_cliches_zh("这件事众所周知。").unwrap();
        assert!(result.count >= 1);
    }

    #[test]
    fn no_cliches() {
        let result = detect_cliches_zh("量子處理器達到了驚人的吞吐量。").unwrap();
        assert_eq!(result.count, 0);
    }

    #[test]
    fn multiple_cliches() {
        let result = detect_cliches_zh("眾所周知，這件事毋庸置疑。").unwrap();
        assert!(result.count >= 2);
    }

    #[test]
    fn empty_text_error() {
        let result = detect_cliches_zh("");
        assert!(result.is_err());
    }
}
