use std::io::Read;

use flate2::read::GzDecoder;
use mla_titlecase::{
    util::normalize::{lookup_key, normalized_unique_sorted},
    MapEntry, PluginPayload, RankedEntry,
};
use rmpv::Value;
use serde_json::Value as JsonValue;

use crate::{
    error::{CliError, Result},
    manifest::NormalizationReport,
};

pub(crate) struct NormalizedPayload {
    pub(crate) payload: PluginPayload,
    pub(crate) report: NormalizationReport,
}

pub(crate) fn parse_stopwords_json(raw: &[u8]) -> Result<NormalizedPayload> {
    let value: JsonValue = serde_json::from_slice(raw)?;
    let words: Vec<String> = match value {
        JsonValue::Array(items) => {
            items.into_iter().map(string_value).collect::<std::result::Result<Vec<_>, _>>()?
        }
        JsonValue::Object(map) => map
            .get("en")
            .ok_or_else(|| CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: "expected an `en` array in stopwords JSON".to_string(),
            })?
            .as_array()
            .ok_or_else(|| CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: "expected `en` to be an array".to_string(),
            })?
            .iter()
            .cloned()
            .map(string_value)
            .collect::<std::result::Result<Vec<_>, _>>()?,
        _ => {
            return Err(CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: "expected a JSON array or object".to_string(),
            })
        }
    };
    Ok(word_set_payload(words, 0))
}

pub(crate) fn parse_scowl_word_list(raw: &[u8]) -> Result<NormalizedPayload> {
    let raw = std::str::from_utf8(raw).map_err(|error| CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: format!("SCOWL input must be UTF-8: {error}"),
    })?;

    let mut extracted = Vec::new();
    let mut ignored_records = 0;
    let mut current_headword: Option<String> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let without_comment = trimmed.split('#').next().unwrap_or(trimmed).trim();
        let Some((_metadata, body)) = without_comment.split_once(':') else {
            ignored_records += 1;
            continue;
        };

        let mut sections = body.split(':');
        let Some(main_section) = sections.next() else {
            ignored_records += 1;
            continue;
        };

        let headword = resolve_scowl_headword(main_section, current_headword.as_deref());
        let Some(headword) = headword else {
            ignored_records += 1;
            continue;
        };
        current_headword = Some(headword.clone());
        extracted.extend(extract_scowl_words(&headword, Some(&headword)));

        for variant_section in sections {
            extracted.extend(extract_scowl_words(variant_section, Some(&headword)));
        }
    }

    let input_records = extracted.len();
    Ok(word_set_payload_with_counts(extracted, input_records, ignored_records))
}

pub(crate) fn parse_wordfreq_msgpack(raw: &[u8]) -> Result<NormalizedPayload> {
    let mut decoder = GzDecoder::new(raw);
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    let value = rmpv::decode::read_value(&mut std::io::Cursor::new(bytes)).map_err(|error| {
        CliError::UnsupportedInput {
            path: std::path::PathBuf::from("<memory>"),
            message: format!("invalid wordfreq MessagePack payload: {error}"),
        }
    })?;

    let buckets = parse_cbpack(value)?;
    let mut next_rank = 1_u64;
    let mut input_records = 0_usize;
    let mut deduped = std::collections::BTreeMap::new();

    for bucket in buckets {
        for word in bucket {
            input_records += 1;
            deduped.entry(word).or_insert_with(|| {
                let rank = next_rank;
                next_rank += 1;
                rank
            });
        }
    }

    let entries =
        deduped.into_iter().map(|(word, rank)| RankedEntry { word, rank }).collect::<Vec<_>>();

    Ok(NormalizedPayload {
        report: NormalizationReport {
            input_records,
            output_entries: entries.len(),
            duplicates_removed: input_records.saturating_sub(entries.len()),
            ignored_records: 0,
        },
        payload: PluginPayload::RankedWords { entries },
    })
}

pub(crate) fn canonical_map_payload(entries: Vec<(String, String)>) -> NormalizedPayload {
    map_payload(entries, normalize_single_word_key, |entries| PluginPayload::CanonicalMap {
        entries,
    })
}

pub(crate) fn multiword_map_payload(entries: Vec<(String, String)>) -> NormalizedPayload {
    map_payload(entries, normalize_phrase_key, |entries| PluginPayload::MultiwordMap { entries })
}

pub(crate) fn protected_spellings_payload(entries: Vec<(String, String)>) -> NormalizedPayload {
    map_payload(entries, normalize_single_word_key, |entries| PluginPayload::ProtectedSpellings {
        entries,
    })
}

fn word_set_payload(words: Vec<String>, ignored_records: usize) -> NormalizedPayload {
    word_set_payload_with_counts(words.clone(), words.len(), ignored_records)
}

fn word_set_payload_with_counts(
    words: Vec<String>,
    input_records: usize,
    ignored_records: usize,
) -> NormalizedPayload {
    let normalized = normalized_unique_sorted(words);
    let output_entries = normalized.len();
    NormalizedPayload {
        report: NormalizationReport {
            input_records,
            output_entries,
            duplicates_removed: input_records.saturating_sub(output_entries),
            ignored_records,
        },
        payload: PluginPayload::WordSet { words: normalized },
    }
}

fn map_payload<F>(
    entries: Vec<(String, String)>,
    normalize_key: F,
    build_payload: impl FnOnce(Vec<MapEntry>) -> PluginPayload,
) -> NormalizedPayload
where
    F: Fn(&str) -> String,
{
    let input_records = entries.len();
    let mut deduped = std::collections::BTreeMap::new();
    let mut valid_records = 0_usize;
    let mut ignored_records = 0_usize;

    for (key, value) in entries {
        let normalized_key = normalize_key(&key);
        let normalized_value = value.trim();
        if normalized_key.is_empty() || normalized_value.is_empty() {
            ignored_records += 1;
            continue;
        }
        valid_records += 1;
        deduped.entry(normalized_key).or_insert_with(|| normalized_value.to_string());
    }

    let entries =
        deduped.into_iter().map(|(key, value)| MapEntry { key, value }).collect::<Vec<_>>();

    NormalizedPayload {
        report: NormalizationReport {
            input_records,
            output_entries: entries.len(),
            duplicates_removed: valid_records.saturating_sub(entries.len()),
            ignored_records,
        },
        payload: build_payload(entries),
    }
}

fn normalize_single_word_key(value: &str) -> String {
    lookup_key(value)
}

fn normalize_phrase_key(value: &str) -> String {
    value.split_whitespace().map(lookup_key).collect::<Vec<_>>().join(" ")
}

fn resolve_scowl_headword(fragment: &str, current_headword: Option<&str>) -> Option<String> {
    let head = fragment.split('<').next().unwrap_or(fragment).trim();
    let candidate = head.rsplit(':').next().unwrap_or(head).trim();
    match candidate {
        "" => None,
        "-" | "~" => current_headword.map(ToOwned::to_owned),
        other => Some(other.to_string()),
    }
}

fn extract_scowl_words(fragment: &str, current_headword: Option<&str>) -> Vec<String> {
    let base = fragment.split('<').next().unwrap_or(fragment).trim();
    let candidate = base.rsplit(':').next().unwrap_or(base).trim();
    let replaced = match candidate {
        "-" | "~" => current_headword.unwrap_or(candidate).to_string(),
        other => other.replace('~', current_headword.unwrap_or("")),
    };

    let mut words = Vec::new();
    let mut current = String::new();
    for ch in replaced.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '\'' | '.' | '-') {
            current.push(ch);
        } else if !current.is_empty() {
            push_candidate(&mut words, &mut current);
        }
    }
    if !current.is_empty() {
        push_candidate(&mut words, &mut current);
    }
    words
}

fn push_candidate(words: &mut Vec<String>, current: &mut String) {
    let candidate = current.trim_matches(|ch: char| ch == '\'' || ch == '-');
    if candidate.chars().any(|ch| ch.is_ascii_alphanumeric()) {
        words.push(candidate.to_string());
    }
    current.clear();
}

fn parse_cbpack(value: Value) -> Result<Vec<Vec<String>>> {
    let array = value.as_array().ok_or_else(|| CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: "wordfreq data must decode to a top-level array".to_string(),
    })?;
    let Some(header) = array.first() else {
        return Err(CliError::UnsupportedInput {
            path: std::path::PathBuf::from("<memory>"),
            message: "wordfreq data is missing the cBpack header".to_string(),
        });
    };
    validate_cbpack_header(header)?;

    array
        .iter()
        .skip(1)
        .map(|bucket| {
            let words = bucket.as_array().ok_or_else(|| CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: "wordfreq bucket must be an array of strings".to_string(),
            })?;
            words
                .iter()
                .map(|word| {
                    word.as_str().map(ToOwned::to_owned).ok_or_else(|| CliError::UnsupportedInput {
                        path: std::path::PathBuf::from("<memory>"),
                        message: "wordfreq bucket entry must be a string".to_string(),
                    })
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect()
}

fn validate_cbpack_header(header: &Value) -> Result<()> {
    let map = header.as_map().ok_or_else(|| CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: "wordfreq header must be a map".to_string(),
    })?;

    let format =
        map.iter().find_map(|(key, value)| (key.as_str() == Some("format")).then_some(value));
    let version =
        map.iter().find_map(|(key, value)| (key.as_str() == Some("version")).then_some(value));

    if format.and_then(Value::as_str) != Some("cB") || version.and_then(Value::as_i64) != Some(1) {
        return Err(CliError::UnsupportedInput {
            path: std::path::PathBuf::from("<memory>"),
            message: "unexpected wordfreq cBpack header".to_string(),
        });
    }

    Ok(())
}

fn string_value(value: JsonValue) -> Result<String> {
    value.as_str().map(ToOwned::to_owned).ok_or_else(|| CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: "expected a string value".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use flate2::{write::GzEncoder, Compression};
    use mla_titlecase::{PluginPayload, RankedEntry};
    use rmpv::Value;

    use super::{parse_scowl_word_list, parse_stopwords_json, parse_wordfreq_msgpack};

    #[test]
    fn parses_stopword_json() {
        let payload = parse_stopwords_json(br#"["and", "the"]"#).unwrap();
        assert_eq!(
            payload.payload,
            PluginPayload::WordSet { words: vec!["and".to_string(), "the".to_string()] }
        );
        assert_eq!(payload.report.input_records, 2);
    }

    #[test]
    fn parses_scowl_pre_lines() {
        let payload = parse_scowl_word_list(
            br#"
35: that <d>: those
35: who <pn>: whom, -, whose, whose
50 [12dicts]: - <n>: who's
"#,
        )
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::WordSet {
                words: vec![
                    "that".to_string(),
                    "those".to_string(),
                    "who".to_string(),
                    "who's".to_string(),
                    "whom".to_string(),
                    "whose".to_string(),
                ],
            }
        );
        assert_eq!(payload.report.input_records, 8);
    }

    #[test]
    fn parses_wordfreq_msgpack_payload() {
        let payload = parse_wordfreq_msgpack(&encode_cbpack(vec![
            vec![],
            vec!["the".to_string()],
            vec!["and".to_string(), "of".to_string()],
        ]))
        .unwrap();

        assert_eq!(
            payload.payload,
            PluginPayload::RankedWords {
                entries: vec![
                    RankedEntry { word: "and".to_string(), rank: 2 },
                    RankedEntry { word: "of".to_string(), rank: 3 },
                    RankedEntry { word: "the".to_string(), rank: 1 },
                ],
            }
        );
        assert_eq!(payload.report.input_records, 3);
    }

    fn encode_cbpack(buckets: Vec<Vec<String>>) -> Vec<u8> {
        let mut top_level = Vec::with_capacity(1 + buckets.len());
        top_level.push(Value::Map(vec![
            (Value::from("format"), Value::from("cB")),
            (Value::from("version"), Value::from(1)),
        ]));
        top_level.extend(
            buckets.into_iter().map(|bucket| {
                Value::Array(bucket.into_iter().map(Value::from).collect::<Vec<_>>())
            }),
        );

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        rmpv::encode::write_value(&mut encoder, &Value::Array(top_level)).unwrap();
        encoder.finish().unwrap()
    }
}
