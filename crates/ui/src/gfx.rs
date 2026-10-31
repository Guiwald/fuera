//! Drawing helpers on top of tiny-skia: rounded polyomino outlines, symbols, gradients.

use fuera_core::Pos;
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, Path, PathBuilder, Pixmap, Point, Rect, Shader, SpreadMode, Stroke, Transform,
};

use crate::theme::Symbol;

/// Outline loops (grid corner points, turning points only) of a set of cells. Each loop is walked with the cells on
/// its right (clockwise on screen, y down); holes come out counter-clockwise.
pub fn outline(cells: &[Pos]) -> Vec<Vec<(i32, i32)>> {
    use std::collections::HashMap;
    let has = |x: i32, y: i32| cells.iter().any(|c| c.x as i32 == x && c.y as i32 == y);
    // Directed edges: from -> to.
    let mut edges: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    let mut count = 0;
    for c in cells {
        let (x, y) = (c.x as i32, c.y as i32);
        if !has(x, y - 1) {
            edges.entry((x, y)).or_default().push((x + 1, y));
            count += 1;
        }
        if !has(x + 1, y) {
            edges.entry((x + 1, y)).or_default().push((x + 1, y + 1));
            count += 1;
        }
        if !has(x, y + 1) {
            edges.entry((x + 1, y + 1)).or_default().push((x, y + 1));
            count += 1;
        }
        if !has(x - 1, y) {
            edges.entry((x, y + 1)).or_default().push((x, y));
            count += 1;
        }
    }
    let mut loops = Vec::new();
    let mut used = 0;
    while used < count {
        // Pick any remaining edge as a start (smallest for determinism).
        let start = *edges.iter().filter(|(_, v)| !v.is_empty()).map(|(k, _)| k).min().unwrap();
        let mut pts = vec![start];
        let mut cur = start;
        let mut prev_dir: Option<(i32, i32)> = None;
        loop {
            let outs = edges.get_mut(&cur).unwrap();
            // At a pinch point prefer the right turn, which keeps diagonal neighbours as separate loops.
            let pick = if outs.len() > 1 {
                let pd = prev_dir.unwrap_or((1, 0));
                let mut best = 0;
                for (i, o) in outs.iter().enumerate() {
                    let d = (o.0 - cur.0, o.1 - cur.1);
                    if pd.0 * d.1 - pd.1 * d.0 > 0 {
                        best = i;
                    }
                }
                best
            } else {
                0
            };
            let next = outs.swap_remove(pick);
            used += 1;
            prev_dir = Some((next.0 - cur.0, next.1 - cur.1));
            cur = next;
            if cur == start {
                break;
            }
            pts.push(cur);
        }
        // Keep only turning points.
        let n = pts.len();
        let mut turns = Vec::new();
        for i in 0..n {
            let a = pts[(i + n - 1) % n];
            let b = pts[i];
            let c = pts[(i + 1) % n];
            let d1 = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
            let d2 = ((c.0 - b.0).signum(), (c.1 - b.1).signum());
            if d1 != d2 {
                turns.push(b);
            }
        }
        loops.push(turns);
    }
    loops
}

const K: f32 = 0.552_284_8;

/// Rounded path of outline loops, in pixels. `inset` moves every edge towards the inside (negative: outside);
/// `r` is the radius of convex corners, `rc` of concave ones.
pub fn rounded(loops: &[Vec<(i32, i32)>], ox: f32, oy: f32, cell: f32, inset: f32, r: f32, rc: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    for lp in loops {
        let n = lp.len();
        if n < 4 {
            continue;
        }
        // Inset corner points and their directions.
        let mut pts = Vec::with_capacity(n);
        for i in 0..n {
            let a = lp[(i + n - 1) % n];
            let b = lp[i];
            let c = lp[(i + 1) % n];
            let din = (((b.0 - a.0).signum()) as f32, ((b.1 - a.1).signum()) as f32);
            let dout = (((c.0 - b.0).signum()) as f32, ((c.1 - b.1).signum()) as f32);
            let rn = |d: (f32, f32)| (-d.1, d.0);
            let (n1, n2) = (rn(din), rn(dout));
            let p = (ox + b.0 as f32 * cell + inset * (n1.0 + n2.0), oy + b.1 as f32 * cell + inset * (n1.1 + n2.1));
            let convex = din.0 * dout.1 - din.1 * dout.0 > 0.0;
            pts.push((p, din, dout, if convex { r } else { rc }));
        }
        // Clamp radii to half of the adjacent edges.
        let len = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() + (a.1 - b.1).abs();
        let mut radii: Vec<f32> = pts.iter().map(|p| p.3).collect();
        for i in 0..n {
            let prev = len(pts[(i + n - 1) % n].0, pts[i].0);
            let next = len(pts[i].0, pts[(i + 1) % n].0);
            radii[i] = radii[i].min(prev / 2.0).min(next / 2.0).max(0.0);
        }
        for i in 0..=n {
            let (p, din, dout, _) = pts[i % n];
            let rr = radii[i % n];
            let s = (p.0 - din.0 * rr, p.1 - din.1 * rr);
            let e = (p.0 + dout.0 * rr, p.1 + dout.1 * rr);
            if i == 0 {
                pb.move_to(e.0, e.1);
                continue;
            }
            pb.line_to(s.0, s.1);
            let c1 = (p.0 - din.0 * rr * (1.0 - K), p.1 - din.1 * rr * (1.0 - K));
            let c2 = (p.0 + dout.0 * rr * (1.0 - K), p.1 + dout.1 * rr * (1.0 - K));
            pb.cubic_to(c1.0, c1.1, c2.0, c2.1, e.0, e.1);
        }
        pb.close();
    }
    pb.finish()
}

pub fn rrect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0).max(0.0);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r * (1.0 - K), y, x + w, y + r * (1.0 - K), x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r * (1.0 - K), x + w - r * (1.0 - K), y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r * (1.0 - K), y + h, x, y + h - r * (1.0 - K), x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r * (1.0 - K), x + r * (1.0 - K), y, x + r, y);
    pb.close();
    pb.finish()
}

pub fn circle(cx: f32, cy: f32, r: f32) -> Option<Path> {
    PathBuilder::from_circle(cx, cy, r.max(0.1))
}

/// A symbol centred on (cx, cy), fitting a circle of radius `r`.
pub fn symbol(s: Symbol, cx: f32, cy: f32, r: f32) -> Option<Path> {
    let mut pb = PathBuilder::new();
    match s {
        Symbol::Circle => return circle(cx, cy, r * 0.82),
        Symbol::Square => return rrect(cx - r * 0.72, cy - r * 0.72, r * 1.44, r * 1.44, r * 0.25),
        Symbol::Diamond => {
            pb.move_to(cx, cy - r);
            pb.line_to(cx + r * 0.8, cy);
            pb.line_to(cx, cy + r);
            pb.line_to(cx - r * 0.8, cy);
        }
        Symbol::Triangle => {
            pb.move_to(cx, cy - r * 0.95);
            pb.line_to(cx + r * 0.95, cy + r * 0.72);
            pb.line_to(cx - r * 0.95, cy + r * 0.72);
        }
        Symbol::Star => {
            for i in 0..10 {
                let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
                let rr = if i % 2 == 0 { r } else { r * 0.45 };
                let (x, y) = (cx + rr * a.cos(), cy + rr * a.sin() + r * 0.08);
                if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
            }
        }
        Symbol::Hexagon => {
            for i in 0..6 {
                let a = i as f32 * std::f32::consts::PI / 3.0;
                let (x, y) = (cx + r * 0.9 * a.cos(), cy + r * 0.9 * a.sin());
                if i == 0 { pb.move_to(x, y) } else { pb.line_to(x, y) }
            }
        }
        Symbol::Plus => {
            let t = r * 0.34;
            pb.move_to(cx - t, cy - r * 0.9);
            pb.line_to(cx + t, cy - r * 0.9);
            pb.line_to(cx + t, cy - t);
            pb.line_to(cx + r * 0.9, cy - t);
            pb.line_to(cx + r * 0.9, cy + t);
            pb.line_to(cx + t, cy + t);
            pb.line_to(cx + t, cy + r * 0.9);
            pb.line_to(cx - t, cy + r * 0.9);
            pb.line_to(cx - t, cy + t);
            pb.line_to(cx - r * 0.9, cy + t);
            pb.line_to(cx - r * 0.9, cy - t);
            pb.line_to(cx - t, cy - t);
        }
        Symbol::Heart => {
            let y0 = cy - r * 0.45;
            pb.move_to(cx, cy + r * 0.9);
            pb.cubic_to(cx - r * 0.2, cy + r * 0.6, cx - r * 1.05, cy + r * 0.15, cx - r * 0.95, y0);
            pb.cubic_to(cx - r * 0.85, cy - r * 1.05, cx - r * 0.1, cy - r * 1.05, cx, y0 + r * 0.05);
            pb.cubic_to(cx + r * 0.1, cy - r * 1.05, cx + r * 0.85, cy - r * 1.05, cx + r * 0.95, y0);
            pb.cubic_to(cx + r * 1.05, cy + r * 0.15, cx + r * 0.2, cy + r * 0.6, cx, cy + r * 0.9);
        }
    }
    pb.close();
    pb.finish()
}

/// Arrow (chevron) pointing in direction (dx, dy).
pub fn chevron(cx: f32, cy: f32, size: f32, dx: f32, dy: f32) -> Option<Path> {
    // Points for an arrow pointing east, then rotated.
    let pts = [(-0.35, -0.55), (0.25, 0.0), (-0.35, 0.55), (-0.05, 0.0)];
    let mut pb = PathBuilder::new();
    for (i, (x, y)) in pts.iter().enumerate() {
        // rotate (x, y) by direction (dx, dy): east (1,0) is identity.
        let rx = x * dx - y * dy;
        let ry = x * dy + y * dx;
        let (px, py) = (cx + rx * size, cy + ry * size);
        if i == 0 { pb.move_to(px, py) } else { pb.line_to(px, py) }
    }
    pb.close();
    pb.finish()
}

pub fn solid(c: Color) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(c);
    p.anti_alias = true;
    p
}

pub fn vgrad(y0: f32, y1: f32, stops: &[(f32, Color)]) -> Paint<'static> {
    let mut p = Paint::default();
    p.anti_alias = true;
    let stops: Vec<GradientStop> = stops.iter().map(|(t, c)| GradientStop::new(*t, *c)).collect();
    if let Some(s) = LinearGradient::new(Point::from_xy(0.0, y0), Point::from_xy(0.0, y1.max(y0 + 0.5)), stops, SpreadMode::Pad, Transform::identity()) {
        p.shader = s;
    } else {
        p.shader = Shader::SolidColor(Color::WHITE);
    }
    p
}

pub fn fill(pm: &mut Pixmap, path: &Option<Path>, paint: &Paint, t: Transform) {
    if let Some(p) = path {
        pm.fill_path(p, paint, FillRule::Winding, t, None);
    }
}

pub fn stroke(pm: &mut Pixmap, path: &Option<Path>, paint: &Paint, width: f32, t: Transform) {
    if let Some(p) = path {
        let s = Stroke { width, line_cap: tiny_skia::LineCap::Round, line_join: tiny_skia::LineJoin::Round, ..Default::default() };
        pm.stroke_path(p, paint, &s, t, None);
    }
}

pub fn fill_rect(pm: &mut Pixmap, x: f32, y: f32, w: f32, h: f32, paint: &Paint) {
    if let Some(r) = Rect::from_xywh(x, y, w.max(0.01), h.max(0.01)) {
        pm.fill_rect(r, paint, Transform::identity(), None);
    }
}

/// Draws a sprite with opacity and scale about its centre.
pub fn blit(dst: &mut Pixmap, src: &Pixmap, x: f32, y: f32, alpha: f32, scale: f32) {
    let mut p = tiny_skia::PixmapPaint { opacity: alpha.clamp(0.0, 1.0), quality: tiny_skia::FilterQuality::Bilinear, ..Default::default() };
    if (scale - 1.0).abs() < 0.001 && x.fract() == 0.0 && y.fract() == 0.0 {
        p.quality = tiny_skia::FilterQuality::Nearest;
        dst.draw_pixmap(x as i32, y as i32, src.as_ref(), &p, Transform::identity(), None);
    } else {
        let (w, h) = (src.width() as f32, src.height() as f32);
        let t = Transform::from_translate(x + w / 2.0, y + h / 2.0).pre_scale(scale, scale).pre_translate(-w / 2.0, -h / 2.0);
        dst.draw_pixmap(0, 0, src.as_ref(), &p, t, None);
    }
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t)
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

/// Spring-like overshoot used for pops.
pub fn ease_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}
