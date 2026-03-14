# Plugin format

The workspace uses a shared plugin schema defined in the library crate.

## JSON format

JSON plugins serialize the full `LexiconPlugin` value.

Top-level fields:

- `metadata`
- `payload`

`metadata` includes schema/version information, source identifiers, upstream URL, prepared timestamp, checksum, license summary, and optional notice text.

`payload` is one of:

- `word-set`
- `canonical-map`
- `ranked-words`
- `protected-spellings`

## FST format

FST plugins use a single binary file with:

1. a fixed header (`MLATFST1`)
2. serialized metadata length
3. payload kind tag
4. payload byte length
5. metadata JSON bytes
6. payload bytes

Payload encoding strategy:

- `WordSet`: `fst::Set`
- `RankedWords`: `fst::Map<word, rank>`
- `CanonicalMap` / `ProtectedSpellings`: `fst::Map<word, value_offset>` plus a string table

## Compatibility and validation

- schema version mismatches fail explicitly
- duplicate or empty entries fail validation
- both JSON and FST loaders validate before registration
- `inspect-plugin` reports format, payload kind, entry counts, and key metadata
