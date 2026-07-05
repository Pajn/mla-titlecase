use serde::Deserialize;
use serde_json::Value;

use crate::{
    cli::PreparePayloadKind,
    error::Result,
    normalize::{
        canonical_map_payload, multiword_map_payload, protected_spellings_payload,
        NormalizedPayload,
    },
    sources::{
        is_multiword, validate_payload_kind, FetchOptions, PrepareOptions, ResolvedSource,
        SourceDefinition, SourceId,
    },
};

const ENDPOINT: &str = "https://api.crossref.org/journals";
const DEFAULT_LIMIT: usize = 250;
const MAX_ROWS: usize = 1000;
/// Descriptive identifier for Crossref's "polite pool" (URL form, per Crossref's
/// User-Agent guidance), which grants more reliable service than the anonymous
/// pool.
const POLITE_USER_AGENT: &str = concat!(
    "mla-titlecase-cli/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/Pajn/mla-titlecase)"
);

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Crossref,
        description: "Journal, periodical, and publisher names from the Crossref REST API",
        license_summary: "Crossref metadata is freely reusable; Crossref asserts no copyright over it (CC0-style)",
        notice: "The default Crossref flow queries the public REST API (api.crossref.org) and records the first request URL in the fetch manifest. It pages through results with cursor paging, accumulating up to --limit journals.",
        default_url: ENDPOINT,
        recommended: true,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(
    client: &reqwest::blocking::Client,
    options: &FetchOptions,
) -> Result<ResolvedSource> {
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT);
    let mut items = Vec::<Value>::new();
    let mut source_url = String::new();
    // Deep offset paging on `/journals` is capped at 10k results, so use cursor
    // paging instead (start at "*", then follow `message.next-cursor`), which
    // has no such ceiling. `rows` stays fixed across requests as cursor paging
    // expects, and the final page is truncated to `limit`.
    let mut cursor = "*".to_string();

    while items.len() < limit {
        let mut request = client
            .get(ENDPOINT)
            .header(reqwest::header::USER_AGENT, POLITE_USER_AGENT)
            .query(&[("rows", MAX_ROWS.to_string()), ("cursor", cursor.clone())]);
        if let Some(query) = options.query.as_ref() {
            request = request.query(&[("query", query)]);
        }

        let response = request.send()?.error_for_status()?;
        if source_url.is_empty() {
            source_url = response.url().to_string();
        }
        let page: CrossrefPage = response.json()?;
        let page_len = page.message.items.len();
        let total_results = page.message.total_results;
        let next_cursor = page.message.next_cursor;
        items.extend(page.message.items);

        // Stop once the corpus (or query result set) is drained: an empty page,
        // no further cursor, or having collected every reported result. The
        // `total_results` check avoids a final empty round-trip when the corpus
        // size is an exact multiple of `rows`.
        if page_len == 0 || (total_results > 0 && items.len() >= total_results) {
            break;
        }
        match next_cursor {
            Some(next) => cursor = next,
            None => break,
        }
    }
    items.truncate(limit);

    let bytes = serde_json::to_vec_pretty(&serde_json::json!({ "message": { "items": items } }))?;
    Ok(ResolvedSource {
        bytes,
        source_url,
        source_version: None,
        license_summary: definition().license_summary.to_string(),
        notice: Some(definition().notice.to_string()),
    })
}

pub(crate) fn prepare(raw: &[u8], options: PrepareOptions) -> Result<NormalizedPayload> {
    let requested_kind = options.payload_kind.unwrap_or(PreparePayloadKind::MultiwordMap);
    validate_payload_kind(
        SourceId::Crossref,
        requested_kind,
        &[
            PreparePayloadKind::CanonicalMap,
            PreparePayloadKind::MultiwordMap,
            PreparePayloadKind::ProtectedSpellings,
        ],
    )?;

    let parsed = parse_items(raw)?;
    let mut payload = match requested_kind {
        PreparePayloadKind::CanonicalMap => canonical_map_payload(parsed.single_word_entries),
        PreparePayloadKind::MultiwordMap => multiword_map_payload(parsed.multiword_entries),
        PreparePayloadKind::ProtectedSpellings => {
            protected_spellings_payload(parsed.single_word_entries)
        }
        _ => unreachable!("validated above"),
    };
    payload.report.ignored_records += parsed.ignored_records;
    Ok(payload)
}

struct ParsedItems {
    single_word_entries: Vec<(String, String)>,
    multiword_entries: Vec<(String, String)>,
    ignored_records: usize,
}

fn parse_items(raw: &[u8]) -> Result<ParsedItems> {
    let response: CrossrefResponse = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for item in response.message.items {
        let mut kept = false;
        for surface in [item.title, item.publisher].into_iter().flatten() {
            let surface = surface.trim().to_string();
            if surface.is_empty() {
                continue;
            }
            kept = true;
            if is_multiword(&surface) {
                multiword_entries.push((surface.clone(), surface));
            } else {
                single_word_entries.push((surface.clone(), surface));
            }
        }
        if !kept {
            ignored_records += 1;
        }
    }

    Ok(ParsedItems { single_word_entries, multiword_entries, ignored_records })
}

/// Raw page shape used while fetching: items stay as `Value` so accumulated
/// pages can be re-serialized verbatim for `prepare` to parse later.
#[derive(Debug, Deserialize)]
struct CrossrefPage {
    message: CrossrefPageMessage,
}

#[derive(Debug, Deserialize)]
struct CrossrefPageMessage {
    #[serde(default)]
    items: Vec<Value>,
    #[serde(rename = "next-cursor", default)]
    next_cursor: Option<String>,
    #[serde(rename = "total-results", default)]
    total_results: usize,
}

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize)]
struct CrossrefMessage {
    items: Vec<CrossrefItem>,
}

#[derive(Debug, Deserialize)]
struct CrossrefItem {
    title: Option<String>,
    publisher: Option<String>,
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/crossref-sample.json");

    #[test]
    fn prepares_multiword_journal_and_publisher_names() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::MultiwordMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::MultiwordMap {
                entries: vec![
                    MapEntry {
                        key: "johns hopkins university press".to_string(),
                        value: "Johns Hopkins University Press".to_string(),
                    },
                    MapEntry {
                        key: "modern language association".to_string(),
                        value: "Modern Language Association".to_string(),
                    },
                    MapEntry {
                        key: "studies in the novel".to_string(),
                        value: "Studies in the Novel".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_journal_acronyms() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![MapEntry { key: "pmla".to_string(), value: "PMLA".to_string() }],
            }
        );
    }
}
