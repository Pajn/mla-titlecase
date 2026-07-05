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

    for (index, ch) in lowered.char_indices() {
        if make_upper && ch.is_alphabetic() {
            append_uppercase(&mut result, ch, locale);
            make_upper = false;
            continue;
        }

        result.push(ch);
        if ch.is_alphanumeric() {
            // A leading digit occupies the capitalized position: `42nd`, not `42Nd`.
            make_upper = false;
        } else if matches!(ch, '\'' | '\u{2019}')
            && capitalizes_after_apostrophe(&lowered, index, ch)
        {
            make_upper = true;
        }
    }

    result
}

/// Contraction endings that keep the letter after an apostrophe lowercase.
const CONTRACTION_SUFFIXES: &[&str] =
    &["all", "cause", "d", "em", "er", "ll", "m", "n", "re", "s", "t", "til", "ve"];

/// An apostrophe recapitalizes only after a single-letter prefix (`O'Neill`,
/// `D'Angelo`), and never before a contraction ending (`don't`, `y'all`).
fn capitalizes_after_apostrophe(lowered: &str, index: usize, apostrophe: char) -> bool {
    let mut preceding = lowered[..index].chars().rev();
    if !preceding.next().is_some_and(char::is_alphabetic) {
        return false;
    }
    if preceding.next().is_some_and(char::is_alphabetic) {
        return false;
    }

    let following: String = lowered[index + apostrophe.len_utf8()..]
        .chars()
        .take_while(|ch| ch.is_alphanumeric())
        .collect();
    !CONTRACTION_SUFFIXES.contains(&following.as_str())
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
    fn capitalizes_after_single_letter_apostrophe_prefixes() {
        assert_eq!(capitalize_with_locale("o'neill", LocaleProfile::English), "O'Neill");
        assert_eq!(capitalize_with_locale("d'angelo", LocaleProfile::English), "D'Angelo");
        assert_eq!(capitalize_with_locale("rock'n'roll", LocaleProfile::English), "Rock'n'Roll");
    }

    #[test]
    fn keeps_contraction_endings_lowercase() {
        assert_eq!(capitalize_with_locale("don't", LocaleProfile::English), "Don't");
        assert_eq!(capitalize_with_locale("it's", LocaleProfile::English), "It's");
        assert_eq!(capitalize_with_locale("wasn't", LocaleProfile::English), "Wasn't");
        assert_eq!(capitalize_with_locale("y'all", LocaleProfile::English), "Y'all");
        assert_eq!(capitalize_with_locale("o'er", LocaleProfile::English), "O'er");
    }

    #[test]
    fn leaves_digit_led_words_lowercase() {
        assert_eq!(capitalize_with_locale("42nd", LocaleProfile::English), "42nd");
        assert_eq!(capitalize_with_locale("3rd", LocaleProfile::English), "3rd");
    }

    #[test]
    fn handles_dutch_and_turkish_casing() {
        assert_eq!(capitalize_with_locale("ijsselmeer", LocaleProfile::Dutch), "IJsselmeer");
        assert_eq!(lowercase_with_locale("IĞDIR", LocaleProfile::Turkish), "ığdır");
        assert_eq!(capitalize_with_locale("istanbul", LocaleProfile::Turkish), "İstanbul");
    }
}
