use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    cli::PreparePayloadKind,
    error::{CliError, Result},
    manifest::{NormalizationReport, PreparedLexicon, PreparedMetadata, RawSourceManifest},
    normalize::NormalizedPayload,
};

pub(crate) mod github;
pub(crate) mod scowl;
pub(crate) mod stopwords_iso;
pub(crate) mod wikidata;
pub(crate) mod wordfreq;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceId {
    Scowl,
    StopwordsIso,
    Wikidata,
    Wordfreq,
}

impl SourceId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Scowl => "scowl",
            Self::StopwordsIso => "stopwords-iso",
            Self::Wikidata => "wikidata",
            Self::Wordfreq => "wordfreq",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SourceDefinition {
    pub(crate) id: SourceId,
    pub(crate) description: &'static str,
    pub(crate) license_summary: &'static str,
    pub(crate) notice: &'static str,
    pub(crate) default_url: &'static str,
    pub(crate) recommended: bool,
    pub(crate) requires_acknowledgement: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSource {
    pub(crate) bytes: Vec<u8>,
    pub(crate) source_url: String,
    pub(crate) source_version: Option<String>,
    pub(crate) license_summary: String,
    pub(crate) notice: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PrepareContext {
    pub(crate) source_url: Option<String>,
    pub(crate) source_version: Option<String>,
    pub(crate) input_checksum: String,
    pub(crate) license_summary: Option<String>,
    pub(crate) notice: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FetchOptions {
    pub(crate) query: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PrepareOptions {
    pub(crate) payload_kind: Option<PreparePayloadKind>,
}

pub(crate) fn all_sources() -> [SourceDefinition; 4] {
    [
        scowl::definition(),
        stopwords_iso::definition(),
        wikidata::definition(),
        wordfreq::definition(),
    ]
}

pub(crate) fn source_definition(source: SourceId) -> SourceDefinition {
    match source {
        SourceId::Scowl => scowl::definition(),
        SourceId::StopwordsIso => stopwords_iso::definition(),
        SourceId::Wikidata => wikidata::definition(),
        SourceId::Wordfreq => wordfreq::definition(),
    }
}

pub(crate) fn fetch_default(
    source: SourceId,
    client: &reqwest::blocking::Client,
    options: &FetchOptions,
) -> Result<ResolvedSource> {
    match source {
        SourceId::Scowl => scowl::fetch(client),
        SourceId::StopwordsIso => stopwords_iso::fetch(client),
        SourceId::Wikidata => wikidata::fetch(client, options),
        SourceId::Wordfreq => wordfreq::fetch(client),
    }
}

pub(crate) fn prepare_source(
    source: SourceId,
    raw: &[u8],
    context: PrepareContext,
    options: PrepareOptions,
) -> Result<PreparedLexicon> {
    let definition = source_definition(source);
    let NormalizedPayload { payload, report } = match source {
        SourceId::Scowl => scowl::prepare(raw, options)?,
        SourceId::StopwordsIso => stopwords_iso::prepare(raw, options)?,
        SourceId::Wikidata => wikidata::prepare(raw, options)?,
        SourceId::Wordfreq => wordfreq::prepare(raw, options)?,
    };

    Ok(PreparedLexicon {
        metadata: PreparedMetadata {
            source_id: definition.id.as_str().to_string(),
            source_version: context.source_version,
            source_url: context.source_url,
            prepared_at: chrono::Utc::now().to_rfc3339(),
            input_checksum: context.input_checksum,
            license_summary: context
                .license_summary
                .unwrap_or_else(|| definition.license_summary.to_string()),
            notice: context
                .notice
                .or_else(|| (!definition.notice.is_empty()).then(|| definition.notice.to_string())),
        },
        report,
        payload,
    })
}

pub(crate) fn prepare_context_from_manifest(
    source: SourceId,
    manifest: Option<RawSourceManifest>,
    source_url_override: Option<String>,
    input_checksum: String,
) -> PrepareContext {
    let definition = source_definition(source);
    PrepareContext {
        source_url: source_url_override
            .or_else(|| manifest.as_ref().map(|item| item.source_url.clone())),
        source_version: manifest.as_ref().and_then(|item| item.source_version.clone()),
        input_checksum,
        license_summary: manifest
            .as_ref()
            .map(|item| item.license_summary.clone())
            .or_else(|| Some(definition.license_summary.to_string())),
        notice: manifest
            .and_then(|item| item.notice)
            .or_else(|| (!definition.notice.is_empty()).then(|| definition.notice.to_string())),
    }
}

pub(crate) fn default_manifest_path(input: &Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.manifest.json", input.display()))
}

pub(crate) fn require_acknowledgement(source: SourceId, acknowledged: bool) -> Result<()> {
    let definition = source_definition(source);
    if definition.requires_acknowledgement && !acknowledged {
        return Err(CliError::MissingAcknowledgement(definition.id.as_str()));
    }
    Ok(())
}

pub(crate) fn format_prepare_summary(report: &NormalizationReport) -> String {
    format!(
        "{} input records -> {} normalized entries ({} duplicates removed, {} ignored)",
        report.input_records,
        report.output_entries,
        report.duplicates_removed,
        report.ignored_records
    )
}

pub(crate) fn require_payload_kind(
    source: SourceId,
    requested: Option<PreparePayloadKind>,
    expected: PreparePayloadKind,
) -> Result<()> {
    if let Some(kind) = requested {
        if kind != expected {
            return Err(CliError::SourceMetadata(format!(
                "{} only supports --payload-kind {}",
                source.as_str(),
                payload_kind_name(expected)
            )));
        }
    }
    Ok(())
}

pub(crate) const fn payload_kind_name(kind: PreparePayloadKind) -> &'static str {
    match kind {
        PreparePayloadKind::WordSet => "word-set",
        PreparePayloadKind::CanonicalMap => "canonical-map",
        PreparePayloadKind::MultiwordMap => "multiword-map",
        PreparePayloadKind::RankedWords => "ranked-words",
        PreparePayloadKind::ProtectedSpellings => "protected-spellings",
    }
}

#[cfg(test)]
mod tests {
    use super::{all_sources, source_definition, SourceId};

    #[test]
    fn exposes_source_registry() {
        let sources = all_sources();
        assert_eq!(sources.len(), 4);
        assert_eq!(source_definition(SourceId::Scowl).id.as_str(), "scowl");
    }
}
