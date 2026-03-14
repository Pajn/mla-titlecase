use crate::{error::Result, sources};

pub(crate) fn run() -> Result<()> {
    for source in sources::all_sources() {
        let status = if source.recommended { "recommended" } else { "optional" };
        println!(
            "{:<14} {:<11} {:<50} {}",
            source.id.as_str(),
            status,
            source.license_summary,
            source.description
        );
    }
    Ok(())
}
