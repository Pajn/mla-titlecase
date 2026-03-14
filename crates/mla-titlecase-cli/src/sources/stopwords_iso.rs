use mla_titlecase::PluginPayload;

use crate::{
    error::Result,
    normalize::parse_stopwords_json,
    sources::{SourceDefinition, SourceId},
};

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::StopwordsIso,
        description: "Stopword candidates from stopwords-iso (heuristic only)",
        license_summary: "MIT-like upstream terms; check the current stopwords-iso repository",
        notice:
            "stopwords-iso is useful for heuristics, not for defining MLA small-word semantics.",
        default_url:
            "https://raw.githubusercontent.com/stopwords-iso/stopwords-en/master/stopwords-en.json",
        recommended: false,
        requires_acknowledgement: false,
    }
}

pub(crate) fn prepare(raw: &str) -> Result<PluginPayload> {
    parse_stopwords_json(raw)
}
