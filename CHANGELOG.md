# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Changed

- The small-word list now follows MLA's part-of-speech rule: subordinating conjunctions (`if`, `that`, `than`, `once`) are capitalized, and prepositions of any length (`about`, `among`, `between`, `despite`, `throughout`, `without`, ...) are lowercased.
- Colons are treated as subtitle boundaries on both sides: the last significant word before a colon is now capitalized (`What Dreams Are Made Of: A Study`), matching MLA's first-and-last-word rule for titles and subtitles.

### Fixed

- Contraction endings stay lowercase after apostrophes (`don't` → `Don't`, not `Don'T`); recapitalization now applies only to single-letter prefixes such as `O'Neill` and `D'Angelo`.
- Digit-led words no longer capitalize their first letter (`42nd street` → `42nd Street`, not `42Nd Street`).
- All-caps input is recased instead of being half-preserved as acronyms (`THE WIND IN THE WILLOWS` → `The Wind in the Willows`); a lone all-caps word is still treated as an acronym.
- The first word of a subtitle is capitalized through typographic quotes and other opening punctuation (`“` `‘` `«` `‹` `¿` `¡`) after a colon.
