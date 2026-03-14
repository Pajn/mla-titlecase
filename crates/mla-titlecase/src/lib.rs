//! Core library crate for MLA-style title casing.
//!
//! The initial scaffold keeps the crate buildable while the rule engine,
//! plugin support, and public API are added in later commits.

/// Returns the package version for quick smoke testing during early scaffolding.
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn exposes_package_version() {
        assert!(!version().is_empty());
    }
}
