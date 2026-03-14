#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mla_titlecase::{fst_store, json_store, LexiconPlugin, PluginMetadata, PluginPayload};
use tempfile::tempdir;

fn bench_plugin_load(c: &mut Criterion) {
    let plugin = LexiconPlugin {
        metadata: PluginMetadata::new("bench", "MIT"),
        payload: PluginPayload::WordSet {
            words: vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()],
        },
    };
    let temp = tempdir().expect("tempdir");
    let json_path = temp.path().join("plugin.json");
    let fst_path = temp.path().join("plugin.mlatl");
    json_store::save_json_plugin(&json_path, &plugin).expect("save json plugin");
    fst_store::save_fst_plugin(&fst_path, &plugin).expect("save fst plugin");

    c.bench_function("load_json_plugin", |b| {
        b.iter(|| json_store::load_json_plugin(&json_path).expect("load json plugin"))
    });
    c.bench_function("load_fst_plugin", |b| {
        b.iter(|| fst_store::load_fst_plugin(&fst_path).expect("load fst plugin"))
    });
}

criterion_group!(benches, bench_plugin_load);
criterion_main!(benches);
