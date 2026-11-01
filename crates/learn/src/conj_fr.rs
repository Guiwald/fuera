//! French verb conjugator (France, traditional spelling).
//!
//! Six persons (*je, tu, il/elle, nous, vous, ils/elles*) and the six tenses a learner drills: présent, passé
//! composé, imparfait, futur simple, conditionnel présent and subjonctif présent. Spelling is the traditional one:
//! *il cédera*, *il renouvelle*, *il connaît* (the 1990 variants *cèdera*, *renouvèle*, *connait* are not produced).
//!
//! The engine is rule based:
//! * first group (*-er*): *-cer*/*-ger* spellings (*commençons*, *mangeons*), *e → è* (*lève*, *lèverai*), doubled
//!   consonant for *-eler*/*-eter* (*appelle*, *jette*), *é → è* before a mute ending (*cède*, but *céderai*),
//!   *y → i* (*nettoie*, *paie*), *envoyer* (*enverrai*), *aller*;
//! * second group (*finir*, *finissons*), with a list of the *-ir* verbs that are not in it;
//! * third group by models (*partir*, *ouvrir*, *venir*, *prendre*, *mettre*, *craindre*, *connaître*, *conduire*,
//!   *dire*, *écrire*…), prefixed compounds conjugated from their root (*devenir*, *comprendre*, *permettre*…);
//! * the passé composé with *avoir* or *être*; with *être* the participle shows its agreement as grammars write it
//!   (*suis allé(e)*, *sommes allé(e)s*, *êtes allé(e)(s)*);
//! * pronominal verbs (*se lever*, *s'appeler*) with the clitic in front and *être* in the passé composé.
//!
//! Forms are given without the subject pronoun (*parle*, *ai parlé*, *m'appelle*); [`with_subject`] adds it
//! (*je parle*, *j'ai parlé*, *je m'appelle*).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Person {
    Je,
    Tu,
    Il,
    Nous,
    Vous,
    Ils,
}

impl Person {
    pub const ALL: [Person; 6] = [Person::Je, Person::Tu, Person::Il, Person::Nous, Person::Vous, Person::Ils];

    /// Label on a door: `je`, `tu`, `il / elle`, `nous`, `vous`, `ils / elles`.
    pub fn label(self) -> &'static str {
        match self {
            Person::Je => "je",
            Person::Tu => "tu",
            Person::Il => "il / elle",
            Person::Nous => "nous",
            Person::Vous => "vous",
            Person::Ils => "ils / elles",
        }
    }

    /// Subject pronoun used when the form is read aloud.
    pub fn pronoun(self) -> &'static str {
        match self {
            Person::Je => "je",
            Person::Tu => "tu",
            Person::Il => "il",
            Person::Nous => "nous",
            Person::Vous => "vous",
            Person::Ils => "ils",
        }
    }

    fn i(self) -> usize {
        self as usize
    }

    fn clitic(self) -> &'static str {
        match self {
            Person::Je => "me",
            Person::Tu => "te",
            Person::Il | Person::Ils => "se",
            Person::Nous => "nous",
            Person::Vous => "vous",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    PasseCompose,
    Imparfait,
    Futur,
    Conditionnel,
    Subjonctif,
}

impl Tense {
    pub const ALL: [Tense; 6] = [Tense::Present, Tense::PasseCompose, Tense::Imparfait, Tense::Futur, Tense::Conditionnel, Tense::Subjonctif];

    /// French name as used in French classrooms.
    pub fn name(self) -> &'static str {
        match self {
            Tense::Present => "présent",
            Tense::PasseCompose => "passé composé",
            Tense::Imparfait => "imparfait",
            Tense::Futur => "futur simple",
            Tense::Conditionnel => "conditionnel présent",
            Tense::Subjonctif => "subjonctif présent",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aux {
    Avoir,
    Etre,
}

/// Verbs whose *h* is aspirated: no elision (*je hais*, *je hurle*).
const H_ASPIRE: &[&str] = &[
    "haïr", "hurler", "heurter", "hacher", "haleter", "hanter", "harceler", "hasarder", "hausser", "héler", "hennir", "hérisser", "hisser", "hocher",
    "honnir", "huer", "hâter", "harasser", "hurler", "hiérarchiser", "handicaper", "harponner", "heurter", "houspiller",
];

/// Intransitive verbs conjugated with *être* (with their prefixed forms listed).
const ETRE: &[&str] = &[
    "aller", "venir", "devenir", "revenir", "redevenir", "parvenir", "intervenir", "survenir", "provenir", "arriver", "partir", "repartir", "entrer", "rentrer",
    "sortir", "ressortir", "monter", "remonter", "descendre", "redescendre", "naître", "renaître", "mourir", "rester", "tomber", "retomber",
    "retourner", "apparaître", "décéder", "éclore",
];

/// *-ir* verbs that are not second group, with their model.
const IR_MODELS: &[(&str, Model)] = &[
    ("partir", Model::Partir),
    ("sortir", Model::Partir),
    ("dormir", Model::Partir),
    ("servir", Model::Partir),
    ("sentir", Model::Partir),
    ("mentir", Model::Partir),
    ("ouvrir", Model::Ouvrir),
    ("couvrir", Model::Ouvrir),
    ("offrir", Model::Ouvrir),
    ("souffrir", Model::Ouvrir),
    ("cueillir", Model::Cueillir),
    ("accueillir", Model::Cueillir),
    ("recueillir", Model::Cueillir),
    ("assaillir", Model::Cueillir),
    ("tressaillir", Model::Cueillir),
    ("venir", Model::Venir),
    ("tenir", Model::Venir),
    ("courir", Model::Courir),
    ("mourir", Model::Mourir),
    ("acquérir", Model::Acquerir),
    ("conquérir", Model::Acquerir),
    ("requérir", Model::Acquerir),
    ("fuir", Model::Fuir),
    ("vêtir", Model::Vetir),
    ("bouillir", Model::Bouillir),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Model {
    /// First group (*parler*).
    Er,
    /// Second group (*finir*).
    Finir,
    Partir,
    Ouvrir,
    Cueillir,
    Venir,
    Courir,
    Mourir,
    Acquerir,
    Fuir,
    Vetir,
    Bouillir,
    Hair,
    /// *-dre* (*rendre*): *rends, rend, rendons*.
    Rendre,
    Rompre,
    Battre,
    Mettre,
    Prendre,
    /// *-indre* (*craindre, peindre, joindre*).
    Craindre,
    /// *-aître* (*connaître, paraître*).
    Connaitre,
    Naitre,
    /// *-uire* (*conduire*).
    Conduire,
    Croire,
    Boire,
    Dire,
    Lire,
    Ecrire,
    Rire,
    Suivre,
    Vivre,
    Faire,
    Plaire,
    Taire,
    Conclure,
    Vaincre,
    Resoudre,
    Traire,
    Coudre,
    Moudre,
    Croitre,
    Etre,
    Avoir,
    Aller,
    Voir,
    Recevoir,
    Devoir,
    Pouvoir,
    Vouloir,
    Savoir,
    Valoir,
    Falloir,
    Pleuvoir,
    Asseoir,
    Mouvoir,
}

/// Roots of the third group with their model, longest first when searched; a verb ending in a root (with a prefix
/// in front) follows it: *devenir* (venir), *comprendre* (prendre), *permettre* (mettre).
const ROOTS: &[(&str, Model)] = &[
    ("être", Model::Etre),
    ("avoir", Model::Avoir),
    ("aller", Model::Aller),
    ("prendre", Model::Prendre),
    ("mettre", Model::Mettre),
    ("battre", Model::Battre),
    ("rompre", Model::Rompre),
    ("aindre", Model::Craindre),
    ("eindre", Model::Craindre),
    ("oindre", Model::Craindre),
    ("aître", Model::Connaitre),
    ("croître", Model::Croitre),
    ("uire", Model::Conduire),
    ("suffire", Model::Conduire),
    ("croire", Model::Croire),
    ("boire", Model::Boire),
    ("dire", Model::Dire),
    ("lire", Model::Lire),
    ("crire", Model::Ecrire),
    ("rire", Model::Rire),
    ("suivre", Model::Suivre),
    ("vivre", Model::Vivre),
    ("faire", Model::Faire),
    ("plaire", Model::Plaire),
    ("taire", Model::Taire),
    ("clure", Model::Conclure),
    ("vaincre", Model::Vaincre),
    ("soudre", Model::Resoudre),
    ("traire", Model::Traire),
    ("coudre", Model::Coudre),
    ("moudre", Model::Moudre),
    ("dre", Model::Rendre),
    ("voir", Model::Voir),
    ("cevoir", Model::Recevoir),
    ("devoir", Model::Devoir),
    ("pouvoir", Model::Pouvoir),
    ("vouloir", Model::Vouloir),
    ("savoir", Model::Savoir),
    ("valoir", Model::Valoir),
    ("falloir", Model::Falloir),
    ("pleuvoir", Model::Pleuvoir),
    ("asseoir", Model::Asseoir),
    ("mouvoir", Model::Mouvoir),
    ("haïr", Model::Hair),
];

/// *-eler*/*-eter* verbs that take *è* instead of doubling the consonant.
const ELER_GRAVE: &[&str] = &[
    "acheter", "racheter", "geler", "congeler", "dégeler", "surgeler", "peler", "modeler", "harceler", "ciseler", "démanteler", "marteler", "haleter",
    "crocheter", "fureter", "celer", "déceler", "receler", "écarteler", "corseter",
];

fn ends(s: &str, suf: &str) -> bool {
    s.ends_with(suf)
}

fn cut(s: &str, n: usize) -> &str {
    let mut idx = s.len();
    for _ in 0..n {
        idx = s[..idx].char_indices().last().map(|(i, _)| i).unwrap_or(0);
    }
    &s[..idx]
}

fn is_vowel(c: char) -> bool {
    "aeiouyâàäéèêëîïôöûùüœæ".contains(c)
}

/// Whether a word starts with a sound that takes elision (*j'aime*, *m'appelle*).
fn elides(word: &str, verb: &str) -> bool {
    let Some(c) = word.chars().next() else { return false };
    if is_vowel(c) && c != 'y' {
        return true;
    }
    c == 'h' && !H_ASPIRE.contains(&verb)
}

fn model_of(inf: &str) -> Option<Model> {
    if inf == "aller" {
        return Some(Model::Aller);
    }
    if ends(inf, "er") {
        return Some(Model::Er);
    }
    if inf == "haïr" {
        return Some(Model::Hair);
    }
    if inf == "naître" || inf == "renaître" {
        return Some(Model::Naitre);
    }
    if ends(inf, "ir") && !ends(inf, "oir") {
        if SECOND_GROUP_TIR.contains(&inf) {
            return Some(Model::Finir);
        }
        // Third-group -ir verbs, also with a prefix (devenir, découvrir, endormir…).
        let mut best: Option<(&str, Model)> = None;
        for (root, m) in IR_MODELS {
            if ends(inf, root) && best.is_none_or(|(b, _)| root.len() > b.len()) {
                best = Some((root, *m));
            }
        }
        return Some(best.map(|(_, m)| m).unwrap_or(Model::Finir));
    }
    let mut best: Option<(&str, Model)> = None;
    for (root, m) in ROOTS {
        if ends(inf, root) && best.is_none_or(|(b, _)| root.len() > b.len()) {
            best = Some((root, *m));
        }
    }
    best.map(|(_, m)| m)
}

/// Second-group verbs that end like a third-group root.
const SECOND_GROUP_TIR: &[&str] = &[
    "aboutir", "abrutir", "aplatir", "anéantir", "amortir", "assortir", "avertir", "bâtir", "convertir", "divertir", "garantir", "engloutir",
    "investir", "pervertir", "ralentir", "répartir", "retentir", "rôtir", "sertir", "subvertir", "travestir", "nantir", "lotir", "abêtir", "aguerrir",
    "appauvrir", "asservir", "ouvrager", "atterrir", "enrichir", "nourrir", "pourrir", "guérir", "périr", "chérir", "fleurir",
];

/// Splits a pronominal infinitive: `se lever` → (`lever`, true), `s'appeler` → (`appeler`, true).
fn split_pronominal(inf: &str) -> (&str, bool) {
    if let Some(v) = inf.strip_prefix("se ") {
        (v, true)
    } else if let Some(v) = inf.strip_prefix("s'").or_else(|| inf.strip_prefix("s’")) {
        (v, true)
    } else {
        (inf, false)
    }
}

/// First-group stem changes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ErKind {
    Plain,
    /// *lever*: e → è before a mute ending and in the future.
    EGrave,
    /// *appeler*: consonant doubled before a mute ending and in the future.
    Double,
    /// *céder*: é → è before a mute ending (future keeps é in the traditional spelling).
    EAcute,
    /// *nettoyer*, *essuyer*, *payer*: y → i before a mute ending and in the future.
    Yi,
}

/// For a first-group stem (`lev`, `appel`, `céd`): which change applies.
fn er_kind(inf: &str, stem: &str) -> ErKind {
    if ends(stem, "oy") || ends(stem, "uy") || ends(stem, "ay") {
        return ErKind::Yi;
    }
    let chars: Vec<char> = stem.chars().collect();
    // The last vowel of the stem and what follows it (the u of gu/qu is no vowel: lèguer → lègue, dissèque).
    let Some(vi) = last_vowel(&chars) else { return ErKind::Plain };
    let after = &chars[vi + 1..];
    if after.is_empty() {
        return ErKind::Plain;
    }
    // Only one consonant sound (digraphs ch, gn, ph, th, qu, gu count as one: sécher, régner, disséquer), or a consonant + r/l (sevrer, célébrer, régler). Never x (vexer).
    let after_s: String = after.iter().collect();
    let digraph = matches!(after_s.as_str(), "ch" | "gn" | "ph" | "th" | "qu" | "gu");
    let ok_cluster = !after.contains(&'x') && (after.len() == 1 || digraph || after.len() == 2 && matches!(after[1], 'r' | 'l') && !matches!(after[0], 'r' | 'l'));
    let v = chars[vi];
    if !ok_cluster {
        return ErKind::Plain;
    }
    match v {
        'e' => {
            if (ends(inf, "eler") || ends(inf, "eter")) && !ELER_GRAVE.contains(&inf) {
                ErKind::Double
            } else {
                ErKind::EGrave
            }
        }
        'é' => ErKind::EAcute,
        _ => ErKind::Plain,
    }
}

/// Index of the last vowel of a stem; the u of gu/qu is no vowel (léguer → lègue).
fn last_vowel(chars: &[char]) -> Option<usize> {
    (0..chars.len()).rev().find(|&i| is_vowel(chars[i]) && !(chars[i] == 'u' && i > 0 && matches!(chars[i - 1], 'g' | 'q')))
}

/// The stem before a mute ending (*-e, -es, -ent*, and the future for some kinds).
fn er_mute_stem(stem: &str, kind: ErKind) -> String {
    match kind {
        ErKind::Plain => stem.to_string(),
        ErKind::EGrave | ErKind::EAcute => {
            let chars: Vec<char> = stem.chars().collect();
            let vi = last_vowel(&chars).unwrap_or(0);
            let mut c = chars.clone();
            c[vi] = 'è';
            c.into_iter().collect()
        }
        ErKind::Double => {
            let last = stem.chars().last().unwrap_or('l');
            format!("{stem}{last}")
        }
        ErKind::Yi => format!("{}i", cut(stem, 1)),
    }
}

/// -cer / -ger before an ending starting with a or o.
fn er_soft(stem: &str, ending: &str) -> String {
    let hard = ending.starts_with('a') || ending.starts_with('o') || ending.starts_with('â');
    if hard && ends(stem, "c") {
        format!("{}ç{ending}", cut(stem, 1))
    } else if hard && ends(stem, "g") {
        format!("{stem}e{ending}")
    } else {
        format!("{stem}{ending}")
    }
}

fn er_present(inf: &str, p: Person) -> String {
    let stem = cut(inf, 2);
    let kind = er_kind(inf, stem);
    let end = ["e", "es", "e", "ons", "ez", "ent"][p.i()];
    match p {
        Person::Nous | Person::Vous => er_soft(stem, end),
        _ => format!("{}{end}", er_mute_stem(stem, kind)),
    }
}

fn er_future_stem(inf: &str) -> String {
    if inf == "envoyer" || inf == "renvoyer" {
        return format!("{}err", cut(inf, 4));
    }
    let stem = cut(inf, 2);
    match er_kind(inf, stem) {
        ErKind::EAcute | ErKind::Plain => inf.to_string(),
        k => format!("{}er", er_mute_stem(stem, k)),
    }
}

/// Present tense (without pronoun, without clitic) for the non-first-group models.
fn present(inf: &str, m: Model, p: Person) -> Option<String> {
    let i = p.i();
    let six = |a: [&str; 6]| a[i].to_string();
    // Stem before the model's root ending (the prefix + whatever the model keeps).
    let r = |n: usize| cut(inf, n).to_string();
    Some(match m {
        Model::Er => er_present(inf, p),
        Model::Finir => {
            let s = r(2);
            format!("{s}{}", ["is", "is", "it", "issons", "issez", "issent"][i])
        }
        Model::Hair => {
            let pre = r(4);
            format!("{pre}{}", ["hais", "hais", "hait", "haïssons", "haïssez", "haïssent"][i])
        }
        Model::Partir => {
            // dormir: dors, dort, dormons. The stem's last consonant drops in the singular.
            let s = r(2);
            let short = cut(&s, 1).to_string();
            match p {
                Person::Je | Person::Tu => format!("{short}s"),
                Person::Il => format!("{short}t"),
                _ => format!("{s}{}", ["", "", "", "ons", "ez", "ent"][i]),
            }
        }
        Model::Ouvrir | Model::Cueillir => {
            let s = r(2);
            format!("{s}{}", ["e", "es", "e", "ons", "ez", "ent"][i])
        }
        Model::Venir => {
            // venir, tenir and compounds.
            let pre = r(5);
            let v = if ends(inf, "venir") { "v" } else { "t" };
            format!("{pre}{v}{}", ["iens", "iens", "ient", "enons", "enez", "iennent"][i])
        }
        Model::Courir => {
            let s = r(2);
            format!("{s}{}", ["s", "s", "t", "ons", "ez", "ent"][i])
        }
        Model::Mourir => six(["meurs", "meurs", "meurt", "mourons", "mourez", "meurent"]),
        Model::Acquerir => {
            let pre = r(4);
            format!("{pre}{}", ["iers", "iers", "iert", "érons", "érez", "ièrent"][i])
        }
        Model::Fuir => {
            let pre = r(4);
            format!("{pre}{}", ["fuis", "fuis", "fuit", "fuyons", "fuyez", "fuient"][i])
        }
        Model::Vetir => {
            let s = r(2);
            format!("{s}{}", ["s", "s", "", "ons", "ez", "ent"][i])
        }
        Model::Bouillir => {
            let pre = r(8);
            format!("{pre}{}", ["bous", "bous", "bout", "bouillons", "bouillez", "bouillent"][i])
        }
        Model::Rendre => {
            let s = r(2);
            format!("{s}{}", ["s", "s", "", "ons", "ez", "ent"][i])
        }
        Model::Rompre => {
            let s = r(2);
            format!("{s}{}", ["s", "s", "t", "ons", "ez", "ent"][i])
        }
        Model::Battre => {
            let pre = r(6);
            format!("{pre}{}", ["bats", "bats", "bat", "battons", "battez", "battent"][i])
        }
        Model::Mettre => {
            let pre = r(6);
            format!("{pre}{}", ["mets", "mets", "met", "mettons", "mettez", "mettent"][i])
        }
        Model::Prendre => {
            let pre = r(7);
            format!("{pre}{}", ["prends", "prends", "prend", "prenons", "prenez", "prennent"][i])
        }
        Model::Craindre => {
            // craindre → crain-s, craign-ons.
            let s = r(3); // "crain"
            let g = format!("{}gn", cut(&s, 1));
            match p {
                Person::Je | Person::Tu => format!("{s}s"),
                Person::Il => format!("{s}t"),
                _ => format!("{g}{}", ["", "", "", "ons", "ez", "ent"][i]),
            }
        }
        Model::Connaitre => {
            let pre = r(5); // "conn" + "aître" → pre = "conn"
            format!("{pre}{}", ["ais", "ais", "aît", "aissons", "aissez", "aissent"][i])
        }
        Model::Naitre => {
            let pre = r(6);
            format!("{pre}{}", ["nais", "nais", "naît", "naissons", "naissez", "naissent"][i])
        }
        Model::Croitre => {
            let pre = r(7);
            format!("{pre}{}", ["croîs", "croîs", "croît", "croissons", "croissez", "croissent"][i])
        }
        Model::Conduire => {
            let s = r(2); // "condui"
            format!("{s}{}", ["s", "s", "t", "sons", "sez", "sent"][i])
        }
        Model::Croire => six(["crois", "crois", "croit", "croyons", "croyez", "croient"]),
        Model::Boire => six(["bois", "bois", "boit", "buvons", "buvez", "boivent"]),
        Model::Dire => {
            let pre = r(4);
            // redire: redites; contredire, interdire, médire, prédire: -disez.
            let vous = if pre.is_empty() || pre == "re" { "dites" } else { "disez" };
            format!("{pre}{}", ["dis", "dis", "dit", "disons", vous, "disent"][i])
        }
        Model::Lire => {
            let pre = r(4);
            format!("{pre}{}", ["lis", "lis", "lit", "lisons", "lisez", "lisent"][i])
        }
        Model::Ecrire => {
            let pre = r(5);
            format!("{pre}{}", ["cris", "cris", "crit", "crivons", "crivez", "crivent"][i])
        }
        Model::Rire => {
            let pre = r(4);
            format!("{pre}{}", ["ris", "ris", "rit", "rions", "riez", "rient"][i])
        }
        Model::Suivre => {
            let pre = r(6);
            format!("{pre}{}", ["suis", "suis", "suit", "suivons", "suivez", "suivent"][i])
        }
        Model::Vivre => {
            let pre = r(5);
            format!("{pre}{}", ["vis", "vis", "vit", "vivons", "vivez", "vivent"][i])
        }
        Model::Faire => {
            let pre = r(5);
            format!("{pre}{}", ["fais", "fais", "fait", "faisons", "faites", "font"][i])
        }
        Model::Plaire => {
            let pre = r(6);
            format!("{pre}{}", ["plais", "plais", "plaît", "plaisons", "plaisez", "plaisent"][i])
        }
        Model::Taire => six(["tais", "tais", "tait", "taisons", "taisez", "taisent"]),
        Model::Conclure => {
            let s = r(2);
            format!("{s}{}", ["s", "s", "t", "ons", "ez", "ent"][i])
        }
        Model::Vaincre => {
            let pre = r(7);
            format!("{pre}{}", ["vaincs", "vaincs", "vainc", "vainquons", "vainquez", "vainquent"][i])
        }
        Model::Resoudre => {
            let pre = r(6);
            format!("{pre}{}", ["sous", "sous", "sout", "solvons", "solvez", "solvent"][i])
        }
        Model::Traire => {
            let pre = r(6);
            format!("{pre}{}", ["trais", "trais", "trait", "trayons", "trayez", "traient"][i])
        }
        Model::Coudre => {
            let pre = r(6);
            format!("{pre}{}", ["couds", "couds", "coud", "cousons", "cousez", "cousent"][i])
        }
        Model::Moudre => six(["mouds", "mouds", "moud", "moulons", "moulez", "moulent"]),
        Model::Etre => six(["suis", "es", "est", "sommes", "êtes", "sont"]),
        Model::Avoir => six(["ai", "as", "a", "avons", "avez", "ont"]),
        Model::Aller => six(["vais", "vas", "va", "allons", "allez", "vont"]),
        Model::Voir => {
            let pre = r(4);
            format!("{pre}{}", ["vois", "vois", "voit", "voyons", "voyez", "voient"][i])
        }
        Model::Recevoir => {
            let pre = r(6);
            format!("{pre}{}", ["çois", "çois", "çoit", "cevons", "cevez", "çoivent"][i])
        }
        Model::Devoir => six(["dois", "dois", "doit", "devons", "devez", "doivent"]),
        Model::Pouvoir => six(["peux", "peux", "peut", "pouvons", "pouvez", "peuvent"]),
        Model::Vouloir => six(["veux", "veux", "veut", "voulons", "voulez", "veulent"]),
        Model::Savoir => six(["sais", "sais", "sait", "savons", "savez", "savent"]),
        Model::Valoir => {
            let pre = r(6);
            format!("{pre}{}", ["vaux", "vaux", "vaut", "valons", "valez", "valent"][i])
        }
        Model::Falloir => {
            if p != Person::Il {
                return None;
            }
            "faut".into()
        }
        Model::Pleuvoir => match p {
            Person::Il => "pleut".into(),
            // Figurative plural: *les coups pleuvent*.
            Person::Ils => "pleuvent".into(),
            _ => return None,
        },
        Model::Asseoir => {
            let pre = r(7);
            format!("{pre}{}", ["assieds", "assieds", "assied", "asseyons", "asseyez", "asseyent"][i])
        }
        Model::Mouvoir => {
            let pre = r(7);
            format!("{pre}{}", ["meus", "meus", "meut", "mouvons", "mouvez", "meuvent"][i])
        }
    })
}

/// The stem of the imperfect (from *nous* in the present) or `None` for impersonal verbs.
fn imperfect_stem(inf: &str, m: Model) -> Option<String> {
    match m {
        Model::Etre => Some("ét".into()),
        Model::Falloir => Some("fall".into()),
        Model::Pleuvoir => Some("pleuv".into()),
        Model::Er => Some(cut(inf, 2).to_string()),
        _ => {
            let nous = present(inf, m, Person::Nous)?;
            Some(cut(&nous, 3).to_string())
        }
    }
}

/// Persons an impersonal verb does not have (*falloir*: only *il*; *pleuvoir*: *il*, and *ils* figuratively).
fn impersonal_person(m: Model, p: Person) -> bool {
    match m {
        Model::Falloir => p != Person::Il,
        Model::Pleuvoir => !matches!(p, Person::Il | Person::Ils),
        _ => false,
    }
}

fn add_ending(stem: &str, end: &str, first_group: bool) -> String {
    if first_group { er_soft(stem, end) } else { format!("{stem}{end}") }
}

fn imparfait(inf: &str, m: Model, p: Person) -> Option<String> {
    if impersonal_person(m, p) {
        return None;
    }
    let stem = imperfect_stem(inf, m)?;
    let end = ["ais", "ais", "ait", "ions", "iez", "aient"][p.i()];
    Some(add_ending(&stem, end, m == Model::Er))
}

fn future_stem(inf: &str, m: Model) -> String {
    let r = |n: usize| cut(inf, n).to_string();
    match m {
        Model::Er => er_future_stem(inf),
        Model::Etre => "ser".into(),
        Model::Avoir => "aur".into(),
        Model::Aller => "ir".into(),
        Model::Faire => format!("{}fer", r(5)),
        Model::Venir => format!("{}{}iendr", r(5), if ends(inf, "venir") { "v" } else { "t" }),
        Model::Voir => {
            if ends(inf, "prévoir") || ends(inf, "pourvoir") {
                inf.to_string()
            } else {
                format!("{}verr", r(4))
            }
        }
        Model::Pouvoir => "pourr".into(),
        Model::Vouloir => "voudr".into(),
        Model::Savoir => "saur".into(),
        Model::Devoir => "devr".into(),
        Model::Recevoir => format!("{}cevr", r(6)),
        Model::Courir => format!("{}rr", r(3)),
        Model::Mourir => "mourr".into(),
        Model::Acquerir => format!("{}err", r(4)),
        Model::Valoir => format!("{}vaudr", r(6)),
        Model::Falloir => "faudr".into(),
        Model::Pleuvoir => "pleuvr".into(),
        Model::Asseoir => format!("{}assiér", r(7)),
        Model::Cueillir => format!("{}er", r(2)),
        Model::Mouvoir => format!("{}mouvr", r(7)),
        _ => {
            // Infinitive, without the final e of -re.
            if ends(inf, "re") { r(1) } else { inf.to_string() }
        }
    }
}

fn futur(inf: &str, m: Model, p: Person, conditional: bool) -> Option<String> {
    if impersonal_person(m, p) {
        return None;
    }
    let end = if conditional { ["ais", "ais", "ait", "ions", "iez", "aient"] } else { ["ai", "as", "a", "ons", "ez", "ont"] }[p.i()];
    Some(format!("{}{end}", future_stem(inf, m)))
}

fn subjonctif(inf: &str, m: Model, p: Person) -> Option<String> {
    let i = p.i();
    let r = |n: usize| cut(inf, n).to_string();
    let six = |a: [&str; 6]| Some(a[i].to_string());
    match m {
        Model::Etre => return six(["sois", "sois", "soit", "soyons", "soyez", "soient"]),
        Model::Avoir => return six(["aie", "aies", "ait", "ayons", "ayez", "aient"]),
        Model::Aller => return six(["aille", "ailles", "aille", "allions", "alliez", "aillent"]),
        Model::Faire => return Some(format!("{}{}", r(5), ["fasse", "fasses", "fasse", "fassions", "fassiez", "fassent"][i])),
        Model::Pouvoir => return six(["puisse", "puisses", "puisse", "puissions", "puissiez", "puissent"]),
        Model::Savoir => return six(["sache", "saches", "sache", "sachions", "sachiez", "sachent"]),
        Model::Vouloir => return six(["veuille", "veuilles", "veuille", "voulions", "vouliez", "veuillent"]),
        Model::Valoir => return Some(format!("{}{}", r(6), ["vaille", "vailles", "vaille", "valions", "valiez", "vaillent"][i])),
        Model::Falloir => return if p == Person::Il { Some("faille".into()) } else { None },
        Model::Pleuvoir => return if p == Person::Il { Some("pleuve".into()) } else if p == Person::Ils { Some("pleuvent".into()) } else { None },
        _ => {}
    }
    match p {
        Person::Nous | Person::Vous => {
            let stem = imperfect_stem(inf, m)?;
            Some(add_ending(&stem, if p == Person::Nous { "ions" } else { "iez" }, m == Model::Er))
        }
        _ => {
            let ils = if m == Model::Er { er_present(inf, Person::Ils) } else { present(inf, m, Person::Ils)? };
            let stem = cut(&ils, 3);
            Some(format!("{stem}{}", ["e", "es", "e", "", "", "ent"][i]))
        }
    }
}

/// Past participle (masculine singular).
pub fn participle(infinitive: &str) -> Option<String> {
    let (inf, _) = split_pronominal(infinitive.trim());
    let m = model_of(inf)?;
    let r = |n: usize| cut(inf, n).to_string();
    Some(match m {
        Model::Er | Model::Aller => format!("{}é", r(2)),
        Model::Finir | Model::Partir | Model::Cueillir | Model::Bouillir => format!("{}i", r(2)),
        Model::Hair => format!("{}ï", r(2)),
        Model::Ouvrir => format!("{}ert", r(3)),
        Model::Venir => format!("{}u", r(2)),
        Model::Courir => format!("{}u", r(2)),
        Model::Mourir => "mort".into(),
        Model::Acquerir => format!("{}is", r(4)),
        Model::Fuir => format!("{}i", r(2)),
        Model::Vetir => format!("{}u", r(2)),
        Model::Rendre | Model::Rompre | Model::Battre => format!("{}u", r(2)),
        Model::Mettre => format!("{}is", r(5)),
        Model::Prendre => format!("{}is", r(5)),
        Model::Craindre => format!("{}t", r(3)),
        Model::Connaitre => format!("{}u", r(5)),
        Model::Naitre => format!("{}né", r(6)),
        Model::Croitre => format!("{}crû", r(7)),
        Model::Conduire => {
            if ends(inf, "nuire") || ends(inf, "luire") || ends(inf, "suffire") {
                r(2)
            } else {
                format!("{}t", r(2))
            }
        }
        Model::Croire => "cru".into(),
        Model::Boire => "bu".into(),
        Model::Dire => format!("{}dit", r(4)),
        Model::Lire => format!("{}lu", r(4)),
        Model::Ecrire => format!("{}crit", r(5)),
        Model::Rire => format!("{}ri", r(4)),
        Model::Suivre => format!("{}suivi", r(6)),
        Model::Vivre => format!("{}vécu", r(5)),
        Model::Faire => format!("{}fait", r(5)),
        Model::Plaire => format!("{}plu", r(6)),
        Model::Taire => "tu".into(),
        Model::Conclure => {
            if ends(inf, "inclure") || ends(inf, "occlure") {
                format!("{}s", r(2))
            } else {
                r(2)
            }
        }
        Model::Vaincre => format!("{}vaincu", r(7)),
        Model::Resoudre => format!("{}solu", r(6)),
        Model::Traire => format!("{}trait", r(6)),
        Model::Coudre => format!("{}cousu", r(6)),
        Model::Moudre => "moulu".into(),
        Model::Etre => "été".into(),
        Model::Avoir => "eu".into(),
        Model::Voir => format!("{}vu", r(4)),
        Model::Recevoir => format!("{}çu", r(6)),
        Model::Devoir => "dû".into(),
        Model::Pouvoir => "pu".into(),
        Model::Vouloir => "voulu".into(),
        Model::Savoir => "su".into(),
        Model::Valoir => format!("{}valu", r(6)),
        Model::Falloir => "fallu".into(),
        Model::Pleuvoir => "plu".into(),
        Model::Asseoir => format!("{}assis", r(7)),
        Model::Mouvoir => {
            if inf == "mouvoir" {
                "mû".into()
            } else {
                format!("{}mu", r(7))
            }
        }
    })
}

/// The auxiliary of the passé composé (pronominal verbs: *être*).
pub fn auxiliary(infinitive: &str) -> Aux {
    let (inf, pronominal) = split_pronominal(infinitive.trim());
    if pronominal || ETRE.contains(&inf) {
        Aux::Etre
    } else {
        Aux::Avoir
    }
}

/// Verbs used only in the third person (*falloir*, *pleuvoir*, *neiger*, *il s'agit de*; *se dérouler*,
/// *s'avérer*: things take place, turn out): not drilled person by person.
pub fn impersonal(infinitive: &str) -> bool {
    let full = infinitive.trim();
    let (inf, _) = split_pronominal(full);
    matches!(model_of(inf), Some(Model::Falloir | Model::Pleuvoir))
        || ["neiger", "grêler", "s'agir", "se dérouler", "s'avérer", "se produire", "s'écouler", "advenir"].contains(&full)
}

/// Puts the reflexive clitic in front: `me lève`, `m'appelle`, `nous levons`.
fn with_clitic(form: &str, p: Person, verb: &str) -> String {
    let c = p.clitic();
    if matches!(p, Person::Je | Person::Tu | Person::Il | Person::Ils) && elides(form, verb) {
        format!("{}'{form}", &c[..1])
    } else {
        format!("{c} {form}")
    }
}

/// Conjugates `infinitive` (*parler*, *finir*, *se lever*, *s'appeler*). Returns the form without the subject
/// pronoun, or `None` if the verb is unknown or has no such person (impersonal verbs).
pub fn conjugate(infinitive: &str, tense: Tense, p: Person) -> Option<String> {
    let full = infinitive.trim().to_lowercase();
    let (inf, pronominal) = split_pronominal(&full);
    let m = model_of(inf)?;
    let form = match tense {
        Tense::Present => present(inf, m, p)?,
        Tense::Imparfait => imparfait(inf, m, p)?,
        Tense::Futur => futur(inf, m, p, false)?,
        Tense::Conditionnel => futur(inf, m, p, true)?,
        Tense::Subjonctif => subjonctif(inf, m, p)?,
        Tense::PasseCompose => {
            if impersonal_person(m, p) {
                return None;
            }
            let pp = participle(inf)?;
            let aux = auxiliary(&full);
            let aux_form = match aux {
                Aux::Avoir => present("avoir", Model::Avoir, p)?,
                Aux::Etre => present("être", Model::Etre, p)?,
            };
            let pp = match aux {
                Aux::Avoir => pp,
                // A participle ending in s already has its plural: assis(e), assis(es).
                Aux::Etre if pp.ends_with('s') => format!("{pp}{}", ["(e)", "(e)", "(e)", "(es)", "(e)(es)", "(es)"][p.i()]),
                Aux::Etre => format!("{pp}{}", ["(e)", "(e)", "(e)", "(e)s", "(e)(s)", "(e)s"][p.i()]),
            };
            let f = format!("{aux_form} {pp}");
            return Some(if pronominal { with_clitic(&f, p, inf) } else { f });
        }
    };
    Some(if pronominal { with_clitic(&form, p, inf) } else { form })
}

/// The form with its subject: `je parle`, `j'aime`, `j'ai parlé`, `je m'appelle`, `il pleut`.
pub fn with_subject(form: &str, p: Person, infinitive: &str) -> String {
    let (inf, _) = split_pronominal(infinitive.trim());
    if p == Person::Je && elides(form, inf) {
        format!("j'{form}")
    } else {
        format!("{} {form}", p.pronoun())
    }
}

/// A form as it is read aloud: the agreement marks of the grammar notation dropped (*allé(e)s* → *allés*).
pub fn spoken(form: &str) -> String {
    form.replace("(e)(es)", "").replace("(es)", "").replace("(e)(s)", "s").replace("(e)s", "s").replace("(e)", "").replace("(s)", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(inf: &str, t: Tense, p: Person) -> String {
        conjugate(inf, t, p).unwrap_or_else(|| panic!("{inf} {t:?} {p:?}"))
    }

    #[test]
    fn regular_and_spelling() {
        assert_eq!(c("parler", Tense::Present, Person::Je), "parle");
        assert_eq!(c("manger", Tense::Present, Person::Nous), "mangeons");
        assert_eq!(c("manger", Tense::Imparfait, Person::Nous), "mangions");
        assert_eq!(c("manger", Tense::Imparfait, Person::Je), "mangeais");
        assert_eq!(c("commencer", Tense::Present, Person::Nous), "commençons");
        assert_eq!(c("commencer", Tense::Imparfait, Person::Vous), "commenciez");
        assert_eq!(c("lever", Tense::Present, Person::Ils), "lèvent");
        assert_eq!(c("lever", Tense::Futur, Person::Je), "lèverai");
        assert_eq!(c("acheter", Tense::Present, Person::Je), "achète");
        assert_eq!(c("appeler", Tense::Present, Person::Je), "appelle");
        assert_eq!(c("appeler", Tense::Futur, Person::Nous), "appellerons");
        assert_eq!(c("jeter", Tense::Present, Person::Tu), "jettes");
        assert_eq!(c("céder", Tense::Present, Person::Je), "cède");
        assert_eq!(c("céder", Tense::Futur, Person::Je), "céderai");
        assert_eq!(c("préférer", Tense::Present, Person::Ils), "préfèrent");
        assert_eq!(c("nettoyer", Tense::Present, Person::Je), "nettoie");
        assert_eq!(c("nettoyer", Tense::Futur, Person::Je), "nettoierai");
        assert_eq!(c("envoyer", Tense::Futur, Person::Je), "enverrai");
        assert_eq!(c("créer", Tense::Present, Person::Je), "crée");
        assert_eq!(c("finir", Tense::Present, Person::Nous), "finissons");
        assert_eq!(c("finir", Tense::Subjonctif, Person::Je), "finisse");
    }

    #[test]
    fn irregular() {
        assert_eq!(c("être", Tense::Imparfait, Person::Nous), "étions");
        assert_eq!(c("aller", Tense::Subjonctif, Person::Nous), "allions");
        assert_eq!(c("venir", Tense::Present, Person::Ils), "viennent");
        assert_eq!(c("devenir", Tense::Futur, Person::Je), "deviendrai");
        assert_eq!(c("prendre", Tense::Subjonctif, Person::Je), "prenne");
        assert_eq!(c("comprendre", Tense::Subjonctif, Person::Nous), "comprenions");
        assert_eq!(c("craindre", Tense::Present, Person::Nous), "craignons");
        assert_eq!(c("peindre", Tense::Present, Person::Il), "peint");
        assert_eq!(c("connaître", Tense::Present, Person::Il), "connaît");
        assert_eq!(c("conduire", Tense::Present, Person::Nous), "conduisons");
        assert_eq!(c("boire", Tense::Subjonctif, Person::Nous), "buvions");
        assert_eq!(c("voir", Tense::Futur, Person::Je), "verrai");
        assert_eq!(c("recevoir", Tense::Present, Person::Ils), "reçoivent");
        assert_eq!(c("dire", Tense::Present, Person::Vous), "dites");
        assert_eq!(c("interdire", Tense::Present, Person::Vous), "interdisez");
        assert_eq!(c("dormir", Tense::Present, Person::Je), "dors");
        assert_eq!(c("ouvrir", Tense::Present, Person::Je), "ouvre");
        assert_eq!(c("falloir", Tense::Present, Person::Il), "faut");
        assert!(conjugate("falloir", Tense::Present, Person::Je).is_none());
        assert_eq!(c("rire", Tense::Imparfait, Person::Nous), "riions");
    }

    #[test]
    fn passe_compose() {
        assert_eq!(c("parler", Tense::PasseCompose, Person::Je), "ai parlé");
        assert_eq!(c("aller", Tense::PasseCompose, Person::Nous), "sommes allé(e)s");
        assert_eq!(c("aller", Tense::PasseCompose, Person::Vous), "êtes allé(e)(s)");
        assert_eq!(c("se lever", Tense::PasseCompose, Person::Je), "me suis levé(e)");
        assert_eq!(c("se lever", Tense::PasseCompose, Person::Il), "s'est levé(e)");
        assert_eq!(c("s'appeler", Tense::Present, Person::Je), "m'appelle");
        assert_eq!(c("s'appeler", Tense::Present, Person::Nous), "nous appelons");
        assert_eq!(with_subject("aime", Person::Je, "aimer"), "j'aime");
        assert_eq!(with_subject("ai parlé", Person::Je, "parler"), "j'ai parlé");
        assert_eq!(with_subject("hais", Person::Je, "haïr"), "je hais");
        assert_eq!(spoken("sommes allé(e)s"), "sommes allés");
        assert_eq!(c("s'asseoir", Tense::PasseCompose, Person::Nous), "nous sommes assis(es)");
        assert_eq!(spoken("nous sommes assis(es)"), "nous sommes assis");
        assert_eq!(c("léguer", Tense::Present, Person::Je), "lègue");
        assert!(impersonal("neiger") && impersonal("se dérouler") && !impersonal("se lever"));
    }
}
