//! Chooses what goes into the next language round: which items (due reviews first, a few new ones), as tiles (the
//! blocks) and gates (the doors). The door rule *is* the language rule: colours never give the answer away, there are
//! always more doors than one, and wrong doors bounce.

use std::collections::HashSet;

use crate::content::{Band, Base, Choice, Content, Gender, Target, Word};
use crate::memory::Memory;
use crate::{conj, conj_fr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Spanish word → meaning (or meaning → Spanish once the word is known).
    Words,
    /// Hear the word, find its meaning.
    Listening,
    /// el / la; un / une.
    Gender,
    /// Conjugated forms → persons.
    Verbs,
    SerEstar,
    PorPara,
    /// Preterite or imperfect (passé composé or imparfait), in a sentence.
    Past,
    /// Subjunctive or indicative, in a sentence.
    Subjunctive,
    FalseFriends,
    /// French: être or avoir in the passé composé.
    Auxiliary,
    /// French: du, de la, de l', des, de.
    Partitive,
    /// "¿Existe?" / "Ça existe ?": real words vs. invented ones.
    Placement,
}

impl Mode {
    /// The modes offered on a course's home, in order.
    pub fn menu(target: Target) -> &'static [Mode] {
        match target {
            Target::Es => &[Mode::Words, Mode::Listening, Mode::Gender, Mode::Verbs, Mode::SerEstar, Mode::PorPara, Mode::Past, Mode::Subjunctive, Mode::FalseFriends],
            Target::Fr => &[Mode::Words, Mode::Listening, Mode::Gender, Mode::Verbs, Mode::Partitive, Mode::Auxiliary, Mode::Past, Mode::Subjunctive, Mode::FalseFriends],
        }
    }

    /// Grammar modes the daily session rotates through.
    pub fn grammar(target: Target) -> &'static [Mode] {
        match target {
            Target::Es => &[Mode::Gender, Mode::Verbs, Mode::SerEstar, Mode::PorPara, Mode::Past, Mode::Subjunctive, Mode::FalseFriends],
            Target::Fr => &[Mode::Gender, Mode::Verbs, Mode::Partitive, Mode::FalseFriends, Mode::Auxiliary, Mode::Past, Mode::Subjunctive],
        }
    }

    /// Name of the mode, in the language learnt.
    pub fn name(self, target: Target) -> &'static str {
        match target {
            Target::Es => match self {
                Mode::Words => "Palabras",
                Mode::Listening => "Escucha",
                Mode::Gender => "El o la",
                Mode::Verbs => "Verbos",
                Mode::SerEstar => "Ser o estar",
                Mode::PorPara => "Por o para",
                Mode::Past => "Pasado",
                Mode::Subjunctive => "Subjuntivo",
                Mode::FalseFriends => "Falsos amigos",
                Mode::Auxiliary => "Ser o haber",
                Mode::Partitive => "Partitivos",
                Mode::Placement => "¿Existe?",
            },
            Target::Fr => match self {
                Mode::Words => "Mots",
                Mode::Listening => "Écoute",
                Mode::Gender => "Un ou une",
                Mode::Verbs => "Verbes",
                Mode::SerEstar => "Être",
                Mode::PorPara => "Pour",
                Mode::Past => "Passé",
                Mode::Subjunctive => "Subjonctif",
                Mode::FalseFriends => "Faux amis",
                Mode::Auxiliary => "Être ou avoir",
                Mode::Partitive => "Du, de la, des",
                Mode::Placement => "Ça existe ?",
            },
        }
    }

    /// Lowest band where the mode makes sense.
    pub fn min_band(self, target: Target) -> Band {
        match (target, self) {
            (_, Mode::Words | Mode::Listening | Mode::Gender | Mode::Verbs | Mode::Placement) => Band::A1,
            (Target::Es, Mode::SerEstar | Mode::PorPara | Mode::FalseFriends | Mode::Past) => Band::A2,
            (Target::Fr, Mode::Partitive | Mode::FalseFriends) => Band::A1,
            (Target::Fr, Mode::Auxiliary | Mode::Past) => Band::A2,
            (_, Mode::Subjunctive) => Band::B1,
            _ => Band::A2,
        }
    }

    /// Whether the course has this mode.
    pub fn exists(self, target: Target) -> bool {
        self == Mode::Placement || Mode::menu(target).contains(&self)
    }
}

/// A verb tense of either language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VTense {
    Es(conj::Tense),
    Fr(conj_fr::Tense),
}

impl VTense {
    pub fn name(self) -> &'static str {
        match self {
            VTense::Es(t) => t.name_es(),
            VTense::Fr(t) => t.name(),
        }
    }

    pub fn parse(name: &str) -> Option<VTense> {
        conj::Tense::ALL.into_iter().find(|t| t.name_es() == name).map(VTense::Es).or_else(|| conj_fr::Tense::ALL.into_iter().find(|t| t.name() == name).map(VTense::Fr))
    }
}

/// The paradigm of a verb in a tense: (form shown, door label, spoken with its subject) for each person, persons
/// with the same form merged (*parle*: je / il / elle). Latin-American Spanish has no *vosotros*.
pub fn paradigm(verb: &str, tense: VTense, latam: bool) -> Option<Vec<(String, String, String)>> {
    let mut groups: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut push = |form: String, label: &str, said: String| match groups.iter_mut().find(|(f, _, _)| *f == form) {
        Some((_, ls, _)) => ls.push(label.to_string()),
        None => groups.push((form, vec![label.to_string()], said)),
    };
    match tense {
        VTense::Es(t) => {
            for p in conj::Person::ALL {
                if latam && p == conj::Person::Vosotros {
                    continue;
                }
                let f = conj::conjugate(verb, t, p)?;
                let label = if latam && p == conj::Person::Ellos { "ellos / ustedes" } else { p.pronoun() };
                push(f.clone(), label, format!("{} {f}", p.pronoun()));
            }
        }
        VTense::Fr(t) => {
            for p in conj_fr::Person::ALL {
                let f = conj_fr::conjugate(verb, t, p)?;
                let said = conj_fr::spoken(&conj_fr::with_subject(&f, p, verb));
                push(f, p.label(), said);
            }
        }
    }
    Some(groups.into_iter().map(|(f, ls, said)| (f, merge_labels(&ls), said)).collect())
}

/// `je` + `il / elle` → `je / il / elle`.
fn merge_labels(ls: &[String]) -> String {
    ls.join(" / ")
}

/// A block of the round.
#[derive(Clone, Debug)]
pub struct Tile {
    pub text: String,
    pub key: u16,
    /// Item graded by this tile (None: a distractor tile).
    pub item: Option<String>,
    /// Spanish text to speak once answered (on exit, in feedback): may contain the answer.
    pub say: Option<String>,
    /// Spanish text to speak when the tile is tapped before it is answered: never gives the answer away (the word
    /// without its article, the form without its person, the sentence with a pause for the gap).
    pub ask: Option<String>,
    /// Audio only: the text is hidden (listening).
    pub hidden: bool,
    /// Shown when the tile goes through a wrong door.
    pub explain: String,
    /// Width in cells (2 or 3).
    pub width: u8,
    /// Placement rounds: the band of a real word, `None` for an invented one.
    pub band: Option<Band>,
}

/// A gap sentence read aloud without its answer: the blank becomes a short pause.
pub fn spoken_gap(prompt: &str) -> String {
    let mut out = String::new();
    let mut in_gap = false;
    for ch in prompt.chars() {
        if ch == '_' {
            if !in_gap {
                out.push('…');
                in_gap = true;
            }
        } else {
            in_gap = false;
            out.push(ch);
        }
    }
    out
}

/// A door of the round.
#[derive(Clone, Debug)]
pub struct Gate {
    pub text: String,
    pub key: u16,
    pub pic: Option<String>,
    /// Item failed when a distractor tile is pushed into this door (sentence doors).
    pub item: Option<String>,
    /// Long text: shown in the legend above the board, the door only shows its mark.
    pub long: bool,
}

/// A word to present before the round (new words).
#[derive(Clone, Debug)]
pub struct Intro {
    /// The word as shown (*el perro*, *l'arbre (m.)*).
    pub es: String,
    /// What to say when the card is tapped.
    pub say: String,
    pub meaning: String,
    pub pic: Option<String>,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct Round {
    pub mode: Mode,
    pub tiles: Vec<Tile>,
    pub gates: Vec<Gate>,
    pub intro: Vec<Intro>,
    /// Production direction (meaning → Spanish).
    pub production: bool,
    /// For verbs: the infinitive and tense shown as the round's title.
    pub heading: String,
}

pub struct Planner<'a> {
    pub content: &'a Content,
    pub memory: &'a Memory,
    pub base: Base,
    /// Spanish glosses as in Latin America (*carro*), and the Latin-American Spanish course.
    pub latam: bool,
    /// Learner's working band (from placement or the content seen so far).
    pub band: Band,
    pub now: f64,
    rng: u64,
}

fn width_for(text: &str) -> u8 {
    if text.chars().count() > 11 { 3 } else { 2 }
}

/// Words that make poor vocabulary tiles (grammar words with long glosses).
const NOT_VOCAB: &[&str] = &[
    "ser", "estar", "haber", "no", "sí", "que", "ya", "tan", "muy", "más", "menos", "bien", "être", "avoir", "ne", "pas", "en", "y", "si", "plus", "très",
    "aussi", "donc", "alors",
];

/// Keeps the overall order but alternates parts of speech, so a round mixes nouns, verbs and adjectives instead of
/// six verbs in a row (frequency lists start with verbs).
fn interleave_pos(v: Vec<Word>) -> Vec<Word> {
    let mut groups: Vec<(String, std::collections::VecDeque<Word>)> = Vec::new();
    for w in v {
        let g = if w.pos == "n" || w.pos == "v" || w.pos == "adj" { w.pos.clone() } else { "other".to_string() };
        match groups.iter_mut().find(|(k, _)| *k == g) {
            Some((_, q)) => q.push_back(w),
            None => groups.push((g, std::collections::VecDeque::from([w]))),
        }
    }
    let mut out = Vec::new();
    loop {
        let mut any = false;
        for (k, q) in groups.iter_mut() {
            // Nouns twice as often: they are half of the vocabulary.
            let take = if k == "n" { 2 } else { 1 };
            for _ in 0..take {
                if let Some(w) = q.pop_front() {
                    out.push(w);
                    any = true;
                }
            }
        }
        if !any {
            break;
        }
    }
    out
}

/// Stable short id for a sentence item.
pub fn hash_id(prefix: &str, s: &str) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for b in s.bytes() {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    format!("{prefix}:{h:08x}")
}

impl<'a> Planner<'a> {
    pub fn new(content: &'a Content, memory: &'a Memory, base: Base, band: Band, now: f64, seed: u64) -> Planner<'a> {
        Planner { content, memory, base, latam: content.course.latam, band, now, rng: seed | 1 }
    }

    /// Spanish glosses as in Latin America (the French course for a Latin-American Spanish speaker).
    pub fn with_latam(mut self, latam: bool) -> Planner<'a> {
        self.latam = latam || self.content.course.latam;
        self
    }

    fn target(&self) -> Target {
        self.content.course.target
    }

    fn fr(&self) -> bool {
        self.target() == Target::Fr
    }

    fn rnd(&mut self, n: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        if n == 0 { 0 } else { (self.rng % n as u64) as usize }
    }

    fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.rnd(i + 1);
            v.swap(i, j);
        }
    }

    /// Picks up to `n` items of a pool: due ones first (most forgotten first), then new ones, then the least recently
    /// reviewed. `id` gives the item id of a pool element.
    fn pick<T: Clone>(&mut self, pool: &[T], id: impl Fn(&T) -> String, n: usize, max_new: usize) -> Vec<(T, bool)> {
        let due: HashSet<String> = self.memory.due("", self.now).into_iter().collect();
        let mut out: Vec<(T, bool)> = Vec::new();
        let mut taken: HashSet<String> = HashSet::new();
        let mut due_items: Vec<&T> = pool.iter().filter(|t| due.contains(&id(t))).collect();
        due_items.sort_by(|a, b| {
            let ra = self.memory.retrievability(&id(a), self.now).unwrap_or(1.0);
            let rb = self.memory.retrievability(&id(b), self.now).unwrap_or(1.0);
            ra.partial_cmp(&rb).unwrap_or(std::cmp::Ordering::Equal)
        });
        for t in due_items {
            if out.len() >= n {
                break;
            }
            if taken.insert(id(t)) {
                out.push((t.clone(), false));
            }
        }
        let mut new = 0;
        for t in pool {
            if out.len() >= n || new >= max_new {
                break;
            }
            if self.memory.card(&id(t)).is_none() && taken.insert(id(t)) {
                out.push((t.clone(), true));
                new += 1;
            }
        }
        if out.len() < n {
            // Fill with seen, not-due items, least recently reviewed first.
            let mut rest: Vec<&T> = pool.iter().filter(|t| self.memory.card(&id(t)).is_some() && !taken.contains(&id(t))).collect();
            rest.sort_by(|a, b| {
                let la = self.memory.card(&id(a)).map(|c| c.last).unwrap_or(0.0);
                let lb = self.memory.card(&id(b)).map(|c| c.last).unwrap_or(0.0);
                la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal)
            });
            for t in rest {
                if out.len() >= n {
                    break;
                }
                if taken.insert(id(t)) {
                    out.push((t.clone(), false));
                }
            }
        }
        if out.len() < n {
            // Still short (early on): more new items.
            for t in pool {
                if out.len() >= n {
                    break;
                }
                if taken.insert(id(t)) {
                    out.push((t.clone(), self.memory.card(&id(t)).is_none()));
                }
            }
        }
        out
    }

    /// Words in teaching order: by band (from the learner's band down to A1 for gaps, then upwards), frequency order
    /// inside a band.
    fn word_pool(&self, filter: impl Fn(&Word) -> bool) -> Vec<Word> {
        let mut v: Vec<Word> = self.content.words.iter().filter(|w| filter(w) && w.band <= self.band).cloned().collect();
        // Teach the learner's own band first, lower bands as review material afterwards.
        v.sort_by_key(|w| if w.band == self.band { 0 } else { 1 + (self.band.index() - w.band.index()) });
        let mut above: Vec<Word> = self.content.words.iter().filter(|w| filter(w) && w.band > self.band).cloned().collect();
        above.sort_by_key(|w| w.band);
        v.extend(above);
        interleave_pos(v)
    }

    pub fn plan(&mut self, mode: Mode) -> Option<Round> {
        if !mode.exists(self.target()) {
            return None;
        }
        match mode {
            Mode::Words | Mode::Listening => self.words(mode),
            Mode::Gender => self.gender(),
            Mode::Verbs => self.verbs(),
            Mode::SerEstar | Mode::PorPara | Mode::Auxiliary => self.binary(mode),
            Mode::Past | Mode::Subjunctive | Mode::Partitive => self.sentences(mode),
            Mode::FalseFriends => self.false_friends(),
            Mode::Placement => self.placement(),
        }
    }

    fn words(&mut self, mode: Mode) -> Option<Round> {
        let listening = mode == Mode::Listening;
        let (base, latam) = (self.base, self.latam);
        let content_words = |w: &Word| {
            matches!(w.pos.as_str(), "n" | "v" | "adj" | "adv" | "expr" | "num" | "interj")
                && !w.meaning(base, latam).is_empty()
                && !NOT_VOCAB.contains(&w.lemma.as_str())
                && !w.meaning(base, latam).contains('(')
                && w.meaning(base, latam).chars().count() <= 22
        };
        let mut pool = self.word_pool(content_words);
        if listening {
            // Only words already met, and never a word that sounds like another one (verre, vert, vers).
            let homophones = &self.content.homophones;
            pool.retain(|w| self.memory.card(&w.id()).is_some() && !homophones.contains(&w.lemma));
            if pool.len() < 4 {
                return self.words(Mode::Words);
            }
        }
        // Three words and one decoy door: four doors, the most a board of two door columns offers.
        let n = 3;
        let picked = self.pick(&pool, |w| w.id(), n, if listening { 0 } else { 2 });
        if picked.len() < 2 {
            return None;
        }
        // Direction: production once most picked words are well known.
        let mature = picked.iter().filter(|(w, _)| self.memory.card(&w.id()).is_some_and(|c| c.stability >= 7.0)).count();
        let production = !listening && mature * 2 > picked.len();
        let mut tiles = Vec::new();
        let mut gates = Vec::new();
        let mut intro = Vec::new();
        let mut used_meanings: HashSet<String> = HashSet::new();
        for (k, (w, is_new)) in picked.iter().enumerate() {
            let meaning = w.meaning(base, latam).to_string();
            if !used_meanings.insert(meaning.to_lowercase()) {
                continue;
            }
            let key = k as u16;
            let es = w.with_article();
            let explain = format!("{} = {meaning}{}", w.display(), w.pic.as_ref().map(|p| format!(" {p}")).unwrap_or_default());
            if *is_new {
                intro.push(Intro { es: w.display(), say: w.with_article(), meaning: meaning.clone(), pic: w.pic.clone(), note: w.note_for(base) });
            }
            if production {
                tiles.push(Tile { text: meaning.clone(), key, item: Some(w.id()), say: Some(w.lemma.clone()), ask: None, hidden: false, explain, width: width_for(&meaning), band: None });
                gates.push(Gate { text: es.clone(), key, pic: None, item: None, long: es.chars().count() > 12 });
            } else {
                tiles.push(Tile { text: es.clone(), key, item: Some(w.id()), say: Some(w.lemma.clone()), ask: Some(w.lemma.clone()), hidden: listening, explain, width: if listening { 2 } else { width_for(&es) }, band: None });
                gates.push(Gate { text: meaning.clone(), key, pic: w.pic.clone(), item: None, long: meaning.chars().count() > 12 });
            }
        }
        // One or two distractor doors: same part of speech, preferably the same theme.
        let chosen: HashSet<String> = picked.iter().map(|(w, _)| w.id()).collect();
        let pos: Vec<String> = picked.iter().map(|(w, _)| w.pos.clone()).collect();
        let themes: Vec<String> = picked.iter().map(|(w, _)| w.theme.clone()).collect();
        let mut cands: Vec<&Word> = pool.iter().filter(|w| !chosen.contains(&w.id()) && pos.contains(&w.pos)).collect();
        cands.sort_by_key(|w| if themes.contains(&w.theme) { 0 } else { 1 });
        cands.truncate(40);
        let mut cands: Vec<Word> = cands.into_iter().cloned().collect();
        self.shuffle(&mut cands);
        let mut extra = 0;
        let want = if tiles.len() <= 2 { 2 } else { 1 };
        for w in cands {
            if extra >= want {
                break;
            }
            let meaning = w.meaning(base, latam).to_string();
            if !used_meanings.insert(meaning.to_lowercase()) {
                continue;
            }
            let key = 100 + extra as u16;
            if production {
                let es = w.with_article();
                gates.push(Gate { text: es.clone(), key, pic: None, item: None, long: es.chars().count() > 12 });
            } else {
                gates.push(Gate { text: meaning.clone(), key, pic: w.pic.clone(), item: None, long: meaning.chars().count() > 12 });
            }
            extra += 1;
        }
        self.shuffle(&mut gates);
        Some(Round { mode, tiles, gates, intro, production, heading: String::new() })
    }

    fn gender(&mut self) -> Option<Round> {
        let fr = self.fr();
        let (base, latam) = (self.base, self.latam);
        let mut pool: Vec<Word> = self.content.gender_traps.iter().filter(|w| w.band <= self.band && !w.meaning(base, latam).is_empty()).cloned().collect();
        pool.extend(self.word_pool(|w| w.pos == "n" && matches!(w.gender, Some(Gender::M) | Some(Gender::F)) && !w.note.contains("el agua") && !w.plural() && !w.lemma.contains(' ')));
        let id = |w: &Word| format!("g:{}", w.lemma);
        let picked = self.pick(&pool, id, 6, 3);
        let tiles: Vec<Tile> = picked
            .iter()
            .map(|(w, _)| {
                let key = if w.gender == Some(Gender::F) { 1 } else { 0 };
                let (said, mut explain) = if fr {
                    let un = w.with_indefinite();
                    let le = w.with_article();
                    let shown = if le.starts_with("l'") { format!("{un} · {le}") } else { un.clone() };
                    (un, format!("{shown} = {}", w.meaning(base, latam)))
                } else {
                    let art = if key == 1 { "la" } else { "el" };
                    (format!("{art} {}", w.lemma), format!("{art} {} = {}", w.lemma, w.meaning(base, latam)))
                };
                let note: Vec<&str> = w.note.split(';').map(|p| p.trim()).filter(|p| !p.is_empty() && !p.contains("h aspiré")).collect();
                let note = crate::content::note_for(&note.join("; "), self.target(), base);
                if !note.is_empty() {
                    explain.push_str(&format!(" · {note}"));
                }
                Tile { text: w.lemma.clone(), key, item: Some(format!("g:{}", w.lemma)), say: Some(said), ask: Some(w.lemma.clone()), hidden: false, explain, width: width_for(&w.lemma), band: None }
            })
            .collect();
        let (a, b) = if fr { ("un", "une") } else { ("el", "la") };
        let gates = vec![Gate { text: a.into(), key: 0, pic: None, item: None, long: false }, Gate { text: b.into(), key: 1, pic: None, item: None, long: false }];
        Some(Round { mode: Mode::Gender, tiles, gates, intro: Vec::new(), production: false, heading: String::new() })
    }

    fn tense_for_band(&mut self) -> VTense {
        if self.fr() {
            use conj_fr::Tense as T;
            let options: &[T] = match self.band {
                Band::A1 => &[T::Present],
                Band::A2 => &[T::Present, T::PasseCompose, T::Imparfait],
                Band::B1 => &[T::PasseCompose, T::Imparfait, T::Futur, T::Conditionnel, T::Subjonctif],
                Band::B2 => &[T::Subjonctif, T::Conditionnel, T::Futur, T::PasseCompose, T::Imparfait],
            };
            return VTense::Fr(options[self.rnd(options.len())]);
        }
        use conj::Tense as T;
        let options: &[T] = match self.band {
            Band::A1 => &[T::Present],
            Band::A2 => &[T::Present, T::Preterite, T::Imperfect],
            Band::B1 => &[T::Preterite, T::Imperfect, T::Future, T::PresentSubjunctive],
            Band::B2 => &[T::PresentSubjunctive, T::Conditional, T::ImperfectSubjunctive, T::Future, T::Preterite],
        };
        VTense::Es(options[self.rnd(options.len())])
    }

    fn verbs(&mut self) -> Option<Round> {
        let tense = self.tense_for_band();
        let fr = self.fr();
        let pool: Vec<Word> = self.word_pool(|w| {
            w.pos == "v" && if fr { !conj_fr::impersonal(&w.lemma) && (!w.lemma.contains(' ') || w.lemma.starts_with("se ")) } else { !w.lemma.contains(' ') }
        });
        let id = |w: &Word| format!("c:{}:{}", w.lemma, tense.name());
        let picked = self.pick(&pool, id, 1, 1);
        let (verb, _) = picked.first()?.clone();
        // Persons with the same form share a door (yo / él in the imperfect, je / il / elle for *parle*).
        let groups = paradigm(&verb.lemma, tense, self.latam)?;
        let item = format!("c:{}:{}", verb.lemma, tense.name());
        // Four persons per round (four doors), three of them with their form: one door stays empty, so the last
        // form cannot be placed by elimination.
        let mut idx: Vec<usize> = (0..groups.len()).collect();
        self.shuffle(&mut idx);
        idx.truncate(4);
        let n_tiles = idx.len().saturating_sub(1).max(1);
        let mut tiles = Vec::new();
        for &g in idx.iter().take(n_tiles) {
            let (form, label, said) = &groups[g];
            tiles.push(Tile {
                text: form.clone(),
                key: g as u16,
                item: Some(item.clone()),
                say: Some(said.clone()),
                ask: Some(if fr { conj_fr::spoken(form) } else { form.clone() }),
                hidden: false,
                explain: format!("{label} → {form} ({}, {})", verb.lemma, tense.name()),
                width: width_for(form),
                band: None,
            });
        }
        let gates: Vec<Gate> = idx.iter().map(|&g| Gate { text: groups[g].1.clone(), key: g as u16, pic: None, item: None, long: false }).collect();
        let heading = format!("{} · {}", verb.lemma, tense.name());
        let intro = if self.memory.card(&item).is_none() && !self.memory.cards.contains_key(&verb.id()) {
            vec![Intro { es: verb.lemma.clone(), say: verb.lemma.clone(), meaning: verb.meaning(self.base, self.latam).to_string(), pic: verb.pic.clone(), note: verb.note_for(self.base) }]
        } else {
            Vec::new()
        };
        Some(Round { mode: Mode::Verbs, tiles, gates, intro, production: false, heading })
    }

    fn binary(&mut self, mode: Mode) -> Option<Round> {
        let (pool, prefix, a, b): (Vec<Choice>, &str, &str, &str) = match mode {
            Mode::SerEstar => (self.content.binary_a.clone(), "se", "ser", "estar"),
            Mode::PorPara => (self.content.binary_b.clone(), "pp", "por", "para"),
            _ => (self.content.binary_a.clone(), "ea", "être", "avoir"),
        };
        let (base, latam) = (self.base, self.latam);
        let mut pool: Vec<Choice> = pool.into_iter().filter(|c| c.band <= self.band.max(Band::B1) && !c.translation(base, latam).is_empty()).collect();
        self.shuffle(&mut pool);
        pool.sort_by_key(|c| c.band);
        let picked = self.pick(&pool, |c| hash_id(prefix, &c.prompt), 5, 3);
        let tiles = picked
            .iter()
            .map(|(c, _)| {
                let key = if c.answer == a { 0 } else { 1 };
                Tile {
                    text: c.prompt.clone(),
                    key,
                    item: Some(hash_id(prefix, &c.prompt)),
                    say: Some(c.filled()),
                    ask: Some(spoken_gap(&c.prompt)),
                    hidden: false,
                    explain: format!("{} — {} ({})", c.filled(), c.translation(base, latam), c.why),
                    width: 3,
                    band: None,
                }
            })
            .collect();
        let gates = vec![Gate { text: a.into(), key: 0, pic: None, item: None, long: false }, Gate { text: b.into(), key: 1, pic: None, item: None, long: false }];
        Some(Round { mode, tiles, gates, intro: Vec::new(), production: false, heading: String::new() })
    }

    fn sentences(&mut self, mode: Mode) -> Option<Round> {
        let (pool, prefix) = match mode {
            Mode::Past => (self.content.past.clone(), "pi"),
            Mode::Partitive => (self.content.partitive.clone(), "pa"),
            _ => (self.content.subjunctive.clone(), "sj"),
        };
        let (base, latam) = (self.base, self.latam);
        let mut pool: Vec<Choice> = pool.into_iter().filter(|c| c.band <= self.band.max(Band::B1) && !c.translation(base, latam).is_empty()).collect();
        self.shuffle(&mut pool);
        // Two sentences per round, each with its right form and the competing one (the other tense or mood, another
        // article): two real choices, at most four tiles and three doors — a board size proven free of dead ends
        // (`fuera_core::lattice`).
        let picked = self.pick(&pool, |c| hash_id(prefix, &c.prompt), 2, 2);
        let mut tiles = Vec::new();
        let mut gates = Vec::new();
        let mut seen_forms: HashSet<String> = HashSet::new();
        for (k, (c, _)) in picked.iter().enumerate() {
            let id = hash_id(prefix, &c.prompt);
            let explain = format!("{} — {} ({})", c.filled(), c.translation(base, latam), c.why);
            let inf = c.infinitive().map(|i| format!(" ({i})")).unwrap_or_default();
            gates.push(Gate { text: format!("{}{inf}", c.sentence()), key: k as u16, pic: None, item: Some(id.clone()), long: true });
            if seen_forms.insert(c.answer.clone()) {
                tiles.push(Tile { text: c.answer.clone(), key: k as u16, item: Some(id.clone()), say: Some(c.filled()), ask: Some(c.answer.clone()), hidden: false, explain: explain.clone(), width: width_for(&c.answer), band: None });
            }
            if !c.other.is_empty() && seen_forms.insert(c.other.clone()) {
                tiles.push(Tile { text: c.other.clone(), key: 99, item: None, say: None, ask: Some(c.other.clone()), hidden: false, explain, width: width_for(&c.other), band: None });
            }
        }
        if tiles.iter().all(|t| t.key != 99) {
            return Some(Round { mode, tiles, gates, intro: Vec::new(), production: false, heading: String::new() });
        }
        // The bin for the forms that fit none of the sentences.
        let none = if self.fr() { "aucune" } else { "ninguna" };
        gates.push(Gate { text: none.into(), key: 99, pic: None, item: None, long: false });
        Some(Round { mode, tiles, gates, intro: Vec::new(), production: false, heading: String::new() })
    }

    fn false_friends(&mut self) -> Option<Round> {
        let (base, latam) = (self.base, self.latam);
        let mut pool: Vec<_> = self.content.false_friends.iter().filter(|f| !f.trap(base, latam).is_empty() && !f.truth(base, latam).is_empty() && f.band <= self.band.max(Band::B1)).cloned().collect();
        self.shuffle(&mut pool);
        let picked = self.pick(&pool, |f| format!("ff:{}", f.word), 2, 2);
        let mut tiles = Vec::new();
        let mut gates = Vec::new();
        for (k, (f, _)) in picked.iter().enumerate() {
            let key = k as u16;
            let (truth, trap) = (f.truth(base, latam).to_string(), f.trap(base, latam).to_string());
            let note = crate::content::note_for(&f.note, self.target(), base);
            let explain = format!("{} = {truth} (≠ {trap}){}", f.word, if note.is_empty() { String::new() } else { format!(" · {note}") });
            tiles.push(Tile { text: f.word.clone(), key, item: Some(format!("ff:{}", f.word)), say: Some(f.word.clone()), ask: Some(f.word.clone()), hidden: false, explain, width: width_for(&f.word), band: None });
            gates.push(Gate { text: truth.clone(), key, pic: None, item: None, long: truth.chars().count() > 12 });
            gates.push(Gate { text: trap.clone(), key: 100 + key, pic: None, item: None, long: trap.chars().count() > 12 });
        }
        if tiles.is_empty() {
            return None;
        }
        self.shuffle(&mut gates);
        Some(Round { mode: Mode::FalseFriends, tiles, gates, intro: Vec::new(), production: false, heading: String::new() })
    }

    fn placement(&mut self) -> Option<Round> {
        let (base, latam) = (self.base, self.latam);
        let mut tiles = Vec::new();
        for band in Band::ALL {
            let mut ws: Vec<&Word> = self.content.words.iter().filter(|w| w.band == band && (w.pos == "n" || w.pos == "v") && !w.lemma.contains(' ')).collect();
            if ws.is_empty() {
                continue;
            }
            // Skip the very first words of A1 (too easy to be informative).
            let start = ws.len() / 5;
            ws.drain(..start.min(ws.len().saturating_sub(1)));
            let w = ws[self.rnd(ws.len())];
            tiles.push(Tile { text: w.lemma.clone(), key: 0, item: None, say: None, ask: None, hidden: false, explain: format!("{} = {}", w.display(), w.meaning(base, latam)), width: width_for(&w.lemma), band: Some(band) });
        }
        let none = if self.fr() { "n'existe pas" } else { "no existe" };
        // Two invented words per round (6 tiles: boards stay small enough to be solved quickly).
        let mut added = 0;
        while added < 2 && self.content.pseudo.len() >= 2 {
            let p = self.content.pseudo[self.rnd(self.content.pseudo.len())].clone();
            if tiles.iter().any(|t: &Tile| t.text == p) {
                continue;
            }
            tiles.push(Tile { text: p.clone(), key: 1, item: None, say: None, ask: None, hidden: false, explain: format!("« {p} » {none}"), width: width_for(&p), band: None });
            added += 1;
        }
        self.shuffle(&mut tiles);
        let gates = vec![Gate { text: "existe".into(), key: 0, pic: None, item: None, long: false }, Gate { text: none.into(), key: 1, pic: None, item: None, long: false }];
        Some(Round { mode: Mode::Placement, tiles, gates, intro: Vec::new(), production: false, heading: String::new() })
    }
}

/// Estimates the working band from placement answers: (band of the real word or None, answered right).
pub fn place(answers: &[(Option<Band>, bool)]) -> Band {
    let pseudo: Vec<bool> = answers.iter().filter(|a| a.0.is_none()).map(|a| a.1).collect();
    let false_alarms = pseudo.iter().filter(|ok| !**ok).count() as f32 / pseudo.len().max(1) as f32;
    let mut best = Band::A1;
    for b in Band::ALL {
        let real: Vec<bool> = answers.iter().filter(|a| a.0 == Some(b)).map(|a| a.1).collect();
        if real.is_empty() {
            continue;
        }
        let hit = real.iter().filter(|x| **x).count() as f32 / real.len() as f32;
        // Corrected for guessing (yes to invented words).
        if hit - false_alarms >= 0.6 {
            best = b;
        } else {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::Course;

    const COURSES: [Course; 3] = [Course::SPAIN, Course::LATAM, Course::FRENCH];

    #[test]
    fn every_mode_plans_a_round() {
        for course in COURSES {
            let content = Content::load(course);
            assert!(content.words.len() > 1500, "{course:?}: {}", content.words.len());
            let mem = Memory::default();
            for band in Band::ALL {
                for base in course.target.bases() {
                    let mut p = Planner::new(&content, &mem, base, band, 20_000.0, 7);
                    for &m in Mode::menu(course.target).iter().chain([&Mode::Placement]) {
                        if m.min_band(course.target) > band {
                            continue;
                        }
                        let r = p.plan(m).unwrap_or_else(|| panic!("{course:?}: no round for {m:?} at {band:?}"));
                        assert!(!r.tiles.is_empty(), "{m:?}");
                        assert!(r.gates.len() >= 2, "{m:?}");
                        // Every tile has a door that accepts it.
                        for t in &r.tiles {
                            assert!(r.gates.iter().any(|g| g.key == t.key), "{m:?}: tile {} has no door", t.text);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn tapping_a_tile_never_says_the_answer() {
        for course in COURSES {
            let content = Content::load(course);
            let mem = Memory::default();
            let mut p = Planner::new(&content, &mem, course.target.bases()[0], Band::B2, 20_000.0, 3);
            for _ in 0..30 {
                for &m in Mode::menu(course.target) {
                    let Some(r) = p.plan(m) else { continue };
                    for t in &r.tiles {
                        let Some(ask) = &t.ask else { continue };
                        assert!(!ask.contains('_'), "{m:?}: {ask}");
                        match m {
                            // Not the article, not the person, not the filled-in sentence.
                            Mode::Gender => assert!(!["el ", "la ", "le ", "un ", "une ", "l'"].iter().any(|a| ask.starts_with(a)), "{ask}"),
                            Mode::Verbs => assert!(ask == &t.text || *ask == conj_fr::spoken(&t.text), "{ask}"),
                            Mode::SerEstar | Mode::PorPara | Mode::Auxiliary => assert_ne!(Some(ask), t.say.as_ref()),
                            Mode::Past | Mode::Subjunctive | Mode::Partitive => assert_eq!(ask, &t.text),
                            _ => {}
                        }
                        // A tile showing a meaning must not say the word it is looking for.
                        if !t.hidden && r.gates.iter().any(|g| g.key == t.key && Some(&g.text) == t.say.as_ref()) {
                            panic!("{m:?}: tap would read the door {ask}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn latin_america_has_no_vosotros() {
        let content = Content::load(Course::LATAM);
        assert!(content.words.iter().all(|w| w.lemma != "vosotros" && w.lemma != "coche"));
        assert!(content.words.iter().any(|w| w.lemma == "carro"));
        let g = paradigm("hablar", VTense::Es(conj::Tense::Present), true).unwrap();
        assert_eq!(g.len(), 5);
        assert!(g.iter().any(|(f, l, _)| f == "hablan" && l.contains("ustedes")));
        // The French course shows Latin-American glosses to a Latin-American learner.
        let fr = Content::load(crate::content::Course::FRENCH);
        let m0 = Memory::default();
        let p = Planner::new(&fr, &m0, Base::Es, Band::A1, 20_000.0, 1).with_latam(true);
        let w = fr.words.iter().find(|w| w.lemma == "voiture").unwrap();
        assert_eq!(w.meaning(p.base, p.latam), "carro");
        let mem = Memory::default();
        let mut p = Planner::new(&content, &mem, Base::En, Band::B2, 20_000.0, 11);
        for _ in 0..40 {
            for m in [Mode::Past, Mode::Subjunctive, Mode::SerEstar, Mode::PorPara] {
                let r = p.plan(m).unwrap();
                for t in &r.tiles {
                    assert!(!t.text.ends_with("áis") && !t.text.ends_with("éis"), "{m:?}: {}", t.text);
                }
            }
        }
    }

    #[test]
    fn placement_estimate() {
        let a = [(Some(Band::A1), true), (Some(Band::A2), true), (Some(Band::B1), true), (Some(Band::B2), false), (None, true), (None, true)];
        assert_eq!(place(&a), Band::B1);
        let all_yes = [(Some(Band::A1), true), (Some(Band::A2), true), (Some(Band::B1), true), (Some(Band::B2), true), (None, false), (None, false)];
        assert_eq!(place(&all_yes), Band::A1);
    }
}

/// Text for an item in "Mi cuaderno", and what to speak.
/// `latam`: Spanish glosses as in Latin America (the variant setting; for the Spanish course, its own variant).
pub fn describe(content: &Content, id: &str, base: Base, latam: bool) -> Option<(String, String)> {
    let fr = content.course.target == Target::Fr;
    let (kind, rest) = id.split_once(':')?;
    match kind {
        "w" => {
            let w = content.word(id)?;
            Some((format!("{} = {}", w.display(), w.meaning(base, latam)), w.lemma.clone()))
        }
        "g" => {
            let w = content.gender_traps.iter().chain(content.words.iter().filter(|w| w.pos == "n")).find(|w| w.lemma == rest)?;
            if fr {
                Some((format!("{} = {}", w.with_indefinite(), w.meaning(base, latam)), w.with_indefinite()))
            } else {
                Some((format!("{} = {}", w.with_article(), w.meaning(base, latam)), w.with_article()))
            }
        }
        "c" => {
            let (inf, tname) = rest.rsplit_once(':')?;
            let tense = VTense::parse(tname)?;
            let groups = paradigm(inf, tense, latam)?;
            let forms: Vec<String> = groups.iter().map(|(f, l, _)| format!("{l} {f}")).collect();
            Some((format!("{inf} ({tname}): {}", forms.join(", ")), groups.first().map(|g| g.2.clone()).unwrap_or_default()))
        }
        "se" | "pp" | "pi" | "sj" | "ea" | "pa" => {
            let list = match kind {
                "se" | "ea" => &content.binary_a,
                "pp" => &content.binary_b,
                "pi" => &content.past,
                "pa" => &content.partitive,
                _ => &content.subjunctive,
            };
            let c = list.iter().find(|c| hash_id(kind, &c.prompt) == id)?;
            Some((format!("{} — {}", c.filled(), c.translation(base, latam)), c.filled()))
        }
        "ff" => {
            let f = content.false_friends.iter().find(|f| f.word == rest)?;
            Some((format!("{} = {} (≠ {})", f.word, f.truth(base, latam), f.trap(base, latam)), f.word.clone()))
        }
        _ => None,
    }
}
