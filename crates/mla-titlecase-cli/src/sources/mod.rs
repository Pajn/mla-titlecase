use serde::{Deserialize, Serialize};

use crate::{
    error::{CliError, Result},
    manifest::{PreparedLexicon, PreparedMetadata},
};

pub(crate) mod scowl;
pub(crate) mod stopwords_iso;
pub(crate) mod wordfreq;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, clap::ValueEnum, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceId {
    Scowl,
    StopwordsIso,
    Wordfreq,
}

impl SourceId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Scowl => "scowl",
            Self::StopwordsIso => "stopwords-iso",
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

pub(crate) fn all_sources() -> [SourceDefinition; 3] {
    [scowl::definition(), stopwords_iso::definition(), wordfreq::definition()]
}

pub(crate) fn source_definition(source: SourceId) -> SourceDefinition {
    match source {
        SourceId::Scowl => scowl::definition(),
        SourceId::StopwordsIso => stopwords_iso::definition(),
        SourceId::Wordfreq => wordfreq::definition(),
    }
}

pub(crate) fn prepare_source(
    source: SourceId,
    raw: &str,
    source_url: Option<String>,
    input_checksum: String,
) -> Result<PreparedLexicon> {
    let definition = source_definition(source);
    let payload = match source {
        SourceId::Scowl => scowl::prepare(raw),
        SourceId::StopwordsIso => stopwords_iso::prepare(raw)?,
        SourceId::Wordfreq => wordfreq::prepare(raw)?,
    };

    Ok(PreparedLexicon {
        metadata: PreparedMetadata {
            source_id: definition.id.as_str().to_string(),
            source_version: None,
            source_url,
            prepared_at: chrono::Utc::now().to_rfc3339(),
            input_checksum,
            license_summary: definition.license_summary.to_string(),
            notice: (!definition.notice.is_empty()).then(|| definition.notice.to_string()),
        },
        payload,
    })
}

pub(crate) fn require_acknowledgement(source: SourceId, acknowledged: bool) -> Result<()> {
    let definition = source_definition(source);
    if definition.requires_acknowledgement && !acknowledged {
        return Err(CliError::MissingAcknowledgement(definition.id.as_str()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{all_sources, source_definition, SourceId};

    #[test]
    fn exposes_source_registry() {
        let sources = all_sources();
        assert_eq!(sources.len(), 3);
        assert_eq!(source_definition(SourceId::Scowl).id.as_str(), "scowl");
    }
}
