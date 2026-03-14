use crate::{cli::ShowLicenseArgs, error::Result, sources};

pub(crate) fn run(args: ShowLicenseArgs) -> Result<()> {
    let definition = sources::source_definition(args.source);
    println!("source: {}", definition.id.as_str());
    println!("license: {}", definition.license_summary);
    println!("default-artifact: {}", definition.default_url);
    println!(
        "acknowledgement-required: {}",
        if definition.requires_acknowledgement { "yes" } else { "no" }
    );
    if !definition.notice.is_empty() {
        println!("notice: {}", definition.notice);
    }
    Ok(())
}
