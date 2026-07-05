# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Adverbial-particle detection (`capitalize_phrasal_particles`, on by default): small words are capitalized when no prepositional complement can follow ("Come Up and See Me") or when they follow a curated phrasal-verb head ("Turn Off the Lights", "Burning Down the House"), matching MLA's rule that adverbs are capitalized.
- `AllCapsPolicy` for handling shouting input: `Normalize` (default, unchanged), `Preserve` (return intentional all-caps stylization verbatim), and `NormalizeKnownWords { unknown }` (title-case words a loaded dictionary word-set such as SCOWL recognizes; with SCOWL loaded and unknowns preserved, `SHERLOCK HISTORY` → `SHERLOCK History`). Unrecognized words are cast per `UnknownWordCasing`: `Preserve` (keep as likely acronyms), `TitleCase` (treat as ordinary words), or `PreserveShortAcronyms { max_acronym_len }` (preserve short words as acronyms but title-case longer ones as likely names).
- The built-in abbreviation list grew from 11 to ~50 curated initialisms (`IBM`, `FBI`, `NATO`, `URL`, `PDF`, ...), all guarded to never collide with small words or ordinary English words; `iOS` and `PhD` join the built-in protected spellings.
- The `n` written as `'n'` with flanking apostrophes is kept lowercase as a contraction of "and" (`Rock 'n' Roll`, `Fish 'n' Chips`).
- `titlecase_into(&mut out, input, options)` writes the result into a caller-owned buffer (cleared first), letting bulk callers reuse one `String` across many titles to avoid a per-call allocation.

### Changed

- The small-word list now follows MLA's part-of-speech rule: subordinating conjunctions (`if`, `that`, `than`, `once`) are capitalized, and prepositions of any length (`about`, `among`, `between`, `despite`, `throughout`, `without`, ...) are lowercased.
- Colons are treated as subtitle boundaries on both sides: the last significant word before a colon is now capitalized (`What Dreams Are Made Of: A Study`), matching MLA's first-and-last-word rule for titles and subtitles.
- `a.m.` and `p.m.` join `e.g.` and `i.e.` as dotted abbreviations that stay lowercase instead of being uppercased as initialisms.
- Figure, en, and em dashes are no longer treated as compound hyphens; only `-`, U+2010, and U+2011 join hyphenated compounds.
- The name-particle heuristic now requires both neighboring words to look like name words, so phrases such as "riding the van to victory" no longer lowercase `van` under `NameParticlePolicy::Heuristic`.

### Removed

- The redundant `lowercase_small_words` option. It duplicated `SmallWordPolicy::NeverLowercase`; callers that set `lowercase_small_words: false` should use `small_word_policy: SmallWordPolicy::NeverLowercase` instead. Name-particle lowering is now independent of the small-word policy.

### Fixed

- Protected spellings are never recased anymore: previously a protected word that matched the small-word list (or an `AlwaysLowercase` word-set entry) was force-lowercased, losing its protected form.

- Contraction endings stay lowercase after apostrophes (`don't` → `Don't`, not `Don'T`); recapitalization now applies only to single-letter prefixes such as `O'Neill` and `D'Angelo`.
- Digit-led words no longer capitalize their first letter (`42nd street` → `42nd Street`, not `42Nd Street`).
- All-caps input is recased instead of being half-preserved as acronyms (`THE WIND IN THE WILLOWS` → `The Wind in the Willows`); a lone all-caps word is still treated as an acronym.
- The first element of a hyphenated compound is now capitalized even when it is a small word (`a by-product of war` → `A By-Product of War`); interior small words still lowercase (`State-of-the-Art`).
- English small words and name particles now lowercase the English way under non-English locales, so `IN` no longer renders as Turkish dotless `ın`; `lookup_key` also maps the dotted capital `İ` to ASCII `i` so Turkish-cased words match lexicon entries.
- The first word of a subtitle is capitalized through typographic quotes and other opening punctuation (`“` `‘` `«` `‹` `¿` `¡`) after a colon.
