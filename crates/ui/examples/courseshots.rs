//! Screenshots of the courses (picker, French course for a Spanish speaker, Latin-American Spanish):
//! `cargo run -p fuera-ui --example courseshots -- outdir [lang] [course]`.
use fuera_ui::{App, Host};
use tiny_skia::Pixmap;

struct Fake {
    lang: String,
}
impl Host for Fake {
    fn load(&mut self) -> Option<String> {
        Some(format!("lang={}\n", self.lang))
    }
    fn store(&mut self, _t: &str) {}
    fn today(&self) -> i64 {
        20_400
    }
    fn now_days(&self) -> f64 {
        20_400.5
    }
    fn locale(&self) -> String {
        "es-HN".into()
    }
}

fn shot(app: &mut App, pm: &mut Pixmap, dir: &str, name: &str) {
    for _ in 0..40 {
        app.update(1.0 / 60.0);
    }
    app.draw(pm);
    pm.save_png(format!("{dir}/{name}.png")).unwrap();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let lang = a.get(2).cloned().unwrap_or_else(|| "es".into());
    let course = a.get(3).cloned().unwrap_or_else(|| "fr".into());
    let mut app = App::new(Box::new(Fake { lang }));
    app.resize(412, 915, 0.0, 0.0);
    let mut pm = Pixmap::new(412, 915).unwrap();
    shot(&mut app, &mut pm, dir, "c0-home");
    app.open("learn");
    shot(&mut app, &mut pm, dir, "c1-courses");
    app.open(&format!("course:{course}"));
    shot(&mut app, &mut pm, dir, "c2-learn");
    app.open("session");
    shot(&mut app, &mut pm, dir, "c3-intro");
    app.open("go");
    shot(&mut app, &mut pm, dir, "c4-words");
    for k in 0..9usize {
        app.open(&format!("mode:{k}"));
        app.open("go");
        shot(&mut app, &mut pm, dir, &format!("c5-mode{k}"));
    }
    // Play a round to the end, then report the first word.
    app.open("mode:4");
    app.open("go");
    while app.auto_move() {
        for _ in 0..20 {
            app.update(1.0 / 60.0);
        }
    }
    shot(&mut app, &mut pm, dir, "c6-done");
    app.report_result(0, 9);
    shot(&mut app, &mut pm, dir, "c7-reported");
    app.open("placement");
    shot(&mut app, &mut pm, dir, "c8-placement");
    app.open("settings");
    shot(&mut app, &mut pm, dir, "c9-settings");
}
