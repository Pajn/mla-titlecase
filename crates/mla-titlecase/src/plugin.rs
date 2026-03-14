//! Versioned plugin schema shared by the library and CLI.

use serde::{Deserialize, Serialize};

use crate::{error::Result, lexicon::ExternalLexicons, util::normalize::lookup_key, Error};

/// Current stable plugin schema version.
pub const PLUGIN_SCHEMA_VERSION: u32 = 1;

/// Top-level plugin document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LexiconPlugin {
    /// Provenance and compatibility metadata.
    pub metadata: PluginMetadata,
    /// Plugin payload.
    pub payload: PluginPayload,
}

impl LexiconPlugin {
    /// Returns the payload kind.
    #[must_use]
    pub fn payload_kind(&self) -> PluginPayloadKind {
        self.payload.kind()
    }

    /// Validates the plugin contents.
    pub fn validate(&self) -> Result<()> {
        self.metadata.validate()?;
        self.payload.validate()
    }

    /// Registers the plugin into an [`ExternalLexicons`] container.
    pub fn register_into(&self, external: &mut ExternalLexicons) -> Result<()> {
        self.validate()?;
        external.register_plugin(self)
    }
}

/// Provenance and compatibility metadata for a plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginMetadata {
    /// Stable schema version.
    pub schema_version: u32,
    /// Plugin revision chosen by the producer.
    pub plugin_version: u32,
    /// Source identifier such as `scowl` or `stopwords-iso`.
    pub source_id: String,
    /// Optional upstream source version.
    pub source_version: Option<String>,
    /// Optional upstream URL.
    pub upstream_url: Option<String>,
    /// Producer timestamp string.
    pub prepared_at: String,
    /// Optional checksum covering the generated artifact.
    pub checksum: Option<String>,
    /// Human-readable license summary.
    pub license_summary: String,
    /// Optional notice text to preserve in downstream artifacts.
    pub notice: Option<String>,
}

impl PluginMetadata {
    /// Returns minimal metadata for ad-hoc plugins.
    #[must_use]
    pub fn new(source_id: impl Into<String>, license_summary: impl Into<String>) -> Self {
        Self {
            schema_version: PLUGIN_SCHEMA_VERSION,
            plugin_version: 1,
            source_id: source_id.into(),
            source_version: None,
            upstream_url: None,
            prepared_at: "unknown".to_string(),
            checksum: None,
            license_summary: license_summary.into(),
            notice: None,
        }
    }

    /// Validates the metadata fields.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != PLUGIN_SCHEMA_VERSION {
            return Err(Error::UnsupportedVersion {
                found: self.schema_version,
                expected: PLUGIN_SCHEMA_VERSION,
            });
        }

        if self.source_id.trim().is_empty() {
            return Err(Error::InvalidData("plugin source_id cannot be empty".to_string()));
        }

        if self.license_summary.trim().is_empty() {
            return Err(Error::InvalidData("plugin license_summary cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// Payload kinds supported by the plugin format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PluginPayloadKind {
    /// Case-insensitive word membership.
    WordSet,
    /// Canonical spellings keyed by lowercase lookup keys.
    CanonicalMap,
    /// Ranked or commonness-ordered words.
    RankedWords,
    /// Protected spellings keyed by lowercase lookup keys.
    ProtectedSpellings,
}

impl PluginPayloadKind {
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::WordSet => 1,
            Self::CanonicalMap => 2,
            Self::RankedWords => 3,
            Self::ProtectedSpellings => 4,
        }
    }

    pub(crate) fn from_tag(tag: u8) -> Result<Self> {
        match tag {
            1 => Ok(Self::WordSet),
            2 => Ok(Self::CanonicalMap),
            3 => Ok(Self::RankedWords),
            4 => Ok(Self::ProtectedSpellings),
            _ => Err(Error::InvalidData(format!("unknown plugin payload tag {tag}"))),
        }
    }
}

/// Key/value entry used by canonical and protected payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapEntry {
    /// Case-insensitive lookup key.
    pub key: String,
    /// Canonical output spelling.
    pub value: String,
}

/// Ranked word entry used by ranked payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RankedEntry {
    /// Case-insensitive lookup key.
    pub word: String,
    /// Rank or commonness score.
    pub rank: u64,
}

/// Supported plugin payload values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PluginPayload {
    /// Case-insensitive word membership.
    WordSet {
        /// Stored words.
        words: Vec<String>,
    },
    /// Canonical spellings keyed by lowercase lookup keys.
    CanonicalMap {
        /// Stored key/value entries.
        entries: Vec<MapEntry>,
    },
    /// Ranked words keyed by lowercase lookup keys.
    RankedWords {
        /// Stored ranked entries.
        entries: Vec<RankedEntry>,
    },
    /// Protected spellings keyed by lowercase lookup keys.
    ProtectedSpellings {
        /// Stored key/value entries.
        entries: Vec<MapEntry>,
    },
}

impl PluginPayload {
    /// Returns the payload kind.
    #[must_use]
    pub fn kind(&self) -> PluginPayloadKind {
        match self {
            Self::WordSet { .. } => PluginPayloadKind::WordSet,
            Self::CanonicalMap { .. } => PluginPayloadKind::CanonicalMap,
            Self::RankedWords { .. } => PluginPayloadKind::RankedWords,
            Self::ProtectedSpellings { .. } => PluginPayloadKind::ProtectedSpellings,
        }
    }

    /// Returns the number of stored entries.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::WordSet { words } => words.len(),
            Self::CanonicalMap { entries } | Self::ProtectedSpellings { entries } => entries.len(),
            Self::RankedWords { entries } => entries.len(),
        }
    }

    /// Returns whether the payload is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Self::WordSet { words } => validate_words(words),
            Self::CanonicalMap { entries } | Self::ProtectedSpellings { entries } => {
                validate_map_entries(entries)
            }
            Self::RankedWords { entries } => validate_ranked_entries(entries),
        }
    }
}

fn validate_words(words: &[String]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for word in words {
        let key = lookup_key(word);
        if key.is_empty() {
            return Err(Error::InvalidData("word-set entries cannot be empty".to_string()));
        }
        if !seen.insert(key) {
            return Err(Error::InvalidData("word-set entries must be unique".to_string()));
        }
    }
    Ok(())
}

fn validate_map_entries(entries: &[MapEntry]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for entry in entries {
        let key = lookup_key(&entry.key);
        if key.is_empty() || entry.value.trim().is_empty() {
            return Err(Error::InvalidData(
                "map entries require non-empty keys and values".to_string(),
            ));
        }
        if !seen.insert(key) {
            return Err(Error::InvalidData("map entries must be unique".to_string()));
        }
    }
    Ok(())
}

fn validate_ranked_entries(entries: &[RankedEntry]) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for entry in entries {
        let key = lookup_key(&entry.word);
        if key.is_empty() {
            return Err(Error::InvalidData(
                "ranked-word entries require a non-empty word".to_string(),
            ));
        }
        if !seen.insert(key) {
            return Err(Error::InvalidData("ranked-word entries must be unique".to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LexiconPlugin, MapEntry, PluginMetadata, PluginPayload, PluginPayloadKind, RankedEntry,
    };
    use crate::ExternalLexicons;

    #[test]
    fn validates_payloads() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::CanonicalMap {
                entries: vec![MapEntry { key: "github".to_string(), value: "GitHub".to_string() }],
            },
        };

        assert_eq!(plugin.payload_kind(), PluginPayloadKind::CanonicalMap);
        assert!(plugin.validate().is_ok());
    }

    #[test]
    fn rejects_duplicate_words() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::WordSet { words: vec!["and".to_string(), "AND".to_string()] },
        };

        assert!(plugin.validate().is_err());
    }

    #[test]
    fn validates_ranked_entries() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::RankedWords {
                entries: vec![RankedEntry { word: "common".to_string(), rank: 7 }],
            },
        };

        assert!(plugin.validate().is_ok());
    }

    #[test]
    fn registers_plugins_into_external_lexicons() {
        let plugin = LexiconPlugin {
            metadata: PluginMetadata::new("fixture", "MIT"),
            payload: PluginPayload::ProtectedSpellings {
                entries: vec![MapEntry { key: "github".to_string(), value: "GitHub".to_string() }],
            },
        };
        let mut lexicons = ExternalLexicons::default();

        plugin.register_into(&mut lexicons).unwrap();

        assert_eq!(lexicons.protected_spelling("github"), Some("GitHub"));
    }
}
