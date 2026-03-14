use std::path::Path;

use tempfile::NamedTempFile;

use crate::error::Result;

pub(crate) fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub(crate) fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    ensure_parent_dir(path)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(parent)?;
    use std::io::Write as _;
    temp.write_all(bytes)?;
    temp.persist(path).map_err(|error| error.error)?;
    Ok(())
}

pub(crate) fn copy_file(from: &Path, to: &Path) -> Result<()> {
    write_bytes(to, &std::fs::read(from)?)
}
