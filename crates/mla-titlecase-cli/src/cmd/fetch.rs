use std::path::{Path, PathBuf};

use crate::{
    cli::FetchArgs,
    error::Result,
    fsutil,
    manifest::{save_json, RawSourceManifest},
    sources,
};

pub(crate) fn run(args: FetchArgs) -> Result<()> {
    sources::require_acknowledgement(args.source, args.acknowledge_cc_by_sa)?;
    let definition = sources::source_definition(args.source);
    let resolved = if let Some(path) = args.from_file.as_ref() {
        fsutil::copy_file(path, &args.output)?;
        sources::ResolvedSource {
            bytes: std::fs::read(&args.output)?,
            source_url: format!("file://{}", path.display()),
            source_version: None,
            license_summary: definition.license_summary.to_string(),
            notice: (!definition.notice.is_empty()).then(|| definition.notice.to_string()),
        }
    } else if let Some(url) = args.url.as_ref() {
        let client = sources::github::client()?;
        sources::ResolvedSource {
            bytes: sources::github::download_bytes(&client, url)?,
            source_url: url.clone(),
            source_version: None,
            license_summary: definition.license_summary.to_string(),
            notice: (!definition.notice.is_empty()).then(|| definition.notice.to_string()),
        }
    } else {
        let client = sources::github::client()?;
        sources::fetch_default(
            args.source,
            &client,
            &sources::FetchOptions {
                query: args.query.clone(),
                language: args.language.clone(),
                limit: args.limit,
            },
        )?
    };

    if args.from_file.is_none() {
        fsutil::write_bytes(&args.output, &resolved.bytes)?;
    }
    let manifest = RawSourceManifest::new(
        definition.id.as_str(),
        resolved.source_url,
        resolved.source_version,
        &resolved.bytes,
        resolved.license_summary,
        resolved.notice,
    );
    let manifest_path =
        args.manifest.clone().unwrap_or_else(|| default_manifest_path(&args.output));
    save_json(&manifest_path, &manifest)?;

    println!(
        "fetched {} bytes from {} to {}",
        resolved.bytes.len(),
        manifest.source_url,
        args.output.display()
    );
    if let Some(source_version) = manifest.source_version.as_deref() {
        println!("resolved version: {source_version}");
    }
    Ok(())
}

fn default_manifest_path(output: &Path) -> PathBuf {
    PathBuf::from(format!("{}.manifest.json", output.display()))
}
