//! Every Spanish mode opens a playable round, and playing it to the end (with hints) works, many times over, at every
//! level. Also plays the first campaign levels.
use fuera_ui::{App, Host};

struct Fake {
    save: String,
    day: f64,
}

/// A host that records what is shared and keeps its save where the test can read it.
struct Shared {
    save: std::rc::Rc<std::cell::RefCell<String>>,
    shared: std::rc::Rc<std::cell::RefCell<Vec<String>>>,
    locale: &'static str,
}
impl Host for Shared {
    fn load(&mut self) -> Option<String> {
        Some(self.save.borrow().clone())
    }
    fn store(&mut self, t: &str) {
        *self.save.borrow_mut() = t.to_string();
    }
    fn today(&self) -> i64 {
        20_410
    }
    fn now_days(&self) -> f64 {
        20_410.4
    }
    fn locale(&self) -> String {
        self.locale.into()
    }
    fn share(&mut self, subject: &str, body: &str) -> bool {
        self.shared.borrow_mut().push(format!("{subject}\n{body}"));
        true
    }
}

impl Host for Fake {
    fn load(&mut self) -> Option<String> {
        Some(self.save.clone())
    }
    fn store(&mut self, t: &str) {
        self.save = t.to_string();
    }
    fn today(&self) -> i64 {
        self.day as i64
    }
    fn now_days(&self) -> f64 {
        self.day
    }
}

fn run(app: &mut App, steps: usize) {
    for _ in 0..steps {
        app.update(1.0 / 30.0);
    }
}

#[test]
fn every_mode_plays_through() {
    for placed in ["", "l.placed=B1\n", "l.placed=B2\n"] {
        let mut app = App::new(Box::new(Fake { save: format!("lang=es\n{placed}"), day: 20_400.3 }));
        app.resize(412, 915, 0.0, 0.0);
        let mut pm = tiny_skia::Pixmap::new(412, 915).unwrap();
        app.open("learn");
        for round in 0..9 {
            app.open(&format!("mode:{}", round % 9));
            app.open("go");
            assert_eq!(app.screen_name(), "round", "mode {} did not open (placed {placed:?})", round % 9);
            app.draw(&mut pm);
            let mut moves = 0;
            while app.auto_move() {
                run(&mut app, 20);
                moves += 1;
                assert!(moves < 200, "round never ends");
            }
            run(&mut app, 60);
            app.draw(&mut pm);
            app.open("home");
        }
    }
}

#[test]
fn placement_then_session() {
    let mut app = App::new(Box::new(Fake { save: "lang=fr\n".into(), day: 20_401.5 }));
    app.resize(412, 915, 0.0, 0.0);
    app.open("learn");
    app.open("placement");
    for k in 0..4 {
        let mut n = 0;
        while app.auto_move() {
            run(&mut app, 20);
            n += 1;
        }
        run(&mut app, 60);
        eprintln!("placement round {k}: {n} moves, screen {}", app.screen_name());
        app.open("next");
        eprintln!("  after next: {}", app.screen_name());
    }
    assert_eq!(app.screen_name(), "placed");
    app.open("session");
    for _ in 0..5 {
        app.open("go");
        assert_eq!(app.screen_name(), "round");
        while app.auto_move() {
            run(&mut app, 20);
        }
        run(&mut app, 60);
        app.open("next");
    }
    assert_eq!(app.screen_name(), "learn", "the daily session should end after 5 rounds");
}

#[test]
fn first_campaign_levels_with_hints() {
    let mut app = App::new(Box::new(Fake { save: String::new(), day: 20_400.0 }));
    app.resize(412, 915, 0.0, 0.0);
    for i in 0..12 {
        app.open(&format!("level:{i}"));
        assert_eq!(app.screen_name(), "play");
        let mut n = 0;
        while app.auto_move() {
            run(&mut app, 15);
            n += 1;
            assert!(n < 300);
        }
        run(&mut app, 60);
    }
}

#[test]
fn every_course_plays_through() {
    for course in ["es", "es-419", "fr"] {
        for placed in ["", "l.placed=B1\nfr.l.placed=B1\n", "l.placed=B2\nfr.l.placed=B2\n"] {
            let mut app = App::new(Box::new(Fake { save: format!("lang=es\n{placed}"), day: 20_400.3 }));
            app.resize(412, 915, 0.0, 0.0);
            let mut pm = tiny_skia::Pixmap::new(412, 915).unwrap();
            app.open("learn");
            if placed.is_empty() {
                assert_eq!(app.screen_name(), "courses", "no course yet: the picker comes first");
            }
            app.draw(&mut pm);
            app.open(&format!("course:{course}"));
            assert_eq!(app.screen_name(), "learn");
            app.draw(&mut pm);
            for round in 0..9 {
                app.open(&format!("mode:{round}"));
                app.open("go");
                assert_eq!(app.screen_name(), "round", "{course}: mode {round} did not open (placed {placed:?})");
                app.draw(&mut pm);
                let mut moves = 0;
                while app.auto_move() {
                    run(&mut app, 20);
                    moves += 1;
                    assert!(moves < 200, "round never ends");
                }
                run(&mut app, 60);
                app.draw(&mut pm);
                app.open("home");
            }
        }
    }
}

#[test]
fn courses_keep_their_own_progress_and_old_saves_load() {
    // A 1.0.1 save: Spanish progress under m. / l., no course key.
    let old = "lang=fr\nl.placed=B1\nl.rounds=42\nm.w:casa|n=1,2,3,4,5,6\n".to_string();
    let save = std::rc::Rc::new(std::cell::RefCell::new(old));
    let shared = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut app = App::new(Box::new(Shared { save: save.clone(), shared: shared.clone(), locale: "fr-FR" }));
    app.resize(412, 915, 0.0, 0.0);
    app.open("course:fr");
    app.open("mode:0");
    app.open("go");
    while app.auto_move() {
        run(&mut app, 20);
    }
    run(&mut app, 60);
    app.open("home");
    let s = save.borrow().clone();
    assert!(s.contains("course=fr"), "{s}");
    assert!(s.contains("fr.l.rounds=1"), "French progress under fr.: {s}");
    assert!(s.contains("l.rounds=42") && s.contains("m.w:casa|n="), "Spanish progress kept: {s}");
    app.open("course:es");
    let s = save.borrow().clone();
    assert!(s.contains("course=es") && s.contains("es_variant=es"), "{s}");
}

#[test]
fn an_old_save_stays_in_spanish_from_spain() {
    // A 1.0.1 save on a Spanish-speaking (Honduran) device: still Spain's Spanish, no picker.
    let save = std::rc::Rc::new(std::cell::RefCell::new("lang=es\nl.rounds=3\nm.w:casa|n=1,2,3,4,5,6\n".to_string()));
    let shared = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut app = App::new(Box::new(Shared { save: save.clone(), shared, locale: "es-HN" }));
    app.resize(412, 915, 0.0, 0.0);
    app.open("learn");
    assert_eq!(app.screen_name(), "learn");
    assert_eq!(app.wanted_voice_pack(), Some("es"));
    // A new player on the same device: nothing downloaded before a course is chosen, French offered first.
    let fresh = std::rc::Rc::new(std::cell::RefCell::new("lang=es\n".to_string()));
    let app2 = App::new(Box::new(Shared { save: fresh, shared: Default::default(), locale: "es-HN" }));
    assert_eq!(app2.wanted_voice_pack(), None);
}

#[test]
fn feedback_carries_the_reported_words() {
    let save = std::rc::Rc::new(std::cell::RefCell::new("lang=es\n".to_string()));
    let shared = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut app = App::new(Box::new(Shared { save: save.clone(), shared: shared.clone(), locale: "es-HN" }));
    app.resize(412, 915, 0.0, 0.0);
    app.open("learn");
    app.open("course:fr");
    app.open("mode:0");
    app.open("go");
    while app.auto_move() {
        run(&mut app, 20);
    }
    run(&mut app, 60);
    // Report the first word of the summary as "the voice sounds wrong".
    app.report_result(0, 0);
    app.open("feedback");
    let sent = shared.borrow().clone();
    assert_eq!(sent.len(), 1);
    assert!(sent[0].contains("es→fr"), "{}", sent[0]);
    assert!(sent[0].contains("La voz suena mal"), "{}", sent[0]);
    assert!(save.borrow().contains("fb.flags=fr"), "reports stay saved (a closed share sheet loses nothing)");
    assert!(sent[0].contains("android") || sent[0].contains("linux") || sent[0].contains("macos") || sent[0].contains("windows"));
}
