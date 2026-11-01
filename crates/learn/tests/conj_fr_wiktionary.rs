//! Checks the French conjugator against English Wiktionary's conjugation tables (via kaikki.org, CC BY-SA 4.0) for
//! the most frequent verbs; see `data/README.md`. A slot passes when the output is one of the listed forms
//! (Wiktionary gives both *paie* and *paye*, *céderai* and *cèderai*).

use fuera_learn::conj_fr::{Aux, Person, Tense, auxiliary, conjugate, participle};

const FIXTURE: &str = include_str!("data/forms_fr.tsv");

fn tense(name: &str) -> Option<Tense> {
    Some(match name {
        "present" => Tense::Present,
        "imparfait" => Tense::Imparfait,
        "futur" => Tense::Futur,
        "conditionnel" => Tense::Conditionnel,
        "subjonctif" => Tense::Subjonctif,
        _ => return None,
    })
}

fn person(name: &str) -> Person {
    match name {
        "je" => Person::Je,
        "tu" => Person::Tu,
        "il" => Person::Il,
        "nous" => Person::Nous,
        "vous" => Person::Vous,
        _ => Person::Ils,
    }
}

#[test]
fn french_conjugator_matches_wiktionary() {
    let mut failures = Vec::new();
    let mut slots = 0;
    for line in FIXTURE.lines().skip(1) {
        let c: Vec<&str> = line.split('\t').collect();
        let (inf, t, p, forms) = (c[0], c[1], c[2], c[3]);
        // "-": the verb has no such form (impersonal, defective): the conjugator must give none.
        let ok: Vec<&str> = forms.split(',').map(|f| if f == "-" { "∅" } else { f }).collect();
        slots += 1;
        let got = match t {
            "pp" => participle(inf),
            "aux" => Some(match auxiliary(inf) {
                Aux::Avoir => "avoir".to_string(),
                Aux::Etre => "être".to_string(),
            }),
            _ => match tense(t) {
                Some(tt) => conjugate(inf, tt, person(p)),
                None => continue,
            },
        };
        let got = got.unwrap_or_else(|| "∅".into());
        // Pronominal-only verbs (s'envoler) are listed with their clitic; the fixture's infinitive has none.
        let strip = |f: &str| -> String {
            for c in ["me ", "te ", "se ", "nous ", "vous ", "m'", "t'", "s'"] {
                if let Some(r) = f.strip_prefix(c) {
                    return r.to_string();
                }
            }
            f.to_string()
        };
        if !ok.contains(&got.as_str()) && !ok.iter().any(|f| strip(f) == got) {
            failures.push(format!("{inf} {t} {p}: got {got}, Wiktionary {forms}"));
        }
    }
    let rate = 1.0 - failures.len() as f64 / slots as f64;
    for f in failures.iter() {
        eprintln!("{f}");
    }
    eprintln!("{} slots, {} failures ({:.2} % right)", slots, failures.len(), rate * 100.0);
    assert!(failures.is_empty(), "{} slots differ from Wiktionary", failures.len());
}
