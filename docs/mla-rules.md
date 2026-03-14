# MLA rules implemented

The library implements a pragmatic MLA-focused title-casing engine.

## Small words

The built-in small-word list includes common articles, conjunctions, and prepositions such as `a`, `an`, `and`, `as`, `at`, `by`, `for`, `in`, `of`, `on`, `the`, `to`, and `with`.

By default these words are lowercased only when they appear internally.

## First and last significant words

The first and last significant words are always capitalized, even if they are normally treated as small words.

## Colon behavior

When `capitalize_after_colon` is enabled (the default), the first significant word after a colon is capitalized.

## Hyphen behavior

`HyphenStyle::MlaLike` applies the same MLA logic to each hyphen-separated segment.

`HyphenStyle::CapitalizeBoth` forces capitalization of each segment in a hyphenated compound.

## Acronyms and dotted abbreviations

- all-caps acronyms such as `NASA` are preserved
- dotted initialisms such as `u.s.a.` normalize to `U.S.A.`
- lowercase dotted abbreviations like `e.g.` and `i.e.` remain lowercase

## Protected spellings

The engine preserves:

- user-supplied protected words
- built-in spellings like `GitHub`, `iPhone`, and `macOS`
- external protected-spelling plugins

## Multiword external mappings

When callers load `MultiwordMap` plugins, the engine attempts longest-match phrase lookups across whitespace-separated word runs before it falls back to single-token external maps.

That lets optional plugins preserve forms such as `New York City` without changing the built-in MLA semantics for callers who do not load any external data.

## Name particles

When `NameParticlePolicy::Heuristic` is enabled, common particles such as `van`, `de`, and `von` stay lowercase inside likely personal-name runs.

Locale profiles widen that heuristic carefully for supported languages such as Dutch, French, German, Italian, Spanish, and Turkish while keeping the default English MLA path unchanged.

## Known boundaries

This crate is intentionally MLA-centric. It does not try to become a full named-entity recognizer or a multi-style-guide formatter.
