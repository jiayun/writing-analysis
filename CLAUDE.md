# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust library crate (`writing-analysis`) providing rule-based text analysis and NLP tools. No ML models — all data is embedded at compile time. Published to crates.io.

## Common Commands

```sh
cargo build                        # Build (English only)
cargo build --features chinese     # Build with Chinese support
cargo test                         # Run all tests (English only)
cargo test --features chinese      # Run all tests including Chinese
cargo test <test_name>             # Run a single test
cargo clippy --all-targets -- -D warnings  # Lint (CI uses this)
cargo bench                        # Run benchmarks
```

CI runs `cargo clippy --all-targets -- -D warnings` then `cargo test` on ubuntu-latest.

## Architecture

The library exposes 6 independent analyzers for English, mirrored for Chinese behind the `chinese` feature flag:

| Analyzer | English module | Chinese module (`src/zh/`) |
|----------|---------------|---------------------------|
| Readability | `readability.rs` | `readability.rs` (SCRI) |
| Passive voice | `passive_voice.rs` | `passive_voice.rs` (被/受/遭/給) |
| Cliché detection | `cliche.rs` | `cliche.rs` |
| Filter words | `filter_words.rs` | `filter_words.rs` |
| Sentiment | `sentiment.rs` (AFINN-111) | `sentiment.rs` (negation/intensifier) |
| Sentence variety | `sentence_variety.rs` | `sentence_variety.rs` |

**Key shared modules:**
- `src/lib.rs` — public API surface, `analyze_all()` aggregator, re-exports
- `src/utils.rs` / `src/zh/utils.rs` — text statistics, sentence splitting
- `src/lexicon.rs` / `src/zh/lexicon.rs` — embedded word lists
- `src/zh/segmenter.rs` — Chinese word segmentation (wraps `opencc-jieba-rs`)
- `src/error.rs` — `WritingAnalysisError` and `Result` type

**Pattern:** Each analyzer is a standalone module with a public function (e.g., `detect_cliches`) returning a result struct. The `analyze_all` / `analyze_all_zh` functions aggregate all analyzers. All functions return `Result<T>` and error on empty input.

**Data:** English sentiment lexicon is in `data/afinn-111.txt`. Chinese lexicons and word lists are embedded inline in their respective `lexicon.rs` files.

## Testing

- Unit tests are inside each module
- Integration tests in `tests/integration_tests.rs` with fixtures in `tests/fixtures/`
- Benchmarks in `benches/analysis_benchmark.rs` using Criterion

## Feature Flags

- `default = []` — English-only analysis
- `chinese` — enables `src/zh/` module, adds `opencc-jieba-rs` dependency

## Part of Scrivener Rust Ecosystem

Related crates: `scrivener-rtf` (RTF parser), `scrivener` (`.scriv` project reader/writer).
