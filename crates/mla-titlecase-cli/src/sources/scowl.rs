use crate::{
    cli::PreparePayloadKind,
    error::Result,
    normalize::{parse_scowl_word_list, NormalizedPayload},
    sources::{
        github::{download_bytes, download_text, resolve_file},
        require_payload_kind, PrepareOptions, ResolvedSource, SourceDefinition, SourceId,
    },
};

const OWNER: &str = "en-wl";
const REPO: &str = "wordlist";
const REF: &str = "v2";
const DATA_PATH: &str = "data/scowl-pre.txt";
const COPYRIGHT_PATH: &str = "Copyright";

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Scowl,
        description: "General English word membership extracted from SCOWL v2 preformat data",
        license_summary: "SCOWL / English Speller Database terms; preserve upstream notices",
        notice: "SCOWL-derived outputs should preserve the upstream Copyright notice.",
        default_url: "https://raw.githubusercontent.com/en-wl/wordlist/v2/data/scowl-pre.txt",
        recommended: true,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(client: &reqwest::blocking::Client) -> Result<ResolvedSource> {
    let data = resolve_file(client, OWNER, REPO, DATA_PATH, REF)?;
    let copyright = resolve_file(client, OWNER, REPO, COPYRIGHT_PATH, REF)?;

    Ok(ResolvedSource {
        bytes: download_bytes(client, &data.download_url)?,
        source_url: data.download_url,
        source_version: Some(data.sha),
        license_summary: definition().license_summary.to_string(),
        notice: Some(format!(
            "Source page: {}\n\n{}",
            data.html_url,
            download_text(client, &copyright.download_url)?
        )),
    })
}

pub(crate) fn prepare(raw: &[u8], options: PrepareOptions) -> Result<NormalizedPayload> {
    require_payload_kind(SourceId::Scowl, options.payload_kind, PreparePayloadKind::WordSet)?;
    parse_scowl_word_list(raw)
}
