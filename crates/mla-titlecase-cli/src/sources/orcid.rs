use std::collections::BTreeSet;

use quick_xml::{events::Event, Reader};
use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};

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

const SEARCH_ENDPOINT: &str = "https://pub.orcid.org/v3.0/search/";
const PERSON_ENDPOINT_PREFIX: &str = "https://pub.orcid.org/v3.0";
const DEFAULT_QUERY: &str = "given-names:*";
const DEFAULT_LIMIT: usize = 20;

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Orcid,
        description: "Optional researcher-name data aggregated from the public ORCID search and person APIs",
        license_summary: "ORCID public data is CC0; ORCID trademarks and community norms remain separate from data licensing",
        notice: "This integration aggregates public ORCID search results plus public person records. Review ORCID trademark and community-norm guidance separately from the CC0 data license.",
        default_url: SEARCH_ENDPOINT,
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

    let search_response = client
        .get(SEARCH_ENDPOINT)
        .query(&[("q", query), ("rows", &limit.to_string())])
        .header(ACCEPT, "application/vnd.orcid+xml")
        .send()?
        .error_for_status()?;
    let search_url = search_response.url().to_string();
    let search_xml = search_response.bytes()?.to_vec();
    let ids = extract_search_ids(&search_xml)?;

    let mut people = Vec::new();
    for orcid in ids.into_iter().take(limit) {
        let person = client
            .get(format!("{PERSON_ENDPOINT_PREFIX}/{orcid}/person"))
            .header(ACCEPT, "application/json")
            .send()?
            .error_for_status()?
            .json::<OrcidPersonRecord>()?;
        people.push(FetchedPerson { orcid, person });
    }

    let batch = OrcidRawBatch { query: query.to_string(), search_url: search_url.clone(), people };
    let bytes = serde_json::to_vec_pretty(&batch)?;

    Ok(ResolvedSource {
        bytes,
        source_url: search_url,
        source_version: None,
        license_summary: definition().license_summary.to_string(),
        notice: Some(definition().notice.to_string()),
    })
}

pub(crate) fn prepare(raw: &[u8], options: PrepareOptions) -> Result<NormalizedPayload> {
    let requested_kind = options.payload_kind.unwrap_or(PreparePayloadKind::MultiwordMap);
    validate_payload_kind(
        SourceId::Orcid,
        requested_kind,
        &[
            PreparePayloadKind::CanonicalMap,
            PreparePayloadKind::MultiwordMap,
            PreparePayloadKind::ProtectedSpellings,
        ],
    )?;

    let batch: OrcidRawBatch = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for person in batch.people {
        let surfaces = person.person.surfaces();
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

    let mut payload = match requested_kind {
        PreparePayloadKind::CanonicalMap => canonical_map_payload(single_word_entries),
        PreparePayloadKind::MultiwordMap => multiword_map_payload(multiword_entries),
        PreparePayloadKind::ProtectedSpellings => protected_spellings_payload(single_word_entries),
        _ => unreachable!("validated above"),
    };
    payload.report.ignored_records += ignored_records;
    Ok(payload)
}

fn extract_search_ids(raw: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(raw);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_path = false;
    let mut ids = Vec::new();

    loop {
        match reader.read_event_into(&mut buf).map_err(xml_error)? {
            Event::Start(event) => {
                in_path = local_name(event.name().as_ref()) == b"path";
            }
            Event::End(_) => {
                in_path = false;
            }
            Event::Text(text) if in_path => {
                let value =
                    std::str::from_utf8(text.as_ref()).map_err(xml_error)?.trim().to_string();
                if !value.is_empty() {
                    ids.push(value);
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(ids)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

fn xml_error(error: impl std::fmt::Display) -> CliError {
    CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: format!("invalid ORCID XML payload: {error}"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OrcidRawBatch {
    query: String,
    search_url: String,
    people: Vec<FetchedPerson>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FetchedPerson {
    orcid: String,
    person: OrcidPersonRecord,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrcidPersonRecord {
    name: Option<OrcidName>,
    #[serde(rename = "other-names")]
    other_names: Option<OtherNames>,
}

impl OrcidPersonRecord {
    fn surfaces(&self) -> Vec<String> {
        let mut surfaces = BTreeSet::new();

        if let Some(name) = self.name.as_ref().and_then(OrcidName::display_name) {
            surfaces.insert(name);
        }

        if let Some(other_names) = self.other_names.as_ref() {
            for other_name in &other_names.other_name {
                if let Some(name) = other_name.display_name() {
                    surfaces.insert(name);
                }
            }
        }

        surfaces.into_iter().collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OrcidName {
    #[serde(rename = "credit-name")]
    credit_name: Option<ValueField>,
    #[serde(rename = "given-names")]
    given_names: Option<ValueField>,
    #[serde(rename = "family-name")]
    family_name: Option<ValueField>,
}

impl OrcidName {
    fn display_name(&self) -> Option<String> {
        if let Some(value) = self.credit_name.as_ref().and_then(ValueField::as_str) {
            return Some(value.to_string());
        }

        let mut parts = Vec::new();
        if let Some(value) = self.given_names.as_ref().and_then(ValueField::as_str) {
            parts.push(value);
        }
        if let Some(value) = self.family_name.as_ref().and_then(ValueField::as_str) {
            parts.push(value);
        }
        (!parts.is_empty()).then(|| parts.join(" "))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OtherNames {
    #[serde(rename = "other-name", default)]
    other_name: Vec<OtherName>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtherName {
    content: Option<String>,
    value: Option<String>,
}

impl OtherName {
    fn display_name(&self) -> Option<String> {
        self.content
            .as_deref()
            .or(self.value.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ValueField {
    value: String,
}

impl ValueField {
    fn as_str(&self) -> Option<&str> {
        let value = self.value.trim();
        (!value.is_empty()).then_some(value)
    }
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::{extract_search_ids, prepare};

    const SEARCH_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<search:search xmlns:search="http://www.orcid.org/ns/search" xmlns:common="http://www.orcid.org/ns/common">
  <search:result><common:orcid-identifier><common:path>0000-0002-1825-0097</common:path></common:orcid-identifier></search:result>
  <search:result><common:orcid-identifier><common:path>0000-0001-5109-3700</common:path></common:orcid-identifier></search:result>
</search:search>"#;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/orcid-raw-sample.json");

    #[test]
    fn extracts_ids_from_search_xml() {
        assert_eq!(
            extract_search_ids(SEARCH_XML.as_bytes()).unwrap(),
            vec!["0000-0002-1825-0097".to_string(), "0000-0001-5109-3700".to_string()]
        );
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
                    MapEntry { key: "j. carberry".to_string(), value: "J. Carberry".to_string() },
                    MapEntry {
                        key: "josiah carberry".to_string(),
                        value: "Josiah Carberry".to_string(),
                    },
                    MapEntry {
                        key: "josiah stinkney carberry".to_string(),
                        value: "Josiah Stinkney Carberry".to_string(),
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
                    key: "carberry".to_string(),
                    value: "Carberry".to_string()
                }],
            }
        );
    }
}
