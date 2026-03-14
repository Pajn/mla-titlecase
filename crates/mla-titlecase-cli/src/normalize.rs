use mla_titlecase::{util::normalize::normalized_unique_sorted, PluginPayload, RankedEntry};
use serde_json::Value;

use crate::{error::CliError, error::Result};

pub(crate) fn normalized_word_set(
    words: impl IntoIterator<Item = impl AsRef<str>>,
) -> PluginPayload {
    PluginPayload::WordSet {
        words: normalized_unique_sorted(words.into_iter().map(|word| word.as_ref().to_string())),
    }
}

pub(crate) fn parse_stopwords_json(raw: &str) -> Result<PluginPayload> {
    let value: Value = serde_json::from_str(raw)?;
    let words: Vec<String> = match value {
        Value::Array(items) => {
            items.into_iter().map(string_value).collect::<std::result::Result<Vec<_>, _>>()?
        }
        Value::Object(map) => map
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
    Ok(normalized_word_set(words))
}

pub(crate) fn parse_scowl_word_list(raw: &str) -> PluginPayload {
    let words = raw.lines().filter_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }
        let candidate = trimmed.split_whitespace().next().unwrap_or(trimmed);
        Some(candidate.to_string())
    });
    normalized_word_set(words)
}

pub(crate) fn parse_wordfreq_tsv(raw: &str) -> Result<PluginPayload> {
    let mut entries = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut fields = trimmed.split_whitespace();
        let word = fields.next().ok_or_else(|| CliError::UnsupportedInput {
            path: std::path::PathBuf::from("<memory>"),
            message: "expected a word column".to_string(),
        })?;
        let rank = fields
            .next()
            .ok_or_else(|| CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: "expected a rank column".to_string(),
            })?
            .parse::<u64>()
            .map_err(|error| CliError::UnsupportedInput {
                path: std::path::PathBuf::from("<memory>"),
                message: format!("invalid rank: {error}"),
            })?;
        entries.push(RankedEntry { word: word.to_string(), rank });
    }
    entries.sort_by(|left, right| left.word.cmp(&right.word));
    Ok(PluginPayload::RankedWords { entries })
}

fn string_value(value: Value) -> Result<String> {
    value.as_str().map(ToOwned::to_owned).ok_or_else(|| CliError::UnsupportedInput {
        path: std::path::PathBuf::from("<memory>"),
        message: "expected a string value".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use mla_titlecase::{PluginPayload, RankedEntry};

    use super::{parse_scowl_word_list, parse_stopwords_json, parse_wordfreq_tsv};

    #[test]
    fn parses_stopword_json() {
        let payload = parse_stopwords_json("[\"and\", \"the\"]").unwrap();
        assert_eq!(
            payload,
            PluginPayload::WordSet { words: vec!["and".to_string(), "the".to_string()] }
        );
    }

    #[test]
    fn parses_scowl_like_lists() {
        let payload = parse_scowl_word_list("# comment\nalpha\nBeta\n");
        assert_eq!(
            payload,
            PluginPayload::WordSet { words: vec!["alpha".to_string(), "beta".to_string()] }
        );
    }

    #[test]
    fn parses_wordfreq_tsv() {
        let payload = parse_wordfreq_tsv("common 1\nrare 9\n").unwrap();
        assert_eq!(
            payload,
            PluginPayload::RankedWords {
                entries: vec![
                    RankedEntry { word: "common".to_string(), rank: 1 },
                    RankedEntry { word: "rare".to_string(), rank: 9 },
                ],
            }
        );
    }
}
