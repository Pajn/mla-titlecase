# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Added

- Adverbial-particle detection (`capitalize_phrasal_particles`, on by default): small words are capitalized when no prepositional complement can follow ("Come Up and See Me") or when they follow a curated phrasal-verb head ("Turn Off the Lights", "Burning Down the House"), matching MLA's rule that adverbs are capitalized.

### Changed

- The small-word list now follows MLA's part-of-speech rule: subordinating conjunctions (`if`, `that`, `than`, `once`) are capitalized, and prepositions of any length (`about`, `among`, `between`, `despite`, `throughout`, `without`, ...) are lowercased.
- Colons are treated as subtitle boundaries on both sides: the last significant word before a colon is now capitalized (`What Dreams Are Made Of: A Study`), matching MLA's first-and-last-word rule for titles and subtitles.
- `a.m.` and `p.m.` join `e.g.` and `i.e.` as dotted abbreviations that stay lowercase instead of being uppercased as initialisms.
- Figure, en, and em dashes are no longer treated as compound hyphens; only `-`, U+2010, and U+2011 join hyphenated compounds.
- The name-particle heuristic now requires both neighboring words to look like name words, so phrases such as "riding the van to victory" no longer lowercase `van` under `NameParticlePolicy::Heuristic`.

### Fixed

- Protected spellings are never recased anymore: previously a protected word that matched the small-word list (or an `AlwaysLowercase` word-set entry) was force-lowercased, losing its protected form.

- Contraction endings stay lowercase after apostrophes (`don't` → `Don't`, not `Don'T`); recapitalization now applies only to single-letter prefixes such as `O'Neill` and `D'Angelo`.
- Digit-led words no longer capitalize their first letter (`42nd street` → `42nd Street`, not `42Nd Street`).
- All-caps input is recased instead of being half-preserved as acronyms (`THE WIND IN THE WILLOWS` → `The Wind in the Willows`); a lone all-caps word is still treated as an acronym.
- The first word of a subtitle is capitalized through typographic quotes and other opening punctuation (`“` `‘` `«` `‹` `¿` `¡`) after a colon.
