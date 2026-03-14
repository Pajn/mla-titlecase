# Performance notes

The current workspace includes Criterion benches for:

- repeated title-casing calls
- external lexicon lookups
- JSON versus FST plugin load paths

## JSON vs FST

JSON is easier to inspect and debug, but it pays normal parsing overhead.

FST plugins are designed for compact, deterministic storage and faster machine-oriented loading, especially for word sets and ranked maps.

The library now has a direct-query mmap-backed FST runtime path. That path avoids rebuilding the plugin into heap collections when the caller loads an FST file through `ExternalLexicons::register_mmap_fst_plugin`.

## Recommended usage

- small curated plugins: JSON is usually fine
- larger machine-generated plugins: prefer FST
- development and debugging: start with JSON, switch to FST once the payload stabilizes
- latency-sensitive runtime loading from disk: prefer the mmap-backed FST path

## Caveats

These benches are intended to compare the project's own implementations and regressions, not to serve as universal benchmarks for every workload.
