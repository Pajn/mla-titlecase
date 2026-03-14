# mla-titlecase

`mla-titlecase` is a Rust workspace for MLA-style title casing.

It contains:

- `mla-titlecase`: the library crate that applies the MLA rule engine
- `mla-titlecase-cli`: a companion CLI for preparing and inspecting lexicon plugins

The implementation is intentionally deterministic without any network dependency in the library itself. External lexicons are optional and additive.

More complete documentation, examples, benchmarks, and CLI guidance are added in later commits.
