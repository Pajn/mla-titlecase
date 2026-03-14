use mla_titlecase::PluginPayload;

use crate::{
    error::Result,
    normalize::parse_wordfreq_tsv,
    sources::{SourceDefinition, SourceId},
};

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Wordfreq,
        description: "Ranked words from wordfreq",
        license_summary: "CC-BY-SA data; downstream outputs must preserve attribution and share-alike implications",
        notice: "wordfreq data requires explicit acknowledgement and preserved notice text.",
        default_url: "https://github.com/rspeer/wordfreq",
        recommended: false,
        requires_acknowledgement: true,
    }
}

pub(crate) fn prepare(raw: &str) -> Result<PluginPayload> {
    parse_wordfreq_tsv(raw)
}
