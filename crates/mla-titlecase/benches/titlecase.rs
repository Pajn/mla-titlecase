#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mla_titlecase::{
    titlecase_into, titlecase_mla, titlecase_with_options, ExternalLexicons, LocaleProfile,
    NameParticlePolicy, SmallWordPolicy, TitleCaseOptions,
};

/// A representative batch of titles for the bulk-processing benchmarks.
const BATCH: &[&str] = &[
    "the wind in the willows",
    "love in the time of cholera",
    "a by-product of war",
    "preface: the return of sherlock holmes",
    "state-of-the-art design",
    "rock 'n' roll forever",
    "what dreams are made of: a study",
    "the man who wasn't there",
    "miracle on 34th street",
    "dancing among the stars",
];

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

    let mut particle_options = TitleCaseOptions::default();
    particle_options.name_particle_policy = NameParticlePolicy::Heuristic;
    c.bench_function("titlecase_name_particles", |b| {
        b.iter(|| {
            black_box(titlecase_with_options(
                "ludwig van beethoven and the art of fugue",
                &particle_options,
            ))
        })
    });

    let mut lowercase_options = TitleCaseOptions::default();
    lowercase_options.small_word_policy = SmallWordPolicy::AlwaysLowercase;
    c.bench_function("titlecase_repeated_hot_path", |b| {
        b.iter(|| {
            for _ in 0..50 {
                black_box(titlecase_with_options(LONG_TITLE, &lowercase_options));
            }
        })
    });

    let dutch_options = TitleCaseOptions::with_locale(LocaleProfile::Dutch);
    c.bench_function("titlecase_dutch_locale", |b| {
        b.iter(|| {
            black_box(titlecase_with_options("ijsselmeer and jan van der heijden", &dutch_options))
        })
    });

    let mut lexicons = ExternalLexicons::default();
    lexicons.add_multiword_map([("new york city", "New York City")]);
    let phrase_options = TitleCaseOptions::with_external_lexicons(&lexicons);
    c.bench_function("titlecase_multiword_external", |b| {
        b.iter(|| black_box(titlecase_with_options("new york city stories", &phrase_options)))
    });

    // Bulk processing: allocate a fresh String per title...
    let default_options = TitleCaseOptions::default();
    c.bench_function("titlecase_batch_allocating", |b| {
        b.iter(|| {
            for title in BATCH {
                black_box(titlecase_with_options(title, &default_options));
            }
        })
    });

    // ...versus reusing one buffer across the whole batch.
    c.bench_function("titlecase_batch_reused_buffer", |b| {
        let mut buffer = String::new();
        b.iter(|| {
            for title in BATCH {
                titlecase_into(&mut buffer, title, &default_options);
                black_box(&buffer);
            }
        })
    });
}

criterion_group!(benches, bench_titlecase);
criterion_main!(benches);
