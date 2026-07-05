use crate::{
    config::{LocaleProfile, TitleCaseOptions},
    util::unicode::{lowercase_with_locale, push_capitalized, push_lowercased},
};

pub(crate) fn lowercase_word(word: &str, locale: LocaleProfile) -> String {
    lowercase_with_locale(word, locale)
}

pub(crate) fn is_all_caps_acronym(word: &str) -> bool {
    let mut letters = 0_usize;
    for ch in word.chars() {
        if ch.is_alphabetic() {
            letters += 1;
            if !ch.is_uppercase() {
                return false;
            }
        }
    }
    letters > 1
}

pub(crate) fn is_dotted_abbreviation(word: &str) -> bool {
    word.contains('.')
        && word.chars().any(|ch| ch.is_alphabetic())
        && word.chars().all(|ch| ch.is_alphabetic() || ch == '.' || ch.is_ascii_digit())
}

pub(crate) fn has_internal_caps(word: &str) -> bool {
    let mut seen_initial = false;
    for ch in word.chars() {
        if ch.is_alphabetic() {
            if !seen_initial {
                seen_initial = true;
            } else if ch.is_uppercase() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
pub(crate) fn style_word(word: &str, capitalize: bool, options: &TitleCaseOptions<'_>) -> String {
    let mut out = String::with_capacity(word.len());
    push_styled(&mut out, word, capitalize, options);
    out
}

/// Styles a word directly into an output buffer, avoiding the per-word temporary
/// that `style_word` would allocate before being copied into the result.
pub(crate) fn push_styled(
    out: &mut String,
    word: &str,
    capitalize: bool,
    options: &TitleCaseOptions<'_>,
) {
    if is_all_caps_acronym(word) {
        out.push_str(word);
        return;
    }

    if is_dotted_abbreviation(word) {
        out.push_str(&style_dotted_abbreviation(word));
        return;
    }

    if !capitalize {
        push_lowercased(out, word, options.locale);
        return;
    }

    if options.preserve_existing_caps && has_internal_caps(word) {
        out.push_str(word);
        return;
    }

    push_capitalized(out, word, options.locale);
}

fn style_dotted_abbreviation(word: &str) -> String {
    const LOWERCASE_DOTTED_ABBREVIATIONS: &[&str] = &["a.m.", "e.g.", "i.e.", "p.m."];

    let lowered = word.to_lowercase();
    if LOWERCASE_DOTTED_ABBREVIATIONS.contains(&lowered.as_str()) {
        return lowered;
    }

    let segments: Vec<&str> = word.split('.').filter(|segment| !segment.is_empty()).collect();
    let is_initialism =
        !segments.is_empty() && segments.iter().all(|segment| segment.chars().count() == 1);

    if !is_initialism {
        return word.to_string();
    }

    let mut result = String::with_capacity(word.len());
    for ch in word.chars() {
        if ch.is_alphabetic() {
            for mapped in ch.to_uppercase() {
                result.push(mapped);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{has_internal_caps, is_all_caps_acronym, is_dotted_abbreviation, style_word};
    use crate::config::{LocaleProfile, TitleCaseOptions};
    use crate::util::unicode::capitalize_with_locale as capitalize_word;

    #[test]
    fn capitalizes_after_apostrophe() {
        assert_eq!(capitalize_word("o'neill", LocaleProfile::English), "O'Neill");
    }

    #[test]
    fn detects_internal_caps() {
        assert!(has_internal_caps("iPhone"));
        assert!(!has_internal_caps("Apple"));
    }

    #[test]
    fn detects_acronyms_and_dotted_abbreviations() {
        assert!(is_all_caps_acronym("NASA"));
        assert!(is_dotted_abbreviation("e.g."));
    }

    #[test]
    fn uppercases_initialism_style_abbreviations() {
        let options = TitleCaseOptions::default();
        assert_eq!(style_word("u.s.a.", true, &options), "U.S.A.");
        assert_eq!(style_word("e.g.", true, &options), "e.g.");
        assert_eq!(style_word("a.m.", true, &options), "a.m.");
        assert_eq!(style_word("p.m.", true, &options), "p.m.");
    }

    #[test]
    fn applies_locale_specific_casing() {
        assert_eq!(capitalize_word("ijsselmeer", LocaleProfile::Dutch), "IJsselmeer");
        assert_eq!(capitalize_word("istanbul", LocaleProfile::Turkish), "İstanbul");
    }
}
