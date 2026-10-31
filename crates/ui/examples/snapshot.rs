//! Renders a level to a PNG: `cargo run -p fuera-ui --example snapshot -- level.lvl out.png [width height]`.
use fuera_core::{Game, Level};
use fuera_ui::board::{BoardView, Skin};
use fuera_ui::text::Fonts;
use fuera_ui::{gfx, theme};
use tiny_skia::Pixmap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let level = Level::parse(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    for e in level.validate() {
        eprintln!("warning: {e}");
    }
    let w: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(540);
    let h: u32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(1200);
    let mut pm = Pixmap::new(w, h).unwrap();
    let bg = gfx::vgrad(0.0, h as f32, &[(0.0, theme::col(theme::SEA_TOP)), (1.0, theme::col(theme::SEA_BOTTOM))]);
    gfx::fill_rect(&mut pm, 0.0, 0.0, w as f32, h as f32, &bg);
    let game = Game::new(level.clone());
    let mut fonts = Fonts::new();
    let mut view = BoardView::new(&game, Skin::classic(&level));
    view.layout(&level, 0.0, h as f32 * 0.12, w as f32, h as f32 * 0.7);
    view.select(Some(3));
    view.draw(&mut pm, &game, &mut fonts);
    fonts.draw_shadowed(&mut pm, fuera_ui::text::Face::Title, "Nivel 2", w as f32 / 2.0, h as f32 * 0.08, 44.0, theme::col(theme::WHITE), fuera_ui::text::Align::Center);
    pm.save_png(&a[2]).unwrap();
}
