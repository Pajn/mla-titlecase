# MLA rules implemented

The library implements a pragmatic MLA-focused title-casing engine.

## Small words

The built-in small-word list follows MLA's part-of-speech rule: it lowercases articles (`a`, `an`, `the`), coordinating conjunctions (`and`, `but`, `for`, `nor`, `or`, `so`, `yet`), and prepositions of any length (`of`, `with`, `among`, `between`, `throughout`, ...).

Subordinating conjunctions such as `if`, `that`, `than`, `once`, and `because` are capitalized, as MLA requires. Words that can be either a preposition or a subordinating conjunction (`after`, `before`, `since`, `till`, `until`) are treated as prepositions and lowercased, the more common reading in titles.

By default these words are lowercased only when they appear internally. Set `small_word_policy` to `SmallWordPolicy::NeverLowercase` to disable small-word lowering entirely.

The `n` written as `'n'` with flanking apostrophes is treated as a contraction of "and" and stays lowercase (`Rock 'n' Roll`, `Fish 'n' Chips`), independent of the small-word policy.

## Adverbial particles

MLA capitalizes adverbs, including the particle of a phrasal verb. When `capitalize_phrasal_particles` is enabled (the default), a small word escapes lowering in two situations:

- nothing that could serve as a preposition's complement follows it — the next token is punctuation, a dash, or a coordinating conjunction ("Give Up, Move On", "Come Up and See Me")
- it follows a curated phrasal-verb head, directly or across one object pronoun ("Turn Off the Lights", "Burning Down the House", "Wake Me Up")

Prepositional uses stay lowercase ("Walking down the Street", "Livin' on a Prayer"). Genuinely ambiguous phrases that the heuristic misreads can be pinned with protected words.

## First and last significant words

The first and last significant words are always capitalized, even if they are normally treated as small words.

## Subtitle boundaries

When `capitalize_after_subtitle_boundary` is enabled (the default), a colon, question mark, or exclamation point is treated as a subtitle boundary: the first significant word after it and the last significant word before it are capitalized, matching MLA's rule that the first and last words of both the title and the subtitle are capitalized (`What Now? A Memoir`, `Help! An Inspector Calls`, `Preface: The Return of Sherlock Holmes`).

Periods are not boundaries — they are far more often part of an abbreviation — and figure, en, and em dashes separate clauses rather than subtitles (`Well-Known—a Memoir of Sorts`).

## Hyphen behavior

`HyphenStyle::MlaLike` applies the same MLA logic to each hyphen-separated segment, with one exception: the first element of a hyphenated compound is always capitalized, even when it is a small word (`A By-Product of War`, `The In-Between`). Only interior elements follow the small-word rule (`State-of-the-Art`).

`HyphenStyle::CapitalizeBoth` forces capitalization of each segment in a hyphenated compound.

Only true hyphens (`-`, U+2010, U+2011) join compounds. Figure, en, and em dashes separate clauses and are treated as ordinary punctuation.

## Acronyms and dotted abbreviations

- all-caps acronyms such as `NASA` are preserved when the rest of the title contains lowercase letters, or when the acronym is the only word
- fully all-caps multi-word input is treated as shouting and recased to title case; known abbreviations (`MLA`, `NASA`, `USA`, ...) are restored from the built-in abbreviation list, and protected spellings are kept
- dotted initialisms such as `u.s.a.` normalize to `U.S.A.`
- lowercase dotted abbreviations like `e.g.`, `i.e.`, `a.m.`, and `p.m.` remain lowercase

The built-in abbreviation list applies to all input, not just all-caps titles, so `the ibm story` also becomes `The IBM Story`. For that reason every entry is an initialism that is never an ordinary English word (`us`, `led`, `id` are deliberately excluded so normal titles are never corrupted).

## All-caps policy

`AllCapsPolicy` controls what happens when the whole input is detected as shouting:

- `Normalize` (default) recases the input to title case as described above.
- `Preserve` treats the all-caps input as intentional stylization and returns it verbatim, skipping both small-word lowering and recasing. This suits music and brand metadata where forms like `MONTERO` or `STAY WITH ME` are deliberate. Mixed-case input is not shouting, so it still follows the normal rules.
- `NormalizeKnownWords { unknown }` always title-cases words a loaded dictionary word-set recognizes, and handles unrecognized words according to `unknown`:
  - `UnknownWordCasing::Preserve` keeps unknown words as written, assuming they are acronyms. With a comprehensive word set such as SCOWL loaded, a recognized word recases while an unknown name stays all-caps (`SHERLOCK HISTORY` → `SHERLOCK History`); unknown proper names like `SHERLOCK` stay all-caps unless a canonical map or protected spelling restores them.
  - `UnknownWordCasing::TitleCase` title-cases unknown words instead, assuming they are ordinary words or names (`SHERLOCK` → `Sherlock`). With this setting the dictionary gate has no visible effect on output, so the result matches `Normalize`.
  - `UnknownWordCasing::PreserveShortAcronyms { max_acronym_len }` splits the difference on word length: unknown words with at most `max_acronym_len` letters are preserved as likely acronyms, while longer ones are title-cased as likely names (`THE NSA AND SHERLOCK REPORT` → `The NSA and Sherlock Report` at a threshold of 4). The heuristic is necessarily imperfect — short names (`LEE`) are preserved and long acronyms (`SCOTUS`) are title-cased — so pin the exceptions with a canonical map or protected spellings.

  With no word set loaded, `NormalizeKnownWords` behaves like `Normalize` regardless of `unknown`.

### Dictionary-gated recipe

`NormalizeKnownWords` composes with the external lexicon pipeline:

1. Load SCOWL as a `WordSet` so ordinary English words are recognized and recased.
2. Unknown all-caps words (proper names absent from SCOWL) stay all-caps. Load a `CanonicalMap` or `ProtectedSpellings` plugin from Wikidata, GND, or MusicBrainz to restore their casing (`SHERLOCK` → `Sherlock`); those maps are consulted before the all-caps styling.
3. Pin any remaining brand spellings with protected words.

This keeps the built-in MLA engine authoritative while treating the dictionary and authority data as additive, opt-in signals. Frequency data (`wordfreq` / `RankedWords`) is *not* suitable for this gate: it is case-folded and lists `nasa`, `ibm`, and `fbi` with ordinary frequencies, so membership there does not distinguish a word from an acronym.

## Protected spellings

The engine preserves:

- user-supplied protected words
- built-in spellings like `GitHub`, `iPhone`, and `macOS`
- external protected-spelling plugins

Protected spellings always win: a protected word is never recased, even when it matches the small-word list or an `AlwaysLowercase` word-set entry.

## Multiword external mappings

When callers load `MultiwordMap` plugins, the engine attempts longest-match phrase lookups across whitespace-separated word runs before it falls back to single-token external maps.

That lets optional plugins preserve forms such as `New York City` without changing the built-in MLA semantics for callers who do not load any external data.

## Name particles

When `NameParticlePolicy::Heuristic` is enabled, common particles such as `van`, `de`, and `von` stay lowercase inside likely personal-name runs. A particle only qualifies when the words on both sides look like name words (in particular, neither neighbor is a small word), so "Riding the Van to Victory" keeps its capital while "Ludwig van Beethoven" stays lowered.

Locale profiles widen that heuristic carefully for supported languages such as Dutch, French, German, Italian, Spanish, and Turkish while keeping the default English MLA path unchanged.

## Rich analysis output

`titlecase_analyze` (and `titlecase_analyze_with_options`) return a `TitleCaseAnalysis` alongside the cased string: an overall `Confidence` and one `CasingSpan` per casing decision. That is usually one span per word, but a multiword lexicon match collapses to a single span covering the whole phrase, and `AllCapsPolicy::Preserve` on shouting input records no spans (the input is returned verbatim). Each span carries byte ranges into the input and output, the deciding `CasingRule`, a per-span `Confidence`, and a `changed` flag.

`Confidence` has three tiers:

- `Solid` — a structural MLA rule (first/last word, subtitle boundary, hyphenated-compound start) or an explicit match (protected spelling, abbreviation, dotted initialism, canonical/multiword lexicon, `'n'` contraction, small word, preserved mixed case).
- `Unverified` — an ordinary word capitalized with no lexicon to confirm it is not a name or brand. Correct under MLA, but the open-world "a lexicon might disagree" case applies to nearly every plain word, so it is a separate, lower-priority tier rather than an ambiguity flag.
- `Heuristic` — a decision that could genuinely be wrong: adverbial-particle detection, the name-particle heuristic, all-caps word-versus-acronym classification, and the dual-role prepositions (`after`, `before`, `since`, `till`, `until`).

The title's overall `confidence` is the most concerning tier across every word, so a caller can triage: `Heuristic` titles are the ones most worth a human look. Every word token produces a span, in order; filter on `CasingSpan::changed` to get just the edits. Recording unchanged words too means a heuristic that kept a word as-is — for example a name particle already lowercase in the input — still surfaces and still affects the overall confidence.

The plain `titlecase_*` functions do not compute any of this — the attribution work is compiled out of that path.

## Known boundaries

This crate is intentionally MLA-centric. It does not try to become a full named-entity recognizer or a multi-style-guide formatter.
