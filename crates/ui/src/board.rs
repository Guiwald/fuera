//! The board on screen: layout, tray and doors, glossy bricks (cached as sprites), slide/exit/bounce animations,
//! particles, hit testing.

use fuera_core::rules::{self, Outcome, Result, State};
use fuera_core::{Cell, Dir, DoorKind, Game, Level, Move, Pos};
use tiny_skia::{Pixmap, Transform};

use crate::gfx::{self, blit, fill, fill_rect, rounded, solid, stroke, vgrad};
use crate::text::{Align, Face, Fonts};
use crate::theme::{self, Symbol, col, col_a, shade};

/// How blocks and doors look: colours and symbols in the classic mode, words and pictures in the language modes.
#[derive(Clone, Debug, Default)]
pub struct Skin {
    pub blocks: Vec<Look>,
    /// Look of the outer layer of each block (if it has one).
    pub shells: Vec<Option<Look>>,
    pub doors: Vec<Look>,
}

#[derive(Clone, Debug)]
pub struct Look {
    pub rgb: (u8, u8, u8),
    pub symbol: Option<Symbol>,
    pub label: Option<String>,
    /// Legend mark (A, B, C…) shown on a door whose text is in the legend.
    pub mark: Option<String>,
    /// Audio-only tile: a speaker instead of the text.
    pub speaker: bool,
}

impl Look {
    pub fn key(k: u16) -> Look {
        Look { rgb: theme::key_rgb(k), symbol: theme::key_symbol(k), label: None, mark: None, speaker: false }
    }

    pub fn text(rgb: (u8, u8, u8), label: &str) -> Look {
        Look { rgb, symbol: None, label: Some(label.to_string()), mark: None, speaker: false }
    }
}

impl Skin {
    pub fn classic(level: &Level) -> Skin {
        Skin {
            blocks: level.blocks.iter().map(|b| Look::key(b.key)).collect(),
            shells: level.blocks.iter().map(|b| b.shell.map(Look::key)).collect(),
            doors: level.doors.iter().map(|d| Look::key(d.key)).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Vis {
    x: f32,
    y: f32,
    alpha: f32,
    scale: f32,
}

#[derive(Clone, Debug)]
struct Tween {
    block: usize,
    /// Keyframes: (time from start, x, y, alpha, scale).
    keys: Vec<(f32, f32, f32, f32, f32)>,
    t: f32,
    /// Sprite switches to "no shell" at this time (peel).
    peel_at: Option<f32>,
    /// Burst of particles at this time and place (cells).
    burst: Option<(f32, f32, f32, (u8, u8, u8))>,
}

#[derive(Clone, Copy, Debug)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    size: f32,
    rgb: (u8, u8, u8),
}

/// Something that happened, for sounds and haptics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fx {
    Slide,
    Bump,
    Exit,
    Peel,
    Bounce,
    Win,
    Tap,
    Nope,
}

pub struct BoardView {
    pub skin: Skin,
    /// Pixel rectangle available.
    area: (f32, f32, f32, f32),
    pub cell: f32,
    pub ox: f32,
    pub oy: f32,
    rim: f32,
    vis: Vec<Vis>,
    /// Shell shown on the sprite (may lag the state during a peel animation).
    shell_vis: Vec<bool>,
    tweens: Vec<Tween>,
    particles: Vec<Particle>,
    sprites: Vec<[Option<Pixmap>; 2]>,
    tray: Option<Pixmap>,
    pub selected: Option<usize>,
    /// Hint: block and direction, with an animation clock.
    pub hint: Option<(Move, f32)>,
    pub fx: Vec<Fx>,
    shake: f32,
    clock: f32,
}

const RIM: f32 = 0.42;

impl BoardView {
    pub fn new(game: &Game, skin: Skin) -> BoardView {
        let n = game.level.blocks.len();
        let mut v = BoardView {
            skin,
            area: (0.0, 0.0, 1.0, 1.0),
            cell: 10.0,
            ox: 0.0,
            oy: 0.0,
            rim: 4.0,
            vis: Vec::new(),
            shell_vis: Vec::new(),
            tweens: Vec::new(),
            particles: Vec::new(),
            sprites: vec![[None, None]; n],
            tray: None,
            selected: None,
            hint: None,
            fx: Vec::new(),
            shake: 0.0,
            clock: 0.0,
        };
        v.sync(game);
        v
    }

    /// Snaps the visuals to the game state (after undo, restart…).
    pub fn sync(&mut self, game: &Game) {
        self.tweens.clear();
        self.vis = game.state.blocks.iter().map(|b| Vis { x: b.pos.x as f32, y: b.pos.y as f32, alpha: if b.alive { 1.0 } else { 0.0 }, scale: 1.0 }).collect();
        self.shell_vis = game.state.blocks.iter().map(|b| b.shell).collect();
        self.hint = None;
    }

    pub fn layout(&mut self, level: &Level, x: f32, y: f32, w: f32, h: f32) {
        let area = (x, y, w, h);
        // Room for door signs above and below (language rounds).
        let signs = if self.skin.doors.iter().any(|d| d.label.is_some() || d.mark.is_some()) { 2.0 * SIGN } else { 0.0 };
        let cw = w / (level.w as f32 + 2.0 * RIM);
        let ch = h / (level.h as f32 + 2.0 * RIM + signs);
        let cell = cw.min(ch).floor().max(8.0);
        if area == self.area && cell == self.cell && self.tray.is_some() {
            return;
        }
        self.area = area;
        self.cell = cell;
        self.rim = (cell * RIM).round();
        self.ox = (x + (w - cell * level.w as f32) / 2.0).round();
        self.oy = (y + (h - cell * level.h as f32) / 2.0).round();
        self.tray = None;
        for s in &mut self.sprites {
            *s = [None, None];
        }
    }

    pub fn animating(&self) -> bool {
        !self.tweens.is_empty() || !self.particles.is_empty() || self.shake > 0.0 || self.hint.is_some()
    }

    pub fn busy(&self) -> bool {
        !self.tweens.is_empty()
    }

    /// Finishes running animations at once.
    pub fn finish(&mut self) {
        let tw = std::mem::take(&mut self.tweens);
        for t in tw {
            if let Some(k) = t.keys.last() {
                self.vis[t.block] = Vis { x: k.1, y: k.2, alpha: k.3, scale: k.4 };
            }
            if t.peel_at.is_some() {
                self.shell_vis[t.block] = false;
            }
        }
    }

    /// Cell under a pixel.
    pub fn cell_at(&self, px: f32, py: f32) -> Option<Pos> {
        let x = ((px - self.ox) / self.cell).floor() as i16;
        let y = ((py - self.oy) / self.cell).floor() as i16;
        Some(Pos::new(x, y))
    }

    pub fn block_at(&self, game: &Game, px: f32, py: f32) -> Option<usize> {
        let p = self.cell_at(px, py)?;
        if !game.level.in_bounds(p) {
            return None;
        }
        let o = game.occupancy()[game.level.idx(p)];
        (o > 0).then(|| o as usize - 1)
    }

    /// Starts the animation of a move that was just played (`before` is the state before it).
    pub fn animate(&mut self, game: &Game, before: &State, mv: Move, out: Outcome) {
        self.finish();
        self.hint = None;
        let b = mv.block as usize;
        let level = &game.level;
        let start = before.blocks[b].pos;
        let (sx, sy) = (start.x as f32, start.y as f32);
        let stop = start.step(mv.dir, out.steps);
        let (ex, ey) = (stop.x as f32, stop.y as f32);
        let slide = 0.06 + 0.045 * out.steps as f32;
        let (dx, dy) = mv.dir.delta();
        let (dx, dy) = (dx as f32, dy as f32);
        let mut tw = Tween { block: b, keys: vec![(0.0, sx, sy, 1.0, 1.0)], t: 0.0, peel_at: None, burst: None };
        match out.result {
            Result::Stop { .. } => {
                tw.keys.push((slide, ex, ey, 1.0, 1.0));
                // Small bump into the obstacle.
                tw.keys.push((slide + 0.05, ex + dx * 0.06, ey + dy * 0.06, 1.0, 1.0));
                tw.keys.push((slide + 0.12, ex, ey, 1.0, 1.0));
                self.fx.push(Fx::Slide);
                self.fx.push(Fx::Bump);
            }
            Result::Exit { door } => {
                let shape = &level.blocks[b].shape;
                let len = match mv.dir.axis() {
                    fuera_core::Axis::H => shape.w,
                    fuera_core::Axis::V => shape.h,
                } as f32;
                let beyond = len + RIM;
                let through = 0.10 + 0.05 * len;
                tw.keys.push((slide, ex, ey, 1.0, 1.0));
                tw.keys.push((slide + through, ex + dx * beyond, ey + dy * beyond, 0.0, 0.85));
                let d = &level.doors[door];
                let (bx, by) = door_center(d, self.rim / self.cell);
                tw.burst = Some((slide, bx, by, self.skin.doors[door].rgb));
                self.fx.push(Fx::Slide);
                self.fx.push(Fx::Exit);
            }
            Result::Peel { door } => {
                tw.keys.push((slide, ex, ey, 1.0, 1.0));
                tw.keys.push((slide + 0.08, ex, ey, 1.0, 1.12));
                tw.keys.push((slide + 0.22, ex, ey, 1.0, 1.0));
                tw.peel_at = Some(slide);
                let d = &level.doors[door];
                let (bx, by) = door_center(d, self.rim / self.cell);
                tw.burst = Some((slide, bx, by, self.skin.doors[door].rgb));
                self.fx.push(Fx::Slide);
                self.fx.push(Fx::Peel);
            }
            Result::Bounce { .. } => {
                tw.keys.push((slide, ex + dx * 0.15, ey + dy * 0.15, 1.0, 1.0));
                tw.keys.push((slide + 0.08 + 0.04 * out.steps as f32, sx, sy, 1.0, 1.0));
                self.fx.push(Fx::Bounce);
                self.shake = 0.25;
            }
        }
        self.tweens.push(tw);
    }

    pub fn update(&mut self, dt: f32) {
        self.clock += dt;
        if let Some((_, t)) = &mut self.hint {
            *t += dt;
        }
        self.shake = (self.shake - dt).max(0.0);
        let mut done = Vec::new();
        let mut bursts = Vec::new();
        for (i, tw) in self.tweens.iter_mut().enumerate() {
            let before = tw.t;
            tw.t += dt;
            if let Some(pa) = tw.peel_at
                && before < pa
                && tw.t >= pa
            {
                self.shell_vis[tw.block] = false;
            }
            if let Some((bt, bx, by, rgb)) = tw.burst
                && before < bt
                && tw.t >= bt
            {
                bursts.push((bx, by, rgb));
            }
            let k = &tw.keys;
            let last = k[k.len() - 1];
            let v = if tw.t >= last.0 {
                done.push(i);
                Vis { x: last.1, y: last.2, alpha: last.3, scale: last.4 }
            } else {
                let j = k.iter().position(|kk| kk.0 > tw.t).unwrap_or(k.len() - 1).max(1);
                let (a, b) = (k[j - 1], k[j]);
                let u = gfx::ease_out((tw.t - a.0) / (b.0 - a.0).max(1e-4));
                Vis { x: gfx::lerp(a.1, b.1, u), y: gfx::lerp(a.2, b.2, u), alpha: gfx::lerp(a.3, b.3, u), scale: gfx::lerp(a.4, b.4, u) }
            };
            self.vis[tw.block] = v;
        }
        for i in done.into_iter().rev() {
            self.tweens.remove(i);
        }
        for (x, y, rgb) in bursts {
            self.burst(x, y, rgb, 26);
        }
        for p in &mut self.particles {
            p.life += dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 9.0 * dt;
            p.vx *= 1.0 - 1.5 * dt;
        }
        self.particles.retain(|p| p.life < p.max);
    }

    /// Particles (in cell units) from a point.
    pub fn burst(&mut self, x: f32, y: f32, rgb: (u8, u8, u8), n: usize) {
        let mut seed = (x * 1000.0 + y * 7919.0 + self.clock * 977.0) as u32 | 1;
        let mut rnd = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed % 10_000) as f32 / 10_000.0
        };
        for _ in 0..n {
            let a = rnd() * std::f32::consts::TAU;
            let s = 2.0 + rnd() * 5.0;
            let light = rnd() * 0.5;
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * s,
                vy: a.sin() * s - 2.5,
                life: 0.0,
                max: 0.45 + rnd() * 0.45,
                size: 0.06 + rnd() * 0.08,
                rgb: shade(rgb, light),
            });
        }
    }

    /// Confetti over the whole board (win).
    pub fn celebrate(&mut self, level: &Level) {
        for i in 0..6 {
            let x = level.w as f32 * (0.15 + 0.14 * i as f32);
            let rgb = theme::KEY_COLORS[i % theme::KEY_COLORS.len()];
            self.burst(x, level.h as f32 * 0.4, rgb, 22);
        }
    }

    fn sprite(&mut self, level: &Level, b: usize, shell: bool) -> &Pixmap {
        let si = shell as usize;
        if self.sprites[b][si].is_none() {
            let look = if shell { self.skin.shells[b].clone().unwrap_or_else(|| self.skin.blocks[b].clone()) } else { self.skin.blocks[b].clone() };
            let inner = if shell { Some(self.skin.blocks[b].clone()) } else { None };
            self.sprites[b][si] = Some(render_brick(&level.blocks[b].shape.cells, self.cell, &look, inner.as_ref()));
        }
        self.sprites[b][si].as_ref().unwrap()
    }

    fn render_tray(&self, level: &Level, w: u32, h: u32) -> Pixmap {
        let mut pm = Pixmap::new(w, h).unwrap();
        let c = self.cell;
        let (ox, oy) = (self.ox, self.oy);
        let mut floor = Vec::new();
        let mut solid_cells = Vec::new();
        for y in 0..level.h {
            for x in 0..level.w {
                let p = Pos::new(x, y);
                match level.cell(p) {
                    Cell::Floor => {
                        floor.push(p);
                        solid_cells.push(p);
                    }
                    Cell::Wall => solid_cells.push(p),
                    Cell::Void => {}
                }
            }
        }
        let outer = gfx::outline(&solid_cells);
        let rim = self.rim;
        // Drop shadow, side, top of the plaster rim.
        let shadow = rounded(&outer, ox, oy + c * 0.16, c, -rim, rim * 1.2, rim * 0.6);
        fill(&mut pm, &shadow, &solid(theme::rgba(0, 0, 0, 70)), Transform::identity());
        let side = rounded(&outer, ox, oy + c * 0.08, c, -rim, rim * 1.2, rim * 0.6);
        fill(&mut pm, &side, &solid(col(theme::PLASTER_SIDE)), Transform::identity());
        let top = rounded(&outer, ox, oy, c, -rim, rim * 1.2, rim * 0.6);
        let y0 = oy - rim;
        let y1 = oy + level.h as f32 * c + rim;
        fill(&mut pm, &top, &vgrad(y0, y1, &[(0.0, col(theme::PLASTER)), (1.0, col(shade(theme::PLASTER, -0.04)))]), Transform::identity());
        // Floor: sunken tiles.
        let fl = gfx::outline(&floor);
        let well = rounded(&fl, ox, oy, c, 0.0, c * 0.18, c * 0.12);
        fill(&mut pm, &well, &solid(col(shade(theme::TILE_A, -0.18))), Transform::identity());
        let inner = rounded(&fl, ox, oy + c * 0.05, c, c * 0.02, c * 0.16, c * 0.1);
        fill(&mut pm, &inner, &solid(col(theme::TILE_A)), Transform::identity());
        for p in &floor {
            let (x, y) = (ox + p.x as f32 * c, oy + p.y as f32 * c + c * 0.05);
            if (p.x + p.y) % 2 == 0 {
                let t = gfx::rrect(x + c * 0.06, y + c * 0.06, c * 0.88, c * 0.88, c * 0.12);
                fill(&mut pm, &t, &solid(col(theme::TILE_B)), Transform::identity());
            }
            // Azulejo motif: a small four-petal mark at tile corners.
            let m = gfx::symbol(Symbol::Diamond, x + c * 0.5, y + c * 0.5, c * 0.07);
            fill(&mut pm, &m, &solid(col_a(theme::AZULEJO, 0.45)), Transform::identity());
        }
        // Inner walls: raised plaster.
        for y in 0..level.h {
            for x in 0..level.w {
                if level.cell(Pos::new(x, y)) == Cell::Wall {
                    let (px, py) = (ox + x as f32 * c, oy + y as f32 * c);
                    let r = gfx::rrect(px + c * 0.04, py + c * 0.04, c * 0.92, c * 0.92, c * 0.18);
                    fill(&mut pm, &r, &solid(col(shade(theme::PLASTER_SIDE, 0.2))), Transform::identity());
                    let r = gfx::rrect(px + c * 0.04, py, c * 0.92, c * 0.84, c * 0.18);
                    fill(&mut pm, &r, &solid(col(theme::PLASTER)), Transform::identity());
                }
            }
        }
        pm
    }

    fn draw_doors(&self, pm: &mut Pixmap, game: &Game, fonts: &mut Fonts, need: Option<u16>) {
        let c = self.cell;
        let rim = self.rim;
        for (i, d) in game.level.doors.iter().enumerate() {
            let look = &self.skin.doors[i];
            let open = d.is_open(game.state.exits);
            let (x, y, w, h) = door_rect(d, self.ox, self.oy, c, rim);
            let glow = need == Some(d.key) && open;
            if glow {
                let pulse = 0.5 + 0.5 * (self.clock * 6.0).sin();
                let g = gfx::rrect(x - c * 0.08, y - c * 0.08, w + c * 0.16, h + c * 0.16, c * 0.2);
                fill(pm, &g, &solid(col_a(theme::WHITE, 0.35 + 0.35 * pulse)), Transform::identity());
            }
            let base = if open { look.rgb } else { shade(look.rgb, -0.45) };
            let body = gfx::rrect(x, y, w, h, c * 0.14);
            fill(pm, &gfx::rrect(x, y + c * 0.05, w, h, c * 0.14), &solid(col(shade(base, -0.35))), Transform::identity());
            fill(pm, &body, &vgrad(y, y + h, &[(0.0, col(shade(base, 0.25))), (1.0, col(base))]), Transform::identity());
            let (dx, dy) = d.side.delta();
            let (cx, cy) = (x + w / 2.0, y + h / 2.0);
            let along = d.len as f32 * c;
            // Chevrons along the door, pointing out.
            let n = d.len.max(1) as i32;
            for k in 0..n {
                let off = (k as f32 + 0.5) * c - along / 2.0;
                let (ax, ay) = if dx != 0 { (cx, cy + off) } else { (cx + off, cy) };
                if let Some(s) = look.symbol
                    && look.label.is_none()
                {
                    let _ = s;
                }
                let ch = gfx::chevron(ax, ay, rim * 0.62, dx as f32, dy as f32);
                fill(pm, &ch, &solid(col_a(theme::WHITE, 0.92)), Transform::identity());
            }
            if !open {
                // Shutter bars.
                for k in 0..(d.len as i32 * 3) {
                    let t = (k as f32 + 0.5) / (d.len as f32 * 3.0);
                    if dx != 0 {
                        fill_rect(pm, x, y + t * h - c * 0.03, w, c * 0.06, &solid(col_a(theme::INK, 0.7)));
                    } else {
                        fill_rect(pm, x + t * w - c * 0.03, y, c * 0.06, h, &solid(col_a(theme::INK, 0.7)));
                    }
                }
            }
            if let DoorKind::Shutter { .. } = d.kind {
                let dot = gfx::circle(x + w - c * 0.08, y + c * 0.08, c * 0.05);
                fill(pm, &dot, &solid(col(theme::WHITE)), Transform::identity());
            }
            let text = look.mark.clone().or_else(|| look.label.clone());
            if let Some(label) = text {
                // A sign outside the wall: above top doors, below bottom ones, beside side doors.
                let sign_h = c * 0.72;
                let max_w = if dx != 0 { c * 2.4 } else { (d.len as f32 * c + c * 0.9).max(c * 2.2) };
                let px = fonts.fit(Face::Bold, &label, c * 0.36, max_w - c * 0.3);
                let tw = fonts.width(Face::Bold, &label, px).max(c * 0.5) + c * 0.36;
                let (sx, sy) = match d.side {
                    Dir::N => (cx - tw / 2.0, y - rim * 0.15 - sign_h - c * 0.06),
                    Dir::S => (cx - tw / 2.0, y + h + rim * 0.15 + c * 0.06),
                    Dir::E => (x + w + c * 0.1, cy - sign_h / 2.0),
                    Dir::W => (x - c * 0.1 - tw, cy - sign_h / 2.0),
                };
                let glow_sign = glow;
                fill(pm, &gfx::rrect(sx, sy + c * 0.05, tw, sign_h, sign_h * 0.3), &solid(col_a((0, 0, 0), 0.25)), Transform::identity());
                let bg = if look.mark.is_some() { theme::GOLD } else { theme::PLASTER };
                fill(pm, &gfx::rrect(sx, sy, tw, sign_h, sign_h * 0.3), &solid(col(if glow_sign { shade(bg, 0.3) } else { bg })), Transform::identity());
                fonts.draw(pm, Face::Bold, &label, sx + tw / 2.0, sy + sign_h / 2.0 + px * 0.36, px, col(theme::INK), Align::Center);
            }
        }
    }

    pub fn draw(&mut self, pm: &mut Pixmap, game: &Game, fonts: &mut Fonts) {
        let level = &game.level;
        if self.tray.as_ref().is_none_or(|t| t.width() != pm.width() || t.height() != pm.height()) {
            self.tray = Some(self.render_tray(level, pm.width(), pm.height()));
        }
        let (shx, shy) = if self.shake > 0.0 {
            let s = (self.shake * 60.0).sin() * self.shake * self.cell * 0.12;
            (s, 0.0)
        } else {
            (0.0, 0.0)
        };
        blit(pm, self.tray.as_ref().unwrap(), shx.round(), shy, 1.0, 1.0);
        // Light up the doors that take the held block, in the classic mode only (in language rounds it would give
        // the answer away).
        let need = self.selected.filter(|&b| game.state.blocks[b].alive && level.meta.wrong_door == fuera_core::WrongDoor::Rest).map(|b| rules::needed_key(level, &game.state, b));
        self.draw_doors(pm, game, fonts, need);
        // Draw order: lower rows first so shadows fall naturally; exiting blocks too.
        let mut order: Vec<usize> = (0..self.vis.len()).filter(|&i| self.vis[i].alpha > 0.0).collect();
        order.sort_by(|&a, &b| self.vis[a].y.partial_cmp(&self.vis[b].y).unwrap_or(std::cmp::Ordering::Equal));
        let c = self.cell;
        let pad = (c * PAD).round();
        for b in order {
            let v = self.vis[b];
            let shell = self.shell_vis[b];
            let lifted = self.selected == Some(b);
            let scale = v.scale * if lifted { 1.05 } else { 1.0 };
            let x = (self.ox + v.x * c - pad + shx).round();
            let y = (self.oy + v.y * c - pad - if lifted { c * 0.06 } else { 0.0 }).round();
            let spr = self.sprite(level, b, shell).clone();
            blit(pm, &spr, x, y, v.alpha, scale);
            self.draw_overlays(pm, game, b, x + pad, y + pad, fonts, v.alpha);
        }
        // Particles.
        for p in &self.particles {
            let a = 1.0 - p.life / p.max;
            let r = gfx::rrect(self.ox + p.x * c - p.size * c / 2.0, self.oy + p.y * c - p.size * c / 2.0, p.size * c, p.size * c, p.size * c * 0.3);
            fill(pm, &r, &solid(col_a(p.rgb, a)), Transform::identity());
        }
        // Hint: pulsing ring and an arrow.
        if let Some((mv, t)) = self.hint {
            let b = mv.block as usize;
            if b < self.vis.len() && game.state.blocks[b].alive {
                let shape = &level.blocks[b].shape;
                let v = self.vis[b];
                let loops = gfx::outline(&shape.cells);
                let pulse = 0.5 + 0.5 * (t * 5.0).sin();
                let ring = rounded(&loops, self.ox + v.x * c, self.oy + v.y * c, c, -c * 0.06, c * 0.3, c * 0.1);
                stroke(pm, &ring, &solid(col_a(theme::GOLD, 0.6 + 0.4 * pulse)), c * 0.08, Transform::identity());
                let (dx, dy) = mv.dir.delta();
                let cx = self.ox + (v.x + shape.w as f32 / 2.0) * c + dx as f32 * (shape.w as f32 / 2.0 + 0.2 + 0.25 * pulse) * c;
                let cy = self.oy + (v.y + shape.h as f32 / 2.0) * c + dy as f32 * (shape.h as f32 / 2.0 + 0.2 + 0.25 * pulse) * c;
                let ch = gfx::chevron(cx, cy, c * 0.55, dx as f32, dy as f32);
                fill(pm, &ch, &solid(col(theme::GOLD)), Transform::identity());
                stroke(pm, &ch, &solid(col(shade(theme::GOLD, -0.4))), c * 0.03, Transform::identity());
            }
        }
    }

    fn draw_overlays(&self, pm: &mut Pixmap, game: &Game, b: usize, x: f32, y: f32, fonts: &mut Fonts, alpha: f32) {
        let level = &game.level;
        let def = &level.blocks[b];
        let c = self.cell;
        let st = &game.state;
        if !st.blocks[b].alive {
            return;
        }
        let ice = rules::ice_left(level, st, b);
        if ice > 0 {
            let loops = gfx::outline(&def.shape.cells);
            let p = rounded(&loops, x, y, c, c * 0.04, c * 0.24, c * 0.1);
            fill(pm, &p, &vgrad(y, y + def.shape.h as f32 * c, &[(0.0, col_a(theme::ICE, 0.85 * alpha)), (1.0, col_a(shade(theme::ICE, -0.1), 0.7 * alpha))]), Transform::identity());
            stroke(pm, &p, &solid(col_a(theme::WHITE, 0.9 * alpha)), c * 0.04, Transform::identity());
            // Frost streaks.
            for k in 0..def.shape.cells.len() {
                let cc = def.shape.cells[k];
                let (sx, sy) = (x + cc.x as f32 * c, y + cc.y as f32 * c);
                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(sx + c * 0.2, sy + c * 0.35);
                pb.line_to(sx + c * 0.38, sy + c * 0.2);
                pb.move_to(sx + c * 0.62, sy + c * 0.8);
                pb.line_to(sx + c * 0.8, sy + c * 0.62);
                stroke(pm, &pb.finish(), &solid(col_a(theme::WHITE, 0.8 * alpha)), c * 0.05, Transform::identity());
            }
            let (cx, cy) = anchor_center(&def.shape.cells, x, y, c);
            let disc = gfx::circle(cx, cy, c * 0.24);
            fill(pm, &disc, &solid(col_a((0x3A, 0x8D, 0xC4), alpha)), Transform::identity());
            fonts.draw(pm, Face::Title, &ice.to_string(), cx, cy + c * 0.12, c * 0.34, col_a(theme::WHITE, alpha), Align::Center);
        }
        if rules::is_locked(level, st, b) {
            let (cx, cy) = anchor_center(&def.shape.cells, x, y, c);
            // Chains across the block, then a padlock.
            for cc in &def.shape.cells {
                let (sx, sy) = (x + cc.x as f32 * c, y + cc.y as f32 * c);
                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(sx + c * 0.1, sy + c * 0.1);
                pb.line_to(sx + c * 0.9, sy + c * 0.9);
                pb.move_to(sx + c * 0.9, sy + c * 0.1);
                pb.line_to(sx + c * 0.1, sy + c * 0.9);
                stroke(pm, &pb.finish(), &solid(col_a((0x5B, 0x63, 0x70), 0.9 * alpha)), c * 0.09, Transform::identity());
            }
            draw_padlock(pm, cx, cy, c * 0.5, alpha, def.lock.unwrap_or(0), fonts);
        }
        if def.carries.is_some() {
            let (cx, cy) = anchor_center(&def.shape.cells, x, y, c);
            draw_key_icon(pm, cx + c * 0.22, cy - c * 0.22, c * 0.34, alpha);
        }
        if let Some(axis) = def.axis {
            let (cx, cy) = anchor_center(&def.shape.cells, x, y, c);
            let (dx, dy) = if axis == fuera_core::Axis::H { (1.0, 0.0) } else { (0.0, 1.0) };
            for s in [-1.0f32, 1.0] {
                let ch = gfx::chevron(cx + s * dx * c * 0.3, cy + s * dy * c * 0.3, c * 0.22, s * dx, s * dy);
                fill(pm, &ch, &solid(col_a(theme::WHITE, 0.9 * alpha)), Transform::identity());
            }
        }
        if self.skin.blocks[b].speaker {
            let w = def.shape.w as f32 * c;
            let (cx, cy) = (x + w / 2.0, y + def.shape.h as f32 * c / 2.0);
            crate::icons::draw(pm, crate::icons::Icon::Speaker, cx, cy, c * 0.7, col_a(theme::INK, alpha));
        } else if let Some(label) = &self.skin.blocks[b].label {
            let w = def.shape.w as f32 * c;
            let (cx, cy) = (x + w / 2.0, y + def.shape.h as f32 * c / 2.0);
            let px = fonts.fit(Face::Bold, label, c * 0.36, w - c * 0.24);
            let ink = if st.blocks[b].shell && self.skin.shells[b].is_some() { theme::WHITE } else { theme::INK };
            fonts.draw(pm, Face::Bold, label, cx, cy + px * 0.35, px, col_a(ink, alpha), Align::Center);
        }
    }

    /// Lifts or drops the highlighted block.
    pub fn select(&mut self, b: Option<usize>) {
        self.selected = b;
    }

    pub fn show_hint(&mut self, mv: Option<Move>) {
        self.hint = mv.map(|m| (m, 0.0));
    }

    pub fn door_center_px(&self, level: &Level, door: usize) -> (f32, f32) {
        let (x, y) = door_center(&level.doors[door], self.rim / self.cell);
        (self.ox + x * self.cell, self.oy + y * self.cell)
    }
}

/// Space around a sprite for its shadow.
const PAD: f32 = 0.18;
/// Height of the door signs, in cells.
const SIGN: f32 = 0.95;

fn anchor_center(cells: &[Pos], x: f32, y: f32, c: f32) -> (f32, f32) {
    // Centre of the first cell of the top row: always on the block.
    let p = cells[0];
    (x + (p.x as f32 + 0.5) * c, y + (p.y as f32 + 0.5) * c)
}

/// Door rectangle in pixels (on the rim, outside the floor).
fn door_rect(d: &fuera_core::Door, ox: f32, oy: f32, c: f32, rim: f32) -> (f32, f32, f32, f32) {
    let inset = c * 0.08;
    let len = d.len as f32 * c - 2.0 * inset;
    let thick = rim * 0.86;
    let (x0, y0) = (ox + d.at.x as f32 * c, oy + d.at.y as f32 * c);
    match d.side {
        Dir::E => (x0 + c + rim * 0.07, y0 + inset, thick, len),
        Dir::W => (x0 - rim * 0.93, y0 + inset, thick, len),
        Dir::N => (x0 + inset, y0 - rim * 0.93, len, thick),
        Dir::S => (x0 + inset, y0 + c + rim * 0.07, len, thick),
    }
}

/// Door centre in cell units.
fn door_center(d: &fuera_core::Door, rim_cells: f32) -> (f32, f32) {
    let (x0, y0) = (d.at.x as f32, d.at.y as f32);
    let half = d.len as f32 / 2.0;
    match d.side {
        Dir::E => (x0 + 1.0 + rim_cells / 2.0, y0 + half),
        Dir::W => (x0 - rim_cells / 2.0, y0 + half),
        Dir::N => (x0 + half, y0 - rim_cells / 2.0),
        Dir::S => (x0 + half, y0 + 1.0 + rim_cells / 2.0),
    }
}

/// A glossy brick sprite (with room for its shadow). `inner`: the colour under an outer layer.
pub fn render_brick(cells: &[Pos], c: f32, look: &Look, inner: Option<&Look>) -> Pixmap {
    let w = cells.iter().map(|p| p.x).max().unwrap_or(0) as f32 + 1.0;
    let h = cells.iter().map(|p| p.y).max().unwrap_or(0) as f32 + 1.0;
    let pad = (c * PAD).round();
    let mut pm = Pixmap::new((w * c + 2.0 * pad).ceil() as u32, (h * c + 2.0 * pad).ceil() as u32).unwrap();
    let loops = gfx::outline(cells);
    let (ox, oy) = (pad, pad);
    let m = c * 0.05;
    let r = c * 0.22;
    let rc = c * 0.1;
    let id = Transform::identity();
    let base = look.rgb;
    let shadow = rounded(&loops, ox + c * 0.02, oy + c * 0.11, c, m, r, rc);
    fill(&mut pm, &shadow, &solid(theme::rgba(0, 20, 40, 80)), id);
    let side = rounded(&loops, ox, oy + c * 0.065, c, m, r, rc);
    fill(&mut pm, &side, &solid(col(shade(base, -0.38))), id);
    let top = rounded(&loops, ox, oy, c, m, r, rc);
    let (y0, y1) = (oy, oy + h * c);
    fill(&mut pm, &top, &vgrad(y0, y1, &[(0.0, col(shade(base, 0.22))), (0.55, col(base)), (1.0, col(shade(base, -0.08)))]), id);
    // Inner layer showing through (outer layer = frame).
    if let Some(inn) = inner {
        let core = rounded(&loops, ox, oy, c, m + c * 0.17, r * 0.6, rc);
        fill(&mut pm, &core, &solid(col(shade(inn.rgb, -0.25))), id);
        let core2 = rounded(&loops, ox, oy + c * 0.02, c, m + c * 0.19, r * 0.55, rc);
        fill(&mut pm, &core2, &vgrad(y0, y1, &[(0.0, col(shade(inn.rgb, 0.15))), (1.0, col(inn.rgb))]), id);
    }
    // Bevel highlight.
    let bevel = rounded(&loops, ox, oy, c, m + c * 0.045, r * 0.8, rc);
    stroke(&mut pm, &bevel, &solid(col_a(theme::WHITE, 0.28)), c * 0.035, id);
    // Gloss on each cell and an embossed symbol.
    let shown = inner.unwrap_or(look);
    for p in cells {
        let (cx, cy) = (ox + (p.x as f32 + 0.5) * c, oy + (p.y as f32 + 0.5) * c);
        if inner.is_none() {
            let g = gfx::rrect(cx - c * 0.28, cy - c * 0.36, c * 0.32, c * 0.1, c * 0.05);
            fill(&mut pm, &g, &solid(col_a(theme::WHITE, 0.35)), id);
        }
        if shown.label.is_none()
            && let Some(s) = shown.symbol
        {
            let rs = c * 0.19;
            let lo = gfx::symbol(s, cx, cy + c * 0.025, rs);
            fill(&mut pm, &lo, &solid(col_a(theme::WHITE, 0.45)), id);
            let hi = gfx::symbol(s, cx, cy, rs);
            fill(&mut pm, &hi, &solid(col(shade(shown.rgb, -0.28))), id);
        }
    }
    pm
}

fn draw_padlock(pm: &mut Pixmap, cx: f32, cy: f32, s: f32, alpha: f32, n: u8, fonts: &mut Fonts) {
    let id = Transform::identity();
    let shackle = {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(cx - s * 0.28, cy);
        pb.line_to(cx - s * 0.28, cy - s * 0.3);
        pb.cubic_to(cx - s * 0.28, cy - s * 0.62, cx + s * 0.28, cy - s * 0.62, cx + s * 0.28, cy - s * 0.3);
        pb.line_to(cx + s * 0.28, cy);
        pb.finish()
    };
    stroke(pm, &shackle, &solid(col_a((0x6B, 0x72, 0x80), alpha)), s * 0.14, id);
    let body = gfx::rrect(cx - s * 0.45, cy - s * 0.08, s * 0.9, s * 0.65, s * 0.14);
    fill(pm, &body, &solid(col_a(theme::GOLD, alpha)), id);
    stroke(pm, &body, &solid(col_a(shade(theme::GOLD, -0.45), alpha)), s * 0.05, id);
    if n > 0 {
        fonts.draw(pm, Face::Title, &n.to_string(), cx, cy + s * 0.42, s * 0.48, col_a(theme::INK, alpha), Align::Center);
    }
}

fn draw_key_icon(pm: &mut Pixmap, cx: f32, cy: f32, s: f32, alpha: f32) {
    let id = Transform::identity();
    let ring = gfx::circle(cx - s * 0.25, cy, s * 0.24);
    stroke(pm, &ring, &solid(col_a(shade(theme::GOLD, -0.45), alpha)), s * 0.16, id);
    stroke(pm, &ring, &solid(col_a(theme::GOLD, alpha)), s * 0.09, id);
    let mut pb = tiny_skia::PathBuilder::new();
    pb.move_to(cx, cy);
    pb.line_to(cx + s * 0.5, cy);
    pb.move_to(cx + s * 0.38, cy);
    pb.line_to(cx + s * 0.38, cy + s * 0.18);
    let p = pb.finish();
    stroke(pm, &p, &solid(col_a(shade(theme::GOLD, -0.45), alpha)), s * 0.16, id);
    stroke(pm, &p, &solid(col_a(theme::GOLD, alpha)), s * 0.09, id);
}
