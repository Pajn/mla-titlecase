use serde::Serialize;

use crate::{
    cli::InspectPluginArgs,
    error::{CliError, Result},
};

#[derive(Debug, Serialize)]
struct PluginSummary {
    format: &'static str,
    payload_kind: mla_titlecase::PluginPayloadKind,
    entry_count: usize,
    multiword_entries: usize,
    source_id: String,
    source_version: Option<String>,
    license_summary: String,
    checksum: Option<String>,
    notice_present: bool,
}

pub(crate) fn run(args: InspectPluginArgs) -> Result<()> {
    let (format, plugin) = load_plugin(&args.path)?;
    let summary = PluginSummary {
        format,
        payload_kind: plugin.payload_kind(),
        entry_count: plugin.payload.len(),
        multiword_entries: payload_multiword_entries(&plugin.payload),
        source_id: plugin.metadata.source_id.clone(),
        source_version: plugin.metadata.source_version.clone(),
        license_summary: plugin.metadata.license_summary.clone(),
        checksum: plugin.metadata.checksum.clone(),
        notice_present: plugin.metadata.notice.is_some(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("format: {}", summary.format);
        println!("payload-kind: {:?}", summary.payload_kind);
        println!("entries: {}", summary.entry_count);
        println!("multiword-entries: {}", summary.multiword_entries);
        println!("source-id: {}", summary.source_id);
        if let Some(version) = summary.source_version {
            println!("source-version: {version}");
        }
        println!("license: {}", summary.license_summary);
        if let Some(checksum) = summary.checksum {
            println!("checksum: {checksum}");
        }
        println!("notice-present: {}", summary.notice_present);
    }
    Ok(())
}

fn payload_multiword_entries(payload: &mla_titlecase::PluginPayload) -> usize {
    match payload {
        mla_titlecase::PluginPayload::MultiwordMap { entries } => entries.len(),
        _ => 0,
    }
}

pub(crate) fn load_plugin(
    path: &std::path::Path,
) -> Result<(&'static str, mla_titlecase::LexiconPlugin)> {
    if path.extension().and_then(|value| value.to_str()) == Some("json") {
        return Ok(("json", mla_titlecase::json_store::load_json_plugin(path)?));
    }

    let bytes = std::fs::read(path)?;
    if bytes.starts_with(b"MLATFST1") {
        return Ok(("fst", mla_titlecase::fst_store::load_fst_plugin(path)?));
    }

    if bytes.iter().copied().find(|byte| !byte.is_ascii_whitespace()) == Some(b'{') {
        return Ok(("json", mla_titlecase::json_store::load_json_plugin(path)?));
    }

    Err(CliError::UnsupportedFormat(path.to_path_buf()))
}
