//! FST-backed plugin serialization helpers.

use std::{
    fs::{self, File},
    io::Cursor,
    ops::Range,
    path::Path,
    sync::Arc,
};

use fst::{Map, MapBuilder, Set, SetBuilder, Streamer};
use memmap2::Mmap;

use crate::{
    error::Result,
    plugin::{
        LexiconPlugin, MapEntry, PluginMetadata, PluginPayload, PluginPayloadKind, RankedEntry,
    },
    util::normalize::{lookup_key, normalized_unique_sorted},
    Error,
};

const MAGIC: &[u8; 8] = b"MLATFST1";
const HEADER_LEN: usize = 24;

/// Direct-query FST plugin loaded from owned bytes or an mmap-backed file.
#[derive(Debug)]
pub struct FstRuntimePlugin {
    metadata: PluginMetadata,
    payload: RuntimePayload,
}

#[derive(Debug)]
enum RuntimePayload {
    WordSet(Set<ByteRegion>),
    CanonicalMap { map: Map<ByteRegion>, values: ByteRegion },
    MultiwordMap { map: Map<ByteRegion>, values: ByteRegion },
    RankedWords(Map<ByteRegion>),
    ProtectedSpellings { map: Map<ByteRegion>, values: ByteRegion },
}

#[derive(Debug, Clone)]
enum ByteStorage {
    Owned(Arc<[u8]>),
    Mmap(Arc<Mmap>),
}

#[derive(Debug, Clone)]
struct ByteRegion {
    storage: ByteStorage,
    range: Range<usize>,
}

#[derive(Debug)]
struct PluginLayout {
    kind: PluginPayloadKind,
    metadata_range: Range<usize>,
    payload_range: Range<usize>,
}

/// Loads a plugin from the custom FST-backed binary format.
pub fn load_fst_plugin(path: impl AsRef<Path>) -> Result<LexiconPlugin> {
    parse_plugin(&fs::read(path)?)
}

/// Loads an FST plugin into a direct-query runtime using owned bytes.
pub fn open_fst_runtime_plugin(path: impl AsRef<Path>) -> Result<FstRuntimePlugin> {
    let bytes: Arc<[u8]> = fs::read(path)?.into();
    build_runtime_plugin(ByteStorage::Owned(bytes))
}

/// Loads an FST plugin into a direct-query runtime using a memory map.
pub fn mmap_fst_plugin(path: impl AsRef<Path>) -> Result<FstRuntimePlugin> {
    let file = File::open(path)?;
    #[allow(unsafe_code)]
    let mmap = unsafe {
        // SAFETY: the returned map is only exposed immutably, and the file is
        // opened read-only for the lifetime of the map.
        Mmap::map(&file)?
    };
    build_runtime_plugin(ByteStorage::Mmap(Arc::new(mmap)))
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

impl FstRuntimePlugin {
    /// Returns the plugin metadata.
    #[must_use]
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Returns the stored payload kind.
    #[must_use]
    pub fn payload_kind(&self) -> PluginPayloadKind {
        match self.payload {
            RuntimePayload::WordSet(_) => PluginPayloadKind::WordSet,
            RuntimePayload::CanonicalMap { .. } => PluginPayloadKind::CanonicalMap,
            RuntimePayload::MultiwordMap { .. } => PluginPayloadKind::MultiwordMap,
            RuntimePayload::RankedWords(_) => PluginPayloadKind::RankedWords,
            RuntimePayload::ProtectedSpellings { .. } => PluginPayloadKind::ProtectedSpellings,
        }
    }

    pub(crate) fn contains_word(&self, word: &str) -> bool {
        match &self.payload {
            RuntimePayload::WordSet(set) => set.contains(lookup_key(word)),
            _ => false,
        }
    }

    pub(crate) fn map_value(&self, word: &str) -> Option<&str> {
        let key = lookup_key(word);
        match &self.payload {
            RuntimePayload::CanonicalMap { map, values }
            | RuntimePayload::MultiwordMap { map, values }
            | RuntimePayload::ProtectedSpellings { map, values } => {
                map.get(key).and_then(|offset| read_c_string(values.as_ref(), offset as usize).ok())
            }
            _ => None,
        }
    }

    /// Returns the rank for the given word when the payload stores ranked words.
    #[must_use]
    pub fn rank_of(&self, word: &str) -> Option<u64> {
        let key = lookup_key(word);
        match &self.payload {
            RuntimePayload::RankedWords(map) => map.get(key),
            _ => None,
        }
    }
}

impl AsRef<[u8]> for ByteRegion {
    fn as_ref(&self) -> &[u8] {
        match &self.storage {
            ByteStorage::Owned(bytes) => &bytes[self.range.start..self.range.end],
            ByteStorage::Mmap(mmap) => &mmap[self.range.start..self.range.end],
        }
    }
}

fn build_runtime_plugin(storage: ByteStorage) -> Result<FstRuntimePlugin> {
    let layout = parse_layout(storage_bytes(&storage))?;
    let metadata: PluginMetadata =
        serde_json::from_slice(&storage_bytes(&storage)[layout.metadata_range.clone()])?;
    let payload = runtime_payload(storage, &layout)?;
    let plugin = FstRuntimePlugin { metadata, payload };
    plugin.metadata.validate()?;
    Ok(plugin)
}

fn runtime_payload(storage: ByteStorage, layout: &PluginLayout) -> Result<RuntimePayload> {
    match layout.kind {
        PluginPayloadKind::WordSet => Ok(RuntimePayload::WordSet(Set::new(ByteRegion {
            storage,
            range: layout.payload_range.clone(),
        })?)),
        PluginPayloadKind::CanonicalMap => {
            let (map_range, values_range) =
                parse_map_ranges(storage_bytes(&storage), &layout.payload_range)?;
            Ok(RuntimePayload::CanonicalMap {
                map: Map::new(ByteRegion { storage: storage.clone(), range: map_range })?,
                values: ByteRegion { storage, range: values_range },
            })
        }
        PluginPayloadKind::MultiwordMap => {
            let (map_range, values_range) =
                parse_map_ranges(storage_bytes(&storage), &layout.payload_range)?;
            Ok(RuntimePayload::MultiwordMap {
                map: Map::new(ByteRegion { storage: storage.clone(), range: map_range })?,
                values: ByteRegion { storage, range: values_range },
            })
        }
        PluginPayloadKind::RankedWords => Ok(RuntimePayload::RankedWords(Map::new(ByteRegion {
            storage,
            range: layout.payload_range.clone(),
        })?)),
        PluginPayloadKind::ProtectedSpellings => {
            let (map_range, values_range) =
                parse_map_ranges(storage_bytes(&storage), &layout.payload_range)?;
            Ok(RuntimePayload::ProtectedSpellings {
                map: Map::new(ByteRegion { storage: storage.clone(), range: map_range })?,
                values: ByteRegion { storage, range: values_range },
            })
        }
    }
}

fn parse_plugin(bytes: &[u8]) -> Result<LexiconPlugin> {
    let layout = parse_layout(bytes)?;
    let metadata: PluginMetadata = serde_json::from_slice(&bytes[layout.metadata_range])?;
    let payload = decode_payload(layout.kind, &bytes[layout.payload_range])?;
    let plugin = LexiconPlugin { metadata, payload };
    plugin.validate()?;
    Ok(plugin)
}

fn parse_layout(bytes: &[u8]) -> Result<PluginLayout> {
    if bytes.len() < HEADER_LEN || &bytes[..8] != MAGIC {
        return Err(Error::InvalidData("invalid FST plugin header".to_string()));
    }

    let metadata_len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let kind = PluginPayloadKind::from_tag(bytes[12])?;
    let payload_len = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
    let metadata_start = HEADER_LEN;
    let metadata_end = metadata_start + metadata_len;
    let payload_end = metadata_end + payload_len;

    if payload_end > bytes.len() {
        return Err(Error::InvalidData("truncated FST plugin payload".to_string()));
    }

    Ok(PluginLayout {
        kind,
        metadata_range: metadata_start..metadata_end,
        payload_range: metadata_end..payload_end,
    })
}

fn parse_map_ranges(
    bytes: &[u8],
    payload_range: &Range<usize>,
) -> Result<(Range<usize>, Range<usize>)> {
    let payload = &bytes[payload_range.start..payload_range.end];
    let mut cursor = Cursor::new(payload);
    let map_len = read_u64(&mut cursor)? as usize;
    let map_start = payload_range.start + cursor.position() as usize;
    let map_end = map_start + map_len;
    if map_end > payload_range.end {
        return Err(Error::InvalidData("truncated FST map section".to_string()));
    }

    cursor.set_position((map_end - payload_range.start) as u64);
    let values_len = read_u64(&mut cursor)? as usize;
    let values_start = payload_range.start + cursor.position() as usize;
    let values_end = values_start + values_len;
    if values_end > payload_range.end {
        return Err(Error::InvalidData("truncated FST values section".to_string()));
    }

    Ok((map_start..map_end, values_start..values_end))
}

fn storage_bytes(storage: &ByteStorage) -> &[u8] {
    match storage {
        ByteStorage::Owned(bytes) => bytes,
        ByteStorage::Mmap(mmap) => mmap,
    }
}

fn encode_payload(payload: &PluginPayload) -> Result<Vec<u8>> {
    match payload {
        PluginPayload::WordSet { words } => encode_word_set(words),
        PluginPayload::CanonicalMap { entries }
        | PluginPayload::MultiwordMap { entries }
        | PluginPayload::ProtectedSpellings { entries } => {
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
        PluginPayloadKind::MultiwordMap => {
            Ok(PluginPayload::MultiwordMap { entries: decode_map_entries(bytes)? })
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
    let (map_range, values_range) = parse_map_ranges_for_payload(bytes)?;
    let map = Map::new(&bytes[map_range])?;
    let values = &bytes[values_range];
    let mut stream = map.stream();
    let mut entries = Vec::new();
    while let Some((key, offset)) = stream.next() {
        entries.push(MapEntry {
            key: std::str::from_utf8(key)?.to_string(),
            value: read_c_string(values, offset as usize)?.to_string(),
        });
    }
    Ok(entries)
}

fn parse_map_ranges_for_payload(bytes: &[u8]) -> Result<(Range<usize>, Range<usize>)> {
    let payload_range = 0..bytes.len();
    parse_map_ranges(bytes, &payload_range)
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

fn read_c_string(bytes: &[u8], offset: usize) -> Result<&str> {
    if offset >= bytes.len() {
        return Err(Error::InvalidData("invalid string-table offset".to_string()));
    }
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .map(|relative| offset + relative)
        .ok_or_else(|| Error::InvalidData("unterminated string-table entry".to_string()))?;
    Ok(std::str::from_utf8(&bytes[offset..end])?)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        fst_store::{
            load_fst_plugin, mmap_fst_plugin, open_fst_runtime_plugin, save_fst_plugin,
            FstRuntimePlugin,
        },
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
    fn round_trips_multiword_payloads() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::MultiwordMap {
                entries: vec![MapEntry {
                    key: "new york city".to_string(),
                    value: "New York City".to_string(),
                }],
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
    fn opens_owned_runtime_plugins() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::ProtectedSpellings {
                entries: vec![MapEntry { key: "github".to_string(), value: "GitHub".to_string() }],
            },
        };
        let runtime = save_and_open_runtime(plugin, |path| open_fst_runtime_plugin(path));
        assert_eq!(runtime.map_value("github"), Some("GitHub"));
    }

    #[test]
    fn opens_mmap_runtime_plugins() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::RankedWords {
                entries: vec![RankedEntry { word: "common".to_string(), rank: 7 }],
            },
        };
        let runtime = save_and_open_runtime(plugin, |path| mmap_fst_plugin(path));
        assert_eq!(runtime.rank_of("common"), Some(7));
    }

    #[test]
    fn rejects_invalid_headers() {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("broken.mlatl");
        std::fs::write(&path, b"not-a-plugin").unwrap();

        assert!(load_fst_plugin(&path).is_err());
        assert!(mmap_fst_plugin(&path).is_err());
    }

    fn save_and_open_runtime(
        plugin: LexiconPlugin,
        opener: impl Fn(&std::path::Path) -> crate::Result<FstRuntimePlugin>,
    ) -> FstRuntimePlugin {
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.mlatl");
        save_fst_plugin(&path, &plugin).unwrap();
        opener(&path).unwrap()
    }
}
