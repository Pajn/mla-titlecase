use mla_titlecase::{LexiconPlugin, PluginMetadata};

use crate::manifest::PreparedLexicon;

pub(crate) fn build_plugin(prepared: &PreparedLexicon) -> LexiconPlugin {
    LexiconPlugin {
        metadata: PluginMetadata {
            schema_version: mla_titlecase::PLUGIN_SCHEMA_VERSION,
            plugin_version: 1,
            source_id: prepared.metadata.source_id.clone(),
            source_version: prepared.metadata.source_version.clone(),
            upstream_url: prepared.metadata.source_url.clone(),
            prepared_at: prepared.metadata.prepared_at.clone(),
            checksum: Some(prepared.metadata.input_checksum.clone()),
            license_summary: prepared.metadata.license_summary.clone(),
            notice: prepared.metadata.notice.clone(),
        },
        payload: prepared.payload.clone(),
    }
}
