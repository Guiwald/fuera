//! The content of each course, shipped with the game and checked against Wiktionary: words, grammar choices, false
//! friends, pseudo-words for the placement test.
//!
//! * Spanish (`assets/es/`, CC BY-SA): Spain's Spanish, with a Latin-American variant built from it
//!   (`assets/es/latam.tsv`: words replaced or dropped, sentences that only work in Spain left out).
//! * French (`assets/fr/`, CC BY-SA): France's French, traditional spelling, meanings in Spanish and English.
//!
//! Files are tab-separated with a header line; columns are found by their header name, so the Spanish and French
//! files can name their language columns after the language (`es`, `fr`, `en`).

use std::collections::{HashMap, HashSet};

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

/// The language being learnt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Es,
    Fr,
}

impl Target {
    pub const ALL: [Target; 2] = [Target::Es, Target::Fr];

    pub fn code(self) -> &'static str {
        match self {
            Target::Es => "es",
            Target::Fr => "fr",
        }
    }

    pub fn from_code(s: &str) -> Option<Target> {
        match s {
            "es" => Some(Target::Es),
            "fr" => Some(Target::Fr),
            _ => None,
        }
    }

    /// Languages the meanings can be given in (never the target itself).
    pub fn bases(self) -> [Base; 2] {
        match self {
            Target::Es => [Base::Fr, Base::En],
            Target::Fr => [Base::Es, Base::En],
        }
    }
}

/// A course: the language learnt and, for Spanish, the variety (Spain or Latin America).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Course {
    pub target: Target,
    /// Latin-American Spanish (*ustedes* without *vosotros*, *carro*, *computadora*…). Ignored for French.
    pub latam: bool,
}

impl Course {
    pub const SPAIN: Course = Course { target: Target::Es, latam: false };
    pub const LATAM: Course = Course { target: Target::Es, latam: true };
    pub const FRENCH: Course = Course { target: Target::Fr, latam: false };

    /// Locale for the system voice.
    pub fn locale(self) -> &'static str {
        match (self.target, self.latam) {
            (Target::Es, false) => "es-ES",
            (Target::Es, true) => "es-MX",
            (Target::Fr, _) => "fr-FR",
        }
    }

    /// Name of the recorded-voice pack (`voces-<id>.fvox`).
    pub fn voice_pack(self) -> &'static str {
        match (self.target, self.latam) {
            (Target::Es, false) => "es",
            (Target::Es, true) => "es-419",
            (Target::Fr, _) => "fr",
        }
    }
}

/// The learner's own language, used for meanings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Base {
    Fr,
    En,
    Es,
}

impl Base {
    pub fn code(self) -> &'static str {
        match self {
            Base::Fr => "fr",
            Base::En => "en",
            Base::Es => "es",
        }
    }

    pub fn from_code(s: &str) -> Option<Base> {
        match s {
            "fr" => Some(Base::Fr),
            "en" => Some(Base::En),
            "es" => Some(Base::Es),
            _ => None,
        }
    }
}

/// A gloss in Spanish may differ between Spain and Latin America: written `coche|carro`.
pub fn pick_variant(s: &str, latam: bool) -> &str {
    match s.split_once('|') {
        Some((spain, america)) => {
            if latam {
                america.trim()
            } else {
                spain.trim()
            }
        }
        None => s,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    M,
    F,
    /// Same form for both (*el/la estudiante*, *un/une élève*).
    Mf,
}

#[derive(Clone, Debug)]
pub struct Word {
    /// The word in the language learnt.
    pub lemma: String,
    /// Part of speech: n, v, adj, adv, prep, conj, pron, det, num, interj, expr.
    pub pos: String,
    pub gender: Option<Gender>,
    /// Meanings, by base language (empty when not given).
    pub fr: String,
    pub en: String,
    pub es: String,
    pub band: Band,
    pub theme: String,
    pub pic: Option<String>,
    pub note: String,
    pub target: Target,
}

impl Word {
    pub fn id(&self) -> String {
        format!("w:{}|{}", self.lemma, self.pos)
    }

    pub fn meaning(&self, base: Base, latam: bool) -> &str {
        match base {
            Base::Fr => &self.fr,
            Base::En => &self.en,
            Base::Es => pick_variant(&self.es, latam),
        }
    }

    /// The note as shown to a learner whose language is `base`: false-friend warnings for another language left out
    /// (`≠ EN: …` for a Spanish speaker), and in the French course the bilingual hints cut to the learner's side
    /// (`aussi: hielo / ice` → `aussi: hielo`; `le goûter = la merienda` only in Spanish).
    pub fn note_for(&self, base: Base) -> String {
        note_for(&self.note, self.target, base)
    }

    /// Plural-only noun (*las gafas*, *les gens*).
    pub fn plural(&self) -> bool {
        self.note.split([';', ',']).any(|p| p.trim() == "pl.")
    }

    /// The noun with its article: *el perro*, *el agua*, *la mano*; *le chien*, *la maison*, *l'arbre*, *le héros*,
    /// *les gens*.
    pub fn with_article(&self) -> String {
        if self.pos != "n" {
            return self.lemma.clone();
        }
        match self.target {
            Target::Es => {
                let el = match self.gender {
                    Some(Gender::M) | Some(Gender::Mf) => {
                        if self.plural() {
                            "los"
                        } else {
                            "el"
                        }
                    }
                    Some(Gender::F) => {
                        if self.plural() {
                            "las"
                        } else if self.note.contains("el agua") || self.note.starts_with("el ") {
                            "el"
                        } else {
                            "la"
                        }
                    }
                    None => return self.lemma.clone(),
                };
                format!("{el} {}", self.lemma)
            }
            Target::Fr => {
                if self.gender.is_none() {
                    return self.lemma.clone();
                }
                if self.plural() {
                    return format!("les {}", self.lemma);
                }
                if fr_elides(&self.lemma, &self.note) {
                    return format!("l'{}", self.lemma);
                }
                let a = if self.gender == Some(Gender::F) { "la" } else { "le" };
                format!("{a} {}", self.lemma)
            }
        }
    }

    /// The French noun with its indefinite article, which always shows the gender (*un arbre*, *une eau*).
    pub fn with_indefinite(&self) -> String {
        match self.gender {
            Some(Gender::F) => format!("une {}", self.lemma),
            _ => format!("un {}", self.lemma),
        }
    }

    /// For the new-word card: a French noun whose article hides its gender gets it in brackets (*l'arbre (m.)*).
    pub fn display(&self) -> String {
        let s = self.with_article();
        if self.target == Target::Fr && self.pos == "n" && (s.starts_with("l'") || s.starts_with("les ")) {
            match self.gender {
                Some(Gender::M) => return format!("{s} (m.)"),
                Some(Gender::F) => return format!("{s} (f.)"),
                _ => {}
            }
        }
        s
    }
}

/// See [`Word::note_for`].
pub fn note_for(note: &str, target: Target, base: Base) -> String {
    let mine = match base {
        Base::Fr => "FR",
        Base::En => "EN",
        Base::Es => "ES",
    };
    let mut out = Vec::new();
    for part in note.split(';').map(|p| p.trim()).filter(|p| !p.is_empty()) {
        if let Some(rest) = part.strip_prefix('≠') {
            if rest.trim_start().starts_with(mine) {
                out.push(part.to_string());
            }
            continue;
        }
        // A hint tagged for one base language: `pain (EN) = douleur`.
        if part.contains("(EN)") || part.contains("(ES)") || part.contains("(FR)") {
            if part.contains(&format!("({mine})")) {
                out.push(part.replace(&format!(" ({mine})"), "").replace(&format!("({mine})"), ""));
            }
            continue;
        }
        if target == Target::Fr {
            // The Spanish word with its article (`la leche`): a hint for Spanish speakers only.
            if ["el ", "la ", "los ", "las "].iter().any(|a| part.starts_with(a)) && base != Base::Es {
                continue;
            }
            if let Some((head, rest)) = part.split_once(':').filter(|(h, _)| matches!(h.trim(), "aussi" | "also")) {
                // `hielo / ice, espejo / mirror`: Spanish before the slash, English after.
                let side: Vec<&str> = rest
                    .split(',')
                    .filter_map(|item| {
                        let (es, en) = item.split_once(" / ")?;
                        Some(if base == Base::Es { es.trim() } else { en.trim() })
                    })
                    .filter(|x| !x.is_empty())
                    .collect();
                if !side.is_empty() {
                    out.push(format!("{}: {}", head.trim(), side.join(", ")));
                }
                continue;
            }
            // `prendre conscience = darse cuenta`: a hint in Spanish (grammar like `au = à + le` stays for all).
            if part.contains(" = ") && !part.contains('+') && base != Base::Es {
                continue;
            }
        }
        out.push(if target == Target::Fr { translate_tag(part, base) } else { part.to_string() });
    }
    out.join("; ")
}

/// The French course's grammar tags in the learner's language (`irrégulier` → `irregular`, `pluriel: yeux` →
/// `plural: yeux`, `aussi:` → `también:`).
fn translate_tag(part: &str, base: Base) -> String {
    let es = base == Base::Es;
    let table: [(&str, &str, &str); 10] = [
        ("h aspiré", "h aspirada (sin elisión)", "aspirated h (no elision)"),
        ("irrégulier", "irregular", "irregular"),
        ("familier", "coloquial", "informal"),
        ("affectueux", "afectuoso", "affectionate"),
        ("pluriel:", "plural:", "plural:"),
        ("aussi:", "también:", "also:"),
        ("comme ", "como ", "like "),
        ("voyelle", "vocal", "vowel"),
        ("tutoiement", "tuteo", "informal tu"),
        ("invariable", "invariable", "invariable"),
    ];
    let mut s = part.to_string();
    for (fr, e, n) in table {
        s = s.replace(fr, if es { e } else { n });
    }
    s
}

/// Whether a French word takes the elided article (*l'arbre*, *l'heure*) — vowels and mute *h*; a word marked
/// "h aspiré" in its note keeps the full article (*le héros*).
pub fn fr_elides(word: &str, note: &str) -> bool {
    let Some(c) = word.chars().next() else { return false };
    let c = c.to_lowercase().next().unwrap_or(c);
    if "aeiouâàäéèêëîïôöûùüœæ".contains(c) {
        return true;
    }
    c == 'h' && !note.contains("h aspiré")
}

/// A choice in context: ser/estar, por/para, preterite/imperfect, subjunctive/indicative; être/avoir, partitives,
/// passé composé/imparfait, subjonctif.
#[derive(Clone, Debug)]
pub struct Choice {
    /// Phrase or sentence (with `___` where the answer goes, for sentences).
    pub prompt: String,
    pub answer: String,
    /// The wrong alternative. Empty for two-door choices (ser/estar, por/para, être/avoir).
    pub other: String,
    /// The form that fills the gap when it differs from the answer's label (être/avoir: *sommes* for *être*).
    pub form: String,
    pub fr: String,
    pub en: String,
    pub es: String,
    pub band: Band,
    pub why: String,
}

impl Choice {
    pub fn translation(&self, base: Base, latam: bool) -> &str {
        match base {
            Base::Fr => &self.fr,
            Base::En => &self.en,
            Base::Es => pick_variant(&self.es, latam),
        }
    }

    /// The sentence with the gap filled.
    pub fn filled(&self) -> String {
        let s = self.sentence();
        let fill = if self.form.is_empty() { &self.answer } else { &self.form };
        if s.contains("___") { fix_elision(&s.replace("___", fill)) } else { format!("{} {}", fill, s) }
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

/// Joins an elided form to the next word once a gap is filled: `de l' eau` → `de l'eau`, `j' ai` → `j'ai`.
pub fn fix_elision(s: &str) -> String {
    s.replace("' ", "'").replace("’ ", "’")
}

#[derive(Clone, Debug)]
pub struct FalseFriend {
    /// The word in the language learnt.
    pub word: String,
    /// (true meaning, tempting wrong meaning) by base language; empty trap when it is no false friend there.
    pub fr: (String, String),
    pub en: (String, String),
    pub es: (String, String),
    pub band: Band,
    pub note: String,
}

impl FalseFriend {
    fn pair(&self, base: Base) -> &(String, String) {
        match base {
            Base::Fr => &self.fr,
            Base::En => &self.en,
            Base::Es => &self.es,
        }
    }

    pub fn truth(&self, base: Base, latam: bool) -> &str {
        pick_variant(&self.pair(base).0, latam)
    }

    pub fn trap(&self, base: Base, latam: bool) -> &str {
        pick_variant(&self.pair(base).1, latam)
    }
}

#[derive(Clone, Debug)]
pub struct Content {
    pub course: Course,
    pub words: Vec<Word>,
    /// Nouns whose gender surprises learners.
    pub gender_traps: Vec<Word>,
    /// Spanish: ser/estar. French: être/avoir in the passé composé.
    pub binary_a: Vec<Choice>,
    /// Spanish: por/para. French: unused.
    pub binary_b: Vec<Choice>,
    /// Spanish: preterite/imperfect. French: passé composé/imparfait.
    pub past: Vec<Choice>,
    /// Subjunctive or indicative.
    pub subjunctive: Vec<Choice>,
    /// French: du, de la, de l', des, de.
    pub partitive: Vec<Choice>,
    pub false_friends: Vec<FalseFriend>,
    pub pseudo: Vec<String>,
    /// Words that sound like another word (*verre*, *vert*, *vers*): never in listening rounds.
    pub homophones: HashSet<String>,
}

static ES_LEX_A: &str = include_str!("../../../assets/es/lexicon_a.tsv");
static ES_LEX_B: &str = include_str!("../../../assets/es/lexicon_b.tsv");
static ES_GENERO: &str = include_str!("../../../assets/es/grammar/genero.tsv");
static ES_SER_ESTAR: &str = include_str!("../../../assets/es/grammar/ser_estar.tsv");
static ES_POR_PARA: &str = include_str!("../../../assets/es/grammar/por_para.tsv");
static ES_PRET_IMP: &str = include_str!("../../../assets/es/grammar/pret_imp.tsv");
static ES_SUBJ: &str = include_str!("../../../assets/es/grammar/subjuntivo.tsv");
static ES_FALSOS: &str = include_str!("../../../assets/es/grammar/falsos_amigos.tsv");
static ES_PSEUDO: &str = include_str!("../../../assets/es/grammar/pseudo.tsv");
static ES_LATAM: &str = include_str!("../../../assets/es/latam.tsv");
static ES_LATAM_SKIP: &str = include_str!("../../../assets/es/latam_skip.txt");

static FR_LEX_A: &str = include_str!("../../../assets/fr/lexicon_a.tsv");
static FR_LEX_B: &str = include_str!("../../../assets/fr/lexicon_b.tsv");
static FR_GENRE: &str = include_str!("../../../assets/fr/grammar/genre.tsv");
static FR_ETRE_AVOIR: &str = include_str!("../../../assets/fr/grammar/etre_avoir.tsv");
static FR_PARTITIFS: &str = include_str!("../../../assets/fr/grammar/partitifs.tsv");
static FR_PASSE: &str = include_str!("../../../assets/fr/grammar/passe.tsv");
static FR_SUBJ: &str = include_str!("../../../assets/fr/grammar/subjonctif.tsv");
static FR_FAUX: &str = include_str!("../../../assets/fr/grammar/faux_amis.tsv");
static FR_PSEUDO: &str = include_str!("../../../assets/fr/grammar/pseudo.tsv");
static FR_HOMOPHONES: &str = include_str!("../../../assets/fr/homophones.txt");

/// A TSV file read by column name.
struct Table<'a> {
    cols: HashMap<&'a str, usize>,
    rows: Vec<Vec<&'a str>>,
}

impl<'a> Table<'a> {
    fn parse(text: &'a str) -> Table<'a> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        let cols = lines.next().map(|h| h.split('\t').enumerate().map(|(i, c)| (c.trim(), i)).collect()).unwrap_or_default();
        let rows = lines.map(|l| l.split('\t').collect()).collect();
        Table { cols, rows }
    }

    fn get(&self, r: &[&'a str], name: &str) -> &'a str {
        self.cols.get(name).and_then(|&i| r.get(i)).copied().unwrap_or("").trim()
    }

    /// Column by position (first column = the prompt or the word).
    fn at(&self, r: &[&'a str], i: usize) -> &'a str {
        r.get(i).copied().unwrap_or("").trim()
    }
}

fn gender(s: &str) -> Option<Gender> {
    match s {
        "m" => Some(Gender::M),
        "f" => Some(Gender::F),
        "mf" => Some(Gender::Mf),
        _ => None,
    }
}

fn words(text: &str, target: Target) -> Vec<Word> {
    let t = Table::parse(text);
    t.rows
        .iter()
        .map(|r| Word {
            lemma: t.at(r, 0).to_string(),
            pos: t.get(r, "pos").to_string(),
            gender: gender(t.get(r, "gender")),
            fr: if target == Target::Fr { String::new() } else { t.get(r, "fr").to_string() },
            en: t.get(r, "en").to_string(),
            es: if target == Target::Es { String::new() } else { t.get(r, "es").to_string() },
            band: Band::parse(t.get(r, "band")),
            theme: t.get(r, "theme").to_string(),
            pic: Some(t.get(r, "pic").to_string()).filter(|s| !s.is_empty()),
            note: t.get(r, "note").to_string(),
            target,
        })
        .filter(|w| !w.lemma.is_empty())
        .collect()
}

fn gender_traps(text: &str, target: Target) -> Vec<Word> {
    let t = Table::parse(text);
    t.rows
        .iter()
        .map(|r| Word {
            lemma: t.at(r, 0).to_string(),
            pos: "n".into(),
            gender: gender(t.get(r, "gender")),
            fr: if target == Target::Fr { String::new() } else { t.get(r, "fr").to_string() },
            en: t.get(r, "en").to_string(),
            es: if target == Target::Es { String::new() } else { t.get(r, "es").to_string() },
            band: Band::parse(t.get(r, "band")),
            theme: String::new(),
            pic: None,
            note: t.get(r, "note").to_string(),
            target,
        })
        .filter(|w| !w.lemma.is_empty())
        .collect()
}

fn choices(text: &str, target: Target) -> Vec<Choice> {
    let t = Table::parse(text);
    t.rows
        .iter()
        .map(|r| Choice {
            prompt: t.at(r, 0).to_string(),
            answer: t.get(r, "answer").to_string(),
            other: t.get(r, "other").to_string(),
            form: t.get(r, "form").to_string(),
            fr: if target == Target::Fr { String::new() } else { t.get(r, "fr").to_string() },
            en: t.get(r, "en").to_string(),
            es: if target == Target::Es { String::new() } else { t.get(r, "es").to_string() },
            band: Band::parse(t.get(r, "band")),
            why: t.get(r, "why").to_string(),
        })
        .filter(|c| !c.prompt.is_empty() && !c.answer.is_empty())
        .collect()
}

fn false_friends(text: &str) -> Vec<FalseFriend> {
    let t = Table::parse(text);
    let pair = |r: &[&str], l: &str| (t.get(r, &format!("{l}_true")).to_string(), t.get(r, &format!("{l}_trap")).to_string());
    t.rows
        .iter()
        .map(|r| FalseFriend {
            word: t.at(r, 0).to_string(),
            fr: pair(r, "fr"),
            en: pair(r, "en"),
            es: pair(r, "es"),
            band: Band::parse(t.get(r, "band")),
            note: t.get(r, "note").to_string(),
        })
        .filter(|f| !f.word.is_empty())
        .collect()
}

fn first_column(text: &str) -> Vec<String> {
    let t = Table::parse(text);
    t.rows.iter().map(|r| t.at(r, 0).to_string()).filter(|s| !s.is_empty()).collect()
}

/// Lower-case words of a text (letters only).
pub fn tokens(s: &str) -> impl Iterator<Item = String> + '_ {
    s.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()).map(|w| w.to_lowercase())
}

impl Content {
    pub fn load(course: Course) -> Content {
        match course.target {
            Target::Es => Content::spanish(course.latam),
            Target::Fr => Content::french(),
        }
    }

    fn spanish(latam: bool) -> Content {
        let mut ws = words(ES_LEX_A, Target::Es);
        ws.extend(words(ES_LEX_B, Target::Es));
        let mut c = Content {
            course: if latam { Course::LATAM } else { Course::SPAIN },
            words: ws,
            gender_traps: gender_traps(ES_GENERO, Target::Es),
            binary_a: choices(ES_SER_ESTAR, Target::Es),
            binary_b: choices(ES_POR_PARA, Target::Es),
            past: choices(ES_PRET_IMP, Target::Es),
            subjunctive: choices(ES_SUBJ, Target::Es),
            partitive: Vec::new(),
            false_friends: false_friends(ES_FALSOS),
            pseudo: first_column(ES_PSEUDO),
            homophones: HashSet::new(),
        };
        if latam {
            c.apply_latam();
        }
        c
    }

    fn french() -> Content {
        let mut ws = words(FR_LEX_A, Target::Fr);
        ws.extend(words(FR_LEX_B, Target::Fr));
        Content {
            course: Course::FRENCH,
            words: ws,
            gender_traps: gender_traps(FR_GENRE, Target::Fr),
            binary_a: choices(FR_ETRE_AVOIR, Target::Fr),
            binary_b: Vec::new(),
            past: choices(FR_PASSE, Target::Fr),
            subjunctive: choices(FR_SUBJ, Target::Fr),
            partitive: choices(FR_PARTITIFS, Target::Fr),
            false_friends: false_friends(FR_FAUX),
            pseudo: first_column(FR_PSEUDO),
            homophones: FR_HOMOPHONES.lines().flat_map(|l| l.split_whitespace()).map(|s| s.to_string()).collect(),
        }
    }

    /// Latin-American Spanish: words replaced (*coche* → *carro*) or dropped (*vosotros*), and every sentence that
    /// only works in Spain (a *vosotros* form, a replaced word) left out.
    fn apply_latam(&mut self) {
        struct Swap {
            latam: String,
            gender: String,
            fr: String,
            en: String,
            note: String,
        }
        let t = Table::parse(ES_LATAM);
        // Spain-only words (single words, with their plural or every conjugated form) and phrases.
        let mut spain_only: HashSet<String> = HashSet::new();
        let mut spain_phrases: Vec<(HashSet<String>, Vec<String>)> = Vec::new();
        let mut map: HashMap<(String, String), Swap> = HashMap::new();
        for r in &t.rows {
            let es = t.at(r, 0);
            let swap = Swap {
                latam: t.get(r, "latam").to_string(),
                gender: t.get(r, "gender").to_string(),
                fr: t.get(r, "fr").to_string(),
                en: t.get(r, "en").to_string(),
                note: t.get(r, "note").to_string(),
            };
            let pos = t.get(r, "pos");
            if swap.latam != es {
                if es.contains(' ') {
                    // A phrase: its words in a row, the first one in any conjugated form (te echo de menos).
                    let words: Vec<String> = tokens(es).collect();
                    let mut first: HashSet<String> = HashSet::from([words[0].clone()]);
                    if pos == "v" || pos == "expr" {
                        for t in crate::conj::Tense::ALL {
                            for p in crate::conj::Person::ALL {
                                if let Some(f) = crate::conj::conjugate(&words[0], t, p) {
                                    first.extend(tokens(&f));
                                }
                            }
                        }
                    }
                    spain_phrases.push((first, words[1..].to_vec()));
                } else if pos == "v" {
                    for t in crate::conj::Tense::ALL {
                        for p in crate::conj::Person::ALL {
                            if let Some(f) = crate::conj::conjugate(es, t, p) {
                                spain_only.extend(tokens(&f));
                            }
                        }
                    }
                    spain_only.insert(es.to_lowercase());
                } else {
                    let w = es.to_lowercase();
                    if matches!(pos, "n" | "adj") {
                        let vowel = w.ends_with(['a', 'e', 'i', 'o', 'u']);
                        spain_only.insert(if vowel { format!("{w}s") } else { format!("{w}es") });
                        // enfadado → enfadada, enfadados, enfadadas.
                        if let Some(stem) = w.strip_suffix('o') {
                            for e in ["a", "os", "as"] {
                                spain_only.insert(format!("{stem}{e}"));
                            }
                        }
                    }
                    spain_only.insert(w);
                }
            }
            map.insert((es.to_string(), pos.to_string()), swap);
        }
        let dropped = |w: &Word| map.get(&(w.lemma.clone(), w.pos.clone())).is_some_and(|s| s.latam == "-" || s.latam.is_empty());
        let existing: HashSet<(String, String)> = self.words.iter().filter(|w| !dropped(w)).map(|w| (w.lemma.clone(), w.pos.clone())).collect();
        let mut out = Vec::with_capacity(self.words.len());
        for mut w in std::mem::take(&mut self.words) {
            match map.get(&(w.lemma.clone(), w.pos.clone())) {
                Some(s) if s.latam == "-" || s.latam.is_empty() => continue,
                // The Latin-American word is already in the lexicon: keep that one only.
                Some(s) if s.latam != w.lemma && existing.contains(&(s.latam.clone(), w.pos.clone())) => continue,
                Some(s) => {
                    w.lemma = s.latam.clone();
                    w.note = s.note.clone();
                    if let Some(g) = gender(&s.gender) {
                        w.gender = Some(g);
                    }
                    if !s.fr.is_empty() {
                        w.fr = s.fr.clone();
                    }
                    if !s.en.is_empty() {
                        w.en = s.en.clone();
                    }
                    out.push(w);
                }
                None => {
                    // Notes about Latin America are what the variant now shows: drop them.
                    w.note = w.note.split(';').filter(|p| !p.contains("LatAm")).map(|p| p.trim()).collect::<Vec<_>>().join("; ");
                    out.push(w);
                }
            }
        }
        self.words = out;
        // A Spain word that is also a form of a verb used everywhere (*venga* of *venir*, *mando* of *mandar*) does
        // not mark a sentence as Spain-only.
        let mut shared: HashSet<String> = HashSet::new();
        for w in &self.words {
            if w.pos != "v" || w.lemma.contains(' ') {
                continue;
            }
            for t in crate::conj::Tense::ALL {
                for p in crate::conj::Person::ALL {
                    if p == crate::conj::Person::Vosotros {
                        continue;
                    }
                    if let Some(f) = crate::conj::conjugate(&w.lemma, t, p) {
                        shared.extend(tokens(&f));
                    }
                }
            }
        }
        spain_only.retain(|x| !shared.contains(x));
        // Every vosotros form of every verb of the lexicon.
        for w in &self.words {
            if w.pos != "v" || w.lemma.contains(' ') {
                continue;
            }
            for t in crate::conj::Tense::ALL {
                if let Some(f) = crate::conj::conjugate(&w.lemma, t, crate::conj::Person::Vosotros) {
                    spain_only.extend(tokens(&f).filter(|x| x != "os"));
                }
            }
        }
        for x in ["vosotros", "vosotras", "os", "vuestro", "vuestra", "vuestros", "vuestras"] {
            spain_only.insert(x.to_string());
        }
        // Sentences reviewed by hand as Spain-only (perfect tense for today, *quedar*, *rellenar*…).
        let skip: HashSet<&str> = ES_LATAM_SKIP.lines().map(|l| l.trim()).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
        let keep = |c: &Choice| {
            let toks: Vec<String> = tokens(&c.filled()).chain(tokens(&c.prompt)).chain(tokens(&c.other)).collect();
            let has_phrase = |(first, rest): &(HashSet<String>, Vec<String>)| {
                (0..toks.len()).any(|i| first.contains(&toks[i]) && toks.len() > i + rest.len() && toks[i + 1..=i + rest.len()] == rest[..])
            };
            !skip.contains(c.prompt.as_str()) && !toks.iter().any(|x| spain_only.contains(x)) && !spain_phrases.iter().any(has_phrase)
        };
        for list in [&mut self.binary_a, &mut self.binary_b, &mut self.past, &mut self.subjunctive] {
            list.retain(keep);
        }
        self.gender_traps.retain(|w| !spain_only.contains(&w.lemma) && !skip.contains(w.lemma.as_str()));
        self.false_friends.retain(|f| !spain_only.contains(&f.word) && !skip.contains(f.word.as_str()));
    }

    pub fn word(&self, id: &str) -> Option<&Word> {
        let rest = id.strip_prefix("w:")?;
        let (lemma, pos) = rest.split_once('|')?;
        self.words.iter().find(|w| w.lemma == lemma && w.pos == pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_american_variant() {
        let spain = Content::load(Course::SPAIN);
        let america = Content::load(Course::LATAM);
        let n = |c: &Content| c.binary_a.len() + c.binary_b.len() + c.past.len() + c.subjunctive.len();
        // Most sentences work on both sides of the Atlantic.
        assert!(n(&america) * 100 >= n(&spain) * 85, "{} of {}", n(&america), n(&spain));
        let has = |c: &Content, s: &str| c.binary_a.iter().chain(&c.binary_b).chain(&c.past).chain(&c.subjunctive).any(|x| x.prompt.contains(s));
        // Spain-only: coger in any form, plurals of replaced words, a perfect tense for today.
        assert!(!has(&america, "Coge el 5") && has(&spain, "Coge el 5"));
        assert!(!has(&america, "móviles"));
        assert!(!has(&america, "cogimos"));
        assert!(!has(&america, "Tu madre ha preguntado"));
        // Shared verb forms are not mistaken for Spain words (venga: venir, mando: mandar).
        assert!(has(&america, "venga") || !has(&spain, "venga"));
        let w = |c: &Content, l: &str| c.words.iter().find(|w| w.lemma == l).map(|w| w.gender);
        assert_eq!(w(&america, "refrigerador"), Some(Some(Gender::M)));
        assert_eq!(w(&america, "frijol"), Some(Some(Gender::M)));
        assert!(w(&america, "coche").is_none() && w(&america, "vosotros").is_none());
    }

    #[test]
    fn french_articles() {
        let fr = Content::load(Course::FRENCH);
        let w = |l: &str| fr.words.iter().find(|w| w.lemma == l && w.pos == "n").unwrap_or_else(|| panic!("{l}"));
        assert_eq!(w("arbre").with_article(), "l'arbre");
        assert_eq!(w("arbre").display(), "l'arbre (m.)");
        assert_eq!(w("héros").with_article(), "le héros");
        assert_eq!(w("homme").with_article(), "l'homme");
        assert_eq!(w("maison").with_article(), "la maison");
        assert_eq!(w("gens").with_article(), "les gens");
        assert_eq!(w("eau").with_indefinite(), "une eau");
        assert_eq!(pick_variant("coche|carro", true), "carro");
        assert_eq!(pick_variant("coche|carro", false), "coche");
        assert_eq!(note_for("aussi: hielo / ice, espejo / mirror", Target::Fr, Base::En), "aussi: ice, mirror");
        assert_eq!(note_for("h aspiré; ≠ EN: hazard = danger", Target::Fr, Base::Es), "h aspirada (sin elisión)");
        assert_eq!(note_for("h aspiré; ≠ EN: hazard = danger", Target::Fr, Base::En), "aspirated h (no elision); ≠ EN: hazard = danger");
        assert_eq!(note_for("le goûter = la merienda", Target::Fr, Base::En), "");
        assert_eq!(note_for("au = à + le", Target::Fr, Base::En), "au = à + le");
        assert_eq!(note_for("irrégulier (comme tenir)", Target::Fr, Base::Es), "irregular (como tenir)");
        assert_eq!(note_for("pluriel: yeux", Target::Fr, Base::En), "plural: yeux");
        assert_eq!(note_for("pain (EN) = douleur", Target::Fr, Base::En), "pain = douleur");
        assert_eq!(note_for("pain (EN) = douleur", Target::Fr, Base::Es), "");
        assert_eq!(note_for("la leche", Target::Fr, Base::En), "");
        assert_eq!(note_for("la leche", Target::Fr, Base::Es), "la leche");
    }
}
