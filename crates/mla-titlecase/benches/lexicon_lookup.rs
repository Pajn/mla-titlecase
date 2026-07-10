#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mla_titlecase::{titlecase_with_options, ExternalLexicons, SmallWordPolicy, TitleCaseOptions};

const LARGE_WORD_COUNT: usize = 20_000;

fn bench_lookup(c: &mut Criterion) {
    let mut external = ExternalLexicons::default();
    external.add_word_set((0..LARGE_WORD_COUNT).map(synthetic_word));
    external.add_canonical_map(
        (0..5_000).map(|index| (synthetic_word(index + 30_000), format!("Canon{index:05}"))),
    );
    external.add_protected_spellings(
        (0..5_000).map(|index| (synthetic_word(index + 60_000), format!("Brand{index:05}"))),
    );

    let mut options = TitleCaseOptions::default();
    options.external_lexicons = Some(&external);
    options.small_word_policy = SmallWordPolicy::AlwaysLowercase;

    let protected_hit = synthetic_word(60_042);
    let canonical_hit = synthetic_word(30_077);
    let word_set_hit = synthetic_word(10);
    let hit_title = format!("{protected_hit} and {canonical_hit} with {word_set_hit} in practice");
    let miss_title = "unlisted tools and ordinary names for title casing";

    c.bench_function("lookup_large_external_hits", |b| {
        b.iter(|| black_box(titlecase_with_options(&hit_title, &options)))
    });

    c.bench_function("lookup_large_external_misses", |b| {
        b.iter(|| black_box(titlecase_with_options(miss_title, &options)))
    });
}

fn synthetic_word(index: usize) -> String {
    let mut value = (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xA5A5_A5A5_5A5A_5A5A;
    let mut word = String::with_capacity(12);
    for _ in 0..12 {
        let letter = ((value & 0x1f) as u8 % 26) + b'a';
        word.push(char::from(letter));
        value ^= value >> 7;
        value = value.rotate_left(11).wrapping_mul(0x2545_F491_4F6C_DD1D);
    }
    word
}

criterion_group!(benches, bench_lookup);
criterion_main!(benches);
