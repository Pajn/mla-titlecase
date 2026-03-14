use crate::{
    error::Result,
    normalize::{parse_wordfreq_msgpack, NormalizedPayload},
    sources::{
        github::{download_bytes, download_text, resolve_file},
        ResolvedSource, SourceDefinition, SourceId,
    },
};

const OWNER: &str = "rspeer";
const REPO: &str = "wordfreq";
const REF: &str = "master";
const DATA_PATH: &str = "wordfreq/data/small_en.msgpack.gz";
const NOTICE_PATH: &str = "NOTICE.md";

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Wordfreq,
        description: "Ranked English words from the wordfreq small cBpack dataset",
        license_summary:
            "Apache-2.0 package with included CC-BY-SA-derived data; preserve attribution and notice text",
        notice: "wordfreq data requires explicit acknowledgement and preserved upstream notice text.",
        default_url:
            "https://raw.githubusercontent.com/rspeer/wordfreq/master/wordfreq/data/small_en.msgpack.gz",
        recommended: false,
        requires_acknowledgement: true,
    }
}

pub(crate) fn fetch(client: &reqwest::blocking::Client) -> Result<ResolvedSource> {
    let data = resolve_file(client, OWNER, REPO, DATA_PATH, REF)?;
    let notice = resolve_file(client, OWNER, REPO, NOTICE_PATH, REF)?;

    Ok(ResolvedSource {
        bytes: download_bytes(client, &data.download_url)?,
        source_url: data.download_url,
        source_version: Some(data.sha),
        license_summary: definition().license_summary.to_string(),
        notice: Some(download_text(client, &notice.download_url)?),
    })
}

pub(crate) fn prepare(raw: &[u8]) -> Result<NormalizedPayload> {
    parse_wordfreq_msgpack(raw)
}
