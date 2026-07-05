# MLA rules implemented

The library implements a pragmatic MLA-focused title-casing engine.

## Small words

The built-in small-word list follows MLA's part-of-speech rule: it lowercases articles (`a`, `an`, `the`), coordinating conjunctions (`and`, `but`, `for`, `nor`, `or`, `so`, `yet`), and prepositions of any length (`of`, `with`, `among`, `between`, `throughout`, ...).

Subordinating conjunctions such as `if`, `that`, `than`, `once`, and `because` are capitalized, as MLA requires. Words that can be either a preposition or a subordinating conjunction (`after`, `before`, `since`, `till`, `until`) are treated as prepositions and lowercased, the more common reading in titles.

By default these words are lowercased only when they appear internally.

## Adverbial particles

MLA capitalizes adverbs, including the particle of a phrasal verb. When `capitalize_phrasal_particles` is enabled (the default), a small word escapes lowering in two situations:

- nothing that could serve as a preposition's complement follows it — the next token is punctuation, a dash, or a coordinating conjunction ("Give Up, Move On", "Come Up and See Me")
- it follows a curated phrasal-verb head, directly or across one object pronoun ("Turn Off the Lights", "Burning Down the House", "Wake Me Up")

Prepositional uses stay lowercase ("Walking down the Street", "Livin' on a Prayer"). Genuinely ambiguous phrases that the heuristic misreads can be pinned with protected words.

## First and last significant words

The first and last significant words are always capitalized, even if they are normally treated as small words.

## Colon behavior

When `capitalize_after_colon` is enabled (the default), a colon is treated as a subtitle boundary: the first significant word after it and the last significant word before it are capitalized, matching MLA's rule that the first and last words of both the title and the subtitle are capitalized.

## Hyphen behavior

`HyphenStyle::MlaLike` applies the same MLA logic to each hyphen-separated segment.

`HyphenStyle::CapitalizeBoth` forces capitalization of each segment in a hyphenated compound.

Only true hyphens (`-`, U+2010, U+2011) join compounds. Figure, en, and em dashes separate clauses and are treated as ordinary punctuation.

## Acronyms and dotted abbreviations

- all-caps acronyms such as `NASA` are preserved when the rest of the title contains lowercase letters, or when the acronym is the only word
- fully all-caps multi-word input is treated as shouting and recased to title case; known abbreviations (`MLA`, `NASA`, `USA`, ...) are restored from the built-in abbreviation list, and protected spellings are kept
- dotted initialisms such as `u.s.a.` normalize to `U.S.A.`
- lowercase dotted abbreviations like `e.g.`, `i.e.`, `a.m.`, and `p.m.` remain lowercase

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

## Known boundaries

This crate is intentionally MLA-centric. It does not try to become a full named-entity recognizer or a multi-style-guide formatter.
