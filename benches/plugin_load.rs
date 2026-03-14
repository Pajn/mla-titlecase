#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mla_titlecase::{fst_store, json_store, LexiconPlugin, PluginMetadata, PluginPayload};
use tempfile::tempdir;

const LARGE_PLUGIN_WORDS: usize = 25_000;

fn bench_plugin_load(c: &mut Criterion) {
    let plugin = LexiconPlugin {
        metadata: PluginMetadata::new("bench", "MIT"),
        payload: PluginPayload::WordSet {
            words: (0..LARGE_PLUGIN_WORDS).map(synthetic_word).collect(),
        },
    };
    let temp = tempdir().expect("tempdir");
    let json_path = temp.path().join("plugin.json");
    let fst_path = temp.path().join("plugin.mlatl");
    json_store::save_json_plugin(&json_path, &plugin).expect("save json plugin");
    fst_store::save_fst_plugin(&fst_path, &plugin).expect("save fst plugin");

    c.bench_function("load_json_plugin_large_word_set", |b| {
        b.iter(|| json_store::load_json_plugin(&json_path).expect("load json plugin"))
    });
    c.bench_function("load_fst_plugin_large_word_set", |b| {
        b.iter(|| fst_store::load_fst_plugin(&fst_path).expect("load fst plugin"))
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

criterion_group!(benches, bench_plugin_load);
criterion_main!(benches);
