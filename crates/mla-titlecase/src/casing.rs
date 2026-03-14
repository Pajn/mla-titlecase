use crate::config::{LocaleProfile, TitleCaseOptions};

pub(crate) fn lowercase_word(word: &str, _locale: LocaleProfile) -> String {
    word.to_lowercase()
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

pub(crate) fn style_word(word: &str, capitalize: bool, options: &TitleCaseOptions<'_>) -> String {
    if is_all_caps_acronym(word) {
        return word.to_string();
    }

    if is_dotted_abbreviation(word) {
        return style_dotted_abbreviation(word);
    }

    if !capitalize {
        return lowercase_word(word, options.locale);
    }

    if options.preserve_existing_caps && has_internal_caps(word) {
        return word.to_string();
    }

    capitalize_word(word, options.locale)
}

fn style_dotted_abbreviation(word: &str) -> String {
    const LOWERCASE_DOTTED_ABBREVIATIONS: &[&str] = &["e.g.", "i.e."];

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

pub(crate) fn capitalize_word(word: &str, _locale: LocaleProfile) -> String {
    let mut result = String::with_capacity(word.len());
    let mut make_upper = true;

    for ch in word.chars() {
        if ch.is_alphabetic() {
            if make_upper {
                for mapped in ch.to_uppercase() {
                    result.push(mapped);
                }
                make_upper = false;
            } else {
                for mapped in ch.to_lowercase() {
                    result.push(mapped);
                }
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

#[cfg(test)]
mod tests {
    use super::{
        capitalize_word, has_internal_caps, is_all_caps_acronym, is_dotted_abbreviation, style_word,
    };
    use crate::config::{LocaleProfile, TitleCaseOptions};

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
    }
}
