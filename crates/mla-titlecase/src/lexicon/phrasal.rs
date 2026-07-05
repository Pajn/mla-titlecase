//! Detection data for adverbial particles in phrasal verbs.
//!
//! MLA capitalizes adverbs, including the particle of a phrasal verb
//! ("Turn Off the Lights"), while lowercasing the same words used as
//! prepositions ("Walking down the Street").

/// Small words that can act as adverbial particles. Every entry must also be
/// on the small-word list; words like `out` that MLA never lowercases do not
/// need to be here.
const ADVERBIAL_PARTICLES: &[&str] = &[
    "about", "across", "along", "around", "behind", "by", "down", "in", "off", "on", "over",
    "past", "through", "under", "up",
];

pub(crate) fn is_adverbial_particle(word: &str) -> bool {
    ADVERBIAL_PARTICLES.contains(&word)
}

/// Curated phrasal verbs: every listed form of the verb, and the particles it
/// takes adverbially. Particles are omitted when the prepositional reading
/// dominates in titles (`look on the bright side`, `livin' on a prayer`).
const PHRASAL_VERBS: &[(&[&str], &[&str])] = &[
    (&["back", "backed", "backing", "backs"], &["down", "off", "up"]),
    (&["blew", "blow", "blowing", "blown", "blows"], &["off", "over", "up"]),
    (&["break", "breaking", "breaks", "broke", "broken"], &["down", "in", "off", "through", "up"]),
    (
        &["bring", "bringing", "brings", "brought"],
        &["about", "along", "down", "in", "on", "over", "up"],
    ),
    (&["burn", "burned", "burning", "burns", "burnt"], &["down", "off", "up"]),
    (&["call", "called", "calling", "calls"], &["in", "off", "up"]),
    (&["calm", "calmed", "calming", "calms"], &["down"]),
    (&["carried", "carries", "carry", "carrying"], &["on", "over"]),
    (&["catch", "catches", "catching", "caught"], &["on", "up"]),
    (&["cheer", "cheered", "cheering", "cheers"], &["on", "up"]),
    (&["check", "checked", "checking", "checks"], &["in", "off", "up"]),
    (&["clean", "cleaned", "cleaning", "cleans"], &["up"]),
    (&["close", "closed", "closes", "closing"], &["down", "in", "off", "up"]),
    (
        &["came", "come", "comes", "coming"],
        &["about", "along", "around", "by", "down", "in", "on", "over", "through", "up"],
    ),
    (&["cool", "cooled", "cooling", "cools"], &["down", "off"]),
    (&["count", "counted", "counting", "counts"], &["down", "off", "up"]),
    (&["cut", "cuts", "cutting"], &["down", "off", "up"]),
    (&["die", "died", "dies", "dying"], &["down", "off"]),
    (&["drop", "dropped", "dropping", "drops"], &["by", "in", "off"]),
    (
        &["fall", "fallen", "falling", "falls", "fell"],
        &["behind", "down", "off", "over", "through"],
    ),
    (&["fill", "filled", "filling", "fills"], &["in", "up"]),
    (
        &["get", "gets", "getting", "got", "gotten"],
        &["along", "around", "by", "down", "in", "off", "on", "over", "through", "up"],
    ),
    (&["gave", "give", "given", "gives", "giving"], &["in", "off", "over", "up"]),
    (&["go", "goes", "going", "gone", "went"], &["by", "off", "on", "over", "through", "under"]),
    (&["grew", "grow", "growing", "grown", "grows"], &["up"]),
    (&["hang", "hanging", "hangs", "hung"], &["about", "around", "in", "on", "up"]),
    (&["held", "hold", "holding", "holds"], &["down", "off", "on", "up"]),
    (&["hurried", "hurries", "hurry", "hurrying"], &["along", "up"]),
    (&["keep", "keeping", "keeps", "kept"], &["off", "on", "up"]),
    (&["knock", "knocked", "knocking", "knocks"], &["down", "off"]),
    (&["laid", "lay", "laying", "lays"], &["down", "off"]),
    (&["let", "lets", "letting"], &["down", "in", "off", "on", "up"]),
    (&["lie", "lies", "lying"], &["down"]),
    (&["lift", "lifted", "lifting", "lifts"], &["off", "up"]),
    (&["light", "lighting", "lights", "lit"], &["up"]),
    (&["line", "lined", "lines", "lining"], &["up"]),
    (&["look", "looked", "looking", "looks"], &["around", "over", "up"]),
    (&["made", "make", "makes", "making"], &["over", "up"]),
    (&["move", "moved", "moves", "moving"], &["along", "in", "on", "over", "up"]),
    (&["open", "opened", "opening", "opens"], &["up"]),
    (&["pass", "passed", "passes", "passing"], &["by", "on", "over", "up"]),
    (&["paid", "pay", "paying", "pays"], &["off", "up"]),
    (&["pick", "picked", "picking", "picks"], &["off", "up"]),
    (&["pull", "pulled", "pulling", "pulls"], &["in", "off", "over", "through", "up"]),
    (&["push", "pushed", "pushes", "pushing"], &["on", "through"]),
    (&["put", "puts", "putting"], &["down", "in", "off", "on", "up"]),
    (&["rise", "risen", "rises", "rising", "rose"], &["up"]),
    (&["roll", "rolled", "rolling", "rolls"], &["along", "around", "by", "on", "over", "up"]),
    (&["ran", "run", "running", "runs"], &["around", "down", "off", "up"]),
    (&["set", "sets", "setting"], &["down", "off", "up"]),
    (&["settle", "settled", "settles", "settling"], &["down", "in"]),
    (&["shake", "shaken", "shakes", "shaking", "shook"], &["off", "up"]),
    (&["show", "showed", "showing", "shown", "shows"], &["off", "up"]),
    (&["shut", "shuts", "shutting"], &["down", "in", "off", "up"]),
    (&["sat", "sit", "sits", "sitting"], &["down", "in", "up"]),
    (&["slow", "slowed", "slowing", "slows"], &["down"]),
    (&["speak", "speaking", "speaks", "spoke", "spoken"], &["up"]),
    (&["stand", "standing", "stands", "stood"], &["by", "down", "up"]),
    (&["step", "stepped", "stepping", "steps"], &["down", "in", "up"]),
    (&["straighten", "straightened", "straightening", "straightens"], &["up"]),
    (&["take", "taken", "takes", "taking", "took"], &["down", "in", "off", "on", "over", "up"]),
    (&["tear", "tearing", "tears", "tore", "torn"], &["down", "off", "up"]),
    (&["threw", "throw", "throwing", "thrown", "throws"], &["down", "in", "off", "up"]),
    (&["turn", "turned", "turning", "turns"], &["around", "down", "in", "off", "on", "over", "up"]),
    (&["wake", "wakes", "waking", "woke", "woken"], &["up"]),
    (&["warm", "warmed", "warming", "warms"], &["up"]),
    (&["wash", "washed", "washes", "washing"], &["off", "up"]),
    (&["wear", "wearing", "wears", "wore", "worn"], &["down", "off"]),
    (&["wind", "winding", "winds", "wound"], &["down", "up"]),
    (&["work", "worked", "working", "works"], &["off", "through", "up"]),
    (&["wrap", "wrapped", "wrapping", "wraps"], &["up"]),
    (&["write", "writes", "writing", "written", "wrote"], &["down", "in", "off", "up"]),
];

pub(crate) fn is_phrasal_verb_pair(verb: &str, particle: &str) -> bool {
    if lookup_pair(verb, particle) {
        return true;
    }
    // Song titles often drop the g: "Runnin' Down a Dream".
    verb.ends_with("in") && lookup_pair(&format!("{verb}g"), particle)
}

fn lookup_pair(verb: &str, particle: &str) -> bool {
    PHRASAL_VERBS
        .iter()
        .any(|(forms, particles)| forms.contains(&verb) && particles.contains(&particle))
}

#[cfg(test)]
mod tests {
    use super::{is_adverbial_particle, is_phrasal_verb_pair, ADVERBIAL_PARTICLES, PHRASAL_VERBS};
    use crate::lexicon::is_small_word;

    #[test]
    fn matches_known_pairs() {
        assert!(is_phrasal_verb_pair("turn", "off"));
        assert!(is_phrasal_verb_pair("burning", "down"));
        assert!(is_phrasal_verb_pair("woke", "up"));
        assert!(!is_phrasal_verb_pair("walking", "down"));
        assert!(!is_phrasal_verb_pair("living", "on"));
    }

    #[test]
    fn normalizes_dropped_g_gerunds() {
        assert!(is_phrasal_verb_pair("runnin", "down"));
        assert!(!is_phrasal_verb_pair("livin", "on"));
    }

    #[test]
    fn detects_adverbial_particles() {
        assert!(is_adverbial_particle("up"));
        assert!(!is_adverbial_particle("of"));
    }

    #[test]
    fn every_particle_is_a_small_word() {
        for particle in ADVERBIAL_PARTICLES {
            assert!(is_small_word(particle), "{particle} is not a small word");
        }
        for (forms, particles) in PHRASAL_VERBS {
            assert!(!forms.is_empty());
            for particle in *particles {
                assert!(is_adverbial_particle(particle), "{particle} is not an adverbial particle");
            }
        }
    }
}
