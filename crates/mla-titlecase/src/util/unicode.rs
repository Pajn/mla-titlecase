//! Lightweight Unicode- and locale-related helpers.

use crate::config::LocaleProfile;

/// Returns whether the string has at least one cased character.
#[must_use]
pub fn has_cased_letter(value: &str) -> bool {
    value.chars().any(|ch| ch.is_uppercase() || ch.is_lowercase())
}

/// Lowercases a string using locale-aware handling for the supported profiles.
#[must_use]
pub fn lowercase_with_locale(value: &str, locale: LocaleProfile) -> String {
    match locale {
        LocaleProfile::Turkish => value
            .chars()
            .flat_map(|ch| match ch {
                'I' => "ı".chars().collect::<Vec<_>>(),
                '\u{0130}' => "i".chars().collect::<Vec<_>>(),
                _ => ch.to_lowercase().collect(),
            })
            .collect(),
        _ => value.to_lowercase(),
    }
}

/// Title-capitalizes a token using locale-aware handling for the supported profiles.
#[must_use]
pub fn capitalize_with_locale(value: &str, locale: LocaleProfile) -> String {
    if value.is_empty() {
        return String::new();
    }

    if locale == LocaleProfile::Dutch {
        let lowered = lowercase_with_locale(value, locale);
        if let Some(suffix) = lowered.strip_prefix("ij") {
            return format!("IJ{suffix}");
        }
    }

    let lowered = lowercase_with_locale(value, locale);
    let mut result = String::with_capacity(lowered.len());
    let mut make_upper = true;

    for ch in lowered.chars() {
        if ch.is_alphabetic() {
            if make_upper {
                append_uppercase(&mut result, ch, locale);
                make_upper = false;
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
            if matches!(ch, '\'' | '\u{2019}') {
                make_upper = true;
            }
        }
    }

    result
}

fn append_uppercase(output: &mut String, ch: char, locale: LocaleProfile) {
    match (locale, ch) {
        (LocaleProfile::Turkish, 'i') => output.push('\u{0130}'),
        (LocaleProfile::Turkish, 'ı') => output.push('I'),
        _ => {
            for mapped in ch.to_uppercase() {
                output.push(mapped);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::LocaleProfile;

    use super::{capitalize_with_locale, has_cased_letter, lowercase_with_locale};

    #[test]
    fn detects_cased_letters() {
        assert!(has_cased_letter("Rust"));
        assert!(!has_cased_letter("123"));
    }

    #[test]
    fn handles_dutch_and_turkish_casing() {
        assert_eq!(capitalize_with_locale("ijsselmeer", LocaleProfile::Dutch), "IJsselmeer");
        assert_eq!(lowercase_with_locale("IĞDIR", LocaleProfile::Turkish), "ığdır");
        assert_eq!(capitalize_with_locale("istanbul", LocaleProfile::Turkish), "İstanbul");
    }
}
