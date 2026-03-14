# mla-titlecase

`mla-titlecase` is a Rust workspace for MLA-style title casing.

It ships two crates:

- `mla-titlecase`: a library crate with a deterministic MLA rule engine and optional external lexicon support
- `mla-titlecase-cli`: a companion CLI for fetching, preparing, building, inspecting, and diffing lexicon plugins

The library stays useful without any downloaded data. Built-in MLA rules remain authoritative; external lexicons are additive and opt-in.

## Quickstart

### Library

```rust
use mla_titlecase::titlecase_mla;

assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");
```

You can also protect spellings or register external lexicons:

```rust
use mla_titlecase::{titlecase_with_options, ExternalLexicons, TitleCaseOptions};

let mut lexicons = ExternalLexicons::default();
lexicons.add_protected_spellings([("github", "GitHub")]);

let options = TitleCaseOptions::with_external_lexicons(&lexicons);
assert_eq!(
    titlecase_with_options("github in practice", &options),
    "GitHub in Practice"
);
```

### CLI

List supported sources:

```bash
cargo run -p mla-titlecase-cli -- lexicon list-sources
```

Fetch a real upstream source, prepare it, and build a plugin:

```bash
cargo run -p mla-titlecase-cli -- \
  lexicon fetch stopwords-iso \
  --output /tmp/stopwords-raw.json

cargo run -p mla-titlecase-cli -- \
  lexicon prepare stopwords-iso \
  --input /tmp/stopwords-raw.json \
  --output /tmp/stopwords-prepared.json

cargo run -p mla-titlecase-cli -- \
  lexicon build-plugin /tmp/stopwords-prepared.json \
  --format fst \
  --output /tmp/stopwords.mlatl
```

For authority-style sources such as Wikidata, choose the payload shape during `prepare`:

```bash
cargo run -p mla-titlecase-cli -- \
  lexicon fetch wikidata \
  --output /tmp/wikidata.json \
  --language en \
  --limit 250

cargo run -p mla-titlecase-cli -- \
  lexicon prepare wikidata \
  --input /tmp/wikidata.json \
  --output /tmp/wikidata-prepared.json \
  --payload-kind multiword-map
```

Inspect or diff plugins:

```bash
cargo run -p mla-titlecase-cli -- lexicon inspect-plugin /tmp/stopwords.mlatl --json
cargo run -p mla-titlecase-cli -- lexicon diff-plugin left.json right.mlatl --json
```

## JSON vs FST plugins

- Use JSON plugins when you want readable artifacts that are easy to inspect or edit manually.
- Use FST plugins when you want compact, deterministic, machine-oriented artifacts.
- Use `ExternalLexicons::register_mmap_fst_plugin` when you want the runtime to query an FST plugin directly from a memory map.
- Both formats round-trip through the same library schema.

## Licensing notes for fetchable sources

The CLI preserves source metadata and notice text in prepared/plugin artifacts.

- `scowl` is the recommended general English membership source.
- `gnd` is a narrower CC0 authority source aimed at German and European person-name coverage.
- `musicbrainz` is a music-specific CC0 source for artist names and stylized forms.
- `stopwords-iso` is convenient heuristic input, but it does not define MLA semantics.
- `wikidata` is the first optional authority-style source and defaults to a live CC0 SPARQL query.
- `wordfreq` is opt-in and requires `--acknowledge-cc-by-sa`.

## Docs index

- `docs/architecture.md`
- `docs/mla-rules.md`
- `docs/lexicon-sources.md`
- `docs/plugin-format.md`
- `docs/performance.md`

## Examples and benches

Examples live in `examples/` and are wired into the library crate manifest so `cargo test --examples -p mla-titlecase` compiles them.

Criterion benches live in `benches/` and cover title casing, lookup behavior, and plugin load costs.

## Contributor note

Before sending changes, run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```
