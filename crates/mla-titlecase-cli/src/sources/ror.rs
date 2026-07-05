use serde::Deserialize;
use serde_json::Value;

use crate::{
    cli::PreparePayloadKind,
    error::{CliError, Result},
    normalize::{
        canonical_map_payload, multiword_map_payload, protected_spellings_payload,
        NormalizedPayload,
    },
    sources::{
        is_multiword, validate_payload_kind, FetchOptions, PrepareOptions, ResolvedSource,
        SourceDefinition, SourceId,
    },
};

const ENDPOINT: &str = "https://api.ror.org/organizations";
const DEFAULT_LIMIT: usize = 250;
/// Safety ceiling on the number of ROR pages (20 organizations each) fetched
/// for a single request. High enough to satisfy any realistic `--limit`
/// (~200k organizations); exceeding it is reported as an error rather than
/// silently returning fewer results than requested.
const PAGE_CAP: usize = 10_000;

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Ror,
        description: "Research-organization and university names from the ROR registry",
        license_summary: "ROR data is released under CC0 1.0",
        notice: "The default ROR flow queries the public REST API (api.ror.org) and records the request URL in the fetch manifest. Acronyms are dropped: they collide with ordinary words.",
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
    // Whether we stopped for a legitimate reason (reached `limit` or drained the
    // registry) rather than by exhausting the page ceiling with data still left.
    let mut satisfied = false;

    for page in 1..=PAGE_CAP {
        let mut request = client.get(ENDPOINT).query(&[("page", page.to_string())]);
        if let Some(query) = options.query.as_ref() {
            request = request.query(&[("query", query)]);
        }
        let response = request.send()?.error_for_status()?;
        if page == 1 {
            source_url = response.url().to_string();
        }
        let body: RorResponse = response.json()?;
        if body.items.is_empty() {
            // Registry exhausted before reaching `limit`; returning fewer is expected.
            satisfied = true;
            break;
        }
        items.extend(body.items);
        if items.len() >= limit {
            items.truncate(limit);
            satisfied = true;
            break;
        }
    }

    if !satisfied {
        return Err(CliError::SourceMetadata(format!(
            "ROR paging stopped at the {PAGE_CAP}-page cap with only {} of the requested {limit} organizations; lower --limit",
            items.len(),
        )));
    }

    let bytes = serde_json::to_vec_pretty(&serde_json::json!({ "items": items }))?;
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
        SourceId::Ror,
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
    let response: RorResponse = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for item in response.items {
        let surfaces = organization_names(&item);
        if surfaces.is_empty() {
            ignored_records += 1;
            continue;
        }
        for surface in surfaces {
            if is_multiword(&surface) {
                multiword_entries.push((surface.clone(), surface));
            } else {
                single_word_entries.push((surface.clone(), surface));
            }
        }
    }

    Ok(ParsedItems { single_word_entries, multiword_entries, ignored_records })
}

/// Collects an organization's names, handling both the v2 (`names` with typed
/// entries) and v1 (`name` / `aliases` / `labels`) schemas. Acronyms are
/// excluded on both paths: mapping "mit" to a name would rewrite ordinary words.
fn organization_names(item: &Value) -> Vec<String> {
    let mut names = Vec::new();

    if let Some(Value::Array(entries)) = item.get("names") {
        for entry in entries {
            let types: Vec<&str> = entry
                .get("types")
                .and_then(Value::as_array)
                .map(|types| types.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if types.contains(&"acronym") {
                continue;
            }
            push_name(&mut names, entry.get("value").and_then(Value::as_str));
        }
    } else {
        push_name(&mut names, item.get("name").and_then(Value::as_str));
        if let Some(Value::Array(aliases)) = item.get("aliases") {
            for alias in aliases {
                push_name(&mut names, alias.as_str());
            }
        }
        if let Some(Value::Array(labels)) = item.get("labels") {
            for label in labels {
                push_name(&mut names, label.get("label").and_then(Value::as_str));
            }
        }
    }

    names
}

fn push_name(names: &mut Vec<String>, value: Option<&str>) {
    if let Some(text) = value.map(str::trim).filter(|text| !text.is_empty()) {
        let owned = text.to_string();
        if !names.contains(&owned) {
            names.push(owned);
        }
    }
}

#[derive(Debug, Deserialize)]
struct RorResponse {
    #[serde(default)]
    items: Vec<Value>,
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/ror-sample.json");

    #[test]
    fn prepares_multiword_organization_names_without_acronyms() {
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
                        key: "harvard university".to_string(),
                        value: "Harvard University".to_string(),
                    },
                    MapEntry {
                        key: "karolinska institutet".to_string(),
                        value: "Karolinska Institutet".to_string(),
                    },
                    MapEntry {
                        key: "université harvard".to_string(),
                        value: "Université Harvard".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_organization_names() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        // "Harvard" is the only single-word surface; the "KI"/"HU" acronyms are
        // dropped rather than emitted as single-word entries.
        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![MapEntry {
                    key: "harvard".to_string(),
                    value: "Harvard".to_string(),
                }],
            }
        );
    }
}
