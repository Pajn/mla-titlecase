//! Integration tests for the `mla-titlecase` CLI.

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/fixtures").join(name)
}

#[test]
fn lists_sources() {
    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args(["lexicon", "list-sources"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scowl"))
        .stdout(predicate::str::contains("stopwords-iso"))
        .stdout(predicate::str::contains("wikidata"))
        .stdout(predicate::str::contains("wordfreq"));
}

#[test]
fn shows_license_details() {
    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args(["lexicon", "show-license", "stopwords-iso"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stopwords-iso"))
        .stdout(predicate::str::contains("heuristics"));
}

#[test]
fn shows_wikidata_license_details() {
    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args(["lexicon", "show-license", "wikidata"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wikidata"))
        .stdout(predicate::str::contains("CC0"));
}

#[test]
fn fetches_from_local_fixture_and_writes_manifest() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("raw.json");
    let manifest = temp.path().join("raw.manifest.json");

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "fetch",
            "stopwords-iso",
            "--from-file",
            fixture("stopwords-en.json").to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--manifest",
            manifest.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(output.exists());
    assert!(manifest.exists());
}

#[test]
fn prepare_build_inspect_and_diff_plugins() {
    let temp = tempdir().unwrap();
    let raw = temp.path().join("raw.json");
    let raw_manifest = temp.path().join("raw.json.manifest.json");
    let prepared = temp.path().join("prepared.json");
    let json_plugin = temp.path().join("plugin.json");
    let fst_plugin = temp.path().join("plugin.mlatl");

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "fetch",
            "stopwords-iso",
            "--from-file",
            fixture("stopwords-en.json").to_str().unwrap(),
            "--output",
            raw.to_str().unwrap(),
            "--manifest",
            raw_manifest.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "prepare",
            "stopwords-iso",
            "--input",
            raw.to_str().unwrap(),
            "--output",
            prepared.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("prepared 4 stopwords-iso entries"))
        .stdout(predicate::str::contains("4 input records -> 4 normalized entries"));

    let prepared_json: Value = serde_json::from_slice(&std::fs::read(&prepared).unwrap()).unwrap();
    assert_eq!(prepared_json["metadata"]["source_id"], "stopwords-iso");
    assert_eq!(prepared_json["report"]["input_records"], 4);
    assert!(prepared_json["metadata"]["source_url"].as_str().unwrap().starts_with("file://"));

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "build-plugin",
            prepared.to_str().unwrap(),
            "--output",
            json_plugin.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "build-plugin",
            prepared.to_str().unwrap(),
            "--output",
            fst_plugin.to_str().unwrap(),
            "--format",
            "fst",
        ])
        .assert()
        .success();

    let inspect = Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args(["lexicon", "inspect-plugin", json_plugin.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect: Value = serde_json::from_slice(&inspect).unwrap();
    assert_eq!(inspect["format"], "json");
    assert_eq!(inspect["entry_count"], 4);

    let diff = Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "diff-plugin",
            json_plugin.to_str().unwrap(),
            fst_plugin.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let diff: Value = serde_json::from_slice(&diff).unwrap();
    assert_eq!(diff["added"], 0);
    assert_eq!(diff["removed"], 0);
    assert_eq!(diff["changed"], 0);
}

#[test]
fn wordfreq_prepare_requires_acknowledgement() {
    let temp = tempdir().unwrap();
    let prepared = temp.path().join("prepared.json");

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "prepare",
            "wordfreq",
            "--input",
            fixture("wordfreq-mini.txt").to_str().unwrap(),
            "--output",
            prepared.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--acknowledge-cc-by-sa"));
}

#[test]
fn prepare_build_inspect_and_diff_wikidata_plugins() {
    let temp = tempdir().unwrap();
    let raw = temp.path().join("wikidata.json");
    let raw_manifest = temp.path().join("wikidata.json.manifest.json");
    let prepared = temp.path().join("wikidata-prepared.json");
    let json_plugin = temp.path().join("wikidata.json.plugin");
    let fst_plugin = temp.path().join("wikidata.mlatl");

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "fetch",
            "wikidata",
            "--from-file",
            fixture("wikidata-sample.json").to_str().unwrap(),
            "--output",
            raw.to_str().unwrap(),
            "--manifest",
            raw_manifest.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "prepare",
            "wikidata",
            "--input",
            raw.to_str().unwrap(),
            "--output",
            prepared.to_str().unwrap(),
            "--payload-kind",
            "multiword-map",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("prepared 3 wikidata entries"))
        .stdout(predicate::str::contains("3 normalized entries"));

    let prepared_json: Value = serde_json::from_slice(&std::fs::read(&prepared).unwrap()).unwrap();
    assert_eq!(prepared_json["metadata"]["source_id"], "wikidata");
    assert_eq!(prepared_json["payload"]["kind"], "multiword-map");

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "build-plugin",
            prepared.to_str().unwrap(),
            "--output",
            json_plugin.to_str().unwrap(),
            "--format",
            "json",
        ])
        .assert()
        .success();

    Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "build-plugin",
            prepared.to_str().unwrap(),
            "--output",
            fst_plugin.to_str().unwrap(),
            "--format",
            "fst",
        ])
        .assert()
        .success();

    let inspect = Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args(["lexicon", "inspect-plugin", json_plugin.to_str().unwrap(), "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let inspect: Value = serde_json::from_slice(&inspect).unwrap();
    assert_eq!(inspect["payload_kind"], "multiword-map");
    assert_eq!(inspect["multiword_entries"], 3);

    let diff = Command::cargo_bin("mla-titlecase")
        .unwrap()
        .args([
            "lexicon",
            "diff-plugin",
            json_plugin.to_str().unwrap(),
            fst_plugin.to_str().unwrap(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let diff: Value = serde_json::from_slice(&diff).unwrap();
    assert_eq!(diff["added"], 0);
    assert_eq!(diff["removed"], 0);
    assert_eq!(diff["changed"], 0);
}
