//! Renders every screen to PNG files with a fake host: `cargo run -p fuera-ui --example shots -- outdir [w h]`.
use fuera_ui::{App, Host, KeyEvent, PointerEvent};
use tiny_skia::Pixmap;

struct Fake;
impl Host for Fake {
    fn load(&mut self) -> Option<String> { Some("lang=es\nc0=3\n".into()) }
    fn store(&mut self, _t: &str) {}
}

fn shot(app: &mut App, pm: &mut Pixmap, dir: &str, name: &str) {
    for _ in 0..30 { app.update(1.0 / 60.0); }
    app.draw(pm);
    pm.save_png(format!("{dir}/{name}.png")).unwrap();
}

fn tap(app: &mut App, x: f32, y: f32) {
    app.pointer(PointerEvent::Down, x, y);
    app.pointer(PointerEvent::Up, x, y);
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let dir = &a[1];
    let w: u32 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(412);
    let h: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(915);
    let mut app = App::new(Box::new(Fake));
    app.resize(w, h, 0.0, 0.0);
    let mut pm = Pixmap::new(w, h).unwrap();
    shot(&mut app, &mut pm, dir, "1-home");
    app.key(KeyEvent::Enter);
    shot(&mut app, &mut pm, dir, "2-play");
    app.key(KeyEvent::Arrow(fuera_core::Dir::E));
    shot(&mut app, &mut pm, dir, "3-moved");
    app.key(KeyEvent::Hint);
    shot(&mut app, &mut pm, dir, "4-hint");
    app.key(KeyEvent::Back);
    shot(&mut app, &mut pm, dir, "5-levels");
    let _ = tap;
}
