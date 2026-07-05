use std::collections::{BTreeMap, BTreeSet};

use reqwest::header::ACCEPT;
use serde::Deserialize;

use crate::{
    cli::PreparePayloadKind,
    error::{CliError, Result},
    normalize::{
        canonical_map_payload, multiword_map_payload, protected_spellings_payload,
        NormalizedPayload,
    },
    sources::{
        payload_kind_name, FetchOptions, PrepareOptions, ResolvedSource, SourceDefinition, SourceId,
    },
};

const ENDPOINT: &str = "https://query.wikidata.org/sparql";
const DEFAULT_LANGUAGE: &str = "en";
const DEFAULT_LIMIT: usize = 250;

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Wikidata,
        description: "Optional CC0 authority-style names from a live Wikidata SPARQL query",
        license_summary: "Wikidata structured data is available under CC0 1.0",
        notice: "The default Wikidata flow queries the public SPARQL endpoint and records the exact query URL in the fetch manifest.",
        default_url: ENDPOINT,
        recommended: true,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(
    client: &reqwest::blocking::Client,
    options: &FetchOptions,
) -> Result<ResolvedSource> {
    let language = options.language.as_deref().unwrap_or(DEFAULT_LANGUAGE);
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT);
    let query = options.query.clone().unwrap_or_else(|| default_query(language, limit));

    let response = client
        .get(ENDPOINT)
        .query(&[("format", "json"), ("query", query.as_str())])
        .header(ACCEPT, "application/sparql-results+json")
        .send()?
        .error_for_status()?;
    let source_url = response.url().to_string();
    let bytes = response.bytes()?.to_vec();

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
    if !matches!(
        requested_kind,
        PreparePayloadKind::CanonicalMap
            | PreparePayloadKind::MultiwordMap
            | PreparePayloadKind::ProtectedSpellings
    ) {
        return Err(CliError::SourceMetadata(format!(
            "{} supports only --payload-kind canonical-map, multiword-map, or protected-spellings (received {})",
            SourceId::Wikidata.as_str(),
            payload_kind_name(requested_kind)
        )));
    }

    let parsed = parse_rows(raw)?;
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

fn default_query(language: &str, limit: usize) -> String {
    format!(
        concat!(
            "PREFIX wd: <http://www.wikidata.org/entity/>\n",
            "PREFIX wdt: <http://www.wikidata.org/prop/direct/>\n",
            "PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n",
            "PREFIX skos: <http://www.w3.org/2004/02/skos/core#>\n",
            "SELECT ?item ?itemLabel ?altLabel WHERE {{\n",
            "  VALUES ?class {{ wd:Q5 wd:Q43229 wd:Q4830453 }}\n",
            "  ?item wdt:P31/wdt:P279* ?class .\n",
            "  ?item rdfs:label ?itemLabel .\n",
            "  FILTER(LANG(?itemLabel) = \"{language}\")\n",
            "  OPTIONAL {{\n",
            "    ?item skos:altLabel ?altLabel .\n",
            "    FILTER(LANG(?altLabel) = \"{language}\")\n",
            "  }}\n",
            "}}\n",
            "LIMIT {limit}\n"
        ),
        language = language,
        limit = limit
    )
}

struct ParsedRows {
    single_word_entries: Vec<(String, String)>,
    multiword_entries: Vec<(String, String)>,
    ignored_records: usize,
}

fn parse_rows(raw: &[u8]) -> Result<ParsedRows> {
    let response: SparqlResponse = serde_json::from_slice(raw)?;
    let mut entities = BTreeMap::<String, EntityAccumulator>::new();
    let mut ignored_records = 0_usize;

    for binding in response.results.bindings {
        let Some(label) = binding
            .item_label
            .as_ref()
            .map(|value| value.value.trim())
            .filter(|value| !value.is_empty())
        else {
            ignored_records += 1;
            continue;
        };

        let entity = entities.entry(binding.item.value).or_default();
        entity.label.get_or_insert_with(|| label.to_string());
        if let Some(alias) = binding
            .alt_label
            .as_ref()
            .map(|value| value.value.trim())
            .filter(|value| !value.is_empty())
        {
            entity.aliases.insert(alias.to_string());
        }
    }

    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    for entity in entities.into_values() {
        let Some(label) = entity.label else {
            ignored_records += 1;
            continue;
        };

        for surface in std::iter::once(label).chain(entity.aliases) {
            if is_multiword(&surface) {
                multiword_entries.push((surface.clone(), surface));
            } else {
                single_word_entries.push((surface.clone(), surface));
            }
        }
    }

    Ok(ParsedRows { single_word_entries, multiword_entries, ignored_records })
}

fn is_multiword(value: &str) -> bool {
    value.split_whitespace().nth(1).is_some()
}

#[derive(Debug, Default)]
struct EntityAccumulator {
    label: Option<String>,
    aliases: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
struct SparqlResponse {
    results: SparqlResults,
}

#[derive(Debug, Deserialize)]
struct SparqlResults {
    bindings: Vec<SparqlBinding>,
}

#[derive(Debug, Deserialize)]
struct SparqlBinding {
    item: SparqlValue,
    #[serde(rename = "itemLabel")]
    item_label: Option<SparqlValue>,
    #[serde(rename = "altLabel")]
    alt_label: Option<SparqlValue>,
}

#[derive(Debug, Deserialize)]
struct SparqlValue {
    value: String,
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::{default_query, prepare};

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/wikidata-sample.json");

    #[test]
    fn builds_default_query_with_language_and_limit() {
        let query = default_query("fr", 42);
        assert!(query.contains("LANG(?itemLabel) = \"fr\""));
        assert!(query.contains("LIMIT 42"));
    }

    #[test]
    fn prepares_multiword_payloads() {
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
                        key: "ludwig van beethoven".to_string(),
                        value: "Ludwig van Beethoven".to_string(),
                    },
                    MapEntry {
                        key: "new york city".to_string(),
                        value: "New York City".to_string(),
                    },
                    MapEntry {
                        key: "new york, new york".to_string(),
                        value: "New York, New York".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_canonical_payloads() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![
                    MapEntry { key: "ebay".to_string(), value: "eBay".to_string() },
                    MapEntry { key: "nyc".to_string(), value: "NYC".to_string() },
                ],
            }
        );
    }
}
