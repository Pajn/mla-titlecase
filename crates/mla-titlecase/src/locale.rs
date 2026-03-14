use crate::config::{LocaleProfile, NameParticlePolicy};

pub(crate) fn resolve_locale_profile(tag: &str) -> LocaleProfile {
    let primary = tag.split(['-', '_']).next().unwrap_or(tag).trim().to_ascii_lowercase();

    match primary.as_str() {
        "de" => LocaleProfile::German,
        "es" => LocaleProfile::Spanish,
        "fr" => LocaleProfile::French,
        "it" => LocaleProfile::Italian,
        "nl" => LocaleProfile::Dutch,
        "tr" => LocaleProfile::Turkish,
        _ => LocaleProfile::English,
    }
}

pub(crate) const fn locale_tag(locale: LocaleProfile) -> &'static str {
    match locale {
        LocaleProfile::English => "en",
        LocaleProfile::Dutch => "nl",
        LocaleProfile::French => "fr",
        LocaleProfile::German => "de",
        LocaleProfile::Italian => "it",
        LocaleProfile::Spanish => "es",
        LocaleProfile::Turkish => "tr",
    }
}

pub(crate) const fn default_name_particle_policy(locale: LocaleProfile) -> NameParticlePolicy {
    match locale {
        LocaleProfile::English | LocaleProfile::Turkish => NameParticlePolicy::Disabled,
        LocaleProfile::Dutch
        | LocaleProfile::French
        | LocaleProfile::German
        | LocaleProfile::Italian
        | LocaleProfile::Spanish => NameParticlePolicy::Heuristic,
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{LocaleProfile, NameParticlePolicy};

    use super::{default_name_particle_policy, locale_tag, resolve_locale_profile};

    #[test]
    fn resolves_bcp47_tags() {
        assert_eq!(resolve_locale_profile("nl-NL"), LocaleProfile::Dutch);
        assert_eq!(resolve_locale_profile("de_DE"), LocaleProfile::German);
        assert_eq!(resolve_locale_profile("unknown"), LocaleProfile::English);
    }

    #[test]
    fn exposes_profile_defaults() {
        assert_eq!(locale_tag(LocaleProfile::French), "fr");
        assert_eq!(
            default_name_particle_policy(LocaleProfile::Spanish),
            NameParticlePolicy::Heuristic
        );
    }
}
