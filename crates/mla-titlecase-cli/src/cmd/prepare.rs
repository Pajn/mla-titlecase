use crate::{
    checksum::file_sha256_hex,
    cli::PrepareArgs,
    error::Result,
    manifest::{load_raw_manifest, save_json},
    sources,
};

pub(crate) fn run(args: PrepareArgs) -> Result<()> {
    sources::require_acknowledgement(args.source, args.acknowledge_cc_by_sa)?;
    let raw = std::fs::read(&args.input)?;
    let manifest_path =
        args.manifest.clone().unwrap_or_else(|| sources::default_manifest_path(&args.input));
    let manifest = manifest_path.exists().then(|| load_raw_manifest(&manifest_path)).transpose()?;
    let context = sources::prepare_context_from_manifest(
        args.source,
        manifest,
        args.source_url.clone(),
        file_sha256_hex(&args.input)?,
    );
    let prepared = sources::prepare_source(args.source, &raw, context)?;
    save_json(&args.output, &prepared)?;

    println!(
        "prepared {} {} entries to {}",
        prepared.entry_count(),
        prepared.metadata.source_id,
        args.output.display()
    );
    println!("{}", sources::format_prepare_summary(&prepared.report));
    Ok(())
}
