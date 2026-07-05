# MLA rules implemented

The library implements a pragmatic MLA-focused title-casing engine.

## Small words

The built-in small-word list follows MLA's part-of-speech rule: it lowercases articles (`a`, `an`, `the`), coordinating conjunctions (`and`, `but`, `for`, `nor`, `or`, `so`, `yet`), and prepositions of any length (`of`, `with`, `among`, `between`, `throughout`, ...).

Subordinating conjunctions such as `if`, `that`, `than`, `once`, and `because` are capitalized, as MLA requires. Words that can be either a preposition or a subordinating conjunction (`after`, `before`, `since`, `till`, `until`) are treated as prepositions and lowercased, the more common reading in titles. Similarly, `up`, `down`, `off`, `over`, and `past` are treated as prepositions even though MLA capitalizes them when they act as adverbs (as in phrasal verbs); a static list cannot disambiguate part of speech.

By default these words are lowercased only when they appear internally.

## First and last significant words

The first and last significant words are always capitalized, even if they are normally treated as small words.

## Colon behavior

When `capitalize_after_colon` is enabled (the default), a colon is treated as a subtitle boundary: the first significant word after it and the last significant word before it are capitalized, matching MLA's rule that the first and last words of both the title and the subtitle are capitalized.

## Hyphen behavior

`HyphenStyle::MlaLike` applies the same MLA logic to each hyphen-separated segment.

`HyphenStyle::CapitalizeBoth` forces capitalization of each segment in a hyphenated compound.

Only true hyphens (`-`, U+2010, U+2011) join compounds. Figure, en, and em dashes separate clauses and are treated as ordinary punctuation.

## Acronyms and dotted abbreviations

- all-caps acronyms such as `NASA` are preserved
- dotted initialisms such as `u.s.a.` normalize to `U.S.A.`
- lowercase dotted abbreviations like `e.g.`, `i.e.`, `a.m.`, and `p.m.` remain lowercase

## Protected spellings

The engine preserves:

- user-supplied protected words
- built-in spellings like `GitHub`, `iPhone`, and `macOS`
- external protected-spelling plugins

## Multiword external mappings

When callers load `MultiwordMap` plugins, the engine attempts longest-match phrase lookups across whitespace-separated word runs before it falls back to single-token external maps.

That lets optional plugins preserve forms such as `New York City` without changing the built-in MLA semantics for callers who do not load any external data.

## Name particles

When `NameParticlePolicy::Heuristic` is enabled, common particles such as `van`, `de`, and `von` stay lowercase inside likely personal-name runs. A particle only qualifies when the words on both sides look like name words (in particular, neither neighbor is a small word), so "Riding the Van to Victory" keeps its capital while "Ludwig van Beethoven" stays lowered.

Locale profiles widen that heuristic carefully for supported languages such as Dutch, French, German, Italian, Spanish, and Turkish while keeping the default English MLA path unchanged.

## Known boundaries

This crate is intentionally MLA-centric. It does not try to become a full named-entity recognizer or a multi-style-guide formatter.
