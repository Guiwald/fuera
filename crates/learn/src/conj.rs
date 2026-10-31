//! Spanish (Spain) verb conjugator.
//!
//! Covers the six persons (with *vosotros*) of the seven simple tenses a learner
//! drills: present, preterite (*indefinido*), imperfect, future, conditional,
//! present subjunctive and the `-ra` imperfect subjunctive. Spelling follows the
//! current RAE orthography (2010): monosyllables such as *fue*, *dio*, *vio*,
//! *fie*, *rio*, *hui* or *riais* carry no written accent; *dé* and *sé* keep
//! their diacritic accent.
//!
//! The engine is rule based: regular endings, stem changes (e→ie, o→ue, e→i,
//! u→ue, i→ie), spelling changes driven by the sound of the stem's last
//! consonant (busqué, cojo, sigo, averigüé…), y-insertion (-uir, oír), the
//! accented weak vowel of *enviar*/*actuar*/*reunir*/*reír*, irregular *yo*
//! stems, strong preterites and irregular future stems. A small lexicon says
//! which verbs take which irregularity, and prefixed compounds of irregular roots
//! (*mantener*, *deshacer*, *prever*, *producir*…) are conjugated from their root.
//!
//! Pronominal infinitives (`levantarse`) yield forms with the clitic in front:
//! `me levanto`, `os levantáis`.

/// Grammatical person (Spain: *vosotros* is the informal plural).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Person {
    Yo,
    Tu,
    El,
    Nosotros,
    Vosotros,
    Ellos,
}

impl Person {
    /// All persons in paradigm order.
    pub const ALL: [Person; 6] = [
        Person::Yo,
        Person::Tu,
        Person::El,
        Person::Nosotros,
        Person::Vosotros,
        Person::Ellos,
    ];

    /// Subject pronoun: `yo`, `tú`, `él`, `nosotros`, `vosotros`, `ellos`.
    pub fn pronoun(self) -> &'static str {
        match self {
            Person::Yo => "yo",
            Person::Tu => "tú",
            Person::El => "él",
            Person::Nosotros => "nosotros",
            Person::Vosotros => "vosotros",
            Person::Ellos => "ellos",
        }
    }

    /// Reflexive clitic used with pronominal verbs.
    fn clitic(self) -> &'static str {
        match self {
            Person::Yo => "me",
            Person::Tu => "te",
            Person::El | Person::Ellos => "se",
            Person::Nosotros => "nos",
            Person::Vosotros => "os",
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// The simple tenses handled by [`conjugate`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tense {
    Present,
    Preterite,
    Imperfect,
    Future,
    Conditional,
    PresentSubjunctive,
    /// The `-ra` forms (*hablara*); the `-se` forms are not generated.
    ImperfectSubjunctive,
}

impl Tense {
    /// All tenses, in the order they are usually taught.
    pub const ALL: [Tense; 7] = [
        Tense::Present,
        Tense::Preterite,
        Tense::Imperfect,
        Tense::Future,
        Tense::Conditional,
        Tense::PresentSubjunctive,
        Tense::ImperfectSubjunctive,
    ];

    /// Spanish name as used in Spain's classrooms.
    pub fn name_es(self) -> &'static str {
        match self {
            Tense::Present => "presente",
            Tense::Preterite => "pretérito indefinido",
            Tense::Imperfect => "pretérito imperfecto",
            Tense::Future => "futuro simple",
            Tense::Conditional => "condicional simple",
            Tense::PresentSubjunctive => "presente de subjuntivo",
            Tense::ImperfectSubjunctive => "pretérito imperfecto de subjuntivo",
        }
    }
}

/// Conjugates `infinitive` (e.g. `hablar`, `tener`, `levantarse`).
///
/// Returns `None` when the input does not look like a Spanish infinitive
/// (it must end in `-ar`, `-er`, `-ir` or `-ír`, optionally followed by `se`).
/// Impersonal *hay* is out of scope: `haber` gives the auxiliary `ha`.
pub fn conjugate(infinitive: &str, tense: Tense, person: Person) -> Option<String> {
    let inf = infinitive.trim().to_lowercase();
    let (base, pronominal) = split_pronominal(&inf)?;
    let form = conj_inf(base, tense, person)?;
    Some(if pronominal {
        format!("{} {form}", person.clitic())
    } else {
        form
    })
}

/// Whether `infinitive` is irregular in `tense`: at least one form differs from
/// what the regular paradigm gives once the automatic spelling rules are
/// applied. Spelling-only changes (*busqué*, *cojo*, *leyó*, *averigüé*) do not
/// count; stem changes, irregular stems, y-insertion, *-zc-* and the accented
/// weak vowel of *envío*/*reúno* do. Non-verbs are not irregular.
pub fn is_irregular(infinitive: &str, tense: Tense) -> bool {
    let inf = infinitive.trim().to_lowercase();
    let Some((base, _)) = split_pronominal(&inf) else {
        return false;
    };
    Person::ALL.iter().any(|&p| {
        let actual = conj_inf(base, tense, p);
        let regular = conj_simple(base, Spec::NONE, Mode::Regular, tense, p);
        actual != regular
    })
}

// ---------------------------------------------------------------------------
// Infinitive analysis
// ---------------------------------------------------------------------------

fn split_pronominal(inf: &str) -> Option<(&str, bool)> {
    if inf.is_empty()
        || !inf
            .chars()
            .all(|c| c.is_ascii_lowercase() || "ñáéíóúü".contains(c))
    {
        return None;
    }
    if let Some(base) = inf.strip_suffix("se")
        && class_of(base).is_some()
    {
        return Some((base, true));
    }
    class_of(inf).map(|_| (inf, false))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Ar,
    Er,
    Ir,
}

/// Splits an infinitive into its stem and conjugation class.
fn class_of(inf: &str) -> Option<(&str, Class)> {
    let (stem, class) = if let Some(s) = inf.strip_suffix("ar") {
        (s, Class::Ar)
    } else if let Some(s) = inf.strip_suffix("er") {
        (s, Class::Er)
    } else {
        (
            inf.strip_suffix("ir").or_else(|| inf.strip_suffix("ír"))?,
            Class::Ir,
        )
    };
    (!stem.is_empty() || inf == "ir").then_some((stem, class))
}

/// Vowel alternation in the stressed stem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    None,
    /// pensar → pienso (word-initially *ye*: errar → yerro)
    EIe,
    /// contar → cuento (word-initially *hue*: oler → huelo; after g *güe*)
    OUe,
    /// pedir → pido
    EI,
    /// jugar → juego
    UUe,
    /// adquirir → adquiero
    IIe,
}

/// Irregularity profile of a verb (all stems are relative to that verb).
#[derive(Clone, Copy, Debug)]
struct Spec {
    change: Change,
    /// Irregular stem of present *yo* and of the whole present subjunctive.
    yo: Option<&'static str>,
    /// Strong preterite stem (tuv-, hic-, dij-…).
    pret: Option<&'static str>,
    /// Irregular future/conditional stem (tendr-, har-…).
    fut: Option<&'static str>,
    /// Written accent on the stem's weak vowel when stressed (envío, reúno).
    accent: bool,
    /// y-insertion before a/e/o endings (oír → oyes).
    y_insert: bool,
    /// -ir stem changer without the e→i / o→u change of unstressed forms.
    no_minor: bool,
    /// vowel + -cer verb without -zc- (cocer → cuezo, mecer → mezo).
    no_zc: bool,
}

impl Spec {
    const NONE: Spec = Spec {
        change: Change::None,
        yo: None,
        pret: None,
        fut: None,
        accent: false,
        y_insert: false,
        no_minor: false,
        no_zc: false,
    };
    const fn ch(change: Change) -> Spec {
        Spec {
            change,
            ..Spec::NONE
        }
    }
    const fn yo(self, s: &'static str) -> Spec {
        Spec {
            yo: Some(s),
            ..self
        }
    }
    const fn pret(self, s: &'static str) -> Spec {
        Spec {
            pret: Some(s),
            ..self
        }
    }
    const fn fut(self, s: &'static str) -> Spec {
        Spec {
            fut: Some(s),
            ..self
        }
    }
    const fn y(self) -> Spec {
        Spec {
            y_insert: true,
            ..self
        }
    }
    const fn no_minor(self) -> Spec {
        Spec {
            no_minor: true,
            ..self
        }
    }
    const fn no_zc(self) -> Spec {
        Spec {
            no_zc: true,
            ..self
        }
    }
}

use Change::{EI, EIe, IIe, OUe, UUe};

/// Verbs with their own tables for some tenses (see [`special_table`]).
const SPECIAL: &[(&str, Spec)] = &[
    ("ser", Spec::NONE.yo("se")),
    ("ir", Spec::NONE.yo("vay")),
    ("estar", Spec::NONE.pret("estuv")),
    ("haber", Spec::NONE.yo("hay").pret("hub").fut("habr")),
    ("dar", Spec::NONE),
    ("saber", Spec::NONE.yo("sep").pret("sup").fut("sabr")),
    ("ver", Spec::NONE.yo("ve")),
];

/// Irregular roots whose prefixed compounds conjugate the same way
/// (tener → mantener, poner → componer, ver → prever). `ducir` and `facer`
/// take any prefix (producir, satisfacer, licuefacer).
const ROOTS: &[(&str, Spec)] = &[
    ("tener", Spec::ch(EIe).yo("teng").pret("tuv").fut("tendr")),
    ("venir", Spec::ch(EIe).yo("veng").pret("vin").fut("vendr")),
    ("poner", Spec::NONE.yo("pong").pret("pus").fut("pondr")),
    ("hacer", Spec::NONE.yo("hag").pret("hic").fut("har")),
    ("decir", Spec::ch(EI).yo("dig").pret("dij").fut("dir")),
    ("traer", Spec::NONE.yo("traig").pret("traj")),
    ("ducir", Spec::NONE.pret("duj")),
    ("facer", Spec::NONE.yo("fag").pret("fic").fut("far")),
    ("valer", Spec::NONE.yo("valg").fut("valdr")),
    ("salir", Spec::NONE.yo("salg").fut("saldr")),
    ("caer", Spec::NONE.yo("caig")),
    ("oír", Spec::NONE.yo("oig").y()),
    ("querer", Spec::ch(EIe).pret("quis").fut("querr")),
    ("asir", Spec::NONE.yo("asg")),
    ("quirir", Spec::ch(IIe)),
    ("volver", Spec::ch(OUe)),
    ("solver", Spec::ch(OUe)),
    ("mover", Spec::ch(OUe)),
    ("seguir", Spec::ch(EI)),
    ("pedir", Spec::ch(EI)),
    ("medir", Spec::ch(EI)),
    ("vestir", Spec::ch(EI)),
    ("sentir", Spec::ch(EIe)),
    ("vertir", Spec::ch(EIe)),
    ("ferir", Spec::ch(EIe)),
    ("contar", Spec::ch(OUe)),
    ("costar", Spec::ch(OUe)),
    ("cordar", Spec::ch(OUe)),
    ("probar", Spec::ch(OUe)),
    ("mostrar", Spec::ch(OUe)),
    ("forzar", Spec::ch(OUe)),
    ("colgar", Spec::ch(OUe)),
    ("volar", Spec::ch(OUe)),
    ("poblar", Spec::ch(OUe)),
    ("cerrar", Spec::ch(EIe)),
    ("dormir", Spec::ch(OUe)),
    ("morir", Spec::ch(OUe)),
    ("cocer", Spec::ch(OUe).no_zc()),
];

/// Other verbs with irregular stems (not used as compounding roots).
const IRREGULAR: &[(&str, Spec)] = &[
    ("poder", Spec::ch(OUe).pret("pud").fut("podr")),
    ("caber", Spec::NONE.yo("quep").pret("cup").fut("cabr")),
    ("andar", Spec::NONE.pret("anduv")),
    ("desandar", Spec::NONE.pret("desanduv")),
    ("raer", Spec::NONE.yo("raig")),
    // decir compounds with a regular future (DPD prefers predeciré).
    ("bendecir", Spec::ch(EI).yo("bendig").pret("bendij")),
    ("maldecir", Spec::ch(EI).yo("maldig").pret("maldij")),
    ("predecir", Spec::ch(EI).yo("predig").pret("predij")),
    ("jugar", Spec::ch(UUe)),
    ("cernir", Spec::ch(EIe).no_minor()),
    ("hendir", Spec::ch(EIe).no_minor()),
    ("concernir", Spec::ch(EIe).no_minor()),
    ("discernir", Spec::ch(EIe).no_minor()),
    ("mecer", Spec::NONE.no_zc()),
    ("remecer", Spec::NONE.no_zc()),
];

/// e → ie (the -ir ones also take e → i in unstressed forms: sintió, sintamos).
#[rustfmt::skip]
const E_IE: &[&str] = &[
    "abnegar", "acertar", "acrecentar", "adherir", "alentar", "aneblar", "apacentar", "apretar",
    "arrendar", "arrepentir", "ascender", "asentar", "aserrar", "aspaventar", "atender",
    "atravesar", "aventar", "beldar", "calentar", "cegar", "cerner", "comenzar", "concertar",
    "condescender", "confesar", "contender", "defender", "denegar", "dentar", "desacertar",
    "desalentar", "desasosegar", "desatender", "descender", "desconcertar", "desentender",
    "desenterrar", "desgobernar", "deshelar", "desherbar", "desmembrar", "desmentir", "despertar",
    "despezar", "desplegar", "desterrar", "desventar", "digerir", "distender", "emparentar",
    "empedrar", "empezar", "encender", "encomendar", "enlenzar", "enmendar", "enmerdar",
    "ensangrentar", "entender", "enterrar", "erguir", "errar", "escalentar", "escarmentar",
    "estregar", "extender", "fregar", "gobernar", "hacendar", "heder", "helar", "hender", "herbar",
    "herir", "herrar", "herventar", "hervir", "incensar", "ingerir", "injerir", "malentender",
    "malherir", "manifestar", "melar", "mentar", "mentir", "merendar", "negar", "nevar", "pensar",
    "perder", "plegar", "precalentar", "quebrar", "recalentar", "recentar", "recomendar",
    "recomenzar", "refregar", "regar", "rehervir", "remendar", "renegar", "repensar", "replegar",
    "requebrar", "requerir", "resquebrar", "restregar", "retemblar", "reventar", "salpimentar",
    "sarmentar", "segar", "sembrar", "sementar", "sentar", "serrar", "sobreentender",
    "sobrentender", "sosegar", "soterrar", "subarrendar", "subentender", "subtender", "sugerir",
    "temblar", "tender", "tentar", "transcender", "trascender", "trasegar", "travesar", "tropezar",
    "ventar", "verter", "zaherir",
];

/// o → ue (the -ir ones also take o → u in unstressed forms: durmió, durmamos).
#[rustfmt::skip]
const O_UE: &[&str] = &[
    "abuñolar", "aclocar", "agorar", "almorzar", "alongar", "amoblar", "amolar", "apostar",
    "asonar", "atronar", "avergonzar", "clocar", "colar", "condoler", "consolar", "consonar",
    "degollar", "demoler", "denostar", "descollar", "desconsolar", "descornar", "desencontrar",
    "desollar", "destorcer", "desvergonzar", "disonar", "dolar", "doler", "emporcar", "enclocar",
    "encontrar", "enllocar", "enrodar", "ensoñar", "envergonzar", "holgar", "hollar", "llover",
    "mancornar", "moler", "morder", "oler", "reencontrar", "regoldar", "remorder", "renovar",
    "resollar", "resonar", "retorcer", "retostar", "revolcar", "rodar", "rogar", "soldar", "soler",
    "soltar", "sonar", "soñar", "torcer", "tostar", "trastrocar", "trocar", "tronar", "volcar",
];

/// e → i (-ir verbs).
#[rustfmt::skip]
const E_I: &[&str] = &[
    "ceñir", "colegir", "comedir", "competir", "concebir", "constreñir", "corregir", "derretir",
    "descomedir", "desteñir", "elegir", "embestir", "estreñir", "gemir", "henchir", "heñir",
    "preconcebir", "reelegir", "regir", "rendir", "repetir", "reñir", "servir", "teñir",
];

/// Verbs whose stressed stem has an accented weak vowel: envío, actúo (most
/// -uar verbs are detected automatically), reúno, prohíbo, aíslo…
#[rustfmt::skip]
const ACCENT: &[&str] = &[
    "adiar", "ahijar", "ahilar", "ahincar", "ahitar", "ahumar", "ahusar", "airar", "aislar",
    "aliar", "amnistiar", "amohinar", "ampliar", "ansiar", "arcaizar", "arriar", "ataviar",
    "atraillar", "aullar", "aunar", "aupar", "averiar", "aviar", "cabrahigar", "cariar",
    "chirriar", "ciar", "cohibir", "confiar", "contrariar", "criar", "desafiar", "desainar",
    "desataviar", "descafeinar", "descarriar", "desconfiar", "desliar", "desvariar", "desviar",
    "embaular", "enfriar", "enraizar", "enviar", "escalofriar", "espiar", "esquiar", "estriar",
    "europeizar", "expiar", "extasiar", "extraviar", "fiar", "gloriar", "guiar", "hastiar",
    "hebraizar", "homogeneizar", "inventariar", "judaizar", "liar", "malcriar", "maullar", "piar",
    "porfiar", "prohibir", "prohijar", "raizar", "recriar", "reenviar", "rehilar", "rehuir",
    "rehundir", "rehusar", "resfriar", "reunir", "reusar", "rociar", "sahumar", "vaciar", "variar",
];

/// Prefixes recognised in compounds of [`ROOTS`] (applied recursively).
const PREFIXES: &[&str] = &[
    "a", "ab", "abs", "ad", "ante", "bien", "circun", "co", "com", "con", "contra", "contro", "de",
    "des", "di", "dis", "en", "entre", "equi", "es", "ex", "im", "in", "inter", "intro", "mal",
    "man", "o", "ob", "per", "pos", "pre", "pro", "re", "retro", "sobre", "sos", "su", "sub",
    "subs", "super", "sus", "tra", "tras", "trans", "yuxta",
];

/// Prefixes that keep the conjugation of *any* listed irregular verb
/// (reencender, sobrecalentar, desceñir, rejugar, resaber). Prefixes such as
/// `pre`, `con`, `a` or `in` are excluded on purpose: presentar, contentar,
/// atentar and intentar are regular although sentar and tentar are not.
const SAFE_PREFIXES: &[&str] = &["contra", "des", "entre", "re", "sobre", "super"];

/// Regular verbs that look like compounds of an irregular one.
const NOT_COMPOUNDS: &[&str] = &["desairar", "destronar", "encostar"];

fn find(table: &[(&'static str, Spec)], inf: &str) -> Option<Spec> {
    table.iter().find(|(w, _)| *w == inf).map(|&(_, s)| s)
}

fn is_root(inf: &str) -> bool {
    inf == "ver" || find(ROOTS, inf).is_some()
}

/// Verbs whose irregularity comes from the lexicon lists (see [`derive_spec`]).
fn is_listed(inf: &str) -> bool {
    E_IE.contains(&inf)
        || O_UE.contains(&inf)
        || E_I.contains(&inf)
        || ACCENT.contains(&inf)
        || inf.ends_with("eír")
        || find(SPECIAL, inf).is_some()
        || find(IRREGULAR, inf).is_some()
}

/// For a prefixed compound, the verb after the prefix: `mantener` → `tener`,
/// `descomponer` → `componer`, `reproducir` → `producir`, `sobrecalentar` →
/// `calentar`, `autoimponer` → `imponer`.
fn compound_rest(inf: &str) -> Option<&str> {
    if NOT_COMPOUNDS.contains(&inf) {
        return None;
    }
    // auto- is transparent: autoimponerse, autocorregirse, autoaislarse
    if let Some(rest) = inf.strip_prefix("auto")
        && rest.chars().count() > 2
        && class_of(rest).is_some()
    {
        return Some(rest);
    }
    root_rest(inf).or_else(|| {
        SAFE_PREFIXES.iter().find_map(|p| {
            let rest = inf.strip_prefix(p)?;
            (rest.chars().count() > 3 && is_listed(rest)).then_some(rest)
        })
    })
}

/// Compound of one of the [`ROOTS`], possibly through several prefixes.
fn root_rest(inf: &str) -> Option<&str> {
    for root in ["ducir", "facer"] {
        if let Some(prefix) = inf.strip_suffix(root)
            && !prefix.is_empty()
        {
            return Some(&inf[prefix.len()..]);
        }
    }
    PREFIXES.iter().find_map(|p| {
        let rest = inf.strip_prefix(p)?;
        (rest.chars().count() > 2 && (is_root(rest) || root_rest(rest).is_some())).then_some(rest)
    })
}

/// Irregularity profile of a verb that is neither special, root nor compound.
fn derive_spec(inf: &str) -> Spec {
    let mut s = if E_IE.contains(&inf) {
        Spec::ch(EIe)
    } else if O_UE.contains(&inf) {
        Spec::ch(OUe)
    } else if E_I.contains(&inf) {
        Spec::ch(EI)
    } else {
        Spec::NONE
    };
    if inf.ends_with("eír") {
        // reír, sonreír, freír, desleír, engreír: río, rio, riamos
        s.change = EI;
        s.accent = true;
    }
    let uar = inf.ends_with("uar") && !inf.ends_with("guar") && !inf.ends_with("cuar");
    if uar || inf.ends_with("grafiar") || ACCENT.contains(&inf) {
        s.accent = true;
    }
    s
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Full conjugation with special tables and irregularities.
    Full,
    /// Plain regular paradigm (spelling rules only), for [`is_irregular`].
    Regular,
}

fn conj_inf(inf: &str, tense: Tense, p: Person) -> Option<String> {
    if let Some(spec) = find(SPECIAL, inf) {
        return conj_simple(inf, spec, Mode::Full, tense, p);
    }
    if let Some(spec) = find(IRREGULAR, inf).or_else(|| find(ROOTS, inf)) {
        return conj_simple(inf, spec, Mode::Full, tense, p);
    }
    if is_listed(inf) {
        return conj_simple(inf, derive_spec(inf), Mode::Full, tense, p);
    }
    if let Some(rest) = compound_rest(inf) {
        let prefix = &inf[..inf.len() - rest.len()];
        let form = conj_inf(rest, tense, p)?;
        return Some(prefixed(prefix, &form));
    }
    conj_simple(inf, derive_spec(inf), Mode::Full, tense, p)
}

// ---------------------------------------------------------------------------
// The paradigm engine
// ---------------------------------------------------------------------------

/// How the stem's last consonant sound is spelled before front/back vowels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fin {
    /// No respelling.
    Plain,
    /// [k]: c / qu (buscar, delinquir)
    K,
    /// [g]: g / gu (pagar, seguir)
    G,
    /// [θ]: z / c (empezar, vencer, conocer)
    Z,
    /// [gw]: gu / gü (averiguar, argüir)
    Gw,
    /// [x]: j / g (coger, dirigir)
    X,
}

struct Verb<'a> {
    inf: &'a str,
    class: Class,
    /// Stem without the spelling of its last consonant (see `fin`).
    body: &'a str,
    fin: Fin,
    zc: bool,
    y_insert: bool,
    spec: Spec,
}

fn analyze(inf: &str, spec: Spec, mode: Mode) -> Option<Verb<'_>> {
    let (stem, class) = class_of(inf)?;
    let (body, fin) = match class {
        Class::Ar => {
            if let Some(b) = stem.strip_suffix("gu") {
                (b, Fin::Gw)
            } else if let Some(b) = stem.strip_suffix('c') {
                (b, Fin::K)
            } else if let Some(b) = stem.strip_suffix('g') {
                (b, Fin::G)
            } else if let Some(b) = stem.strip_suffix('z') {
                (b, Fin::Z)
            } else {
                (stem, Fin::Plain)
            }
        }
        Class::Er | Class::Ir => {
            if let Some(b) = stem.strip_suffix("gü") {
                (b, Fin::Gw)
            } else if let Some(b) = stem.strip_suffix("gu") {
                (b, Fin::G)
            } else if let Some(b) = stem.strip_suffix("qu") {
                (b, Fin::K)
            } else if stem.ends_with("sc") {
                (stem, Fin::Plain) // evanescer → evanesco
            } else if let Some(b) = stem.strip_suffix('c') {
                (b, Fin::Z)
            } else if let Some(b) = stem.strip_suffix('g') {
                (b, Fin::X)
            } else {
                (stem, Fin::Plain)
            }
        }
    };
    let full = mode == Mode::Full;
    let zc = full
        && class != Class::Ar
        && fin == Fin::Z
        && !spec.no_zc
        && body.chars().last().is_some_and(is_vowel);
    let uir = class == Class::Ir && (fin == Fin::Gw || (fin == Fin::Plain && body.ends_with('u')));
    let y_insert = full && (spec.y_insert || uir);
    Some(Verb {
        inf,
        class,
        body,
        fin,
        zc,
        y_insert,
        spec,
    })
}

const PRES: [[&str; 6]; 3] = [
    ["o", "as", "a", "amos", "áis", "an"],
    ["o", "es", "e", "emos", "éis", "en"],
    ["o", "es", "e", "imos", "ís", "en"],
];
const PRET_AR: [&str; 6] = ["é", "aste", "ó", "amos", "asteis", "aron"];
const PRET_ERIR: [&str; 6] = ["í", "iste", "ió", "imos", "isteis", "ieron"];
const IMPF_AR: [&str; 6] = ["aba", "abas", "aba", "ábamos", "abais", "aban"];
const IMPF_ERIR: [&str; 6] = ["ía", "ías", "ía", "íamos", "íais", "ían"];
const FUT: [&str; 6] = ["é", "ás", "á", "emos", "éis", "án"];
const COND: [&str; 6] = ["ía", "ías", "ía", "íamos", "íais", "ían"];
const SUBJ_AR: [&str; 6] = ["e", "es", "e", "emos", "éis", "en"];
const SUBJ_ERIR: [&str; 6] = ["a", "as", "a", "amos", "áis", "an"];
const IMPF_SUBJ: [&str; 6] = ["ra", "ras", "ra", "ramos", "rais", "ran"];

/// Persons whose present stem is stressed (the "boot").
fn stressed(i: usize) -> bool {
    matches!(i, 0..=2 | 5)
}

/// Full paradigms (or the parts the engine cannot derive) of the most irregular verbs.
fn special_table(inf: &str, tense: Tense) -> Option<[&'static str; 6]> {
    use Tense::*;
    Some(match (inf, tense) {
        ("ser", Present) => ["soy", "eres", "es", "somos", "sois", "son"],
        ("ser" | "ir", Preterite) => ["fui", "fuiste", "fue", "fuimos", "fuisteis", "fueron"],
        ("ser", Imperfect) => ["era", "eras", "era", "éramos", "erais", "eran"],
        ("ir", Present) => ["voy", "vas", "va", "vamos", "vais", "van"],
        ("ir", Imperfect) => ["iba", "ibas", "iba", "íbamos", "ibais", "iban"],
        ("estar", Present) => ["estoy", "estás", "está", "estamos", "estáis", "están"],
        ("estar", PresentSubjunctive) => ["esté", "estés", "esté", "estemos", "estéis", "estén"],
        ("haber", Present) => ["he", "has", "ha", "hemos", "habéis", "han"],
        ("dar", Present) => ["doy", "das", "da", "damos", "dais", "dan"],
        ("dar", Preterite) => ["di", "diste", "dio", "dimos", "disteis", "dieron"],
        ("dar", PresentSubjunctive) => ["dé", "des", "dé", "demos", "deis", "den"],
        ("saber", Present) => ["sé", "sabes", "sabe", "sabemos", "sabéis", "saben"],
        ("ver", Present) => ["veo", "ves", "ve", "vemos", "veis", "ven"],
        ("ver", Imperfect) => ["veía", "veías", "veía", "veíamos", "veíais", "veían"],
        _ => return None,
    })
}

fn conj_simple(inf: &str, spec: Spec, mode: Mode, tense: Tense, p: Person) -> Option<String> {
    let i = p.index();
    if mode == Mode::Full
        && let Some(table) = special_table(inf, tense)
    {
        return Some(table[i].to_string());
    }
    if tense == Tense::ImperfectSubjunctive {
        // Built on the 3rd person plural preterite: tuvie|ron → tuvie|ra.
        let pret = conj_simple(inf, spec, mode, Tense::Preterite, Person::Ellos)?;
        let mut base = pret.strip_suffix("ron").unwrap_or(&pret).to_string();
        if i == 3 {
            accent_last_vowel(&mut base);
        }
        base.push_str(IMPF_SUBJ[i]);
        return Some(base);
    }
    let v = analyze(inf, spec, mode)?;
    let ci = v.class as usize;
    let form = match tense {
        Tense::Present => match v.spec.yo {
            Some(yo) if i == 0 => format!("{yo}o"),
            _ => {
                let stem = if stressed(i) {
                    major(&v)
                } else {
                    v.body.to_string()
                };
                join(&v, &stem, PRES[ci][i])
            }
        },
        Tense::Preterite => match v.spec.pret {
            Some(strong) => strong_preterite(strong, i),
            None if v.class == Class::Ar => join(&v, v.body, PRET_AR[i]),
            None => {
                let stem = if i == 2 || i == 5 {
                    minor(&v)
                } else {
                    v.body.to_string()
                };
                join(&v, &stem, PRET_ERIR[i])
            }
        },
        Tense::Imperfect => {
            let ending = if v.class == Class::Ar {
                IMPF_AR[i]
            } else {
                IMPF_ERIR[i]
            };
            join(&v, v.body, ending)
        }
        Tense::Future | Tense::Conditional => {
            let stem = v
                .spec
                .fut
                .map_or_else(|| v.inf.replace('í', "i"), str::to_string);
            let ending = if tense == Tense::Future {
                FUT[i]
            } else {
                COND[i]
            };
            stem + ending
        }
        Tense::PresentSubjunctive => {
            let endings = if v.class == Class::Ar {
                &SUBJ_AR
            } else {
                &SUBJ_ERIR
            };
            match v.spec.yo {
                Some(yo) => format!("{yo}{}", endings[i]),
                None => {
                    let stem = if stressed(i) { major(&v) } else { minor(&v) };
                    join(&v, &stem, endings[i])
                }
            }
        }
        Tense::ImperfectSubjunctive => unreachable!("derived from the preterite above"),
    };
    Some(unaccent_monosyllable(form))
}

fn strong_preterite(stem: &str, i: usize) -> String {
    match i {
        // hic-o → hizo, satisfic-o → satisfizo
        2 if stem.ends_with('c') => format!("{}zo", &stem[..stem.len() - 1]),
        // dij-eron, traj-eron, conduj-eron
        5 if stem.ends_with('j') => format!("{stem}eron"),
        _ => format!("{stem}{}", ["e", "iste", "o", "imos", "isteis", "ieron"][i]),
    }
}

/// Stem of the stressed (boot) persons: main vowel change, accented weak vowel.
fn major(v: &Verb) -> String {
    let mut stem = v.body.to_string();
    match v.spec.change {
        Change::None => {}
        EIe => replace_last(&mut stem, 'e', |at, before| match (at, before) {
            (0, _) => "ye",
            _ => "ie",
        }),
        OUe => replace_last(&mut stem, 'o', |at, before| match (at, before) {
            (0, _) => "hue",
            (_, Some('g')) => "üe",
            _ => "ue",
        }),
        EI => replace_last(&mut stem, 'e', |_, _| "i"),
        UUe => replace_last(&mut stem, 'u', |_, _| "ue"),
        IIe => replace_last(&mut stem, 'i', |_, _| "ie"),
    }
    if v.spec.accent
        && let Some(at) = stem.rfind(['i', 'u'])
    {
        let accented = if stem[at..].starts_with('i') {
            "í"
        } else {
            "ú"
        };
        stem.replace_range(at..at + 1, accented);
    }
    stem
}

/// Stem of the unstressed persons that change in -ir verbs (sintió, durmamos, pidieron).
fn minor(v: &Verb) -> String {
    let mut stem = v.body.to_string();
    if v.class == Class::Ir && !v.spec.no_minor {
        match v.spec.change {
            EIe | EI => replace_last(&mut stem, 'e', |_, _| "i"),
            OUe => replace_last(&mut stem, 'o', |_, _| "u"),
            _ => {}
        }
    }
    stem
}

/// Replaces the last ASCII vowel `target` of `stem` with `with(index, previous char)`.
fn replace_last(
    stem: &mut String,
    target: char,
    with: impl Fn(usize, Option<char>) -> &'static str,
) {
    if let Some(at) = stem.rfind(target) {
        let before = stem[..at].chars().last();
        let new = with(at, before);
        stem.replace_range(at..at + 1, new);
    }
}

/// Spelling of the stem's final consonant before `next`.
fn spell(fin: Fin, next: Option<char>, zc: bool) -> &'static str {
    let front = matches!(next, Some('e' | 'é' | 'i' | 'í'));
    let back = matches!(next, Some('a' | 'á' | 'o' | 'ó'));
    match fin {
        Fin::Plain => "",
        Fin::K => {
            if front {
                "qu"
            } else {
                "c"
            }
        }
        Fin::G => {
            if front {
                "gu"
            } else {
                "g"
            }
        }
        Fin::Z if front => "c",
        Fin::Z => {
            if zc && back {
                "zc"
            } else {
                "z"
            }
        }
        Fin::Gw => {
            if front {
                "gü"
            } else {
                "gu"
            }
        }
        Fin::X => {
            if front {
                "g"
            } else {
                "j"
            }
        }
    }
}

/// Attaches `ending` to `stem` applying the spelling rules at the junction.
fn join(v: &Verb, stem: &str, ending: &str) -> String {
    let first = ending.chars().next();
    let mut s = String::with_capacity(stem.len() + ending.len() + 4);
    s.push_str(stem);
    if v.y_insert && matches!(first, Some('a' | 'e' | 'o' | 'á' | 'é' | 'ó')) {
        // construyo, oyes, arguyáis
        s.push_str(spell(v.fin, Some('y'), false));
        s.push('y');
        s.push_str(ending);
        return s;
    }
    s.push_str(spell(v.fin, first, v.zc));
    if let Some(after) = ending.strip_prefix('i') {
        let last = s.chars().last();
        if after.chars().next().is_some_and(is_vowel) {
            // Unstressed i before a vowel: absorbed (rio, bulló, tiñeron),
            // or written y between vowels (leyó, huyeron, arguyó).
            if s.ends_with(['i', 'ñ']) || s.ends_with("ll") {
                s.push_str(after);
                return s;
            }
            let vowel_final = match v.fin {
                Fin::Gw => true,
                Fin::Plain => last.is_some_and(is_vowel),
                _ => false,
            };
            if vowel_final {
                if v.fin == Fin::Gw && s.ends_with("gü") {
                    s.truncate(s.len() - "ü".len());
                    s.push('u'); // arguyó
                }
                s.push('y');
                s.push_str(after);
                return s;
            }
        } else if v.fin == Fin::Plain && matches!(last, Some('a' | 'e' | 'o')) {
            // Stressed i in hiatus: leíste, oímos, reímos.
            s.push('í');
            s.push_str(after);
            return s;
        }
    }
    s.push_str(ending);
    s
}

// ---------------------------------------------------------------------------
// Orthography helpers
// ---------------------------------------------------------------------------

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'e' | 'i' | 'o' | 'u' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ü'
    )
}

fn is_strong(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'o' | 'á' | 'é' | 'ó')
}

fn is_accented(c: char) -> bool {
    matches!(c, 'á' | 'é' | 'í' | 'ó' | 'ú')
}

fn strip_accent(c: char) -> char {
    match c {
        'á' => 'a',
        'é' => 'e',
        'í' => 'i',
        'ó' => 'o',
        'ú' => 'u',
        _ => c,
    }
}

fn with_accent(c: char) -> char {
    match c {
        'a' => 'á',
        'e' => 'é',
        'i' => 'í',
        'o' => 'ó',
        'u' => 'ú',
        _ => c,
    }
}

/// Number of syllables by the orthographic rules used for accentuation:
/// two strong vowels, or an accented i/u next to a strong vowel, are in
/// hiatus; anything else adjacent is a diphthong/triphthong (ui is always one).
fn syllables(word: &str) -> usize {
    let cs: Vec<char> = word.chars().collect();
    let mut count = 0;
    let mut prev: Option<char> = None;
    for (k, &c) in cs.iter().enumerate() {
        let silent_u = c == 'u'
            && k > 0
            && matches!(cs[k - 1], 'q' | 'g')
            && matches!(cs.get(k + 1), Some('e' | 'i' | 'é' | 'í'));
        let final_y = c == 'y' && k + 1 == cs.len() && k > 0 && is_vowel(cs[k - 1]);
        let vowel = (is_vowel(c) && !silent_u) || final_y;
        if !vowel {
            // A silent u does not separate the vowels around it (guie, quie).
            if !silent_u {
                prev = None;
            }
            continue;
        }
        let c = if final_y { 'i' } else { c };
        let hiatus = |a: char, b: char| {
            let accented_weak = |x: char| matches!(x, 'í' | 'ú');
            (is_strong(a) && is_strong(b))
                || (accented_weak(a) && is_strong(b))
                || (is_strong(a) && accented_weak(b))
        };
        match prev {
            Some(p) if !hiatus(p, c) => {}
            _ => count += 1,
        }
        prev = Some(c);
    }
    count
}

/// Monosyllables take no written accent (fue, dio, vio, fie, rio, hui, riais),
/// except the diacritic ones *dé* and *sé*.
fn unaccent_monosyllable(form: String) -> String {
    if form == "dé" || form == "sé" || !form.chars().any(is_accented) || syllables(&form) != 1 {
        return form;
    }
    form.chars().map(strip_accent).collect()
}

/// Accent the last vowel of `s` (hablá-ramos, tuvié-ramos).
fn accent_last_vowel(s: &mut String) {
    if let Some((at, c)) = s.char_indices().rev().find(|&(_, c)| is_vowel(c)) {
        s.replace_range(at..at + c.len_utf8(), &with_accent(c).to_string());
    }
}

/// Prefix + root form, restoring written accents the compound needs:
/// a monosyllabic root form becomes an oxytone (ves → prevés, vio → previó,
/// ven → prevén), and a stressed initial i/u (after a silent h or not) is in
/// hiatus with the prefix's vowel (hice → rehíce, uso → reúso).
fn prefixed(prefix: &str, form: &str) -> String {
    let chars: Vec<char> = form.chars().collect();
    let unaccented = !chars.iter().any(|&c| is_accented(c));
    let open_end = form.ends_with(|c: char| is_vowel(c) || c == 'n' || c == 's');
    let n = syllables(form);
    let initial = usize::from(chars.first() == Some(&'h')); // skip a silent h
    let stressed = if unaccented && open_end && n == 1 {
        chars
            .iter()
            .rposition(|&c| is_strong(c))
            .or_else(|| chars.iter().rposition(|&c| is_vowel(c)))
    } else if unaccented
        && open_end
        && n == 2
        && prefix.ends_with(is_strong)
        && matches!(chars.get(initial), Some('i' | 'u'))
        && chars.get(initial + 1).is_some_and(|&c| !is_vowel(c))
    {
        Some(initial)
    } else {
        None
    };
    if stressed.is_none() {
        return format!("{prefix}{form}");
    }
    let accented: String = chars
        .iter()
        .enumerate()
        .map(|(k, &c)| {
            if Some(k) == stressed {
                with_accent(c)
            } else {
                c
            }
        })
        .collect();
    format!("{prefix}{accented}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all(inf: &str, t: Tense) -> Vec<String> {
        Person::ALL
            .iter()
            .map(|&p| conjugate(inf, t, p).unwrap())
            .collect()
    }

    #[test]
    fn regular_verbs() {
        assert_eq!(
            all("hablar", Tense::Present),
            ["hablo", "hablas", "habla", "hablamos", "habláis", "hablan"]
        );
        assert_eq!(
            all("comer", Tense::Preterite),
            [
                "comí",
                "comiste",
                "comió",
                "comimos",
                "comisteis",
                "comieron"
            ]
        );
        assert_eq!(all("vivir", Tense::ImperfectSubjunctive)[3], "viviéramos");
    }

    #[test]
    fn pronominal() {
        assert_eq!(
            all("levantarse", Tense::Present),
            [
                "me levanto",
                "te levantas",
                "se levanta",
                "nos levantamos",
                "os levantáis",
                "se levantan"
            ]
        );
        assert_eq!(
            conjugate("irse", Tense::Present, Person::Yo).unwrap(),
            "me voy"
        );
        assert_eq!(
            conjugate("sentarse", Tense::Preterite, Person::Ellos).unwrap(),
            "se sentaron"
        );
    }

    #[test]
    fn irregular_and_compounds() {
        assert_eq!(
            conjugate("tener", Tense::Future, Person::Yo).unwrap(),
            "tendré"
        );
        assert_eq!(
            conjugate("mantener", Tense::Preterite, Person::El).unwrap(),
            "mantuvo"
        );
        assert_eq!(
            conjugate("prever", Tense::Present, Person::Tu).unwrap(),
            "prevés"
        );
        assert_eq!(
            conjugate("prever", Tense::Preterite, Person::El).unwrap(),
            "previó"
        );
        assert_eq!(
            conjugate("hacer", Tense::Preterite, Person::El).unwrap(),
            "hizo"
        );
        assert_eq!(
            conjugate("dar", Tense::PresentSubjunctive, Person::El).unwrap(),
            "dé"
        );
        assert_eq!(
            conjugate("reír", Tense::Preterite, Person::El).unwrap(),
            "rio"
        );
        assert_eq!(
            conjugate("sonreír", Tense::Preterite, Person::El).unwrap(),
            "sonrió"
        );
        assert_eq!(
            conjugate("huir", Tense::Present, Person::Vosotros).unwrap(),
            "huis"
        );
        assert_eq!(
            conjugate("haber", Tense::Present, Person::El).unwrap(),
            "ha"
        );
    }

    #[test]
    fn not_a_verb() {
        assert_eq!(conjugate("casa", Tense::Present, Person::Yo), None);
        assert_eq!(conjugate("", Tense::Present, Person::Yo), None);
        assert_eq!(conjugate("ar", Tense::Present, Person::Yo), None);
        assert_eq!(conjugate("hola mundo", Tense::Present, Person::Yo), None);
    }

    #[test]
    fn irregularity() {
        assert!(!is_irregular("hablar", Tense::Present));
        assert!(!is_irregular("buscar", Tense::Preterite)); // busqué is spelling only
        assert!(!is_irregular("leer", Tense::Preterite)); // leyó likewise
        assert!(is_irregular("pensar", Tense::Present));
        assert!(!is_irregular("pensar", Tense::Preterite));
        assert!(is_irregular("pedir", Tense::Preterite));
        assert!(is_irregular("conocer", Tense::Present));
        assert!(is_irregular("enviar", Tense::Present));
        assert!(is_irregular("tener", Tense::Future));
        assert!(!is_irregular("ser", Tense::Future));
        assert!(
            is_irregular("levantarse", Tense::Present) == is_irregular("levantar", Tense::Present)
        );
        assert!(!is_irregular("xyz", Tense::Present));
    }

    #[test]
    fn syllable_counting() {
        for (w, n) in [
            ("fue", 1),
            ("guie", 1),
            ("riais", 1),
            ("hui", 1),
            ("reí", 2),
            ("río", 2),
            ("leéis", 2),
            ("hay", 1),
            ("que", 1),
            ("argüí", 2),
        ] {
            assert_eq!(syllables(w), n, "{w}");
        }
    }
}
