use std::collections::BTreeMap;

use serde::Serialize;

use crate::{cli::DiffPluginArgs, cmd::inspect_plugin::load_plugin, error::Result};

#[derive(Debug, Serialize)]
struct DiffSummary {
    left_format: String,
    right_format: String,
    payload_kind_changed: bool,
    metadata_changed: bool,
    added: usize,
    removed: usize,
    changed: usize,
}

pub(crate) fn run(args: DiffPluginArgs) -> Result<()> {
    let (left_format, left) = load_plugin(&args.left)?;
    let (right_format, right) = load_plugin(&args.right)?;
    let (added, removed, changed) = diff_payloads(&left.payload, &right.payload);
    let summary = DiffSummary {
        left_format: left_format.to_string(),
        right_format: right_format.to_string(),
        payload_kind_changed: left.payload_kind() != right.payload_kind(),
        metadata_changed: left.metadata != right.metadata,
        added,
        removed,
        changed,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("left-format: {}", summary.left_format);
        println!("right-format: {}", summary.right_format);
        println!("payload-kind-changed: {}", summary.payload_kind_changed);
        println!("metadata-changed: {}", summary.metadata_changed);
        println!("added: {}", summary.added);
        println!("removed: {}", summary.removed);
        println!("changed: {}", summary.changed);
    }
    Ok(())
}

fn diff_payloads(
    left: &mla_titlecase::PluginPayload,
    right: &mla_titlecase::PluginPayload,
) -> (usize, usize, usize) {
    use mla_titlecase::PluginPayload;

    match (left, right) {
        (PluginPayload::WordSet { words: left }, PluginPayload::WordSet { words: right }) => {
            let left: std::collections::BTreeSet<_> = left.iter().collect();
            let right: std::collections::BTreeSet<_> = right.iter().collect();
            (right.difference(&left).count(), left.difference(&right).count(), 0)
        }
        (
            PluginPayload::CanonicalMap { entries: left },
            PluginPayload::CanonicalMap { entries: right },
        )
        | (
            PluginPayload::ProtectedSpellings { entries: left },
            PluginPayload::ProtectedSpellings { entries: right },
        ) => diff_maps(
            left.iter().map(|entry| (&entry.key, &entry.value)),
            right.iter().map(|entry| (&entry.key, &entry.value)),
        ),
        (
            PluginPayload::RankedWords { entries: left },
            PluginPayload::RankedWords { entries: right },
        ) => diff_maps(
            left.iter().map(|entry| (&entry.word, &entry.rank)),
            right.iter().map(|entry| (&entry.word, &entry.rank)),
        ),
        _ => (right.len(), left.len(), 0),
    }
}

fn diff_maps<'a, T: PartialEq + 'a>(
    left: impl Iterator<Item = (&'a String, &'a T)>,
    right: impl Iterator<Item = (&'a String, &'a T)>,
) -> (usize, usize, usize) {
    let left: BTreeMap<_, _> = left.collect();
    let right: BTreeMap<_, _> = right.collect();
    let added = right.keys().filter(|key| !left.contains_key(*key)).count();
    let removed = left.keys().filter(|key| !right.contains_key(*key)).count();
    let changed = right
        .iter()
        .filter(|(key, value)| left.get(*key).is_some_and(|other| *other != **value))
        .count();
    (added, removed, changed)
}
