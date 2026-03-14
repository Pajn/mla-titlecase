# Lexicon sources

External sources are optional CLI inputs. They do not replace the built-in MLA rule engine.

The repository does **not** commit fetched raw data or prepared outputs. End users are expected to review the upstream licenses and notices first, then run the CLI locally to fetch and prepare the sources they are allowed to use.

## Supported default sources

| Source | Default upstream artifact | Output kind | License / notice handling |
| --- | --- | --- | --- |
| `gnd` | live query to `lobid.org/gnd/search` | `CanonicalMap`, `MultiwordMap`, or `ProtectedSpellings` | GND authority data served by lobid is CC0; the exact request URL is preserved in the fetch manifest |
| `musicbrainz` | live query to `musicbrainz.org/ws/2/artist/` | `CanonicalMap`, `MultiwordMap`, or `ProtectedSpellings` | MusicBrainz core database data is CC0; the exact request URL is preserved in the fetch manifest |
| `orcid` | live aggregation of `pub.orcid.org` search plus person records | `CanonicalMap`, `MultiwordMap`, or `ProtectedSpellings` | ORCID public data is CC0; manifests also preserve a note about separate ORCID trademark/community guidance |
| `scowl` | `en-wl/wordlist:data/scowl-pre.txt` on `v2` | `WordSet` | Preserve the upstream `Copyright` notice |
| `stopwords-iso` | `stopwords-iso/stopwords-en:stopwords-en.json` on `master` | `WordSet` | MIT license text is preserved in fetch metadata |
| `wikidata` | live query to `query.wikidata.org/sparql` | `CanonicalMap`, `MultiwordMap`, or `ProtectedSpellings` | Structured data is CC0; the exact query URL is preserved in the fetch manifest |
| `wordfreq` | `rspeer/wordfreq:wordfreq/data/small_en.msgpack.gz` on `master` | `RankedWords` | Requires `--acknowledge-cc-by-sa` and preserves `NOTICE.md` |

`lexicon fetch` resolves each default artifact through the GitHub Contents API first. That gives the CLI a commit-pinned raw download URL plus the blob SHA, which is recorded as `source_version` in the fetch manifest and then carried into prepared/plugin metadata.

## Fetch and prepare flow

The intended out-of-box flow is:

```bash
cargo run -p mla-titlecase-cli -- lexicon fetch scowl --output /tmp/scowl.txt
cargo run -p mla-titlecase-cli -- lexicon prepare scowl --input /tmp/scowl.txt --output /tmp/scowl.json
cargo run -p mla-titlecase-cli -- lexicon build-plugin /tmp/scowl.json --format fst --output /tmp/scowl.mlatl
```

`prepare` automatically reads the adjacent fetch manifest at `<input>.manifest.json` when it exists, so the prepared file inherits:

- the commit-pinned upstream URL
- the resolved source version / blob SHA
- the preserved license summary
- the preserved notice text

The prepared JSON also includes a normalization report with input counts, deduplicated output counts, duplicates removed, and ignored records.

## Source-specific notes

## `scowl`

SCOWL is the recommended general-English source for word membership. The CLI currently consumes the upstream `scowl-pre.txt` artifact from the SCOWL v2 repository and normalizes it into a single `WordSet`.

Important details:

- the upstream file is richer than a plain word list
- the parser extracts headwords and inflected forms from the SCOWL preformat
- phrase-like entries are flattened to token-level words because the library's external word-set hooks are token-oriented

This makes SCOWL a strong additive membership source, but it still does not redefine MLA small-word behavior.

## `stopwords-iso`

`stopwords-iso` is the simplest source. The English artifact is plain JSON and normalizes directly into a `WordSet`.

It is useful for:

- experiments
- diagnostics
- heuristic small-word candidate lists

It is **not** authoritative for MLA semantics. The built-in MLA list remains the source of truth unless a caller explicitly opts into broader lowering behavior.

## `wordfreq`

`wordfreq` provides ranked word data rather than a plain membership set. The default artifact is the English `small_en.msgpack.gz` cBpack dataset.

The CLI:

- decompresses the gzip payload
- decodes the MessagePack cBpack structure
- expands frequency buckets into deterministic rank-ordered entries
- stores them as a `RankedWords` payload

Because the upstream package includes CC-BY-SA-derived data and an explicit notice, the CLI requires `--acknowledge-cc-by-sa` for both `fetch` and `prepare`.

## `wikidata`

`wikidata` is the first optional authority-style source. Instead of downloading a fixed artifact from a Git repository, the CLI issues a live SPARQL query against `query.wikidata.org` and records the full resolved query URL in the fetch manifest.

The default query targets a manageable slice of entity classes:

- humans
- organizations
- creative works

and supports:

- `--language <tag>` to choose the label/alias language
- `--limit <n>` to cap the live query size
- `--query <sparql>` to override the built-in query entirely

The raw SPARQL JSON can then be prepared into one of three plugin payloads:

- `CanonicalMap` for single-token names with authoritative casing
- `MultiwordMap` for multiword entities and aliases
- `ProtectedSpellings` for single-token protected forms

Example flow:

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

Wikidata is useful when you want broad optional coverage for people, organizations, works, and aliases. It is less attractive when you need a small, deterministic, fully curated source: live query results can be noisy if your filters are too broad, and larger queries will cost more time and memory than the GitHub-backed sources.

## `gnd`

`gnd` uses the public lobid search API for the German National Library authority file. The default fetch is intentionally narrower than Wikidata: it queries `lobid.org/gnd/search` with `q=*`, filters to `type:Person`, and preserves the full request URL in the fetch manifest.

That makes GND a good fit when you want:

- German and European authority-style names
- person-name normalization in German-heavy titles
- a narrower, higher-signal complement to Wikidata

The CLI currently extracts both heading-style names and person-entity name components when they are available, so a single fetch can preserve forms such as:

- `Beethoven, Ludwig van`
- `Ludwig van Beethoven`

Like Wikidata, `prepare` supports:

- `--payload-kind canonical-map`
- `--payload-kind multiword-map`
- `--payload-kind protected-spellings`

GND is better than Wikidata when you want a more focused authority source for European personal names. It is overkill when you just need general English word membership, and it is less attractive than Wikidata when you want broad multilingual coverage across organizations, works, and places.

## `musicbrainz`

`musicbrainz` is deliberately narrow and deliberately explicit: the current CLI integration uses the public `musicbrainz.org/ws/2/artist/` JSON web service for artist records only. It does not claim to support every MusicBrainz dump or every non-core dataset.

That makes it a strong fit for:

- artist names
- bands and performers
- stylized single-token names such as `P!nk`
- music-heavy deployments where brand-like casing matters

The default fetch uses a small live query against the artist endpoint and accepts:

- `--query <lucene-query>` to target a specific artist subset
- `--limit <n>` to control response size

During `prepare`, the CLI preserves artist `name`, `sort-name`, and alias surfaces, then lets you choose:

- `--payload-kind protected-spellings` for stylized single-token names
- `--payload-kind canonical-map` for single-token canonical surfaces
- `--payload-kind multiword-map` for multiword artist and alias names

Use MusicBrainz when the deployment is music-heavy and stylized artist names matter. Prefer Wikidata or GND when you need a broader general-purpose authority source.

## `orcid`

`orcid` is the narrowest of the authority-style sources in this CLI. The implementation is intentionally explicit about scope:

- it queries the public ORCID search API for a small set of record identifiers
- it fetches the corresponding public person records
- it writes an aggregated raw JSON artifact so `prepare` can run deterministically afterward

This source is useful when you care about:

- researcher names
- academic and scholarly titles
- publication-adjacent person-name normalization

The current flow supports:

- `--query <orcid-search-query>` for the public search
- `--limit <n>` for the number of person records fetched
- `--payload-kind canonical-map`, `multiword-map`, or `protected-spellings` during `prepare`

The important licensing nuance is that ORCID's public data is CC0, but ORCID's name, logo, and community guidance are not the same thing as the data license. The CLI preserves that distinction in source notices so downstream users do not mistake trademark/display guidance for a license restriction on the public data itself.

## Licensing expectations

The CLI preserves upstream licensing context in manifests and generated plugins, but it does not make license decisions for the user.

Before redistributing generated artifacts, check the upstream terms yourself:

- whether attribution must travel with the artifact
- whether share-alike obligations apply to your output
- whether any notice text must remain visible downstream

That is especially important for `wordfreq`, whose packaged data includes attribution-sensitive and share-alike-sensitive upstream material.
