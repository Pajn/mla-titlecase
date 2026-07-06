# mla-titlecase

MLA-style title casing for Rust: a deterministic rule engine with optional,
opt-in lexicon support.

The built-in MLA rules are authoritative and need no downloaded data. External
lexicons (names, brands, stop words) only ever *add* to the defaults — they are
loaded explicitly by the caller and never change the baseline behavior.

```rust
use mla_titlecase::titlecase_mla;

assert_eq!(titlecase_mla("the wind in the willows"), "The Wind in the Willows");
```

## What it does

- Capitalizes the first and last words of the title and of each subtitle (colon
  boundary), and any word that is not a small word.
- Lowercases MLA small words (articles, coordinating conjunctions, prepositions
  of any length) when they appear internally.
- Handles the awkward cases: subordinating conjunctions stay capitalized,
  phrasal-verb particles are capitalized (`Turn Off the Lights`), acronyms and
  dotted initialisms are preserved (`NASA`, `U.S.A.`), hyphenated compounds are
  cased per segment, and contractions keep their spelling (`O'Neill`, `Don't`).

See [`docs/mla-rules.md`](../../docs/mla-rules.md) for the full rule set.

## The four entry points

| Function | Use when |
| --- | --- |
| `titlecase_mla(input)` | One title, default MLA options. |
| `titlecase_with_options(input, &options)` | You need to change behavior (below). |
| `titlecase_into(&mut buf, input, &options)` | Bulk processing — reuse one `String` buffer across many titles to avoid a per-call allocation. |
| `titlecase_analyze(input)` | You want to know *why* each word was cased and how much to trust it. |

### Options

`TitleCaseOptions` controls behavior; every field has a sensible MLA default, so
override only what you need:

```rust
use mla_titlecase::{titlecase_with_options, TitleCaseOptions};

let options = TitleCaseOptions::with_protected_words(&["PostgreSQL"]);
assert_eq!(
    titlecase_with_options("learning postgresql with github", &options),
    "Learning PostgreSQL with GitHub",
);
```

| Field | Default | Controls |
| --- | --- | --- |
| `preserve_existing_caps` | `true` | Keep mixed-case input like `iPhone`. |
| `capitalize_after_colon` | `true` | Treat colons as subtitle boundaries. |
| `capitalize_phrasal_particles` | `true` | Capitalize adverbial particles (`Give Up`). |
| `all_caps_policy` | `Normalize` | How to handle all-caps ("shouting") input. |
| `small_word_policy` | `Mla` | Which words to lowercase internally. |
| `hyphen_style` | `MlaLike` | Casing of hyphenated compounds. |
| `name_particle_policy` | `Disabled` | Lowercase `van`/`de`/`von` in personal names. |
| `locale` | `English` | Locale-aware casing (see below). |
| `protected_words` | `&[]` | Spellings that are never recased. |
| `external_lexicons` | `None` | Additive lexicon data (see below). |

`AllCapsPolicy` is worth calling out for catalog data: `Normalize` recases
`THE WIND` to `The Wind`; `Preserve` leaves intentional stylization like
`MONTERO` untouched; `NormalizeKnownWords` recases dictionary words but keeps
likely acronyms, given a word-set lexicon such as SCOWL.

### Locales

The default profile is English MLA. Opt-in profiles add locale-aware casing for
Dutch (`IJ` digraph), French, German, Italian, Spanish, and Turkish (dotted/
dotless `i`), and turn on sensible name-particle handling:

```rust
use mla_titlecase::{titlecase_with_options, LocaleProfile, TitleCaseOptions};

let options = TitleCaseOptions::with_locale(LocaleProfile::Dutch);
assert_eq!(
    titlecase_with_options("ijsselmeer and jan van der heijden", &options),
    "IJsselmeer and Jan van der Heijden",
);
```

### Rich analysis

`titlecase_analyze` returns the cased string plus, for every word, the rule that
decided its casing and how much to trust it — useful for flagging titles that
relied on a heuristic for human review:

```rust
use mla_titlecase::{titlecase_analyze, CasingRule, Confidence};

let analysis = titlecase_analyze("turn off the lights");
assert_eq!(analysis.output, "Turn Off the Lights");
// "off" is a phrasal-verb particle — a heuristic, so the title is flagged.
assert_eq!(analysis.confidence, Confidence::Heuristic);
assert!(analysis.spans.iter().any(|span| span.rule == CasingRule::AdverbialParticle));
```

`Confidence` has three tiers: `Solid` (structural rules and explicit matches),
`Unverified` (an ordinary word capitalized with no lexicon to confirm it isn't a
name), and `Heuristic` (a guess that could be wrong).

## External lexicons and plugins

`ExternalLexicons` holds additive data — canonical spellings, protected
spellings, multiword phrases, and word sets — that you register before casing:

```rust
use mla_titlecase::{titlecase_with_options, ExternalLexicons, TitleCaseOptions};

let mut lexicons = ExternalLexicons::default();
lexicons.add_multiword_map([("new york city", "New York City")]);

let options = TitleCaseOptions::with_external_lexicons(&lexicons);
assert_eq!(
    titlecase_with_options("new york city stories", &options),
    "New York City Stories",
);
```

Lexicon data can also be loaded from prebuilt plugin files (JSON or the compact
FST format). The companion [`mla-titlecase-cli`](../mla-titlecase-cli/README.md)
fetches upstream sources and builds those plugins; see
[`docs/plugin-format.md`](../../docs/plugin-format.md) for the on-disk schema.

## More

- [`docs/mla-rules.md`](../../docs/mla-rules.md) — the exact casing rules.
- [`docs/architecture.md`](../../docs/architecture.md) — how the engine is put together.
- [`docs/performance.md`](../../docs/performance.md) — the bulk-processing path.
