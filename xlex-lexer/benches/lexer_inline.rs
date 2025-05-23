use criterion::{black_box, criterion_group, criterion_main, Criterion};

use xlex_lexer::lexer::{Config, LexerInline, DefaultClassifier};

fn bench_inline_lexer_with_defaults(c: &mut Criterion) {
    let input = "hello 123 world! 💥 привет $S%^& asd\n".repeat(100_000);
    let config = Config::default();
    let classifier = DefaultClassifier;

    c.bench_function("lexer_inline_with_defaults", |b| {
        b.iter(|| {
            let lexer = LexerInline::new(&config, &classifier, black_box(&input));
            
            for _ in lexer {}
        })
    });
}

criterion_group!(benches, bench_inline_lexer_with_defaults);
criterion_main!(benches);