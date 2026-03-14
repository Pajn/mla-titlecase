//! FST-backed plugin serialization helpers.

use std::{fs, io::Cursor, path::Path};

use fst::{Map, MapBuilder, Set, SetBuilder, Streamer};

use crate::{
    error::Result,
    plugin::{
        LexiconPlugin, MapEntry, PluginMetadata, PluginPayload, PluginPayloadKind, RankedEntry,
    },
    util::normalize::{lookup_key, normalized_unique_sorted},
    Error,
};

const MAGIC: &[u8; 8] = b"MLATFST1";

/// Loads a plugin from the custom FST-backed binary format.
pub fn load_fst_plugin(path: impl AsRef<Path>) -> Result<LexiconPlugin> {
    parse_plugin(&fs::read(path)?)
}

/// Saves a plugin to the custom FST-backed binary format.
pub fn save_fst_plugin(path: impl AsRef<Path>, plugin: &LexiconPlugin) -> Result<()> {
    plugin.validate()?;
    let metadata = serde_json::to_vec(&plugin.metadata)?;
    let payload = encode_payload(&plugin.payload)?;
    let mut bytes =
        Vec::with_capacity(MAGIC.len() + 4 + 1 + 3 + 8 + metadata.len() + payload.len());
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    bytes.push(plugin.payload_kind().tag());
    bytes.extend_from_slice(&[0_u8; 3]);
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&metadata);
    bytes.extend_from_slice(&payload);
    fs::write(path, bytes)?;
    Ok(())
}

fn parse_plugin(bytes: &[u8]) -> Result<LexiconPlugin> {
    if bytes.len() < 24 || &bytes[..8] != MAGIC {
        return Err(Error::InvalidData("invalid FST plugin header".to_string()));
    }

    let metadata_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let kind = PluginPayloadKind::from_tag(bytes[12])?;
    let payload_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
    let metadata_start = 24;
    let metadata_end = metadata_start + metadata_len;
    let payload_end = metadata_end + payload_len;

    if payload_end > bytes.len() {
        return Err(Error::InvalidData("truncated FST plugin payload".to_string()));
    }

    let metadata: PluginMetadata = serde_json::from_slice(&bytes[metadata_start..metadata_end])?;
    let payload = decode_payload(kind, &bytes[metadata_end..payload_end])?;
    let plugin = LexiconPlugin { metadata, payload };
    plugin.validate()?;
    Ok(plugin)
}

fn encode_payload(payload: &PluginPayload) -> Result<Vec<u8>> {
    match payload {
        PluginPayload::WordSet { words } => encode_word_set(words),
        PluginPayload::CanonicalMap { entries } | PluginPayload::ProtectedSpellings { entries } => {
            encode_map_entries(entries)
        }
        PluginPayload::RankedWords { entries } => encode_ranked_entries(entries),
    }
}

fn decode_payload(kind: PluginPayloadKind, bytes: &[u8]) -> Result<PluginPayload> {
    match kind {
        PluginPayloadKind::WordSet => Ok(PluginPayload::WordSet { words: decode_word_set(bytes)? }),
        PluginPayloadKind::CanonicalMap => {
            Ok(PluginPayload::CanonicalMap { entries: decode_map_entries(bytes)? })
        }
        PluginPayloadKind::RankedWords => {
            Ok(PluginPayload::RankedWords { entries: decode_ranked_entries(bytes)? })
        }
        PluginPayloadKind::ProtectedSpellings => {
            Ok(PluginPayload::ProtectedSpellings { entries: decode_map_entries(bytes)? })
        }
    }
}

fn encode_word_set(words: &[String]) -> Result<Vec<u8>> {
    let mut builder = SetBuilder::memory();
    for word in normalized_unique_sorted(words.iter().map(String::as_str)) {
        builder.insert(word)?;
    }
    Ok(builder.into_inner()?)
}

fn decode_word_set(bytes: &[u8]) -> Result<Vec<String>> {
    let set = Set::new(bytes)?;
    let mut stream = set.stream();
    let mut words = Vec::new();
    while let Some(word) = stream.next() {
        words.push(std::str::from_utf8(word)?.to_string());
    }
    Ok(words)
}

fn encode_map_entries(entries: &[MapEntry]) -> Result<Vec<u8>> {
    let mut normalized: Vec<MapEntry> = entries
        .iter()
        .map(|entry| MapEntry { key: lookup_key(&entry.key), value: entry.value.clone() })
        .collect();
    normalized.sort_by(|left, right| left.key.cmp(&right.key));

    let mut values = Vec::new();
    let mut builder = MapBuilder::memory();
    for entry in normalized {
        let offset = values.len() as u64;
        builder.insert(entry.key, offset)?;
        values.extend_from_slice(entry.value.as_bytes());
        values.push(0);
    }
    let map = builder.into_inner()?;

    let mut result = Vec::with_capacity(16 + map.len() + values.len());
    result.extend_from_slice(&(map.len() as u64).to_le_bytes());
    result.extend_from_slice(&map);
    result.extend_from_slice(&(values.len() as u64).to_le_bytes());
    result.extend_from_slice(&values);
    Ok(result)
}

fn decode_map_entries(bytes: &[u8]) -> Result<Vec<MapEntry>> {
    let mut cursor = Cursor::new(bytes);
    let map_len = read_u64(&mut cursor)? as usize;
    let start = cursor.position() as usize;
    let end = start + map_len;
    if end > bytes.len() {
        return Err(Error::InvalidData("truncated FST map section".to_string()));
    }
    cursor.set_position(end as u64);
    let values_len = read_u64(&mut cursor)? as usize;
    let values_start = cursor.position() as usize;
    let values_end = values_start + values_len;
    if values_end > bytes.len() {
        return Err(Error::InvalidData("truncated FST values section".to_string()));
    }

    let map = Map::new(&bytes[start..end])?;
    let values = &bytes[values_start..values_end];
    let mut stream = map.stream();
    let mut entries = Vec::new();
    while let Some((key, offset)) = stream.next() {
        entries.push(MapEntry {
            key: std::str::from_utf8(key)?.to_string(),
            value: read_c_string(values, offset as usize)?,
        });
    }
    Ok(entries)
}

fn encode_ranked_entries(entries: &[RankedEntry]) -> Result<Vec<u8>> {
    let mut normalized: Vec<RankedEntry> = entries
        .iter()
        .map(|entry| RankedEntry { word: lookup_key(&entry.word), rank: entry.rank })
        .collect();
    normalized.sort_by(|left, right| left.word.cmp(&right.word));

    let mut builder = MapBuilder::memory();
    for entry in normalized {
        builder.insert(entry.word, entry.rank)?;
    }
    Ok(builder.into_inner()?)
}

fn decode_ranked_entries(bytes: &[u8]) -> Result<Vec<RankedEntry>> {
    let map = Map::new(bytes)?;
    let mut stream = map.stream();
    let mut entries = Vec::new();
    while let Some((word, rank)) = stream.next() {
        entries.push(RankedEntry { word: std::str::from_utf8(word)?.to_string(), rank });
    }
    Ok(entries)
}

fn read_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
    let mut bytes = [0_u8; 8];
    std::io::Read::read_exact(cursor, &mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

fn read_c_string(bytes: &[u8], offset: usize) -> Result<String> {
    if offset >= bytes.len() {
        return Err(Error::InvalidData("invalid string-table offset".to_string()));
    }
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|relative| offset + relative)
        .ok_or_else(|| Error::InvalidData("unterminated string-table entry".to_string()))?;
    Ok(String::from_utf8(bytes[offset..end].to_vec())?)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        fst_store::{load_fst_plugin, save_fst_plugin},
        plugin::{LexiconPlugin, MapEntry, PluginMetadata, PluginPayload, RankedEntry},
    };

    #[test]
    fn round_trips_word_sets() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::WordSet {
                words: vec!["beta".to_string(), "alpha".to_string()],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.mlatl");

        save_fst_plugin(&path, &plugin).unwrap();
        let loaded = load_fst_plugin(&path).unwrap();

        assert_eq!(
            loaded,
            LexiconPlugin {
                metadata: plugin.metadata,
                payload: PluginPayload::WordSet {
                    words: vec!["alpha".to_string(), "beta".to_string()],
                },
            }
        );
    }

    #[test]
    fn round_trips_map_payloads() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::CanonicalMap {
                entries: vec![MapEntry { key: "github".to_string(), value: "GitHub".to_string() }],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.mlatl");

        save_fst_plugin(&path, &plugin).unwrap();
        let loaded = load_fst_plugin(&path).unwrap();

        assert_eq!(loaded, plugin);
    }

    #[test]
    fn round_trips_ranked_payloads() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::RankedWords {
                entries: vec![RankedEntry { word: "common".to_string(), rank: 3 }],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.mlatl");

        save_fst_plugin(&path, &plugin).unwrap();
        let loaded = load_fst_plugin(&path).unwrap();

        assert_eq!(loaded, plugin);
    }

    #[test]
    fn rejects_invalid_headers() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("broken.mlatl");
        std::fs::write(&path, b"not-a-plugin").unwrap();

        assert!(load_fst_plugin(&path).is_err());
    }
}
