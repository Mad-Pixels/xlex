use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Cursor;

use xlex_lexer::lexer::{Config, LexerStream, DefaultClassifier};

fn bench_stream_lexer_with_defaults(c: &mut Criterion) {
    let input = "hello 123 world! 💥 привет $S%^& asd\n".repeat(100_000);
    let config = Config::default();
    let classifier = DefaultClassifier;

    c.bench_function("lexer_stream_with_defaults", |b| {
        b.iter(|| {
            let reader = Cursor::new(black_box(&input));
            let lexer = LexerStream::new(&config, &classifier, reader);
            
            for _ in lexer {}
        })
    });
}

criterion_group!(benches, bench_stream_lexer_with_defaults);
criterion_main!(benches);