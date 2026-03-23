#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mla_titlecase::{fst_store, json_store, LexiconPlugin, PluginMetadata, PluginPayload};
use tempfile::tempdir;

const LARGE_PLUGIN_WORDS: usize = 25_000;

fn bench_plugin_load(c: &mut Criterion) {
    let word_set_plugin = LexiconPlugin {
        metadata: PluginMetadata::new("bench", "MIT"),
        payload: PluginPayload::WordSet {
            words: (0..LARGE_PLUGIN_WORDS).map(synthetic_word).collect(),
        },
    };
    let temp = tempdir().expect("tempdir");
    let json_path = temp.path().join("plugin.json");
    let fst_path = temp.path().join("plugin.mlatl");
    json_store::save_json_plugin(&json_path, &word_set_plugin).expect("save json plugin");
    fst_store::save_fst_plugin(&fst_path, &word_set_plugin).expect("save fst plugin");

    c.bench_function("load_json_plugin_large_word_set", |b| {
        b.iter(|| json_store::load_json_plugin(&json_path).expect("load json plugin"))
    });
    c.bench_function("load_fst_plugin_large_word_set", |b| {
        b.iter(|| fst_store::load_fst_plugin(&fst_path).expect("load fst plugin"))
    });
    c.bench_function("mmap_fst_plugin_large_word_set", |b| {
        b.iter(|| fst_store::mmap_fst_plugin(&fst_path).expect("mmap fst plugin"))
    });

    let multiword_plugin = LexiconPlugin {
        metadata: PluginMetadata::new("bench-multiword", "MIT"),
        payload: PluginPayload::MultiwordMap {
            entries: (0..LARGE_PLUGIN_WORDS)
                .map(|index| {
                    let phrase = synthetic_phrase(index);
                    mla_titlecase::MapEntry { key: phrase.clone(), value: phrase }
                })
                .collect(),
        },
    };
    let multiword_json_path = temp.path().join("multiword.json");
    let multiword_fst_path = temp.path().join("multiword.mlatl");
    json_store::save_json_plugin(&multiword_json_path, &multiword_plugin)
        .expect("save multiword json plugin");
    fst_store::save_fst_plugin(&multiword_fst_path, &multiword_plugin)
        .expect("save multiword fst plugin");

    c.bench_function("load_json_plugin_large_multiword_map", |b| {
        b.iter(|| json_store::load_json_plugin(&multiword_json_path).expect("load multiword json"))
    });
    c.bench_function("load_fst_plugin_large_multiword_map", |b| {
        b.iter(|| fst_store::load_fst_plugin(&multiword_fst_path).expect("load multiword fst"))
    });
    c.bench_function("mmap_fst_plugin_large_multiword_map", |b| {
        b.iter(|| fst_store::mmap_fst_plugin(&multiword_fst_path).expect("mmap multiword fst"))
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

fn synthetic_phrase(index: usize) -> String {
    format!(
        "{} {} {}",
        synthetic_word(index * 3),
        synthetic_word(index * 3 + 1),
        synthetic_word(index * 3 + 2)
    )
}

criterion_group!(benches, bench_plugin_load);
criterion_main!(benches);
