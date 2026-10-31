//! Checks the conjugator against Wiktionary conjugation tables (via kaikki.org,
//! CC BY-SA 4.0) for ~500 frequent and irregular verbs; see `data/README.md`.

use std::collections::{BTreeMap, BTreeSet};

use fuera_learn::conj::{Person, Tense, conjugate};

const FIXTURE: &str = include_str!("data/forms.tsv");

fn tense(name: &str) -> Tense {
    match name {
        "Present" => Tense::Present,
        "Preterite" => Tense::Preterite,
        "Imperfect" => Tense::Imperfect,
        "Future" => Tense::Future,
        "Conditional" => Tense::Conditional,
        "PresentSubjunctive" => Tense::PresentSubjunctive,
        "ImperfectSubjunctive" => Tense::ImperfectSubjunctive,
        other => panic!("unknown tense {other}"),
    }
}

fn person(name: &str) -> Person {
    match name {
        "Yo" => Person::Yo,
        "Tu" => Person::Tu,
        "El" => Person::El,
        "Nosotros" => Person::Nosotros,
        "Vosotros" => Person::Vosotros,
        "Ellos" => Person::Ellos,
        other => panic!("unknown person {other}"),
    }
}

struct Report {
    verbs: usize,
    slots: usize,
    rows: usize,
    rows_ok: usize,
    failures: Vec<String>,
}

/// Compares every `infinitive, tense, person, form` row of `fixture` with
/// [`conjugate`]; a slot passes when the output is one of its listed forms.
fn check(fixture: &str) -> Report {
    // (infinitive, tense, person) -> accepted forms (Wiktionary may list several)
    let mut slots: BTreeMap<(&str, &str, &str), Vec<&str>> = BTreeMap::new();
    let mut rows = 0usize;
    for line in fixture
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("infinitive\t"))
    {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "bad fixture line: {line:?}");
        slots
            .entry((cols[0], cols[1], cols[2]))
            .or_default()
            .push(cols[3]);
        rows += 1;
    }
    let verbs = slots.keys().map(|k| k.0).collect::<BTreeSet<_>>().len();
    let mut failures = Vec::new();
    let mut rows_ok = 0usize;
    for (&(inf, t, p), forms) in &slots {
        let got = conjugate(inf, tense(t), person(p));
        if got.as_deref().is_some_and(|g| forms.contains(&g)) {
            rows_ok += forms.len();
        } else {
            failures.push(format!(
                "{inf} {t} {p}: got {got:?}, expected {}",
                forms.join(" | ")
            ));
        }
    }
    let report = Report {
        verbs,
        slots: slots.len(),
        rows,
        rows_ok,
        failures,
    };
    eprintln!(
        "fixture: {} verbs, {} slots, {} rows checked, {} rows passed, {} slots failed",
        report.verbs,
        report.slots,
        report.rows,
        report.rows_ok,
        report.failures.len()
    );
    report
}

#[test]
fn conjugator_matches_wiktionary() {
    let report = check(FIXTURE);
    assert!(
        report.verbs >= 400,
        "fixture too small: {} verbs",
        report.verbs
    );
    assert!(
        report.failures.is_empty(),
        "{} of {} slots differ from Wiktionary:\n{}",
        report.failures.len(),
        report.slots,
        report
            .failures
            .iter()
            .take(200)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Exploratory run over a larger fixture in the same format, e.g. every verb
/// of the kaikki dump (`build_forms.py` with a bigger `TOP_N`):
/// `FUERA_CONJ_FIXTURE=/path/forms_all.tsv cargo test -p fuera-learn -- --ignored --nocapture`.
/// Rare and defective verbs are expected to differ; it only requires 99% of slots.
#[test]
#[ignore = "needs FUERA_CONJ_FIXTURE pointing to a large kaikki-derived fixture"]
fn conjugator_matches_large_fixture() {
    let path = std::env::var("FUERA_CONJ_FIXTURE").expect("set FUERA_CONJ_FIXTURE");
    let text = std::fs::read_to_string(path).expect("readable fixture");
    let report = check(&text);
    for f in report.failures.iter().take(300) {
        eprintln!("{f}");
    }
    let ok = report.slots - report.failures.len();
    assert!(
        ok * 100 >= report.slots * 99,
        "only {ok} of {} slots match",
        report.slots
    );
}
