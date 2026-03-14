# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-14

### Added

- **English analysis**
  - Readability scoring: Flesch-Kincaid, Flesch Reading Ease, SMOG, Coleman-Liau, ARI
  - Passive voice detection with 90+ irregular past participles and adjective exclusion list
  - Cliche detection with 85 built-in English cliches
  - Filter word detection for 20 common filler words
  - Sentiment analysis using AFINN-111 lexicon (2400+ words)
  - Sentence variety analysis: length stats, starter diversity, structure variety score
  - `analyze_all()` aggregated analysis function

- **Chinese analysis** (feature flag: `chinese`)
  - Chinese readability: SCRI (Simple Chinese Readability Index) and common character ratio (top 3000)
  - Passive voice detection: 被/受/遭/給 + 為...所 patterns with exclusion list
  - Cliche detection: ~100 entries covering overused idioms, bureaucratic phrases, writing cliches
  - Filter word detection: ~26 common filler words
  - Sentiment analysis: 500+ word lexicon with negation and intensifier handling
  - Sentence variety analysis using word segmentation
  - Word segmentation via opencc-jieba-rs (supports both Traditional and Simplified Chinese)
  - `analyze_all_zh()` aggregated analysis function

[0.1.0]: https://github.com/jiayun/writing-analysis/releases/tag/v0.1.0
