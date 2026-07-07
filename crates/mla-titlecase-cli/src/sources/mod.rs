use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    cli::PreparePayloadKind,
    error::{CliError, Result},
    manifest::{NormalizationReport, PreparedLexicon, PreparedMetadata, RawSourceManifest},
    normalize::NormalizedPayload,
};

pub(crate) mod cldr;
pub(crate) mod crossref;
pub(crate) mod github;
pub(crate) mod gnd;
pub(crate) mod musicbrainz;
pub(crate) mod natural_earth;
pub(crate) mod orcid;
pub(crate) mod ror;
pub(crate) mod scowl;
pub(crate) mod stopwords_iso;
pub(crate) mod wikidata;
pub(crate) mod wordfreq;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceId {
    Cldr,
    Crossref,
    Gnd,
    Musicbrainz,
    NaturalEarth,
    Orcid,
    Ror,
    Scowl,
    StopwordsIso,
    Wikidata,
    Wordfreq,
}

impl SourceId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Cldr => "cldr",
            Self::Crossref => "crossref",
            Self::Gnd => "gnd",
            Self::Musicbrainz => "musicbrainz",
            Self::NaturalEarth => "natural-earth",
            Self::Orcid => "orcid",
            Self::Ror => "ror",
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

pub(crate) fn all_sources() -> [SourceDefinition; 11] {
    [
        cldr::definition(),
        crossref::definition(),
        gnd::definition(),
        musicbrainz::definition(),
        natural_earth::definition(),
        orcid::definition(),
        ror::definition(),
        scowl::definition(),
        stopwords_iso::definition(),
        wikidata::definition(),
        wordfreq::definition(),
    ]
}

pub(crate) fn source_definition(source: SourceId) -> SourceDefinition {
    match source {
        SourceId::Cldr => cldr::definition(),
        SourceId::Crossref => crossref::definition(),
        SourceId::Gnd => gnd::definition(),
        SourceId::Musicbrainz => musicbrainz::definition(),
        SourceId::NaturalEarth => natural_earth::definition(),
        SourceId::Orcid => orcid::definition(),
        SourceId::Ror => ror::definition(),
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
        SourceId::Cldr => cldr::fetch(client),
        SourceId::Crossref => crossref::fetch(client, options),
        SourceId::Gnd => gnd::fetch(client, options),
        SourceId::Musicbrainz => musicbrainz::fetch(client, options),
        SourceId::NaturalEarth => natural_earth::fetch(client),
        SourceId::Orcid => orcid::fetch(client, options),
        SourceId::Ror => ror::fetch(client, options),
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
        SourceId::Cldr => cldr::prepare(raw, options)?,
        SourceId::Crossref => crossref::prepare(raw, options)?,
        SourceId::Gnd => gnd::prepare(raw, options)?,
        SourceId::Musicbrainz => musicbrainz::prepare(raw, options)?,
        SourceId::NaturalEarth => natural_earth::prepare(raw, options)?,
        SourceId::Orcid => orcid::prepare(raw, options)?,
        SourceId::Ror => ror::prepare(raw, options)?,
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

/// Whether a surface form has more than one whitespace-separated token, i.e.
/// belongs in a multiword map rather than a single-word map.
pub(crate) fn is_multiword(value: &str) -> bool {
    value.split_whitespace().nth(1).is_some()
}

/// Validates that `requested` is one of `allowed`, producing the shared
/// "supports only --payload-kind ..." error otherwise. Used by sources that
/// accept several payload kinds.
pub(crate) fn validate_payload_kind(
    source: SourceId,
    requested: PreparePayloadKind,
    allowed: &[PreparePayloadKind],
) -> Result<()> {
    if allowed.contains(&requested) {
        return Ok(());
    }
    let names: Vec<&str> = allowed.iter().copied().map(payload_kind_name).collect();
    let list = match names.as_slice() {
        [] => String::new(),
        [only] => (*only).to_string(),
        [head @ .., last] => format!("{}, or {}", head.join(", "), last),
    };
    Err(CliError::SourceMetadata(format!(
        "{} supports only --payload-kind {} (received {})",
        source.as_str(),
        list,
        payload_kind_name(requested)
    )))
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
    use super::{all_sources, is_multiword, source_definition, SourceId};

    #[test]
    fn is_multiword_covers_shared_contract() {
        assert!(!is_multiword(""));
        assert!(!is_multiword("word"));
        assert!(is_multiword("two words"));
        // Irregular whitespace (leading, trailing, tabs, runs) is still one token
        // per non-empty run, so this counts as multiword.
        assert!(is_multiword("  united \t states  "));
        assert!(!is_multiword("   solo   "));
    }

    #[test]
    fn exposes_source_registry() {
        let sources = all_sources();
        assert_eq!(sources.len(), 11);
        assert_eq!(source_definition(SourceId::Scowl).id.as_str(), "scowl");
        assert_eq!(source_definition(SourceId::Crossref).id.as_str(), "crossref");
        assert_eq!(source_definition(SourceId::Cldr).id.as_str(), "cldr");
        assert_eq!(source_definition(SourceId::NaturalEarth).id.as_str(), "natural-earth");
        assert_eq!(source_definition(SourceId::Ror).id.as_str(), "ror");
    }
}
