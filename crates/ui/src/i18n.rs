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
    Frozen => ["Gelé : encore {} sortie(s)", "Frozen: {} more exit(s)", "Congelado: faltan {} salida(s)"],
    Locked => ["Enchaîné : fais sortir la clé {}", "Chained: get key {} out", "Encadenado: saca la llave {}"],
    LaneH => ["Ne bouge qu'à l'horizontale", "Moves sideways only", "Solo se mueve en horizontal"],
    LaneV => ["Ne bouge qu'à la verticale", "Moves up and down only", "Solo se mueve en vertical"],
    NoHint => ["Pas de solution d'ici : annule un coup.", "No way out from here: undo a move.", "No hay solución desde aquí: deshaz."],
    Language => ["Langue", "Language", "Idioma"],
    Sound => ["Sons", "Sounds", "Sonidos"],
    Vibration => ["Vibrations", "Vibration", "Vibración"],
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
}
