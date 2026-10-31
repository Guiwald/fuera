//! The Spanish content shipped with the game (`assets/es/`, CC BY-SA sources checked against Wiktionary): words,
//! grammar choices, false friends, pseudo-words for the placement test.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Band {
    A1,
    A2,
    B1,
    B2,
}

impl Band {
    pub const ALL: [Band; 4] = [Band::A1, Band::A2, Band::B1, Band::B2];

    pub fn parse(s: &str) -> Band {
        match s.trim() {
            "A1" => Band::A1,
            "A2" => Band::A2,
            "B1" => Band::B1,
            _ => Band::B2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Band::A1 => "A1",
            Band::A2 => "A2",
            Band::B1 => "B1",
            Band::B2 => "B2",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// The learner's own language, used for meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Fr,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    M,
    F,
    /// Same form for both (*el/la estudiante*).
    Mf,
}

#[derive(Clone, Debug)]
pub struct Word {
    pub es: String,
    /// Part of speech: n, v, adj, adv, prep, conj, pron, det, num, interj, expr.
    pub pos: String,
    pub gender: Option<Gender>,
    pub fr: String,
    pub en: String,
    pub band: Band,
    pub theme: String,
    pub pic: Option<String>,
    pub note: String,
}

impl Word {
    pub fn id(&self) -> String {
        format!("w:{}|{}", self.es, self.pos)
    }

    pub fn meaning(&self, base: Base) -> &str {
        match base {
            Base::Fr => &self.fr,
            Base::En => &self.en,
        }
    }

    /// The noun with its article (*el perro*, *el agua*, *la mano*).
    pub fn with_article(&self) -> String {
        if self.pos != "n" {
            return self.es.clone();
        }
        let el = match self.gender {
            Some(Gender::M) | Some(Gender::Mf) => "el",
            Some(Gender::F) => {
                if self.note.contains("el agua") || self.note.starts_with("el ") {
                    "el"
                } else {
                    "la"
                }
            }
            None => return self.es.clone(),
        };
        format!("{el} {}", self.es)
    }
}

/// A two-way choice in context: ser/estar, por/para, preterite/imperfect, subjunctive/indicative.
#[derive(Clone, Debug)]
pub struct Choice {
    /// Phrase or sentence (with `___` where the answer goes, for sentences).
    pub prompt: String,
    pub answer: String,
    /// The wrong alternative (preterite/imperfect, subjunctive/indicative). Empty for ser/estar, por/para.
    pub other: String,
    pub fr: String,
    pub en: String,
    pub band: Band,
    pub why: String,
}

impl Choice {
    pub fn translation(&self, base: Base) -> &str {
        match base {
            Base::Fr => &self.fr,
            Base::En => &self.en,
        }
    }

    /// The sentence with the gap filled.
    pub fn filled(&self) -> String {
        let s = match self.prompt.rfind(" (") {
            Some(i) if self.prompt.ends_with(')') => &self.prompt[..i],
            _ => &self.prompt,
        };
        if s.contains("___") { s.replace("___", &self.answer) } else { format!("{} {}", self.answer, s) }
    }

    /// The infinitive in brackets at the end of a sentence (`(ir)`), if any.
    pub fn infinitive(&self) -> Option<&str> {
        let s = self.prompt.trim_end();
        if !s.ends_with(')') {
            return None;
        }
        let i = s.rfind('(')?;
        Some(&s[i + 1..s.len() - 1])
    }

    /// The sentence without the bracketed infinitive.
    pub fn sentence(&self) -> &str {
        match self.prompt.rfind(" (") {
            Some(i) if self.prompt.ends_with(')') => &self.prompt[..i],
            _ => &self.prompt,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FalseFriend {
    pub es: String,
    pub fr_true: String,
    pub fr_trap: String,
    pub en_true: String,
    pub en_trap: String,
    pub band: Band,
    pub note: String,
}

impl FalseFriend {
    pub fn truth(&self, base: Base) -> &str {
        match base {
            Base::Fr => &self.fr_true,
            Base::En => &self.en_true,
        }
    }

    pub fn trap(&self, base: Base) -> &str {
        match base {
            Base::Fr => &self.fr_trap,
            Base::En => &self.en_trap,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Content {
    pub words: Vec<Word>,
    /// Nouns whose gender surprises learners.
    pub gender_traps: Vec<Word>,
    pub ser_estar: Vec<Choice>,
    pub por_para: Vec<Choice>,
    pub pret_imp: Vec<Choice>,
    pub subjunctive: Vec<Choice>,
    pub false_friends: Vec<FalseFriend>,
    pub pseudo: Vec<String>,
}

static LEX_A: &str = include_str!("../../../assets/es/lexicon_a.tsv");
static LEX_B: &str = include_str!("../../../assets/es/lexicon_b.tsv");
static GENERO: &str = include_str!("../../../assets/es/grammar/genero.tsv");
static SER_ESTAR: &str = include_str!("../../../assets/es/grammar/ser_estar.tsv");
static POR_PARA: &str = include_str!("../../../assets/es/grammar/por_para.tsv");
static PRET_IMP: &str = include_str!("../../../assets/es/grammar/pret_imp.tsv");
static SUBJ: &str = include_str!("../../../assets/es/grammar/subjuntivo.tsv");
static FALSOS: &str = include_str!("../../../assets/es/grammar/falsos_amigos.tsv");
static PSEUDO: &str = include_str!("../../../assets/es/grammar/pseudo.tsv");

fn rows(text: &str) -> impl Iterator<Item = Vec<&str>> {
    text.lines().skip(1).filter(|l| !l.trim().is_empty()).map(|l| l.split('\t').collect())
}

fn col<'a>(r: &[&'a str], i: usize) -> &'a str {
    r.get(i).copied().unwrap_or("").trim()
}

fn gender(s: &str) -> Option<Gender> {
    match s {
        "m" => Some(Gender::M),
        "f" => Some(Gender::F),
        "mf" => Some(Gender::Mf),
        _ => None,
    }
}

fn words(text: &str) -> Vec<Word> {
    rows(text)
        .map(|r| Word {
            es: col(&r, 0).to_string(),
            pos: col(&r, 1).to_string(),
            gender: gender(col(&r, 2)),
            fr: col(&r, 3).to_string(),
            en: col(&r, 4).to_string(),
            band: Band::parse(col(&r, 5)),
            theme: col(&r, 6).to_string(),
            pic: Some(col(&r, 7).to_string()).filter(|s| !s.is_empty()),
            note: col(&r, 8).to_string(),
        })
        .filter(|w| !w.es.is_empty())
        .collect()
}

fn choices(text: &str, with_other: bool) -> Vec<Choice> {
    rows(text)
        .map(|r| {
            let o = if with_other { 1 } else { 0 };
            Choice {
                prompt: col(&r, 0).to_string(),
                answer: col(&r, 1).to_string(),
                other: if with_other { col(&r, 2).to_string() } else { String::new() },
                fr: col(&r, 2 + o).to_string(),
                en: col(&r, 3 + o).to_string(),
                band: Band::parse(col(&r, 4 + o)),
                why: col(&r, 5 + o).to_string(),
            }
        })
        .filter(|c| !c.prompt.is_empty())
        .collect()
}

impl Content {
    pub fn load() -> Content {
        let mut words_all = words(LEX_A);
        words_all.extend(words(LEX_B));
        let gender_traps = rows(GENERO)
            .map(|r| Word {
                es: col(&r, 0).to_string(),
                pos: "n".into(),
                gender: gender(col(&r, 1)),
                fr: col(&r, 2).to_string(),
                en: col(&r, 3).to_string(),
                band: Band::parse(col(&r, 4)),
                theme: String::new(),
                pic: None,
                note: col(&r, 5).to_string(),
            })
            .collect();
        let false_friends = rows(FALSOS)
            .map(|r| FalseFriend {
                es: col(&r, 0).to_string(),
                fr_true: col(&r, 1).to_string(),
                fr_trap: col(&r, 2).to_string(),
                en_true: col(&r, 3).to_string(),
                en_trap: col(&r, 4).to_string(),
                band: Band::parse(col(&r, 5)),
                note: col(&r, 6).to_string(),
            })
            .collect();
        Content {
            words: words_all,
            gender_traps,
            ser_estar: choices(SER_ESTAR, false),
            por_para: choices(POR_PARA, false),
            pret_imp: choices(PRET_IMP, true),
            subjunctive: choices(SUBJ, true),
            false_friends,
            pseudo: rows(PSEUDO).map(|r| col(&r, 0).to_string()).filter(|s| !s.is_empty()).collect(),
        }
    }

    pub fn word(&self, id: &str) -> Option<&Word> {
        let rest = id.strip_prefix("w:")?;
        let (es, pos) = rest.split_once('|')?;
        self.words.iter().find(|w| w.es == es && w.pos == pos)
    }
}
