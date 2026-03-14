use mla_titlecase::PluginPayload;

use crate::normalize::parse_scowl_word_list;
use crate::sources::{SourceDefinition, SourceId};

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Scowl,
        description: "General English word membership from SCOWL-style lists",
        license_summary: "SCOWL licensing applies; preserve upstream notices",
        notice: "SCOWL-derived outputs should preserve the upstream license notice.",
        default_url: "https://wordlist.aspell.net/scowl-readme/",
        recommended: true,
        requires_acknowledgement: false,
    }
}

pub(crate) fn prepare(raw: &str) -> PluginPayload {
    parse_scowl_word_list(raw)
}
