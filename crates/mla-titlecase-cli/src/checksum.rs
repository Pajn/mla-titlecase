use sha2::{Digest, Sha256};

use crate::error::Result;

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub(crate) fn file_sha256_hex(path: &std::path::Path) -> Result<String> {
    Ok(sha256_hex(&std::fs::read(path)?))
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn hashes_bytes() {
        assert_eq!(
            sha256_hex(b"mla-titlecase"),
            "6e0e92818196eb1f1c217da40ca6aa0da8ead07d8cb4945ccda84ba468d9cd88"
        );
    }
}
