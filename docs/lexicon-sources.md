# Lexicon sources

The library works with no downloaded data. These sources are *optional* inputs
for the CLI: you fetch one, prepare it, and build a plugin the library can load.
None of them replace the built-in MLA rule engine — they only add canonical
spellings, protected forms, or word-set membership.

The fetch → prepare → build-plugin pipeline itself is documented once, in the
[CLI README](../crates/mla-titlecase-cli/README.md). This page is the catalog:
what each source is and when to reach for it.

> The repository does **not** commit fetched or prepared data. Review each
> upstream's license yourself, then run the CLI locally to produce artifacts you
> are allowed to use.

## At a glance

| Source | Upstream | Output kind | License |
| --- | --- | --- | --- |
| `scowl` | `en-wl/wordlist:data/scowl-pre.txt` (v2) | `WordSet` | Preserve the upstream `Copyright` notice |
| `stopwords-iso` | `stopwords-iso/stopwords-en:stopwords-en.json` | `WordSet` | MIT |
| `wordfreq` | `rspeer/wordfreq:.../small_en.msgpack.gz` | `RankedWords` | CC-BY-SA (requires `--acknowledge-cc-by-sa`) |
| `wikidata` | live SPARQL at `query.wikidata.org` | `CanonicalMap` / `MultiwordMap` / `ProtectedSpellings` | CC0 |
| `gnd` | live query at `lobid.org/gnd/search` | `CanonicalMap` / `MultiwordMap` / `ProtectedSpellings` | CC0 |
| `musicbrainz` | live query at `musicbrainz.org/ws/2/artist/` | `CanonicalMap` / `MultiwordMap` / `ProtectedSpellings` | CC0 |
| `orcid` | live aggregation of `pub.orcid.org` | `CanonicalMap` / `MultiwordMap` / `ProtectedSpellings` | CC0 (data); trademark guidance separate |

GitHub-artifact sources (`scowl`, `stopwords-iso`, `wordfreq`) are resolved
through the GitHub Contents API to a commit-pinned URL and blob SHA, recorded as
`source_version` in the fetch manifest. Live sources instead record the exact
request URL.

The `--payload-kind` flag on `prepare` chooses the shape for the authority-style
sources: `canonical-map` for single-token canonical spellings, `multiword-map`
for multiword names and aliases, `protected-spellings` for stylized single-token
forms.

## General-English membership

### `scowl`

- **Provides:** a `WordSet` of English headwords and inflected forms, parsed from
  the SCOWL preformat (phrase-like entries are flattened to token level, since
  the library's word-set hooks are token-oriented).
- **Use it when:** you want general English word membership — for example to
  drive `AllCapsPolicy::NormalizeKnownWords`, or `SmallWordPolicy::AlwaysLowercase`.
- **Note:** the recommended general-English source. Additive only; it never
  redefines MLA small-word behavior.

### `stopwords-iso`

- **Provides:** a `WordSet` from a plain JSON stop-word list — the simplest source.
- **Use it when:** experimenting, running diagnostics, or building heuristic
  small-word candidate lists.
- **Avoid it for:** authoritative MLA semantics. The built-in list stays the
  source of truth unless a caller explicitly opts into broader lowering.

### `wordfreq`

- **Provides:** `RankedWords` from the English `small_en.msgpack.gz` MessagePack
  dataset (the CLI decompresses the gzip, decodes the MessagePack, and expands
  frequency buckets into deterministic rank-ordered entries).
- **Use it when:** you want word-frequency rank rather than plain membership.
- **License:** includes CC-BY-SA-derived data; `fetch` and `prepare` both
  require `--acknowledge-cc-by-sa` and preserve the upstream `NOTICE.md`.

## Personal and entity names

### `wikidata`

- **Provides:** people, organizations, and creative works via a live SPARQL query.
- **Use it when:** you want broad, general-purpose authority coverage across many
  entity types.
- **Avoid it for:** a small, fully curated set — broad filters return noisy
  results, and large queries cost more time and memory than the GitHub sources.
- **Flags:** `--language <tag>`, `--limit <n>`, `--query <sparql>` to override
  the built-in query.

### `gnd`

- **Provides:** German National Library authority names (filtered to `Person`),
  including both heading-style and person-entity components — a single fetch can
  yield `Beethoven, Ludwig van` and `Ludwig van Beethoven`.
- **Use it when:** you want German and European personal names, or a narrower,
  higher-signal complement to Wikidata.
- **Avoid it for:** general English membership, or broad multilingual coverage
  across organizations and works (prefer Wikidata there).

### `musicbrainz`

- **Provides:** artist, band, and performer names — including stylized single-token
  forms like `P!nk` — from the public artist web service (`name`, `sort-name`,
  and aliases).
- **Use it when:** the deployment is music-heavy and brand-like artist casing
  matters.
- **Flags:** `--query <lucene>`, `--limit <n>`.

### `orcid`

- **Provides:** researcher names, aggregated from the public ORCID search and the
  matching person records.
- **Use it when:** you care about academic and publication-adjacent names.
- **License:** ORCID's public data is CC0, but its name, logo, and community
  guidance are *not* the data license; the CLI preserves that distinction in the
  source notices.
- **Flags:** `--query <orcid-search>`, `--limit <n>`.

## Licensing expectations

The CLI preserves upstream licensing context in manifests and generated plugins,
but it does not make license decisions for you. Before redistributing a generated
artifact, check the upstream terms yourself:

- whether attribution must travel with the artifact,
- whether share-alike obligations apply to your output,
- whether any notice text must remain visible downstream.

This matters most for `wordfreq`, whose packaged data is attribution- and
share-alike-sensitive.
