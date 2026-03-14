#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mla_titlecase::{titlecase_with_options, ExternalLexicons, TitleCaseOptions};

fn bench_lookup(c: &mut Criterion) {
    let mut external = ExternalLexicons::default();
    external.add_word_set(["and", "the", "of", "in", "under", "through"]);
    external.add_protected_spellings([("github", "GitHub"), ("copilot", "Copilot")]);
    let options = TitleCaseOptions::with_external_lexicons(&external);

    c.bench_function("lookup_external_words", |b| {
        b.iter(|| black_box(titlecase_with_options("github and copilot in practice", &options)))
    });
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
