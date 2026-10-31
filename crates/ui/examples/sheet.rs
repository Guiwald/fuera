//! Renders several campaign levels side by side: `cargo run -p fuera-ui --example sheet -- out.png 1 8 16 …`.
use fuera_core::Game;
use fuera_ui::board::{BoardView, Skin};
use fuera_ui::text::{Align, Face, Fonts};
use fuera_ui::{gfx, theme};
use tiny_skia::Pixmap;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let levels = fuera_ui::app::parse_pack(&std::fs::read_to_string("assets/levels/campaign.txt").unwrap());
    let picks: Vec<usize> = a[2..].iter().map(|s| s.parse::<usize>().unwrap() - 1).collect();
    let (cw, ch) = (300u32, 420u32);
    let cols = 5u32.min(picks.len() as u32);
    let rows = (picks.len() as u32).div_ceil(cols);
    let mut pm = Pixmap::new(cw * cols, ch * rows).unwrap();
    let bg = gfx::vgrad(0.0, (ch * rows) as f32, &[(0.0, theme::col(theme::SEA_TOP)), (1.0, theme::col(theme::SEA_BOTTOM))]);
    gfx::fill_rect(&mut pm, 0.0, 0.0, (cw * cols) as f32, (ch * rows) as f32, &bg);
    let mut fonts = Fonts::new();
    for (k, &i) in picks.iter().enumerate() {
        let (x, y) = ((k as u32 % cols * cw) as f32, (k as u32 / cols * ch) as f32);
        let game = Game::new(levels[i].clone());
        let mut v = BoardView::new(&game, Skin::classic(&levels[i]));
        v.layout(&levels[i], x + 8.0, y + 40.0, cw as f32 - 16.0, ch as f32 - 50.0);
        v.draw(&mut pm, &game, &mut fonts);
        let t = format!("{} · opt {}", i + 1, levels[i].meta.optimal.unwrap_or(0));
        fonts.draw(&mut pm, Face::Title, &t, x + cw as f32 / 2.0, y + 28.0, 22.0, theme::col(theme::WHITE), Align::Center);
    }
    pm.save_png(&a[1]).unwrap();
}
