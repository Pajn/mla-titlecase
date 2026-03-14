# Lexicon sources

External sources are optional CLI inputs. They do not replace the built-in MLA rule engine.

The repository does **not** commit fetched raw data or prepared outputs. End users are expected to review the upstream licenses and notices first, then run the CLI locally to fetch and prepare the sources they are allowed to use.

## Supported default sources

| Source | Default upstream artifact | Output kind | License / notice handling |
| --- | --- | --- | --- |
| `scowl` | `en-wl/wordlist:data/scowl-pre.txt` on `v2` | `WordSet` | Preserve the upstream `Copyright` notice |
| `stopwords-iso` | `stopwords-iso/stopwords-en:stopwords-en.json` on `master` | `WordSet` | MIT license text is preserved in fetch metadata |
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

## Licensing expectations

The CLI preserves upstream licensing context in manifests and generated plugins, but it does not make license decisions for the user.

Before redistributing generated artifacts, check the upstream terms yourself:

- whether attribution must travel with the artifact
- whether share-alike obligations apply to your output
- whether any notice text must remain visible downstream

That is especially important for `wordfreq`, whose packaged data includes attribution-sensitive and share-alike-sensitive upstream material.
