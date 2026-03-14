# Architecture

## Workspace layout

The repository is split into two crates:

- `crates/mla-titlecase`: authoritative MLA rule engine plus plugin schema/loaders
- `crates/mla-titlecase-cli`: source-aware tooling for fetch/prepare/build/inspect workflows

The library is intentionally network-free. The CLI owns download concerns, source manifests, and source-specific parsing.

## Title-casing flow

1. `tokenizer.rs` preserves whitespace and punctuation while splitting word segments.
2. `context.rs` identifies first/last significant words, colon boundaries, and hyphen context.
3. `rules.rs` applies MLA precedence rules.
4. `casing.rs` preserves acronyms, dotted abbreviations, protected spellings, and mixed-case forms when configured.
5. optional locale profiles and multiword external lexicons are consulted only when callers opt in through `TitleCaseOptions` or loaded plugins.

## Lexicon lookup precedence

Lookup stays deterministic:

1. user-provided protected words
2. built-in protected spellings and abbreviations
3. external multiword maps, then external protected spellings and canonical maps
4. built-in MLA small-word list
5. opt-in external word sets only when `SmallWordPolicy::AlwaysLowercase` is chosen

Built-ins remain authoritative for default MLA semantics.

## Plugin paths

The shared plugin schema lives in `plugin.rs`.

- `json_store.rs` handles readable `.json` plugins.
- `fst_store.rs` handles compact `.mlatl` plugins backed by `fst` sets/maps.
- `ExternalLexicons` registers validated plugins into the same lookup container used by the rule engine.

## Extension points

The library includes explicit extension hooks for:

- name-particle heuristics via `NameParticlePolicy`
- locale-aware casing via `LocaleProfile`
- phrase-level canonical overrides via `ExternalLexicons::add_multiword_map`

Both remain opt-in and conservative in the current implementation.

## CLI source flows

The CLI now has two broad source families:

- commit-pinned GitHub artifact sources such as `scowl`, `stopwords-iso`, and `wordfreq`
- live authority-style API sources such as `wikidata`, `gnd`, `musicbrainz`, and `orcid`

Authority-style sources preserve the exact request URL in the fetch manifest and use `prepare --payload-kind ...` to choose an appropriate plugin shape for the downloaded names.
