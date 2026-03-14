use std::collections::{BTreeMap, BTreeSet};

use crate::{
    plugin::{LexiconPlugin, PluginPayload},
    util::normalize::lookup_key,
    Result,
};

/// Additive external lexicon container.
#[derive(Debug, Clone, Default)]
pub struct ExternalLexicons {
    word_sets: Vec<BTreeSet<String>>,
    canonical_maps: Vec<BTreeMap<String, String>>,
    protected_maps: Vec<BTreeMap<String, String>>,
    ranked_words: Vec<BTreeMap<String, u64>>,
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

    /// Adds a word-set lexicon matched case-insensitively.
    pub fn add_word_set<I, S>(&mut self, words: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = BTreeSet::new();
        for word in words {
            set.insert(lookup_key(word.as_ref()));
        }
        self.word_sets.push(set);
    }

    /// Adds a canonical spelling map.
    pub fn add_canonical_map<I, K, V>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut map = BTreeMap::new();
        for (key, value) in entries {
            map.insert(lookup_key(key.as_ref()), value.as_ref().to_string());
        }
        self.canonical_maps.push(map);
    }

    /// Adds a protected-spelling map.
    pub fn add_protected_spellings<I, K, V>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut map = BTreeMap::new();
        for (key, value) in entries {
            map.insert(lookup_key(key.as_ref()), value.as_ref().to_string());
        }
        self.protected_maps.push(map);
    }

    /// Adds ranked words keyed case-insensitively.
    pub fn add_ranked_words<I, S>(&mut self, entries: I)
    where
        I: IntoIterator<Item = (S, u64)>,
        S: AsRef<str>,
    {
        let mut map = BTreeMap::new();
        for (word, rank) in entries {
            map.insert(lookup_key(word.as_ref()), rank);
        }
        self.ranked_words.push(map);
    }

    pub(crate) fn contains_word(&self, word: &str) -> bool {
        let key = lookup_key(word);
        self.word_sets.iter().any(|set| set.contains(&key))
    }

    pub(crate) fn canonical_spelling(&self, word: &str) -> Option<&str> {
        let key = lookup_key(word);
        self.canonical_maps.iter().rev().find_map(|map| map.get(&key).map(String::as_str))
    }

    pub(crate) fn protected_spelling(&self, word: &str) -> Option<&str> {
        let key = lookup_key(word);
        self.protected_maps.iter().rev().find_map(|map| map.get(&key).map(String::as_str))
    }

    /// Returns the rank for a previously added ranked-word entry.
    #[must_use]
    pub fn rank_of(&self, word: &str) -> Option<u64> {
        let key = lookup_key(word);
        self.ranked_words.iter().rev().find_map(|map| map.get(&key).copied())
    }
}

#[cfg(test)]
mod tests {
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
}
