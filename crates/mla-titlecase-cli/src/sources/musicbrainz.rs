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

const ENDPOINT: &str = "https://musicbrainz.org/ws/2/artist/";
const DEFAULT_QUERY: &str = "*:*";
const DEFAULT_LIMIT: usize = 250;

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Musicbrainz,
        description: "Optional CC0 artist-name data from the public MusicBrainz JSON web service",
        license_summary:
            "MusicBrainz core database data is CC0; this CLI uses the public artist JSON web service only",
        notice: "The current MusicBrainz integration targets artist records from the public /ws/2/artist JSON API and preserves the full request URL in the fetch manifest.",
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
        .query(&[("query", query), ("fmt", "json"), ("limit", &limit.to_string())])
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
    let requested_kind = options.payload_kind.unwrap_or(PreparePayloadKind::ProtectedSpellings);
    validate_payload_kind(
        SourceId::Musicbrainz,
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
    let response: ArtistSearchResponse = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for artist in response.artists {
        let surfaces = artist.surfaces();
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
struct ArtistSearchResponse {
    #[serde(default)]
    artists: Vec<ArtistRecord>,
}

#[derive(Debug, Deserialize)]
struct ArtistRecord {
    name: Option<String>,
    #[serde(rename = "sort-name")]
    sort_name: Option<String>,
    #[serde(default)]
    aliases: Vec<AliasRecord>,
}

impl ArtistRecord {
    fn surfaces(&self) -> Vec<String> {
        let mut surfaces = BTreeSet::new();

        if let Some(name) = self.name.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
            surfaces.insert(name.to_string());
        }
        if let Some(name) =
            self.sort_name.as_deref().map(str::trim).filter(|value| !value.is_empty())
        {
            surfaces.insert(name.to_string());
        }
        for alias in &self.aliases {
            if let Some(name) =
                alias.name.as_deref().map(str::trim).filter(|value| !value.is_empty())
            {
                surfaces.insert(name.to_string());
            }
        }

        surfaces.into_iter().collect()
    }
}

#[derive(Debug, Deserialize)]
struct AliasRecord {
    name: Option<String>,
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/musicbrainz-sample.json");

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
                    MapEntry { key: "dana owens".to_string(), value: "Dana Owens".to_string() },
                    MapEntry {
                        key: "latifah, queen".to_string(),
                        value: "Latifah, Queen".to_string(),
                    },
                    MapEntry {
                        key: "queen latifah".to_string(),
                        value: "Queen Latifah".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_payloads() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::ProtectedSpellings) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::ProtectedSpellings {
                entries: vec![
                    MapEntry { key: "p!nk".to_string(), value: "P!nk".to_string() },
                    MapEntry { key: "queen".to_string(), value: "Queen".to_string() },
                ],
            }
        );
    }
}
