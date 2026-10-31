//! Level generator: random boards checked by the solver, graded, and kept when they hit a target difficulty.
//!
//! Generate-and-test: carve a board shape, place doors on its walls, pack blocks, add special elements, then solve.
//! Every level that comes out is solvable by construction of the check (BFS), with its optimal move count.

use crate::geom::{Axis, Dir, Pos};
use crate::level::{BlockDef, Cell, Door, DoorKind, Key, Level, Meta, Shape};
use crate::rules::{self, State};
use crate::solver::{self, Analysis, Solve};

/// Small deterministic RNG (SplitMix64), so a seed always gives the same level on every platform.
#[derive(Clone, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + self.below((hi - lo + 1).max(1) as usize) as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        (self.next_u64() % 10_000) as f32 / 10_000.0 < p
    }

    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len())]
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outline {
    Rect,
    /// Notch cut from the top middle.
    U,
    /// Two corners cut.
    L,
    /// Four corners cut.
    Plus,
    /// Hole in the middle.
    Ring,
    /// Notches top and bottom middle.
    H,
    Random,
}

#[derive(Clone, Debug)]
pub struct Params {
    pub w: i16,
    pub h: i16,
    pub outline: Outline,
    pub colors: usize,
    /// Fraction of floor cells covered by blocks.
    pub fill: f32,
    /// Allowed shapes (as lists of cells).
    pub shapes: Vec<Shape>,
    pub walls: usize,
    pub ice: usize,
    pub lanes: usize,
    pub shells: usize,
    pub locks: usize,
    pub shutters: usize,
    /// Probability that a door is longer than 1 when it could be.
    pub wide_doors: f32,
    /// Second door for some colours.
    pub extra_doors: usize,
    /// Stop adding blocks at this count.
    pub max_blocks: usize,
}

impl Default for Params {
    fn default() -> Params {
        Params {
            w: 6,
            h: 7,
            outline: Outline::Rect,
            colors: 3,
            fill: 0.5,
            shapes: vec![Shape::rect(1, 1)],
            walls: 0,
            ice: 0,
            lanes: 0,
            shells: 0,
            locks: 0,
            shutters: 0,
            wide_doors: 0.3,
            extra_doors: 0,
            max_blocks: 62,
        }
    }
}

pub fn shapes_by_name(names: &str) -> Vec<Shape> {
    let mut v = Vec::new();
    for n in names.split(',') {
        let cells: Vec<(i16, i16)> = match n.trim() {
            "1" => vec![(0, 0)],
            "2h" => vec![(0, 0), (1, 0)],
            "2v" => vec![(0, 0), (0, 1)],
            "3h" => vec![(0, 0), (1, 0), (2, 0)],
            "3v" => vec![(0, 0), (0, 1), (0, 2)],
            "sq" => vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            "l1" => vec![(0, 0), (0, 1), (1, 1)],
            "l2" => vec![(1, 0), (0, 1), (1, 1)],
            "l3" => vec![(0, 0), (1, 0), (0, 1)],
            "l4" => vec![(0, 0), (1, 0), (1, 1)],
            "t" => vec![(0, 0), (1, 0), (2, 0), (1, 1)],
            _ => continue,
        };
        v.push(Shape::new(cells.into_iter().map(|(x, y)| Pos::new(x, y)).collect()));
    }
    v
}

fn carve(p: &Params, rng: &mut Rng) -> Vec<Cell> {
    let (w, h) = (p.w, p.h);
    let mut cells = vec![Cell::Floor; (w * h) as usize];
    let mut cut = |x0: i16, y0: i16, x1: i16, y1: i16| {
        for y in y0.max(0)..y1.min(h) {
            for x in x0.max(0)..x1.min(w) {
                cells[(y * w + x) as usize] = Cell::Void;
            }
        }
    };
    match p.outline {
        Outline::Rect => {}
        Outline::U => {
            let a = rng.range(1, ((w - 1) / 2).max(1) as i32) as i16;
            let depth = rng.range(1, (h - 2).max(1) as i32) as i16;
            cut(a, 0, w - a, depth);
        }
        Outline::L => {
            let cw = rng.range(1, (w / 2) as i32) as i16;
            let ch = rng.range(1, (h / 2) as i32) as i16;
            if rng.chance(0.5) {
                cut(w - cw, 0, w, ch);
                cut(0, h - ch, cw, h);
            } else {
                cut(0, 0, cw, ch);
                cut(w - cw, h - ch, w, h);
            }
        }
        Outline::Plus => {
            let c = rng.range(1, ((w.min(h) - 1) / 3).max(1) as i32) as i16;
            cut(0, 0, c, c);
            cut(w - c, 0, w, c);
            cut(0, h - c, c, h);
            cut(w - c, h - c, w, h);
        }
        Outline::Ring => {
            let m = rng.range(1, 2) as i16;
            cut(m + 1, m + 1, w - m - 1, h - m - 1);
        }
        Outline::H => {
            let a = (w / 3).max(1);
            let d = rng.range(1, (h / 3).max(1) as i32) as i16;
            cut(a, 0, w - a, d);
            cut(a, h - d, w - a, h);
        }
        Outline::Random => {
            // Bite random rectangles off the edges.
            for _ in 0..rng.range(1, 3) {
                let bw = rng.range(1, (w / 2) as i32) as i16;
                let bh = rng.range(1, (h / 3).max(1) as i32) as i16;
                match rng.below(4) {
                    0 => {
                        let x = rng.range(0, (w - bw) as i32) as i16;
                        cut(x, 0, x + bw, bh)
                    }
                    1 => {
                        let x = rng.range(0, (w - bw) as i32) as i16;
                        cut(x, h - bh, x + bw, h)
                    }
                    2 => {
                        let y = rng.range(0, (h - bh) as i32) as i16;
                        cut(0, y, bw.min(w / 3).max(1), y + bh)
                    }
                    _ => {
                        let y = rng.range(0, (h - bh) as i32) as i16;
                        cut(w - bw.min(w / 3).max(1), y, w, y + bh)
                    }
                }
            }
        }
    }
    cells
}

fn connected(w: i16, h: i16, cells: &[Cell]) -> bool {
    let floor: Vec<usize> = (0..cells.len()).filter(|&i| cells[i] == Cell::Floor).collect();
    let Some(&start) = floor.first() else { return false };
    let mut seen = vec![false; cells.len()];
    let mut stack = vec![start];
    seen[start] = true;
    let mut n = 0;
    while let Some(i) = stack.pop() {
        n += 1;
        let (x, y) = ((i as i16) % w, (i as i16) / w);
        for d in Dir::ALL {
            let q = Pos::new(x, y).step(d, 1);
            if q.x >= 0 && q.y >= 0 && q.x < w && q.y < h {
                let j = (q.y * w + q.x) as usize;
                if !seen[j] && cells[j] == Cell::Floor {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
    }
    n == floor.len()
}

/// One random candidate (not checked for solvability).
pub fn candidate(p: &Params, rng: &mut Rng) -> Option<Level> {
    let (w, h) = (p.w, p.h);
    let mut cells = carve(p, rng);
    if !connected(w, h, &cells) {
        return None;
    }
    // Inner walls, keeping the floor connected.
    let mut placed = 0;
    for _ in 0..p.walls * 10 {
        if placed >= p.walls {
            break;
        }
        let i = rng.below(cells.len());
        if cells[i] != Cell::Floor {
            continue;
        }
        let (x, y) = ((i as i16) % w, (i as i16) / w);
        if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
            continue;
        }
        cells[i] = Cell::Wall;
        if connected(w, h, &cells) {
            placed += 1;
        } else {
            cells[i] = Cell::Floor;
        }
    }
    let mut level = Level { w, h, cells, doors: Vec::new(), blocks: Vec::new(), start: Vec::new(), meta: Meta::default() };
    // Doors: one per colour (plus extras), on free stretches of wall.
    let mut edges: Vec<(Pos, Dir)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let p0 = Pos::new(x, y);
            if !level.is_floor(p0) {
                continue;
            }
            for d in Dir::ALL {
                // Doors only on the outer rim (not on inner walls).
                if level.cell(p0.step(d, 1)) == Cell::Void {
                    edges.push((p0, d));
                }
            }
        }
    }
    rng.shuffle(&mut edges);
    let n_doors = p.colors + p.extra_doors.min(p.colors);
    let mut taken: Vec<(Pos, Dir)> = Vec::new();
    for k in 0..n_doors {
        let key = (k % p.colors) as Key;
        for &(p0, d) in &edges {
            if taken.contains(&(p0, d)) {
                continue;
            }
            // Avoid a second door right next to an existing one of another colour on the same wall.
            let along = if d.axis() == Axis::H { Dir::S } else { Dir::E };
            let mut len = 1;
            let maxlen = if rng.chance(p.wide_doors) { rng.range(2, 3) } else { 1 };
            while len < maxlen {
                let q = p0.step(along, len);
                if level.is_floor(q) && level.cell(q.step(d, 1)) == Cell::Void && !taken.contains(&(q, d)) {
                    len += 1;
                } else {
                    break;
                }
            }
            let kind = if level.doors.iter().filter(|dd| matches!(dd.kind, DoorKind::Shutter { .. })).count() < p.shutters {
                DoorKind::Shutter { open: rng.chance(0.5) }
            } else {
                DoorKind::Plain
            };
            let door = Door { side: d, at: p0, len: len as u8, key, kind };
            for c in door.cells() {
                taken.push((c, d));
            }
            level.doors.push(door);
            break;
        }
    }
    if level.doors.len() < p.colors {
        return None;
    }
    // Blocks.
    let floor_n = level.cells.iter().filter(|c| **c == Cell::Floor).count();
    let target = ((floor_n as f32) * p.fill).round() as usize;
    let mut occ = vec![false; (w * h) as usize];
    let mut covered = 0;
    let names: Vec<char> = Level::NAMES.chars().collect();
    let mut tries = 0;
    while covered < target && tries < 400 && level.blocks.len() < names.len().min(p.max_blocks) {
        tries += 1;
        let shape = rng.pick(&p.shapes).clone();
        let at = Pos::new(rng.range(0, (w - shape.w) as i32) as i16, rng.range(0, (h - shape.h) as i32) as i16);
        let fits = shape.cells.iter().all(|c| {
            let q = at.add(*c);
            level.is_floor(q) && !occ[level.idx(q)]
        });
        if !fits {
            continue;
        }
        // Colours with a door wide enough for this shape.
        let ok: Vec<Key> = (0..p.colors as Key).filter(|&k| level.doors.iter().any(|d| d.key == k && d.len as i16 >= shape.width_across(d.side))).collect();
        if ok.is_empty() {
            continue;
        }
        let key = *rng.pick(&ok);
        for c in &shape.cells {
            let q = at.add(*c);
            let i = level.idx(q);
            occ[i] = true;
        }
        covered += shape.cells.len();
        let name = names[level.blocks.len()];
        level.blocks.push(BlockDef::plain(name, shape, key));
        level.start.push(at);
    }
    if level.blocks.len() < 2 {
        return None;
    }
    // Drop doors whose colour has no block.
    level.doors.retain(|d| level.blocks.iter().any(|b| b.key == d.key));
    // Specials.
    let n = level.blocks.len();
    let mut idx: Vec<usize> = (0..n).collect();
    rng.shuffle(&mut idx);
    let mut it = idx.into_iter();
    for _ in 0..p.ice {
        if let Some(i) = it.next() {
            level.blocks[i].ice = rng.range(1, 3.min(n as i32 - 1).max(1)) as u8;
        }
    }
    for _ in 0..p.lanes {
        if let Some(i) = it.next() {
            level.blocks[i].axis = Some(if rng.chance(0.5) { Axis::H } else { Axis::V });
        }
    }
    for _ in 0..p.shells {
        if let Some(i) = it.next() {
            let shape = level.blocks[i].shape.clone();
            let own = level.blocks[i].key;
            let ok: Vec<Key> = level.doors.iter().filter(|d| d.key != own && d.len as i16 >= shape.width_across(d.side)).map(|d| d.key).collect();
            if !ok.is_empty() {
                level.blocks[i].shell = Some(*rng.pick(&ok));
            }
        }
    }
    for k in 0..p.locks {
        if let (Some(a), Some(b)) = (it.next(), it.next()) {
            level.blocks[a].carries = Some(k as u8 + 1);
            level.blocks[b].lock = Some(k as u8 + 1);
        }
    }
    if !level.validate().is_empty() {
        return None;
    }
    Some(level)
}

/// Grade of a solved level.
#[derive(Clone, Debug)]
pub struct Grade {
    pub optimal: u16,
    pub blocks: usize,
    /// Lower bound of the moves (each block alone on the board).
    pub bound: u16,
    /// Moves above that bound: how much the blocks get in each other's way.
    pub slack: u16,
    pub explored: usize,
    pub analysis: Option<Analysis>,
    /// Blocks that can leave on the very first move.
    pub free_at_start: usize,
    pub score: f32,
}

pub fn grade(level: &Level, limit: usize) -> Option<Grade> {
    let st = State::initial(level);
    let rep = solver::solve(level, &st, limit);
    let Solve::Solved(moves) = rep.result else { return None };
    let optimal = moves.len() as u16;
    let blocks = level.blocks.len();
    let bound = solver::Heuristic::new(level).eval(&st).unwrap_or(0) as u16;
    let slack = optimal.saturating_sub(bound);
    let occ = rules::occupancy(level, &st);
    let free_at_start = rules::legal_moves(level, &st, &occ).iter().filter(|(_, _, o)| matches!(o.result, rules::Result::Exit { .. })).count();
    let analysis = (rep.explored < 4_000).then(|| solver::analyze(level, 40_000)).filter(|a| a.complete);
    let mut score = slack as f32 * 1.0 + optimal as f32 * 0.12;
    if let Some(a) = &analysis {
        let dead = a.dead_states as f32 / a.states.max(1) as f32;
        score += dead * 2.0;
        if a.optimal_first_moves <= 1 && a.first_moves > 2 {
            score += 0.8;
        }
    }
    let specials = level.blocks.iter().filter(|b| b.ice > 0 || b.axis.is_some() || b.shell.is_some() || b.lock.is_some()).count()
        + level.doors.iter().filter(|d| matches!(d.kind, DoorKind::Shutter { .. })).count();
    score += specials as f32 * 0.3;
    score -= free_at_start as f32 * 0.2;
    Some(Grade { optimal, blocks, bound, slack, explored: rep.explored, analysis, free_at_start, score })
}

/// Tries `attempts` candidates and returns the one whose score is closest to `target` (within the constraints).
pub fn generate(p: &Params, seed: u64, attempts: usize, target: f32, min_slack: u16, limit: usize) -> Option<(Level, Grade)> {
    let mut rng = Rng(seed);
    let mut best: Option<(Level, Grade, f32)> = None;
    for _ in 0..attempts {
        let Some(mut level) = candidate(p, &mut rng) else { continue };
        let Some(g) = grade(&level, limit) else { continue };
        if g.slack < min_slack {
            continue;
        }
        let dist = (g.score - target).abs();
        if best.as_ref().is_none_or(|b| dist < b.2) {
            level.meta.optimal = Some(g.optimal);
            best = Some((level, g, dist));
            if dist < 0.3 {
                break;
            }
        }
    }
    best.map(|(l, g, _)| (l, g))
}

/// A board for a language round: horizontal word tiles (`(width, key)`), doors on the top and bottom walls
/// (`keys`, all of the same length so a door's size never gives the answer away), wrong doors bounce.
/// Solvable by construction (A*); `slack` is the wanted number of extra (puzzle) moves.
pub fn word_board(tiles: &[(i16, Key)], doors: &[Key], seed: u64, attempts: usize, slack: (u16, u16)) -> Option<Level> {
    let mut rng = Rng(seed);
    let len = tiles.iter().map(|t| t.0).max().unwrap_or(2).max(2);
    let top = doors.len().div_ceil(2);
    let bottom = doors.len() - top;
    let per = top.max(bottom) as i16;
    let w = (per * len + (per - 1).max(0) + 2).clamp(6, 9);
    let h: i16 = if tiles.len() > 6 { 8 } else { 7 };
    let names: Vec<char> = Level::NAMES.chars().collect();
    let mut best: Option<(Level, u16)> = None;
    for _ in 0..attempts {
        let mut level = Level { w, h, cells: vec![Cell::Floor; (w * h) as usize], doors: Vec::new(), blocks: Vec::new(), start: Vec::new(), meta: Meta::default() };
        level.meta.wrong_door = crate::level::WrongDoor::Bounce;
        // Doors, spread along each wall.
        let mut order: Vec<Key> = doors.to_vec();
        rng.shuffle(&mut order);
        for (side, keys, y) in [(Dir::N, &order[..top], 0), (Dir::S, &order[top..], h - 1)] {
            let n = keys.len() as i16;
            if n == 0 {
                continue;
            }
            let free = w - n * len;
            let gap = free / (n + 1).max(1);
            let mut x = gap + (free - gap * (n + 1)) / 2 + (rng.below(2) as i16 * (free > (n + 1) * gap) as i16);
            for &k in keys {
                level.doors.push(Door { side, at: Pos::new(x.min(w - len), y), len: len as u8, key: k, kind: DoorKind::Plain });
                x += len + gap;
            }
        }
        // A wall cell or two as stoppers.
        for _ in 0..rng.range(0, 2) {
            let p = Pos::new(rng.range(0, (w - 1) as i32) as i16, rng.range(2, (h - 3) as i32) as i16);
            level.cells[(p.y * w + p.x) as usize] = Cell::Wall;
        }
        // Tiles on free rows/columns.
        let mut occ = vec![false; (w * h) as usize];
        for (i, c) in level.cells.iter().enumerate() {
            occ[i] = *c != Cell::Floor;
        }
        let mut ok = true;
        for (i, &(tw, key)) in tiles.iter().enumerate() {
            let mut placed = false;
            for _ in 0..200 {
                let at = Pos::new(rng.range(0, (w - tw) as i32) as i16, rng.range(0, (h - 1) as i32) as i16);
                if (0..tw).all(|dx| !occ[((at.y) * w + at.x + dx) as usize]) {
                    for dx in 0..tw {
                        occ[(at.y * w + at.x + dx) as usize] = true;
                    }
                    level.blocks.push(BlockDef::plain(names[i % names.len()], Shape::rect(tw, 1), key));
                    level.start.push(at);
                    placed = true;
                    break;
                }
            }
            if !placed {
                ok = false;
                break;
            }
        }
        if !ok || !level.validate().is_empty() {
            continue;
        }
        // Not two tiles already sitting under their door at the start.
        let st = State::initial(&level);
        let occ8 = rules::occupancy(&level, &st);
        let free = rules::legal_moves(&level, &st, &occ8).iter().filter(|(_, _, o)| matches!(o.result, rules::Result::Exit { .. })).count();
        if free > 1 {
            continue;
        }
        let Some(h0) = solver::Heuristic::new(&level).eval(&st) else { continue };
        let rep = solver::solve(&level, &st, 60_000);
        let Solve::Solved(m) = rep.result else { continue };
        let s = (m.len() as u32).saturating_sub(h0) as u16;
        if s < slack.0 || s > slack.1 {
            if best.is_none() {
                level.meta.optimal = Some(m.len() as u16);
                best = Some((level, s));
            }
            continue;
        }
        level.meta.optimal = Some(m.len() as u16);
        return Some(level);
    }
    best.map(|b| b.0)
}
