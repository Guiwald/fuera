//! Every Spanish mode opens a playable round, and playing it to the end (with hints) works, many times over, at every
//! level. Also plays the first campaign levels.
use fuera_ui::{App, Host};

struct Fake {
    save: String,
    day: f64,
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
