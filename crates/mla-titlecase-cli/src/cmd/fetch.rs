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
    let (bytes, source_url) = if let Some(path) = args.from_file.as_ref() {
        fsutil::copy_file(path, &args.output)?;
        (std::fs::read(&args.output)?, format!("file://{}", path.display()))
    } else {
        let url = args.url.clone().unwrap_or_else(|| definition.default_url.to_string());
        let response = reqwest::blocking::get(&url)?.error_for_status()?;
        (response.bytes()?.to_vec(), url)
    };

    if args.from_file.is_none() {
        fsutil::write_bytes(&args.output, &bytes)?;
    }
    let manifest = RawSourceManifest::new(
        definition.id.as_str(),
        source_url,
        &bytes,
        definition.license_summary,
        (!definition.notice.is_empty()).then(|| definition.notice.to_string()),
    );
    let manifest_path =
        args.manifest.clone().unwrap_or_else(|| default_manifest_path(&args.output));
    save_json(&manifest_path, &manifest)?;

    println!(
        "fetched {} bytes from {} to {}",
        bytes.len(),
        manifest.source_url,
        args.output.display()
    );
    Ok(())
}

fn default_manifest_path(output: &Path) -> PathBuf {
    PathBuf::from(format!("{}.manifest.json", output.display()))
}
