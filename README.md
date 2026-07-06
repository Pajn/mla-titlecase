# mla-titlecase

A Rust workspace for MLA-style title casing.

```rust
use mla_titlecase::titlecase_mla;

assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");
```

The engine is deterministic and needs no downloaded data — the built-in MLA
rules are authoritative. Optional external lexicons (names, brands, stop words)
are additive and opt-in.

## Crates

| Crate | What it is | Start here |
| --- | --- | --- |
| [`mla-titlecase`](crates/mla-titlecase/README.md) | The title-casing library. | [crate README](crates/mla-titlecase/README.md) |
| [`mla-titlecase-cli`](crates/mla-titlecase-cli/README.md) | Tooling that builds the optional lexicon plugins. It does **not** recase text. | [crate README](crates/mla-titlecase-cli/README.md) |

Most users only need the library. Reach for the CLI when you want to enrich
casing with external name/brand data.

## Install

```toml
[dependencies]
mla-titlecase = "0.2"
```

```bash
cargo install mla-titlecase-cli   # optional plugin-building CLI
```

## What to read

- **Using the library** → [`crates/mla-titlecase/README.md`](crates/mla-titlecase/README.md):
  the four entry points, options, locales, and rich analysis output.
- **Building lexicon plugins** → [`crates/mla-titlecase-cli/README.md`](crates/mla-titlecase-cli/README.md):
  the fetch → prepare → build-plugin pipeline.
- **Everything else** → [`docs/`](docs/README.md): the exact rules, architecture,
  plugin format, source catalog, and performance notes.

## Contributing

Before sending changes:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```
