//! The app icon, drawn in code (all sizes come from the same drawing).

use fuera_core::Pos;
use tiny_skia::{Pixmap, Transform};

use crate::board::{Look, render_brick};
use crate::gfx::{self, fill, solid};
use crate::theme::{self, col, shade};

/// Icon of `size` pixels. `rounded`: draw the rounded-square background (desktop, web); Android launchers want the
/// full square.
pub fn app_icon(size: u32, rounded: bool) -> Pixmap {
    let mut pm = Pixmap::new(size, size).unwrap();
    let s = size as f32;
    let id = Transform::identity();
    let bg = gfx::vgrad(0.0, s, &[(0.0, col(shade(theme::SEA_TOP, 0.15))), (1.0, col(theme::SEA_BOTTOM))]);
    let r = if rounded { s * 0.22 } else { 0.0 };
    let m = if rounded { s * 0.04 } else { 0.0 };
    fill(&mut pm, &gfx::rrect(m, m, s - 2.0 * m, s - 2.0 * m, r), &bg, id);
    // Plaster wall with a door on the right.
    let c = s * 0.3;
    let wx = s * 0.66;
    fill(&mut pm, &gfx::rrect(wx, s * 0.2, s * 0.16, s * 0.62, s * 0.06), &solid(col(theme::PLASTER_SIDE)), id);
    fill(&mut pm, &gfx::rrect(wx, s * 0.18, s * 0.16, s * 0.62, s * 0.06), &solid(col(theme::PLASTER)), id);
    let door = theme::KEY_COLORS[7];
    fill(&mut pm, &gfx::rrect(wx + s * 0.03, s * 0.34, s * 0.1, s * 0.32, s * 0.03), &solid(col(door)), id);
    let ch = gfx::chevron(wx + s * 0.08, s * 0.5, s * 0.07, 1.0, 0.0);
    fill(&mut pm, &ch, &solid(col(theme::WHITE)), id);
    // A brick about to slide through, with motion streaks.
    for k in 0..3 {
        let y = s * (0.42 + 0.08 * k as f32);
        let x0 = s * (0.1 + 0.04 * (k % 2) as f32);
        fill(&mut pm, &gfx::rrect(x0, y, s * 0.18, s * 0.035, s * 0.02), &solid(theme::rgba(255, 255, 255, 70)), id);
    }
    let brick = render_brick(&[Pos::new(0, 0)], c, &Look::key(7), None);
    gfx::blit(&mut pm, &brick, (s * 0.33).round(), (s * 0.5 - c / 2.0 - c * 0.18).round(), 1.0, 1.0);
    pm
}
