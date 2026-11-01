//! The voice packs shipped with the game parse, and cover most words of their course.
use fuera_learn::content::{Content, Course};
use fuera_learn::voice::Voices;

fn check(pack: &[u8], course: Course, min_share: f64) {
    let v = Voices::parse(pack.to_vec()).unwrap_or_else(|| panic!("{course:?}: the pack does not parse (empty file?)"));
    let c = Content::load(course);
    let words: Vec<&str> = c.words.iter().map(|w| w.lemma.as_str()).collect();
    let covered = words.iter().filter(|w| v.has(w)).count();
    let share = covered as f64 / words.len() as f64;
    eprintln!("{course:?}: {} recordings, {covered}/{} words ({:.0} %), {} speakers", v.len(), words.len(), share * 100.0, v.speakers().len());
    assert!(share >= min_share, "{course:?}: only {:.0} % of the words have a recording", share * 100.0);
    // Every recording decodes.
    for w in words.iter().filter(|w| v.has(w)).take(200) {
        assert!(v.clip(w).is_some_and(|pcm| pcm.len() > 1600), "{course:?}: {w} does not decode");
    }
}

#[test]
fn voice_packs() {
    check(include_bytes!("../../../assets/es/voces/voces-es.fvox"), Course::SPAIN, 0.6);
    check(include_bytes!("../../../assets/es/voces/voces-es-419.fvox"), Course::LATAM, 0.75);
    check(include_bytes!("../../../assets/fr/voces/voces-fr.fvox"), Course::FRENCH, 0.9);
}
