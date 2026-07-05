use std::collections::BTreeSet;

use reqwest::header::ACCEPT;
use serde::Deserialize;

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

const ENDPOINT: &str = "https://lobid.org/gnd/search";
const DEFAULT_QUERY: &str = "*";
const DEFAULT_FILTER: &str = "type:Person";
const DEFAULT_LIMIT: usize = 250;

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Gnd,
        description: "Optional CC0 authority-style names from the GND / lobid search API",
        license_summary: "GND authority data served by lobid is available under CC0 1.0",
        notice: "The default GND flow queries lobid.org with type:Person filtering and preserves the full request URL in the fetch manifest.",
        default_url: ENDPOINT,
        recommended: false,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(
    client: &reqwest::blocking::Client,
    options: &FetchOptions,
) -> Result<ResolvedSource> {
    let query = options.query.as_deref().unwrap_or(DEFAULT_QUERY);
    let limit = options.limit.unwrap_or(DEFAULT_LIMIT);
    let response = client
        .get(ENDPOINT)
        .query(&[
            ("q", query),
            ("filter", DEFAULT_FILTER),
            ("size", &limit.to_string()),
            ("format", "json"),
        ])
        .header(ACCEPT, "application/json")
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
    validate_payload_kind(
        SourceId::Gnd,
        requested_kind,
        &[
            PreparePayloadKind::CanonicalMap,
            PreparePayloadKind::MultiwordMap,
            PreparePayloadKind::ProtectedSpellings,
        ],
    )?;

    let parsed = parse_records(raw)?;
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

struct ParsedRecords {
    single_word_entries: Vec<(String, String)>,
    multiword_entries: Vec<(String, String)>,
    ignored_records: usize,
}

fn parse_records(raw: &[u8]) -> Result<ParsedRecords> {
    let response: SearchResponse = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for member in response.member {
        let surfaces = member.surfaces();
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

    Ok(ParsedRecords { single_word_entries, multiword_entries, ignored_records })
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    member: Vec<GndMember>,
}

#[derive(Debug, Deserialize)]
struct GndMember {
    #[serde(rename = "preferredName")]
    preferred_name: Option<String>,
    #[serde(rename = "variantName", default)]
    variant_name: Vec<String>,
    #[serde(rename = "preferredNameEntityForThePerson")]
    preferred_person_name: Option<PersonName>,
    #[serde(rename = "variantNameEntityForThePerson", default)]
    variant_person_names: Vec<PersonName>,
}

impl GndMember {
    fn surfaces(&self) -> Vec<String> {
        let mut surfaces = BTreeSet::new();

        if let Some(name) =
            self.preferred_name.as_deref().map(str::trim).filter(|value| !value.is_empty())
        {
            surfaces.insert(name.to_string());
        }

        for name in &self.variant_name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                surfaces.insert(trimmed.to_string());
            }
        }

        if let Some(person) = self.preferred_person_name.as_ref().and_then(PersonName::display_name)
        {
            surfaces.insert(person);
        }

        for person in &self.variant_person_names {
            if let Some(name) = person.display_name() {
                surfaces.insert(name);
            }
        }

        surfaces.into_iter().collect()
    }
}

#[derive(Debug, Deserialize)]
struct PersonName {
    #[serde(default)]
    forename: Vec<String>,
    #[serde(default)]
    prefix: Vec<String>,
    #[serde(default)]
    surname: Vec<String>,
}

impl PersonName {
    fn display_name(&self) -> Option<String> {
        let mut parts = Vec::new();
        parts.extend(self.forename.iter().map(String::as_str).filter(|value| !value.is_empty()));
        parts.extend(self.prefix.iter().map(String::as_str).filter(|value| !value.is_empty()));
        parts.extend(self.surname.iter().map(String::as_str).filter(|value| !value.is_empty()));

        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/gnd-sample.json");

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
                        key: "beethoven, ludwig van".to_string(),
                        value: "Beethoven, Ludwig van".to_string(),
                    },
                    MapEntry {
                        key: "beethoven, maria josepha van".to_string(),
                        value: "Beethoven, Maria Josepha van".to_string(),
                    },
                    MapEntry {
                        key: "ludwig van beethoven".to_string(),
                        value: "Ludwig van Beethoven".to_string(),
                    },
                    MapEntry {
                        key: "maria josepha poll".to_string(),
                        value: "Maria Josepha Poll".to_string(),
                    },
                    MapEntry {
                        key: "maria josepha van beethoven".to_string(),
                        value: "Maria Josepha van Beethoven".to_string(),
                    },
                    MapEntry {
                        key: "poll, maria josepha".to_string(),
                        value: "Poll, Maria Josepha".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_payloads() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![MapEntry {
                    key: "bayreuth".to_string(),
                    value: "Bayreuth".to_string()
                }],
            }
        );
    }
}
