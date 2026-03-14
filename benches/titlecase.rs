#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mla_titlecase::{
    titlecase_mla, titlecase_with_options, NameParticlePolicy, SmallWordPolicy, TitleCaseOptions,
};

const SHORT_TITLE: &str = "the wind in the willows";
const LONG_TITLE: &str =
    "preface: the state-of-the-art guide to the history of github and postgres in practice";
const PUNCTUATION_HEAVY: &str =
    "\"notes from the field\": a study in twenty-first-century tools, workflows, and trade-offs";

fn bench_titlecase(c: &mut Criterion) {
    c.bench_function("titlecase_short", |b| b.iter(|| black_box(titlecase_mla(SHORT_TITLE))));

    c.bench_function("titlecase_long", |b| b.iter(|| black_box(titlecase_mla(LONG_TITLE))));

    c.bench_function("titlecase_punctuation_heavy", |b| {
        b.iter(|| black_box(titlecase_mla(PUNCTUATION_HEAVY)))
    });

    let particle_options = TitleCaseOptions {
        name_particle_policy: NameParticlePolicy::Heuristic,
        ..TitleCaseOptions::default()
    };
    c.bench_function("titlecase_name_particles", |b| {
        b.iter(|| {
            black_box(titlecase_with_options(
                "ludwig van beethoven and the art of fugue",
                &particle_options,
            ))
        })
    });

    let lowercase_options = TitleCaseOptions {
        small_word_policy: SmallWordPolicy::AlwaysLowercase,
        ..TitleCaseOptions::default()
    };
    c.bench_function("titlecase_repeated_hot_path", |b| {
        b.iter(|| {
            for _ in 0..50 {
                black_box(titlecase_with_options(LONG_TITLE, &lowercase_options));
            }
        })
    });
}

criterion_group!(benches, bench_titlecase);
criterion_main!(benches);
