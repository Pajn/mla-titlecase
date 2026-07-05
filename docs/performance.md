# Performance notes

The current workspace includes Criterion benches for:

- repeated title-casing calls
- external lexicon lookups
- JSON versus FST plugin load paths

## Title-casing hot path

The engine is built for bulk processing (millions of titles), so the per-call
path avoids allocations where it can:

- `normalized_key` returns `Cow<str>` and borrows the input unchanged when it is
  already a clean lowercase-ASCII token (the common case), so classifying a word
  usually allocates nothing.
- Casing writes directly into the output buffer (`push_styled`, `push_lowercased`,
  `push_capitalized`) instead of allocating a temporary `String` per word.
  Capitalizing a plain ASCII word takes a single pass with no temporary at all.
- The curated small-word, adverbial-particle, and abbreviation lists are kept
  sorted (guarded by tests) and queried with `binary_search`.

Together these cut per-title time by roughly 35-50% over the naive path in the
`titlecase` bench. Run `cargo bench -p mla-titlecase --bench titlecase` to
compare against the committed baseline.

## Reusing an output buffer

`titlecase_into(&mut out, input, options)` writes into a caller-owned `String`
(clearing it first) instead of returning a fresh one. Reusing a single buffer
across a batch avoids one allocation per title; the `titlecase_batch_reused_buffer`
bench runs a few percent faster than `titlecase_batch_allocating`.

The gain is bounded because the tokenizer still allocates a `Vec` of tokens per
call: those tokens borrow the input string, so the buffer cannot be pooled
across calls without `unsafe`, which the workspace forbids. For most bulk
workloads the saved output allocation plus caller control over the buffer is the
worthwhile part.

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
