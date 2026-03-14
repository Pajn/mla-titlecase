use crate::{
    cli::PreparePayloadKind,
    error::Result,
    normalize::{parse_stopwords_json, NormalizedPayload},
    sources::{
        github::{download_bytes, download_text, resolve_file},
        require_payload_kind, PrepareOptions, ResolvedSource, SourceDefinition, SourceId,
    },
};

const OWNER: &str = "stopwords-iso";
const REPO: &str = "stopwords-en";
const REF: &str = "master";
const DATA_PATH: &str = "stopwords-en.json";
const LICENSE_PATH: &str = "LICENSE";

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::StopwordsIso,
        description: "Heuristic English stopword candidates from stopwords-iso",
        license_summary: "MIT License",
        notice:
            "stopwords-iso is useful for heuristics, not for defining MLA small-word semantics.",
        default_url:
            "https://raw.githubusercontent.com/stopwords-iso/stopwords-en/master/stopwords-en.json",
        recommended: false,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(client: &reqwest::blocking::Client) -> Result<ResolvedSource> {
    let data = resolve_file(client, OWNER, REPO, DATA_PATH, REF)?;
    let license = resolve_file(client, OWNER, REPO, LICENSE_PATH, REF)?;

    Ok(ResolvedSource {
        bytes: download_bytes(client, &data.download_url)?,
        source_url: data.download_url,
        source_version: Some(data.sha),
        license_summary: definition().license_summary.to_string(),
        notice: Some(download_text(client, &license.download_url)?),
    })
}

pub(crate) fn prepare(raw: &[u8], options: PrepareOptions) -> Result<NormalizedPayload> {
    require_payload_kind(
        SourceId::StopwordsIso,
        options.payload_kind,
        PreparePayloadKind::WordSet,
    )?;
    parse_stopwords_json(raw)
}
