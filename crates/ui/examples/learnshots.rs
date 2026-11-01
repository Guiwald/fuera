//! Screenshots of the Spanish mode with a fake host: `cargo run -p fuera-ui --example learnshots -- outdir`.
use fuera_ui::{App, Host};
use tiny_skia::Pixmap;

struct Fake;
impl Host for Fake {
    fn load(&mut self) -> Option<String> {
        Some("lang=fr\n".into())
    }
    fn store(&mut self, _t: &str) {}
    fn today(&self) -> i64 {
        20_400
    }
    fn now_days(&self) -> f64 {
        20_400.5
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
    let mut app = App::new(Box::new(Fake));
    app.resize(412, 915, 0.0, 0.0);
    let mut pm = Pixmap::new(412, 915).unwrap();
    app.open("learn");
    shot(&mut app, &mut pm, dir, "l1-home");
    app.open("session");
    shot(&mut app, &mut pm, dir, "l2-intro");
    app.open("go");
    shot(&mut app, &mut pm, dir, "l3-words");
    // A wrong move: push the first tile into a door until a bounce happens, else play the hint.
    for _ in 0..3 {
        app.auto_move();
        shot(&mut app, &mut pm, dir, "l4-moving");
    }
    for k in [2usize, 3, 4, 5, 6, 7, 8] {
        app.open(&format!("mode:{k}"));
        app.open("go");
        shot(&mut app, &mut pm, dir, &format!("l5-mode{k}"));
    }
    app.open("settings");
    shot(&mut app, &mut pm, dir, "l8-settings");
    app.open("placement");
    shot(&mut app, &mut pm, dir, "l6-placement");
    for _ in 0..12 {
        if !app.auto_move() {
            break;
        }
        for _ in 0..30 {
            app.update(1.0 / 60.0);
        }
    }
    shot(&mut app, &mut pm, dir, "l7-placement-done");
}
