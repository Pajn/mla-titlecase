# Documentation

**New here?** Start with the crate READMEs, not this folder:

- Using the library → [`crates/mla-titlecase/README.md`](../crates/mla-titlecase/README.md)
- Building lexicon plugins → [`crates/mla-titlecase-cli/README.md`](../crates/mla-titlecase-cli/README.md)

These guides go deeper once you know what you need:

| Guide | Read it when you want to… |
| --- | --- |
| [mla-rules.md](mla-rules.md) | Know the exact casing rules — small words, particles, colons, hyphens, acronyms, locales — and where the engine deliberately draws the line. |
| [architecture.md](architecture.md) | Understand how the workspace and the title-casing / lookup pipeline fit together. |
| [plugin-format.md](plugin-format.md) | Read or generate the on-disk JSON / FST plugin schema. |
| [lexicon-sources.md](lexicon-sources.md) | Choose an upstream source to build a plugin from: what it provides, its license, when to prefer it. |
| [performance.md](performance.md) | Process titles in bulk and understand the allocation-light path and benchmarks. |
