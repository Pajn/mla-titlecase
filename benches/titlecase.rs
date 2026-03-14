#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mla_titlecase::{titlecase_mla, titlecase_with_options, NameParticlePolicy, TitleCaseOptions};

fn bench_titlecase(c: &mut Criterion) {
    c.bench_function("titlecase_short", |b| {
        b.iter(|| black_box(titlecase_mla("the wind in the willows")))
    });

    c.bench_function("titlecase_hyphen_heavy", |b| {
        b.iter(|| {
            black_box(titlecase_mla("state-of-the-art tools for twenty-first-century readers"))
        })
    });

    let options = TitleCaseOptions {
        name_particle_policy: NameParticlePolicy::Heuristic,
        ..TitleCaseOptions::default()
    };
    c.bench_function("titlecase_name_particles", |b| {
        b.iter(|| {
            black_box(titlecase_with_options("ludwig van beethoven and the art of fugue", &options))
        })
    });
}

criterion_group!(benches, bench_titlecase);
criterion_main!(benches);
