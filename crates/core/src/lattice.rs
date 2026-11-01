//! Exhaustive proofs about language boards ("lattice boards").
//!
//! On a language board every tile is exactly as wide as a door and starts on the door grid, the side walls are
//! plain, and every column ends at the top and at the bottom either in a plain wall or in a door (wrong doors send
//! the tile back: `WrongDoor::Bounce`). A slide stops at a wall or against another tile, both on the grid, so a tile
//! is always in one of `cols × rows` slots: the board is a small finite structure and so is the set of all its
//! positions.
//!
//! [`prove`] walks through **every** placement of up to `k` tiles, with every possible assignment of target doors,
//! and checks that each one can still be won. If so, no sequence of moves can ever trap the player on any board with
//! that geometry, whatever the words: the generator then never needs to search a board's state graph for dead ends.
//! The proof goes by induction on the number of tiles: a position is winnable if some sequence of moves lets a tile
//! leave by its own door (one tile fewer: winnable by the previous step), computed as a fixed point (repeated sweeps
//! until nothing changes). See `docs/adr/0006-modele-mathematique.md`.

use crate::geom::{Dir, Pos};
use crate::level::{Cell, Door, DoorKind, Key, Level, Meta, WrongDoor};

/// What closes one end of a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Wall,
    Door(Key),
}

/// A lattice board: `cols` columns of `rows` slots, with the top and bottom end of every column.
#[derive(Clone, Debug)]
pub struct Geometry {
    pub cols: usize,
    pub rows: usize,
    pub top: Vec<End>,
    pub bottom: Vec<End>,
}

/// Result of [`prove`].
#[derive(Clone, Debug)]
pub struct Proof {
    /// Positions examined, for each number of tiles (index 0 = one tile).
    pub positions: Vec<u64>,
    /// Most moves ever needed before some tile can leave by its own door, over all positions.
    pub longest: u32,
    /// A position that cannot be won, if any: (slot, target) per tile, slot = row * cols + col.
    pub dead: Option<Vec<(usize, Key)>>,
}

impl Geometry {
    /// The keys of the doors (the possible targets of a tile).
    pub fn keys(&self) -> Vec<Key> {
        let mut v: Vec<Key> = self.top.iter().chain(&self.bottom).filter_map(|e| if let End::Door(k) = e { Some(*k) } else { None }).collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Text picture of a position (`#` walls, door keys as digits, tiles as `target` letters a, b, c…).
    pub fn draw(&self, tiles: &[(usize, Key)]) -> String {
        let keys = self.keys();
        let end = |e: &End| match e {
            End::Wall => '#',
            End::Door(k) => char::from_digit(keys.iter().position(|x| x == k).unwrap_or(0) as u32, 10).unwrap_or('?'),
        };
        let mut s = String::new();
        s.push_str(&self.top.iter().map(end).collect::<String>());
        s.push('\n');
        for r in 0..self.rows {
            for c in 0..self.cols {
                let slot = r * self.cols + c;
                s.push(match tiles.iter().find(|t| t.0 == slot) {
                    Some(t) => (b'a' + keys.iter().position(|x| *x == t.1).unwrap_or(0) as u8) as char,
                    None => '.',
                });
            }
            s.push('\n');
        }
        s.push_str(&self.bottom.iter().map(end).collect::<String>());
        s
    }
}

/// The board layout of a language round: lattice geometry and door width (2 or 3 cells). Boards are always six cells
/// wide and seven rows high.
#[derive(Clone, Debug)]
pub struct Layout {
    pub geometry: Geometry,
    pub len: i16,
}

/// Layouts the game uses, with the number of tiles up to which `fuera-cli prove` has checked every position
/// (keys are placeholders: only which ends share a key matters).
pub fn production() -> Vec<(&'static str, Layout, usize)> {
    vec![
        ("deux catégories, tuiles de 2", Layout::two_categories(0, 1, 2), 6),
        ("deux catégories, tuiles de 3", Layout::two_categories(0, 1, 3), 6),
        ("quatre portes dans les coins", Layout::corners([Some(0), Some(1), Some(2), Some(3)]), 5),
        ("trois portes dans les coins", Layout::corners([Some(0), Some(1), Some(2), None]), 5),
    ]
}

impl Layout {
    pub const ROWS: usize = 7;
    pub const WIDTH: i16 = 6;

    /// Two categories: `top`'s door over every column but the last, `bottom`'s under every column but the first.
    /// Each side column thus has a plain wall at one end to stop against, and two tiles that want to cross can
    /// always get out of each other's way (with doors over the whole width, two rows of three tiles that want to
    /// swap are stuck for good).
    pub fn two_categories(top: Key, bottom: Key, len: i16) -> Layout {
        let cols = (Self::WIDTH / len) as usize;
        let mut t = vec![End::Door(top); cols];
        t[cols - 1] = End::Wall;
        let mut b = vec![End::Door(bottom); cols];
        b[0] = End::Wall;
        Layout { geometry: Geometry { cols, rows: Self::ROWS, top: t, bottom: b }, len }
    }

    /// Three or four doors in the corners (top-left, top-right, bottom-left, bottom-right; `None`: plain wall), with
    /// plain wall above and below the middle column, two-cell doors. Sideways slides stop at the side walls, right
    /// under the doors; the middle column is where tiles park to let others pass.
    pub fn corners(c: [Option<Key>; 4]) -> Layout {
        let end = |k: Option<Key>| k.map(End::Door).unwrap_or(End::Wall);
        Layout { geometry: Geometry { cols: 3, rows: Self::ROWS, top: vec![end(c[0]), End::Wall, end(c[1])], bottom: vec![end(c[2]), End::Wall, end(c[3])] }, len: 2 }
    }

    /// An empty level with this layout's doors (neighbouring ends with the same key make one door).
    pub fn level(&self) -> Level {
        let g = &self.geometry;
        let (w, h) = (g.cols as i16 * self.len, g.rows as i16);
        let mut level = Level { w, h, cells: vec![Cell::Floor; (w * h) as usize], doors: Vec::new(), blocks: Vec::new(), start: Vec::new(), meta: Meta::default() };
        level.meta.wrong_door = WrongDoor::Bounce;
        for (side, ends, y) in [(Dir::N, &g.top, 0), (Dir::S, &g.bottom, h - 1)] {
            let mut c = 0;
            while c < g.cols {
                let End::Door(k) = ends[c] else {
                    c += 1;
                    continue;
                };
                let first = c;
                while c < g.cols && ends[c] == End::Door(k) {
                    c += 1;
                }
                let len = (c - first) as i16 * self.len;
                level.doors.push(Door { side, at: Pos::new(first as i16 * self.len, y), len: len as u8, key: k, kind: DoorKind::Plain });
            }
        }
        level
    }

    /// The layout of a level, if it is a plain lattice board (only floor, plain one-row tiles all as wide as the
    /// grid, plain doors on the grid at the top and bottom, wrong doors bouncing).
    pub fn of_level(level: &Level) -> Option<Layout> {
        let len = level.blocks.first()?.shape.w;
        if len < 1 || level.w % len != 0 || level.meta.wrong_door != WrongDoor::Bounce || level.cells.iter().any(|c| *c != Cell::Floor) {
            return None;
        }
        let plain = |b: &crate::level::BlockDef| b.shape.h == 1 && b.shape.w == len && b.shape.cells.len() == len as usize && b.shell.is_none() && b.axis.is_none() && b.ice == 0 && b.carries.is_none() && b.lock.is_none();
        if !level.blocks.iter().all(plain) || !level.start.iter().all(|p| p.x % len == 0) {
            return None;
        }
        let cols = (level.w / len) as usize;
        let mut top = vec![End::Wall; cols];
        let mut bottom = vec![End::Wall; cols];
        for d in &level.doors {
            if d.kind != DoorKind::Plain || d.at.x % len != 0 || d.len as i16 % len != 0 {
                return None;
            }
            let ends = match d.side {
                Dir::N if d.at.y == 0 => &mut top,
                Dir::S if d.at.y == level.h - 1 => &mut bottom,
                _ => return None,
            };
            for c in (d.at.x / len) as usize..((d.at.x + d.len as i16) / len) as usize {
                ends[c] = End::Door(d.key);
            }
        }
        Some(Layout { geometry: Geometry { cols, rows: level.h as usize, top, bottom }, len })
    }

    /// Most tiles for which every position of this layout is known to be winnable (0: unknown).
    pub fn proven(&self) -> usize {
        let me = self.geometry.symmetric_forms();
        production().into_iter().filter(|(_, l, _)| l.len == self.len && me.contains(&l.geometry.normal())).map(|p| p.2).max().unwrap_or(0)
    }
}

impl Geometry {
    /// Same shape with keys renamed 0, 1, 2… in order of first appearance (top row, then bottom row).
    fn normal(&self) -> (usize, usize, Vec<Option<usize>>) {
        let mut seen: Vec<Key> = Vec::new();
        let ends = self
            .top
            .iter()
            .chain(&self.bottom)
            .map(|e| match e {
                End::Wall => None,
                End::Door(k) => Some(seen.iter().position(|x| x == k).unwrap_or_else(|| {
                    seen.push(*k);
                    seen.len() - 1
                })),
            })
            .collect();
        (self.cols, self.rows, ends)
    }

    /// Normal forms of the four mirror images (the rules are the same in a mirror).
    fn symmetric_forms(&self) -> Vec<(usize, usize, Vec<Option<usize>>)> {
        let flip = |v: &[End]| v.iter().rev().copied().collect::<Vec<End>>();
        let mut out = Vec::new();
        for (t, b) in [
            (self.top.clone(), self.bottom.clone()),
            (flip(&self.top), flip(&self.bottom)),
            (self.bottom.clone(), self.top.clone()),
            (flip(&self.bottom), flip(&self.top)),
        ] {
            out.push(Geometry { cols: self.cols, rows: self.rows, top: t, bottom: b }.normal());
        }
        out
    }
}

/// Binomial coefficients up to n = 32.
fn binomials() -> Vec<[u64; 33]> {
    let mut c = vec![[0u64; 33]; 33];
    for n in 0..33 {
        c[n][0] = 1;
        for k in 1..=n {
            c[n][k] = c[n - 1][k - 1] + if k <= n - 1 { c[n - 1][k] } else { 0 };
        }
    }
    c
}

/// Checks every position of `1..=max_tiles` tiles. Stops at the first number of tiles that has a dead position.
pub fn prove(g: &Geometry, max_tiles: usize) -> Proof {
    let keys = g.keys();
    let t = keys.len().max(1) as u64;
    let slots = g.cols * g.rows;
    assert!(slots <= 32);
    let binom = binomials();
    let mut proof = Proof { positions: Vec::new(), longest: 0, dead: None };
    for k in 1..=max_tiles.min(slots) {
        let combos = binom[slots][k];
        let tcodes = t.pow(k as u32);
        let total = combos * tcodes;
        proof.positions.push(total);
        let mut good = vec![false; total as usize];
        let mut pos = vec![0usize; k];
        let mut tgt = vec![0usize; k];
        let rank = |pos: &[usize], tgt: &[usize]| -> u64 {
            // Tiles sorted by slot; targets follow their tile.
            let mut r = 0u64;
            for (i, &p) in pos.iter().enumerate() {
                r += binom[p][i + 1];
            }
            let mut code = 0u64;
            for &x in tgt.iter().rev() {
                code = code * t + x as u64;
            }
            r * tcodes + code
        };
        let unrank = |mut r: u64, pos: &mut [usize], tgt: &mut [usize]| {
            let mut code = r % tcodes;
            r /= tcodes;
            for i in (0..k).rev() {
                let mut p = i;
                while binom[p + 1][i + 1] <= r {
                    p += 1;
                }
                pos[i] = p;
                r -= binom[p][i + 1];
            }
            for x in tgt.iter_mut() {
                *x = (code % t) as usize;
                code /= t;
            }
        };
        let mut occupied = vec![false; slots];
        // Successors of a position: Some(index) for a move, None for a tile leaving by its own door.
        let mut step = |r: u64, out: &mut Vec<Option<u64>>, pos: &mut Vec<usize>, tgt: &mut Vec<usize>| {
            out.clear();
            unrank(r, pos, tgt);
            occupied.iter_mut().for_each(|o| *o = false);
            for &p in pos.iter() {
                occupied[p] = true;
            }
            for i in 0..k {
                let (c, row) = (pos[i] % g.cols, pos[i] / g.cols);
                let key = keys[tgt[i]];
                for dir in 0..4 {
                    let (mut nc, mut nr) = (c as i32, row as i32);
                    let (dc, dr) = [(-1, 0), (1, 0), (0, -1), (0, 1)][dir];
                    loop {
                        let (xc, xr) = (nc + dc, nr + dr);
                        if xc < 0 || xc >= g.cols as i32 || xr < 0 || xr >= g.rows as i32 {
                            break;
                        }
                        if occupied[xr as usize * g.cols + xc as usize] {
                            break;
                        }
                        nc = xc;
                        nr = xr;
                    }
                    // Reached the top or bottom end with nothing in between: door or wall.
                    let end = if dr == -1 && nr == 0 {
                        Some(g.top[c])
                    } else if dr == 1 && nr == g.rows as i32 - 1 {
                        Some(g.bottom[c])
                    } else {
                        None
                    };
                    match end {
                        Some(End::Door(d)) if d == key => {
                            out.push(None);
                            continue;
                        }
                        Some(End::Door(_)) => continue, // bounces back: not a move
                        _ => {}
                    }
                    if (nc, nr) == (c as i32, row as i32) {
                        continue;
                    }
                    let mut np: Vec<(usize, usize)> = (0..k).map(|j| (pos[j], tgt[j])).collect();
                    np[i].0 = nr as usize * g.cols + nc as usize;
                    np.sort_unstable();
                    let p2: Vec<usize> = np.iter().map(|x| x.0).collect();
                    let t2: Vec<usize> = np.iter().map(|x| x.1).collect();
                    out.push(Some(rank(&p2, &t2)));
                }
            }
        };
        let mut out = Vec::new();
        let (mut pbuf, mut tbuf) = (pos.clone(), tgt.clone());
        // Layer by layer: layer 0 = a tile can leave by its own door now (the rest is winnable by induction), layer
        // n + 1 = a move leads into layer n. Stops when a layer is empty; what is left can never be won.
        let mut layer = 0u32;
        loop {
            let mut fresh = Vec::new();
            for r in 0..total {
                if good[r as usize] {
                    continue;
                }
                step(r, &mut out, &mut pbuf, &mut tbuf);
                if out.iter().any(|o| o.is_none_or(|j| good[j as usize])) {
                    fresh.push(r);
                }
            }
            if fresh.is_empty() {
                break;
            }
            for r in fresh {
                good[r as usize] = true;
            }
            layer += 1;
        }
        // Layers past the first are moves needed before some tile can leave.
        proof.longest = proof.longest.max(layer.saturating_sub(1));
        if let Some(r) = (0..total).find(|&r| !good[r as usize]) {
            unrank(r, &mut pos, &mut tgt);
            proof.dead = Some(pos.iter().zip(&tgt).map(|(&p, &x)| (p, keys[x])).collect());
            return proof;
        }
    }
    proof
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_columns_all_doors() -> Geometry {
        Geometry { cols: 2, rows: 7, top: vec![End::Door(0), End::Door(1)], bottom: vec![End::Door(2), End::Door(3)] }
    }

    #[test]
    fn doors_over_every_column_can_trap() {
        // Two tiles side by side that want each other's column: nothing can stop them anywhere else.
        let p = prove(&two_columns_all_doors(), 2);
        assert!(p.dead.is_some());
    }

    #[test]
    fn parking_column_never_traps_small() {
        let g = Geometry {
            cols: 3,
            rows: 7,
            top: vec![End::Door(0), End::Wall, End::Door(1)],
            bottom: vec![End::Door(2), End::Wall, End::Door(3)],
        };
        let p = prove(&g, 3);
        assert!(p.dead.is_none(), "{}", g.draw(p.dead.as_deref().unwrap_or(&[])));
    }
}
