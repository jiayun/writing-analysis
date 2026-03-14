use criterion::{criterion_group, criterion_main, Criterion};

fn bench_readability(c: &mut Criterion) {
    let text = "The quick brown fox jumped over the lazy dog. ".repeat(100);
    c.bench_function("readability_500_words", |b| {
        b.iter(|| writing_analysis::analyze_readability(&text).unwrap())
    });
}

fn bench_passive_voice(c: &mut Criterion) {
    let text = "The ball was thrown by the boy. She ran quickly. ".repeat(50);
    c.bench_function("passive_voice_100_sentences", |b| {
        b.iter(|| writing_analysis::detect_passive_voice(&text).unwrap())
    });
}

fn bench_sentiment(c: &mut Criterion) {
    let text = "I love this wonderful beautiful amazing great fantastic day. ".repeat(50);
    c.bench_function("sentiment_500_words", |b| {
        b.iter(|| writing_analysis::analyze_sentiment(&text).unwrap())
    });
}

fn bench_analyze_all(c: &mut Criterion) {
    let text = include_str!("../tests/fixtures/simple_text.txt");
    c.bench_function("analyze_all_fixture", |b| {
        b.iter(|| writing_analysis::analyze_all(text).unwrap())
    });
}

fn bench_analyze_all_large(c: &mut Criterion) {
    let text = "The quick brown fox jumped over the lazy dog. \
                She was very happy about the wonderful results. \
                At the end of the day, it was a great success. "
        .repeat(100);
    c.bench_function("analyze_all_10kb", |b| {
        b.iter(|| writing_analysis::analyze_all(&text).unwrap())
    });
}

criterion_group!(
    benches,
    bench_readability,
    bench_passive_voice,
    bench_sentiment,
    bench_analyze_all,
    bench_analyze_all_large,
);
criterion_main!(benches);
