use crate::{cli::ShowLicenseArgs, error::Result, sources};

pub(crate) fn run(args: ShowLicenseArgs) -> Result<()> {
    let definition = sources::source_definition(args.source);
    println!("source: {}", definition.id.as_str());
    println!("license: {}", definition.license_summary);
    if !definition.notice.is_empty() {
        println!("notice: {}", definition.notice);
    }
    println!("default-url: {}", definition.default_url);
    Ok(())
}
