const PARTICLES: &[&str] = &["da", "de", "del", "der", "di", "la", "van", "von"];

pub(crate) fn contains(word: &str) -> bool {
    PARTICLES.contains(&word)
}
