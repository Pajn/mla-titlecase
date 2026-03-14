use crate::{
    checksum::file_sha256_hex, cli::PrepareArgs, error::Result, manifest::save_json, sources,
};

pub(crate) fn run(args: PrepareArgs) -> Result<()> {
    sources::require_acknowledgement(args.source, args.acknowledge_cc_by_sa)?;
    let raw = std::fs::read_to_string(&args.input)?;
    let prepared = sources::prepare_source(
        args.source,
        &raw,
        args.source_url.clone(),
        file_sha256_hex(&args.input)?,
    )?;
    save_json(&args.output, &prepared)?;

    println!(
        "prepared {} {} entries to {}",
        prepared.entry_count(),
        prepared.metadata.source_id,
        args.output.display()
    );
    Ok(())
}
