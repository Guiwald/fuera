//! Colours of the "Mediterranean" look: deep-sea background, white-plaster tray, blue-grey tiles, glossy bricks.

use tiny_skia::Color;

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba8(r, g, b, 255)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

/// Brick colours in key order (see `fuera_core::level::COLOR_NAMES`).
pub const KEY_COLORS: [(u8, u8, u8); 8] = [
    (0xE5, 0x48, 0x4D), // red
    (0x3E, 0x7B, 0xFA), // blue
    (0xF7, 0xC4, 0x30), // yellow
    (0x3D, 0xBA, 0x6F), // green
    (0x8E, 0x5B, 0xE8), // purple
    (0xF7, 0x88, 0x3A), // orange
    (0x2C, 0xC4, 0xD6), // cyan
    (0xF2, 0x5C, 0xA8), // pink
];

/// Neutral brick for word blocks (language modes), where colour must not give the answer away.
pub const NEUTRAL: (u8, u8, u8) = (0xF4, 0xEE, 0xE2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Symbol {
    Circle,
    Square,
    Star,
    Triangle,
    Diamond,
    Plus,
    Hexagon,
    Heart,
}

pub const KEY_SYMBOLS: [Symbol; 8] = [
    Symbol::Circle,
    Symbol::Square,
    Symbol::Star,
    Symbol::Triangle,
    Symbol::Diamond,
    Symbol::Plus,
    Symbol::Hexagon,
    Symbol::Heart,
];

pub fn key_rgb(k: u16) -> (u8, u8, u8) {
    KEY_COLORS.get(k as usize).copied().unwrap_or(NEUTRAL)
}

pub fn key_symbol(k: u16) -> Option<Symbol> {
    KEY_SYMBOLS.get(k as usize).copied()
}

pub fn shade((r, g, b): (u8, u8, u8), f: f32) -> (u8, u8, u8) {
    // f > 0 lightens towards white, f < 0 darkens towards black.
    let m = |c: u8| -> u8 {
        let c = c as f32;
        let v = if f >= 0.0 { c + (255.0 - c) * f } else { c * (1.0 + f) };
        v.round().clamp(0.0, 255.0) as u8
    };
    (m(r), m(g), m(b))
}

pub fn col((r, g, b): (u8, u8, u8)) -> Color {
    rgb(r, g, b)
}

pub fn col_a((r, g, b): (u8, u8, u8), a: f32) -> Color {
    Color::from_rgba8(r, g, b, (a.clamp(0.0, 1.0) * 255.0) as u8)
}

pub const SEA_TOP: (u8, u8, u8) = (0x0C, 0x33, 0x52);
pub const SEA_BOTTOM: (u8, u8, u8) = (0x17, 0x68, 0x88);
pub const PLASTER: (u8, u8, u8) = (0xF7, 0xF3, 0xEA);
pub const PLASTER_SIDE: (u8, u8, u8) = (0xCC, 0xC2, 0xB0);
pub const TILE_A: (u8, u8, u8) = (0xD6, 0xE2, 0xEB);
pub const TILE_B: (u8, u8, u8) = (0xDF, 0xE9, 0xF0);
pub const TILE_LINE: (u8, u8, u8) = (0xBF, 0xD0, 0xDE);
pub const AZULEJO: (u8, u8, u8) = (0x9D, 0xB8, 0xD0);
pub const INK: (u8, u8, u8) = (0x1D, 0x2B, 0x3A);
pub const WHITE: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
pub const GOLD: (u8, u8, u8) = (0xFF, 0xC9, 0x3C);
pub const BUTTON: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
pub const ACCENT: (u8, u8, u8) = (0xFF, 0x7A, 0x59);
pub const ICE: (u8, u8, u8) = (0xD8, 0xF3, 0xFF);
