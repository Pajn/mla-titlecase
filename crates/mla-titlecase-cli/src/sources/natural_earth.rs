use serde::Deserialize;

use crate::{
    cli::PreparePayloadKind,
    error::Result,
    normalize::{
        canonical_map_payload, multiword_map_payload, protected_spellings_payload,
        NormalizedPayload,
    },
    sources::{
        github::{download_bytes, resolve_file},
        is_multiword, validate_payload_kind, PrepareOptions, ResolvedSource, SourceDefinition,
        SourceId,
    },
};

const OWNER: &str = "nvkelso";
const REPO: &str = "natural-earth-vector";
const REF: &str = "master";
const DATA_PATH: &str = "geojson/ne_110m_admin_0_countries.geojson";

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::NaturalEarth,
        description: "Public-domain country and territory names from Natural Earth",
        license_summary: "Natural Earth is released into the public domain",
        notice: "Natural Earth data is in the public domain; no attribution is required, though crediting Natural Earth is appreciated.",
        default_url:
            "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_110m_admin_0_countries.geojson",
        recommended: false,
        requires_acknowledgement: false,
    }
}

pub(crate) fn fetch(client: &reqwest::blocking::Client) -> Result<ResolvedSource> {
    let data = resolve_file(client, OWNER, REPO, DATA_PATH, REF)?;
    Ok(ResolvedSource {
        bytes: download_bytes(client, &data.download_url)?,
        source_url: data.download_url,
        source_version: Some(data.sha),
        license_summary: definition().license_summary.to_string(),
        notice: Some(definition().notice.to_string()),
    })
}

pub(crate) fn prepare(raw: &[u8], options: PrepareOptions) -> Result<NormalizedPayload> {
    let requested_kind = options.payload_kind.unwrap_or(PreparePayloadKind::MultiwordMap);
    validate_payload_kind(
        SourceId::NaturalEarth,
        requested_kind,
        &[
            PreparePayloadKind::CanonicalMap,
            PreparePayloadKind::MultiwordMap,
            PreparePayloadKind::ProtectedSpellings,
        ],
    )?;

    let parsed = parse_features(raw)?;
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

struct ParsedFeatures {
    single_word_entries: Vec<(String, String)>,
    multiword_entries: Vec<(String, String)>,
    ignored_records: usize,
}

fn parse_features(raw: &[u8]) -> Result<ParsedFeatures> {
    let collection: FeatureCollection = serde_json::from_slice(raw)?;
    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for feature in collection.features {
        let mut kept = false;
        // NAME and NAME_LONG are frequently identical (e.g. "South Africa"); skip
        // the second surface when it repeats the first so each feature yields
        // only unique (surface, surface) pairs.
        let mut previous: Option<String> = None;
        for surface in [feature.properties.name, feature.properties.name_long].into_iter().flatten()
        {
            let surface = surface.trim().to_string();
            if surface.is_empty() {
                continue;
            }
            if previous.as_deref() == Some(surface.as_str()) {
                continue;
            }
            previous = Some(surface.clone());
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

    Ok(ParsedFeatures { single_word_entries, multiword_entries, ignored_records })
}

#[derive(Debug, Deserialize)]
struct FeatureCollection {
    #[serde(default)]
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    #[serde(default)]
    properties: FeatureProperties,
}

#[derive(Debug, Default, Deserialize)]
struct FeatureProperties {
    #[serde(rename = "NAME")]
    name: Option<String>,
    #[serde(rename = "NAME_LONG")]
    name_long: Option<String>,
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] =
        include_bytes!("../../../../testdata/fixtures/natural-earth-sample.json");

    #[test]
    fn prepares_multiword_country_names() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::MultiwordMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::MultiwordMap {
                entries: vec![
                    // A distinct NAME_LONG surfaces alongside the short NAME.
                    MapEntry {
                        key: "czech republic".to_string(),
                        value: "Czech Republic".to_string(),
                    },
                    MapEntry { key: "south africa".to_string(), value: "South Africa".to_string() },
                    MapEntry {
                        key: "united states of america".to_string(),
                        value: "United States of America".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_country_names() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![
                    // The short NAME "Czechia" surfaces even when NAME_LONG differs.
                    MapEntry { key: "czechia".to_string(), value: "Czechia".to_string() },
                    MapEntry { key: "france".to_string(), value: "France".to_string() },
                ],
            }
        );
    }
}
