# Lexicon sources

External sources are optional inputs for the CLI. They do not replace the built-in MLA rule engine.

| Source | Best use | Notes |
| --- | --- | --- |
| `scowl` | General English word membership | Recommended default fetchable source |
| `stopwords-iso` | Heuristic stopword candidates | Convenient, but not authoritative for MLA semantics |
| `wordfreq` | Ranked/commonness data | Advanced option with explicit acknowledgement requirements |

## SCOWL

SCOWL is the strongest default general-English source for this project's CLI-supported downloads. It is a good fit for broad membership checks and for expanding external word-set coverage.

Pros:

- strong English backbone
- straightforward normalization into a `WordSet`
- good default recommendation in CLI output

Tradeoffs:

- upstream licensing/notices must still be preserved
- SCOWL membership should not silently redefine MLA small-word semantics

## stopwords-iso

`stopwords-iso` is easy to consume because the English list is plain JSON. It is useful for heuristic candidate lists and quick experiments.

Pros:

- simple JSON input
- easy local fixture testing
- practical for demos and debugging

Tradeoffs:

- stopword lists are not the same thing as MLA small words
- the library only uses these entries as small-word candidates when callers explicitly opt into `SmallWordPolicy::AlwaysLowercase`

## wordfreq

`wordfreq` is useful when ranked/commonness data matters.

Pros:

- rank-oriented payloads
- fits future commonness heuristics and tooling

Tradeoffs:

- preserved notice handling is mandatory
- the CLI requires `--acknowledge-cc-by-sa`
- generated outputs should keep attribution/share-alike implications visible
