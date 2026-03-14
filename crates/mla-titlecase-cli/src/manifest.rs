use std::path::Path;

use chrono::Utc;
use mla_titlecase::plugin::PluginPayload;
use serde::{Deserialize, Serialize};

use crate::{checksum::sha256_hex, error::Result, fsutil};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RawSourceManifest {
    pub(crate) source_id: String,
    pub(crate) source_url: String,
    pub(crate) source_version: Option<String>,
    pub(crate) fetched_at: String,
    pub(crate) checksum: String,
    pub(crate) license_summary: String,
    pub(crate) notice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PreparedMetadata {
    pub(crate) source_id: String,
    pub(crate) source_version: Option<String>,
    pub(crate) source_url: Option<String>,
    pub(crate) prepared_at: String,
    pub(crate) input_checksum: String,
    pub(crate) license_summary: String,
    pub(crate) notice: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct NormalizationReport {
    pub(crate) input_records: usize,
    pub(crate) output_entries: usize,
    pub(crate) duplicates_removed: usize,
    pub(crate) ignored_records: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PreparedLexicon {
    pub(crate) metadata: PreparedMetadata,
    pub(crate) report: NormalizationReport,
    pub(crate) payload: PluginPayload,
}

impl RawSourceManifest {
    pub(crate) fn new(
        source_id: impl Into<String>,
        source_url: impl Into<String>,
        source_version: Option<String>,
        bytes: &[u8],
        license_summary: impl Into<String>,
        notice: Option<String>,
    ) -> Self {
        Self {
            source_id: source_id.into(),
            source_url: source_url.into(),
            source_version,
            fetched_at: Utc::now().to_rfc3339(),
            checksum: sha256_hex(bytes),
            license_summary: license_summary.into(),
            notice,
        }
    }
}

impl PreparedLexicon {
    pub(crate) fn entry_count(&self) -> usize {
        self.payload.len()
    }
}

pub(crate) fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    fsutil::write_bytes(path, &serde_json::to_vec_pretty(value)?)
}

pub(crate) fn load_raw_manifest(path: &Path) -> Result<RawSourceManifest> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

pub(crate) fn load_prepared(path: &Path) -> Result<PreparedLexicon> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}
