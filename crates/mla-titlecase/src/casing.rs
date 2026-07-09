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
    // The first alphanumeric character occupies the capitalized position, so
    // a capital after a leading digit is internal ("3D", "4K") and preserved
    // like any other intentional mixed casing.
    let mut seen_initial = false;
    for ch in word.chars() {
        if seen_initial {
            if ch.is_uppercase() {
                return true;
            }
        } else if ch.is_alphanumeric() {
            seen_initial = true;
        }
    }
    false
}

#[cfg(test)]
pub(crate) fn style_word(
    word: &str,
    capitalize: bool,
    force_capitalize: bool,
    options: &TitleCaseOptions<'_>,
) -> String {
    let mut out = String::with_capacity(word.len());
    let _ = push_styled(&mut out, word, capitalize, force_capitalize, options);
    out
}

/// Which branch [`push_styled`] took, so callers building a rich analysis can
/// attribute the casing decision without repeating the shape checks.
#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StyleOutcome {
    AcronymPreserved,
    DottedAbbreviation,
    MixedCasePreserved,
    Lowercased,
    Capitalized,
}

/// Styles a word directly into an output buffer, avoiding the per-word temporary
/// that `style_word` would allocate before being copied into the result. The
/// returned [`StyleOutcome`] reports which rule applied (free for callers that
/// ignore it).
///
/// `force_capitalize` marks a mandatory-capitalize position (first or last word
/// of the title or a subtitle segment), where MLA's first-and-last-word rule
/// outranks the lowercase dotted-abbreviation list.
pub(crate) fn push_styled(
    out: &mut String,
    word: &str,
    capitalize: bool,
    force_capitalize: bool,
    options: &TitleCaseOptions<'_>,
) -> StyleOutcome {
    if is_all_caps_acronym(word) {
        out.push_str(word);
        return StyleOutcome::AcronymPreserved;
    }

    if is_dotted_abbreviation(word) {
        push_dotted_abbreviation(out, word, force_capitalize);
        return StyleOutcome::DottedAbbreviation;
    }

    if !capitalize {
        push_lowercased(out, word, options.locale);
        return StyleOutcome::Lowercased;
    }

    if options.preserve_existing_caps && has_internal_caps(word) {
        out.push_str(word);
        return StyleOutcome::MixedCasePreserved;
    }

    push_capitalized(out, word, options.locale);
    StyleOutcome::Capitalized
}

fn push_dotted_abbreviation(out: &mut String, word: &str, force_capitalize: bool) {
    // Latin abbreviations stay lowercase; at a mandatory-capitalize position
    // only their first letter rises ("E.g. a Case Study"), matching MLA per
    // titlecaseconverter.com.
    const LATIN_ABBREVIATIONS: &[&str] = &["e.g.", "i.e."];
    // Meridiem markers also stay lowercase mid-title, but they are ordinary
    // initialisms, so a mandatory position restores full caps ("A.M. Radio
    // Days" rather than "A.m."), again per titlecaseconverter.com.
    const MERIDIEM_ABBREVIATIONS: &[&str] = &["a.m.", "p.m."];

    let lowered = word.to_lowercase();
    if LATIN_ABBREVIATIONS.contains(&lowered.as_str()) {
        if force_capitalize {
            let mut chars = lowered.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        } else {
            out.push_str(&lowered);
        }
        return;
    }

    if MERIDIEM_ABBREVIATIONS.contains(&lowered.as_str()) && !force_capitalize {
        out.push_str(&lowered);
        return;
    }

    let segments: Vec<&str> = word.split('.').filter(|segment| !segment.is_empty()).collect();
    let is_initialism =
        !segments.is_empty() && segments.iter().all(|segment| segment.chars().count() == 1);

    if !is_initialism {
        // Irregular dotted words ("example.com") are kept verbatim even at a
        // mandatory position; their casing is not ours to guess.
        out.push_str(word);
        return;
    }

    for ch in word.chars() {
        if ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
    }
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
        // A leading digit occupies the capitalized position, so a capital
        // after it counts as internal ("3D", "4K"); ordinals do not.
        assert!(has_internal_caps("3D"));
        assert!(has_internal_caps("4K"));
        assert!(!has_internal_caps("42nd"));
        assert!(!has_internal_caps("3d"));
    }

    #[test]
    fn detects_acronyms_and_dotted_abbreviations() {
        assert!(is_all_caps_acronym("NASA"));
        assert!(is_dotted_abbreviation("e.g."));
    }

    #[test]
    fn uppercases_initialism_style_abbreviations() {
        let options = TitleCaseOptions::default();
        assert_eq!(style_word("u.s.a.", true, false, &options), "U.S.A.");
        assert_eq!(style_word("e.g.", true, false, &options), "e.g.");
        assert_eq!(style_word("a.m.", true, false, &options), "a.m.");
        assert_eq!(style_word("p.m.", true, false, &options), "p.m.");
    }

    #[test]
    fn mandatory_positions_capitalize_lowercase_abbreviations() {
        let options = TitleCaseOptions::default();
        // Latin abbreviations raise only their first letter.
        assert_eq!(style_word("e.g.", true, true, &options), "E.g.");
        assert_eq!(style_word("i.e.", true, true, &options), "I.e.");
        // Meridiem markers are ordinary initialisms and restore full caps.
        assert_eq!(style_word("a.m.", true, true, &options), "A.M.");
        assert_eq!(style_word("p.m.", true, true, &options), "P.M.");
        // Irregular dotted words are verbatim regardless of position.
        assert_eq!(style_word("example.com", true, true, &options), "example.com");
    }

    #[test]
    fn applies_locale_specific_casing() {
        assert_eq!(capitalize_word("ijsselmeer", LocaleProfile::Dutch), "IJsselmeer");
        assert_eq!(capitalize_word("istanbul", LocaleProfile::Turkish), "İstanbul");
    }
}
