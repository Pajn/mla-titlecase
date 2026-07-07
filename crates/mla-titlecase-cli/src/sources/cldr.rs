use serde_json::Value;

use crate::{
    cli::PreparePayloadKind,
    error::{CliError, Result},
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

const OWNER: &str = "unicode-org";
const REPO: &str = "cldr-json";
const REF: &str = "main";
const DATA_PATH: &str = "cldr-json/cldr-localenames-full/main/en/territories.json";

pub(crate) fn definition() -> SourceDefinition {
    SourceDefinition {
        id: SourceId::Cldr,
        description: "Unicode CLDR English display names for territories and regions",
        license_summary: "Unicode CLDR data is released under the Unicode License",
        notice: "CLDR display names are covered by the Unicode License. This flow uses the English territory display names.",
        default_url:
            "https://raw.githubusercontent.com/unicode-org/cldr-json/main/cldr-json/cldr-localenames-full/main/en/territories.json",
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
        SourceId::Cldr,
        requested_kind,
        &[
            PreparePayloadKind::CanonicalMap,
            PreparePayloadKind::MultiwordMap,
            PreparePayloadKind::ProtectedSpellings,
        ],
    )?;

    let parsed = parse_territories(raw)?;
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

struct ParsedTerritories {
    single_word_entries: Vec<(String, String)>,
    multiword_entries: Vec<(String, String)>,
    ignored_records: usize,
}

fn parse_territories(raw: &[u8]) -> Result<ParsedTerritories> {
    let root: Value = serde_json::from_slice(raw)?;
    let territories = root
        .get("main")
        .and_then(|main| main.as_object())
        .and_then(|main| main.values().next())
        .and_then(|locale| locale.get("localeDisplayNames"))
        .and_then(|names| names.get("territories"))
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CliError::SourceMetadata(
                "CLDR payload did not contain main.<locale>.localeDisplayNames.territories".into(),
            )
        })?;

    let mut single_word_entries = Vec::new();
    let mut multiword_entries = Vec::new();
    let mut ignored_records = 0_usize;

    for (code, value) in territories {
        // Skip alternate forms ("US-alt-short") — they duplicate short codes.
        if code.contains("-alt-") {
            ignored_records += 1;
            continue;
        }
        // Skip UN M49 numeric codes ("001" world, "142" Asia, "419" Latin
        // America): these are region/continent groupings, not place names.
        if code.bytes().all(|byte| byte.is_ascii_digit()) {
            ignored_records += 1;
            continue;
        }
        let Some(name) = value.as_str().map(str::trim).filter(|name| !name.is_empty()) else {
            ignored_records += 1;
            continue;
        };
        // Keep proper place names; drop generic lowercase labels like "world".
        if !name.chars().next().is_some_and(char::is_uppercase) {
            ignored_records += 1;
            continue;
        }
        let name = name.to_string();
        if is_multiword(&name) {
            multiword_entries.push((name.clone(), name));
        } else {
            single_word_entries.push((name.clone(), name));
        }
    }

    Ok(ParsedTerritories { single_word_entries, multiword_entries, ignored_records })
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{MapEntry, PluginPayload};

    use crate::cli::PreparePayloadKind;
    use crate::sources::PrepareOptions;

    use super::prepare;

    const FIXTURE: &[u8] = include_bytes!("../../../../testdata/fixtures/cldr-sample.json");

    #[test]
    fn prepares_multiword_territory_names_and_skips_generics() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::MultiwordMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::MultiwordMap {
                entries: vec![
                    MapEntry { key: "south korea".to_string(), value: "South Korea".to_string() },
                    MapEntry {
                        key: "united kingdom".to_string(),
                        value: "United Kingdom".to_string(),
                    },
                ],
            }
        );
    }

    #[test]
    fn prepares_single_word_territory_names() {
        let payload = prepare(
            FIXTURE,
            PrepareOptions { payload_kind: Some(PreparePayloadKind::CanonicalMap) },
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::CanonicalMap {
                entries: vec![MapEntry { key: "france".to_string(), value: "France".to_string() }],
            }
        );
    }
}
