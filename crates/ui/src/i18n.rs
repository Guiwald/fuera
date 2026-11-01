//! Interface texts in French, English and Spanish.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Fr,
    En,
    Es,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    pub fn from_code(s: &str) -> Option<Lang> {
        match s.get(..2)? {
            "fr" => Some(Lang::Fr),
            "en" => Some(Lang::En),
            "es" => Some(Lang::Es),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
            Lang::Es => "Español",
        }
    }

    pub fn next(self) -> Lang {
        match self {
            Lang::Fr => Lang::En,
            Lang::En => Lang::Es,
            Lang::Es => Lang::Fr,
        }
    }
}

/// `[fr, en, es]` for each message.
macro_rules! msgs {
    ($($id:ident => [$fr:expr, $en:expr, $es:expr]),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum T { $($id),* }
        pub fn tr(lang: Lang, t: T) -> &'static str {
            match t { $(T::$id => match lang { Lang::Fr => $fr, Lang::En => $en, Lang::Es => $es }),* }
        }
    };
}

msgs! {
    Play => ["Jouer", "Play", "Jugar"],
    Levels => ["Niveaux", "Levels", "Niveles"],
    LearnSpanish => ["Apprendre l'espagnol", "Learn Spanish", "Aprender español"],
    LearnFrench => ["Apprendre le français", "Learn French", "Aprender francés"],
    LearnLanguage => ["Apprendre une langue", "Learn a language", "Aprender un idioma"],
    WhichLanguage => ["Quelle langue veux-tu apprendre ?", "Which language do you want to learn?", "¿Qué idioma quieres aprender?"],
    CourseSpain => ["Espagnol d'Espagne", "Spanish from Spain", "Español de España"],
    CourseLatam => ["Espagnol d'Amérique latine", "Latin-American Spanish", "Español de Latinoamérica"],
    CourseFrench => ["Français (France)", "French (France)", "Francés (Francia)"],
    CourseSpainNote => ["vosotros, coche, ordenador", "vosotros, coche, ordenador", "vosotros, coche, ordenador"],
    CourseLatamNote => ["ustedes, carro, computadora", "ustedes, carro, computadora", "ustedes, carro, computadora"],
    CourseFrenchNote => ["avec des voix de France", "with voices from France", "con voces de Francia"],
    ProgressKept => ["Chaque langue garde ses progrès.", "Each language keeps its own progress.", "Cada idioma guarda su progreso."],
    French => ["Français", "French", "Francés"],
    SpanishVariant => ["Espagnol", "Spanish", "Español"],
    Spain => ["Espagne", "Spain", "España"],
    LatinAmerica => ["Amérique latine", "Latin America", "Latinoamérica"],
    Feedback => ["Donner mon avis", "Send feedback", "Dar mi opinión"],
    FeedbackAsk => ["Un avis ? Écris-le ou dis-le à voix haute.", "Any thoughts? Write them or just say them.", "¿Una opinión? Escríbela o dila en voz alta."],
    FeedbackBody => ["Écris ou enregistre ton message ici :", "Write or record your message here:", "Escribe o graba tu mensaje aquí:"],
    FeedbackFlags => ["Mots signalés", "Reported words", "Palabras señaladas"],
    Report => ["Signaler ce mot", "Report this word", "Señalar esta palabra"],
    ReportVoice => ["La voix sonne mal", "The voice sounds wrong", "La voz suena mal"],
    ReportMeaning => ["Traduction bizarre", "Odd translation", "Traducción rara"],
    ReportOther => ["Autre chose", "Something else", "Otra cosa"],
    Cancel => ["Annuler", "Cancel", "Cancelar"],
    Reported => ["Noté, merci ! Il partira avec ton prochain avis.", "Noted, thanks! It goes with your next feedback.", "¡Anotado, gracias! Irá con tu próxima opinión."],
    InsAuxiliary => ["Être ou avoir : quel auxiliaire ?", "Être or avoir: which auxiliary?", "¿Être o avoir? Elige el auxiliar."],
    Settings => ["Réglages", "Settings", "Ajustes"],
    Level => ["Niveau", "Level", "Nivel"],
    Moves => ["Coups", "Moves", "Movimientos"],
    Best => ["Record", "Best", "Récord"],
    Goal => ["Objectif", "Goal", "Objetivo"],
    Undo => ["Annuler", "Undo", "Deshacer"],
    Restart => ["Recommencer", "Restart", "Reiniciar"],
    Hint => ["Indice", "Hint", "Pista"],
    Next => ["Suivant", "Next", "Siguiente"],
    Replay => ["Rejouer", "Replay", "Repetir"],
    Back => ["Retour", "Back", "Volver"],
    Cleared => ["Bravo !", "Cleared!", "¡Fuera!"],
    Stuck => ["Bloqué ! Annule ou recommence.", "Stuck! Undo or restart.", "¡Atascado! Deshaz o reinicia."],
    DeadEnd => ["Impasse : plus de solution d'ici. Annule.", "Dead end: no way out from here. Undo.", "Callejón sin salida: deshaz."],
    Frozen => ["Gelé : encore {} sortie(s)", "Frozen: {} more exit(s)", "Congelado: faltan {} salida(s)"],
    Locked => ["Enchaîné : fais sortir la clé {}", "Chained: get key {} out", "Encadenado: saca la llave {}"],
    LaneH => ["Ne bouge qu'à l'horizontale", "Moves sideways only", "Solo se mueve en horizontal"],
    LaneV => ["Ne bouge qu'à la verticale", "Moves up and down only", "Solo se mueve en vertical"],
    NoHint => ["Pas de solution d'ici : annule un coup.", "No way out from here: undo a move.", "No hay solución desde aquí: deshaz."],
    Language => ["Langue", "Language", "Idioma"],
    Sound => ["Sons", "Sounds", "Sonidos"],
    Vibration => ["Vibrations", "Vibration", "Vibración"],
    SayAnswer => ["Toucher lit la réponse", "Tap reads the answer", "Tocar lee la respuesta"],
    On => ["oui", "on", "sí"],
    Off => ["non", "off", "no"],
    Credits => ["Crédits et licences", "Credits & licences", "Créditos y licencias"],
    Tagline => ["Fais sortir chaque bloc par sa porte.", "Slide every block out through its door.", "Saca cada bloque por su puerta."],
    Tutorial1 => ["Glisse un bloc : il file jusqu'à l'obstacle.", "Swipe a block: it slides until it hits something.", "Desliza un bloque: avanza hasta chocar."],
    Tutorial2 => ["Il sort par la porte de sa couleur.", "It leaves through the door of its colour.", "Sale por la puerta de su color."],
    Tutorial3 => ["Bloqué ? Annule, c'est gratuit.", "Stuck? Undo is free.", "¿Atascado? Deshacer es gratis."],
    IceIntro => ["Glace : fond après des sorties.", "Ice melts after other blocks leave.", "El hielo se derrite cuando salen otros bloques."],
    LaneIntro => ["Flèches : ne bouge que dans ce sens.", "Arrows: moves along that line only.", "Flechas: solo se mueve en esa línea."],
    ShellIntro => ["Double couche : la couche du dessus part d'abord.", "Two layers: the outer one comes off first.", "Dos capas: la de fuera sale primero."],
    LockIntro => ["Chaîne : fais sortir le bloc à la clé.", "Chain: get the key block out first.", "Cadena: saca primero el bloque con la llave."],
    ShutterIntro => ["Volet : s'ouvre et se ferme à chaque sortie.", "Shutter: opens and closes on every exit.", "Persiana: se abre y se cierra con cada salida."],
    WallIntro => ["Murs : ils arrêtent les blocs.", "Walls stop blocks too.", "Las paredes también paran los bloques."],
    BigIntro => ["Gros blocs : il faut une porte assez large.", "Big blocks need a wide enough door.", "Los bloques grandes necesitan una puerta ancha."],
    Daily => ["Défi du jour", "Daily puzzle", "Reto del día"],
    Endless => ["Aléatoire", "Random", "Aleatorio"],
    StarsTotal => ["Étoiles", "Stars", "Estrellas"],
    ComingSoon => ["Bientôt !", "Coming soon!", "¡Muy pronto!"],
    Streak => ["Série", "Streak", "Racha"],
    Days => ["jours", "days", "días"],
    TimeUp => ["Temps écoulé !", "Time's up!", "¡Se acabó el tiempo!"],
    Retry => ["Réessayer", "Try again", "Reintentar"],
    Spanish => ["Espagnol", "Spanish", "Español"],
    TodaySession => ["Séance du jour", "Today's session", "Sesión de hoy"],
    PlacementAsk => ["Quel est ton niveau ? Test « {test} » (3 min)", "What's your level? “{test}” test (3 min)", "¿Qué nivel tienes? Test «{test}» (3 min)"],
    YourLevel => ["Ton niveau", "Your level", "Tu nivel"],
    Notebook => ["Mon cahier", "My notebook", "Mi cuaderno"],
    NotebookEmpty => ["Rien ici pour l'instant : les mots que tu rates viendront s'y ranger.", "Nothing yet: the words you miss will be collected here.", "Nada todavía: aquí se guardan las palabras que fallas."],
    Meanings => ["Sens en", "Meanings in", "Significados en"],
    NewWords => ["Nouveaux mots", "New words", "Palabras nuevas"],
    Go => ["C'est parti !", "Let's go!", "¡Vamos!"],
    TapToHear => ["Touche un mot pour l'écouter.", "Tap a word to hear it.", "Toca una palabra para oírla."],
    RoundDone => ["Manche terminée", "Round complete", "¡Ronda completada!"],
    Right => ["justes", "right", "bien"],
    Continue => ["Continuer", "Continue", "Seguir"],
    GoalDone => ["Objectif du jour atteint !", "Daily goal reached!", "¡Objetivo del día cumplido!"],
    Rounds => ["manches", "rounds", "rondas"],
    Words => ["mots", "words", "palabras"],
    Known => ["acquis", "known", "sabidas"],
    Learning => ["en cours", "learning", "aprendiendo"],
    InsWords => ["Envoie chaque mot vers son sens.", "Send each word to its meaning.", "Lleva cada palabra a su significado."],
    InsProduction => ["Envoie chaque sens vers le mot {lang}.", "Send each meaning to the {Lang} word.", "Lleva cada significado a su palabra en {lang}."],
    InsListening => ["Touche un bloc pour l'écouter, puis envoie-le vers son sens.", "Tap a block to hear it, then send it to its meaning.", "Toca un bloque para oírlo y llévalo a su significado."],
    InsGender => ["El ou la ?", "El or la?", "¿El o la?"],
    InsGenderFr => ["Un ou une ?", "Un or une?", "¿Un o une?"],
    InsVerbs => ["Chaque forme vers sa personne.", "Each form to its person.", "Cada forma con su persona."],
    InsBinary => ["Quel mot va dans le trou ?", "Which word fills the gap?", "¿Qué palabra falta?"],
    InsSentences => ["Chaque forme dans sa phrase ; les autres dans « {none} ».", "Each form into its sentence; the others into “{none}”.", "Cada forma en su frase; las demás a «{none}»."],
    InsFalse => ["Attention aux faux amis !", "Watch out for false friends!", "¡Cuidado con los falsos amigos!"],
    InsPlacement => ["Ce mot existe-t-il en {lang} ?", "Is this a real {Lang} word?", "¿Existe esta palabra en {lang}?"],
    PlacementDone => ["Test terminé", "Test complete", "Test terminado"],
    PlacementNote => ["Les mots plus faciles sont marqués comme acquis ; ils reviendront de temps en temps.", "Easier words are marked as known; they will come back now and then.", "Las palabras más fáciles cuentan como sabidas; volverán de vez en cuando."],
}

/// Name of a language learnt, as an adjective inside a sentence (`le mot {lang}`): espagnol, Spanish, español…
pub fn lang_adj(lang: Lang, target: fuera_learn::content::Target, capital: bool) -> &'static str {
    use fuera_learn::content::Target;
    match (lang, target, capital) {
        (Lang::Fr, Target::Es, _) => "espagnol",
        (Lang::Fr, Target::Fr, _) => "français",
        (Lang::En, Target::Es, _) => "Spanish",
        (Lang::En, Target::Fr, _) => "French",
        (Lang::Es, Target::Es, _) => "español",
        (Lang::Es, Target::Fr, _) => "francés",
    }
}

/// A message with the course filled in: `{lang}` / `{Lang}`, `{none}` (the bin door), `{test}` (the placement test).
pub fn tr_course(lang: Lang, t: T, target: fuera_learn::content::Target) -> String {
    use fuera_learn::content::Target;
    let none = if target == Target::Fr { "aucune" } else { "ninguna" };
    let test = fuera_learn::plan::Mode::Placement.name(target);
    tr(lang, t).replace("{lang}", lang_adj(lang, target, false)).replace("{Lang}", lang_adj(lang, target, true)).replace("{none}", none).replace("{test}", test)
}
