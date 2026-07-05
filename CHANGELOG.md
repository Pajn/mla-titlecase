# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Fixed

- Contraction endings stay lowercase after apostrophes (`don't` → `Don't`, not `Don'T`); recapitalization now applies only to single-letter prefixes such as `O'Neill` and `D'Angelo`.
- Digit-led words no longer capitalize their first letter (`42nd street` → `42nd Street`, not `42Nd Street`).
- All-caps input is recased instead of being half-preserved as acronyms (`THE WIND IN THE WILLOWS` → `The Wind in the Willows`); a lone all-caps word is still treated as an acronym.
- The first word of a subtitle is capitalized through typographic quotes and other opening punctuation (`“` `‘` `«` `‹` `¿` `¡`) after a colon.
