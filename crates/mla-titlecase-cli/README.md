# mla-titlecase-cli

Command-line tooling for building the optional lexicon plugins that the
[`mla-titlecase`](../mla-titlecase/README.md) library can load.

**This binary does not recase text.** Title casing lives in the library. The CLI
exists to turn upstream data (names, brands, stop words, word frequencies) into
plugin files the library can consume. If you only need MLA title casing, you do
not need this crate at all — the library works with zero downloaded data.

The installed executable is named `mla-titlecase`:

```bash
cargo install mla-titlecase-cli   # from crates.io
cargo binstall mla-titlecase-cli  # prebuilt binary from GitHub Releases
```

## The pipeline

Building a plugin is three steps: **fetch** raw upstream data, **prepare** it
into normalized JSON, then **build-plugin** into the format the library loads.

```bash
# 1. fetch a raw source artifact (records provenance in a fetch manifest)
mla-titlecase lexicon fetch stopwords-iso --output /tmp/stopwords-raw.json

# 2. prepare it into normalized JSON (inherits the manifest's provenance)
mla-titlecase lexicon prepare stopwords-iso \
  --input /tmp/stopwords-raw.json \
  --output /tmp/stopwords-prepared.json

# 3. build a plugin: --format json (readable) or fst (compact, mmap-friendly)
mla-titlecase lexicon build-plugin /tmp/stopwords-prepared.json \
  --format fst \
  --output /tmp/stopwords.mlatl
```

The library then loads the result:

```rust,ignore
let mut lexicons = ExternalLexicons::default();
lexicons.register_mmap_fst_plugin("/tmp/stopwords.mlatl")?;
```

Authority-style sources (people, organizations, works) can produce different
plugin shapes; choose one during `prepare` with `--payload-kind`:

```bash
mla-titlecase lexicon fetch wikidata --output /tmp/wikidata.json --language en --limit 250
mla-titlecase lexicon prepare wikidata \
  --input /tmp/wikidata.json \
  --output /tmp/wikidata-prepared.json \
  --payload-kind multiword-map
```

## Commands

All commands live under `lexicon`:

| Command | Purpose |
| --- | --- |
| `list-sources` | List the supported upstream sources. |
| `show-license <source>` | Print a source's licensing details. |
| `fetch <source> --output <path>` | Download a raw source artifact and write a fetch manifest. |
| `prepare <source> --input <raw> --output <json>` | Normalize raw data into a prepared JSON payload. |
| `build-plugin <prepared> --format <json\|fst> --output <path>` | Build a loadable plugin. |
| `inspect-plugin <path> [--json]` | Report a plugin's format, payload kind, and entry counts. |
| `diff-plugin <left> <right> [--json]` | Compare two plugins. |

Run `mla-titlecase lexicon <command> --help` for the full flag list.

## Sources and licensing

`fetch` preserves the exact upstream URL and license/notice text in a manifest,
which `prepare` carries into the plugin metadata — but it does not make license
decisions for you. Review the upstream terms before redistributing any generated
artifact. `wordfreq` in particular carries share-alike-sensitive data and
requires `--acknowledge-cc-by-sa`.

See [`docs/lexicon-sources.md`](../../docs/lexicon-sources.md) for the catalog:
what each source provides, its license, and when to prefer it.
