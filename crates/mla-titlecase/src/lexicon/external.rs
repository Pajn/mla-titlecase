use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    fst_store::{mmap_fst_plugin, open_fst_runtime_plugin, FstRuntimePlugin},
    plugin::{LexiconPlugin, PluginPayload},
    util::normalize::lookup_key,
    Result,
};

/// Additive external lexicon container.
#[derive(Debug, Default)]
pub struct ExternalLexicons {
    word_sets: Vec<WordSetBackend>,
    canonical_maps: Vec<MapBackend>,
    protected_maps: Vec<MapBackend>,
    ranked_words: Vec<RankedBackend>,
}

#[derive(Debug)]
enum WordSetBackend {
    Heap(HashSet<String>),
    Fst(Box<FstRuntimePlugin>),
}

#[derive(Debug)]
enum MapBackend {
    Heap(HashMap<String, String>),
    Fst(Box<FstRuntimePlugin>),
}

#[derive(Debug)]
enum RankedBackend {
    Heap(HashMap<String, u64>),
    Fst(Box<FstRuntimePlugin>),
}

impl ExternalLexicons {
    /// Registers a validated plugin into this lexicon container.
    pub fn register_plugin(&mut self, plugin: &LexiconPlugin) -> Result<()> {
        match &plugin.payload {
            PluginPayload::WordSet { words } => self.add_word_set(words.iter().map(String::as_str)),
            PluginPayload::CanonicalMap { entries } => self.add_canonical_map(
                entries.iter().map(|entry| (entry.key.as_str(), entry.value.as_str())),
            ),
            PluginPayload::ProtectedSpellings { entries } => self.add_protected_spellings(
                entries.iter().map(|entry| (entry.key.as_str(), entry.value.as_str())),
            ),
            PluginPayload::RankedWords { entries } => {
                self.add_ranked_words(entries.iter().map(|entry| (entry.word.as_str(), entry.rank)))
            }
        }
        Ok(())
    }

    /// Registers a direct-query FST plugin from owned bytes.
    pub fn register_fst_plugin(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.register_runtime_plugin(open_fst_runtime_plugin(path)?)
    }

    /// Registers a direct-query FST plugin backed by a memory map.
    pub fn register_mmap_fst_plugin(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.register_runtime_plugin(mmap_fst_plugin(path)?)
    }

    /// Adds a word-set lexicon matched case-insensitively.
    pub fn add_word_set<I, S>(&mut self, words: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = HashSet::new();
        for word in words {
            set.insert(lookup_key(word.as_ref()));
        }
        self.word_sets.push(WordSetBackend::Heap(set));
    }

    /// Adds a canonical spelling map.
    pub fn add_canonical_map<I, K, V>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut map = HashMap::new();
        for (key, value) in entries {
            map.insert(lookup_key(key.as_ref()), value.as_ref().to_string());
        }
        self.canonical_maps.push(MapBackend::Heap(map));
    }

    /// Adds a protected-spelling map.
    pub fn add_protected_spellings<I, K, V>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut map = HashMap::new();
        for (key, value) in entries {
            map.insert(lookup_key(key.as_ref()), value.as_ref().to_string());
        }
        self.protected_maps.push(MapBackend::Heap(map));
    }

    /// Adds ranked words keyed case-insensitively.
    pub fn add_ranked_words<I, S>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (S, u64)>,
        S: AsRef<str>,
    {
        let mut map = HashMap::new();
        for (word, rank) in entries {
            map.insert(lookup_key(word.as_ref()), rank);
        }
        self.ranked_words.push(RankedBackend::Heap(map));
    }

    fn register_runtime_plugin(&mut self, plugin: FstRuntimePlugin) -> Result<()> {
        match plugin.payload_kind() {
            crate::PluginPayloadKind::WordSet => {
                self.word_sets.push(WordSetBackend::Fst(Box::new(plugin)))
            }
            crate::PluginPayloadKind::CanonicalMap => {
                self.canonical_maps.push(MapBackend::Fst(Box::new(plugin)))
            }
            crate::PluginPayloadKind::ProtectedSpellings => {
                self.protected_maps.push(MapBackend::Fst(Box::new(plugin)))
            }
            crate::PluginPayloadKind::RankedWords => {
                self.ranked_words.push(RankedBackend::Fst(Box::new(plugin)))
            }
        }
        Ok(())
    }

    pub(crate) fn contains_word(&self, word: &str) -> bool {
        let key = lookup_key(word);
        self.word_sets.iter().rev().any(|backend| match backend {
            WordSetBackend::Heap(set) => set.contains(&key),
            WordSetBackend::Fst(plugin) => plugin.contains_word(&key),
        })
    }

    pub(crate) fn canonical_spelling(&self, word: &str) -> Option<&str> {
        let key = lookup_key(word);
        self.canonical_maps.iter().rev().find_map(|backend| match backend {
            MapBackend::Heap(map) => map.get(&key).map(String::as_str),
            MapBackend::Fst(plugin) => plugin.map_value(&key),
        })
    }

    pub(crate) fn protected_spelling(&self, word: &str) -> Option<&str> {
        let key = lookup_key(word);
        self.protected_maps.iter().rev().find_map(|backend| match backend {
            MapBackend::Heap(map) => map.get(&key).map(String::as_str),
            MapBackend::Fst(plugin) => plugin.map_value(&key),
        })
    }

    /// Returns the rank for a previously added ranked-word entry.
    #[must_use]
    pub fn rank_of(&self, word: &str) -> Option<u64> {
        let key = lookup_key(word);
        self.ranked_words.iter().rev().find_map(|backend| match backend {
            RankedBackend::Heap(map) => map.get(&key).copied(),
            RankedBackend::Fst(plugin) => plugin.rank_of(&key),
        })
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        fst_store::save_fst_plugin,
        plugin::{LexiconPlugin, MapEntry, PluginMetadata, PluginPayload},
    };

    use super::ExternalLexicons;

    #[test]
    fn stores_and_queries_entries() {
        let mut lexicons = ExternalLexicons::default();
        lexicons.add_word_set(["the", "and"]);
        lexicons.add_canonical_map([("postgres", "Postgres")]);
        lexicons.add_protected_spellings([("github", "GitHub")]);
        lexicons.add_ranked_words([("common", 42)]);

        assert!(lexicons.contains_word("THE"));
        assert_eq!(lexicons.canonical_spelling("POSTGRES"), Some("Postgres"));
        assert_eq!(lexicons.protected_spelling("github"), Some("GitHub"));
        assert_eq!(lexicons.rank_of("common"), Some(42));
    }

    #[test]
    fn queries_mmap_backed_fst_plugins() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::CanonicalMap {
                entries: vec![MapEntry {
                    key: "postgres".to_string(),
                    value: "Postgres".to_string(),
                }],
            },
        };
        let tempdir = tempdir().unwrap();
        let path = tempdir.path().join("plugin.mlatl");
        save_fst_plugin(&path, &plugin).unwrap();

        let mut lexicons = ExternalLexicons::default();
        lexicons.register_mmap_fst_plugin(&path).unwrap();

        assert_eq!(lexicons.canonical_spelling("postgres"), Some("Postgres"));
    }
}
