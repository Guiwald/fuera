//! Small vector icons drawn in a square of size `s` centred on (cx, cy).

use tiny_skia::{Color, Path, PathBuilder, Pixmap, Transform};

use crate::gfx::{self, fill, solid, stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Back,
    Undo,
    Restart,
    Hint,
    Gear,
    Star,
    Lock,
    Play,
    Speaker,
    Book,
    Close,
    Calendar,
    Dice,
}

fn arc(pb: &mut PathBuilder, cx: f32, cy: f32, r: f32, a0: f32, a1: f32) {
    let n = 24;
    for i in 0..=n {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
        if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
    }
}

pub fn star_path(cx: f32, cy: f32, r: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    for i in 0..10 {
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        let rr = if i % 2 == 0 { r } else { r * 0.48 };
        let (x, y) = (cx + rr * a.cos(), cy + rr * a.sin());
        if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
    }
    pb.close();
    pb.finish()
}

pub fn draw(pm: &mut Pixmap, icon: Icon, cx: f32, cy: f32, s: f32, color: Color) {
    let id = Transform::identity();
    let p = solid(color);
    let w = s * 0.12;
    match icon {
        Icon::Back => {
            let mut pb = PathBuilder::new();
            pb.move_to(cx + s * 0.12, cy - s * 0.3);
            pb.line_to(cx - s * 0.18, cy);
            pb.line_to(cx + s * 0.12, cy + s * 0.3);
            stroke(pm, &pb.finish(), &p, w * 1.2, id);
        }
        Icon::Undo | Icon::Restart => {
            let mut pb = PathBuilder::new();
            let (a0, a1) = if icon == Icon::Undo { (-2.6f32, 0.9f32) } else { (-1.2f32, 4.2f32) };
            arc(&mut pb, cx, cy + s * 0.04, s * 0.3, a0, a1);
            stroke(pm, &pb.finish(), &p, w, id);
            // Arrow head at the start of the arc.
            let (hx, hy) = (cx + s * 0.3 * a0.cos(), cy + s * 0.04 + s * 0.3 * a0.sin());
            let t = (a0 - std::f32::consts::FRAC_PI_2).sin_cos();
            let (tx, ty) = (t.1, t.0);
            let mut pb = PathBuilder::new();
            pb.move_to(hx + tx * s * 0.2, hy + ty * s * 0.2);
            pb.line_to(hx - ty * s * 0.14, hy + tx * s * 0.14);
            pb.line_to(hx + ty * s * 0.14, hy - tx * s * 0.14);
            pb.close();
            fill(pm, &pb.finish(), &p, id);
        }
        Icon::Hint => {
            let bulb = gfx::circle(cx, cy - s * 0.08, s * 0.24);
            fill(pm, &bulb, &p, id);
            fill(pm, &gfx::rrect(cx - s * 0.12, cy + s * 0.1, s * 0.24, s * 0.16, s * 0.04), &p, id);
            fill(pm, &gfx::rrect(cx - s * 0.09, cy + s * 0.29, s * 0.18, s * 0.06, s * 0.03), &p, id);
        }
        Icon::Gear => {
            let mut pb = PathBuilder::new();
            for i in 0..16 {
                let a = i as f32 * std::f32::consts::TAU / 16.0;
                let r = if i % 2 == 0 { s * 0.38 } else { s * 0.28 };
                let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
                if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
            }
            pb.close();
            fill(pm, &pb.finish(), &p, id);
            fill(pm, &gfx::circle(cx, cy, s * 0.12), &solid(Color::from_rgba8(0, 0, 0, 90)), id);
        }
        Icon::Star => fill(pm, &star_path(cx, cy, s * 0.42), &p, id),
        Icon::Lock => {
            let mut pb = PathBuilder::new();
            arc(&mut pb, cx, cy - s * 0.08, s * 0.18, std::f32::consts::PI, std::f32::consts::TAU);
            stroke(pm, &pb.finish(), &p, w, id);
            fill(pm, &gfx::rrect(cx - s * 0.26, cy - s * 0.08, s * 0.52, s * 0.4, s * 0.08), &p, id);
        }
        Icon::Play => {
            let mut pb = PathBuilder::new();
            pb.move_to(cx - s * 0.18, cy - s * 0.28);
            pb.line_to(cx + s * 0.3, cy);
            pb.line_to(cx - s * 0.18, cy + s * 0.28);
            pb.close();
            fill(pm, &pb.finish(), &p, id);
        }
        Icon::Speaker => {
            let mut pb = PathBuilder::new();
            pb.move_to(cx - s * 0.32, cy - s * 0.1);
            pb.line_to(cx - s * 0.16, cy - s * 0.1);
            pb.line_to(cx + s * 0.04, cy - s * 0.28);
            pb.line_to(cx + s * 0.04, cy + s * 0.28);
            pb.line_to(cx - s * 0.16, cy + s * 0.1);
            pb.line_to(cx - s * 0.32, cy + s * 0.1);
            pb.close();
            fill(pm, &pb.finish(), &p, id);
            for r in [0.18f32, 0.32] {
                let mut pb = PathBuilder::new();
                arc(&mut pb, cx + s * 0.04, cy, s * r, -0.8, 0.8);
                stroke(pm, &pb.finish(), &p, w * 0.8, id);
            }
        }
        Icon::Book => {
            fill(pm, &gfx::rrect(cx - s * 0.34, cy - s * 0.28, s * 0.32, s * 0.56, s * 0.05), &p, id);
            fill(pm, &gfx::rrect(cx + s * 0.02, cy - s * 0.28, s * 0.32, s * 0.56, s * 0.05), &p, id);
        }
        Icon::Close => {
            let mut pb = PathBuilder::new();
            pb.move_to(cx - s * 0.25, cy - s * 0.25);
            pb.line_to(cx + s * 0.25, cy + s * 0.25);
            pb.move_to(cx + s * 0.25, cy - s * 0.25);
            pb.line_to(cx - s * 0.25, cy + s * 0.25);
            stroke(pm, &pb.finish(), &p, w * 1.2, id);
        }
        Icon::Calendar => {
            stroke(pm, &gfx::rrect(cx - s * 0.3, cy - s * 0.24, s * 0.6, s * 0.54, s * 0.08), &p, w * 0.8, id);
            gfx::fill_rect(pm, cx - s * 0.3, cy - s * 0.24, s * 0.6, s * 0.14, &p);
            for (dx, dy) in [(-0.12, 0.06), (0.12, 0.06), (-0.12, 0.18), (0.12, 0.18)] {
                fill(pm, &gfx::circle(cx + s * dx, cy + s * dy, s * 0.04), &p, id);
            }
        }
        Icon::Dice => {
            stroke(pm, &gfx::rrect(cx - s * 0.3, cy - s * 0.3, s * 0.6, s * 0.6, s * 0.12), &p, w * 0.8, id);
            for (dx, dy) in [(-0.14, -0.14), (0.0, 0.0), (0.14, 0.14)] {
                fill(pm, &gfx::circle(cx + s * dx, cy + s * dy, s * 0.05), &p, id);
            }
        }
    }
}
