//! The rules: a dragged block slides until something stops it; if every row (or column) it covers ends at an open
//! door that accepts it, it leaves the board.

use crate::geom::{Axis, Dir, Pos};
use crate::level::{Key, Level, WrongDoor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState {
    pub pos: Pos,
    pub alive: bool,
    /// The outer layer is still there.
    pub shell: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub blocks: Vec<BlockState>,
    /// Exits so far (drives ice and shutter doors).
    pub exits: u16,
}

impl State {
    pub fn initial(level: &Level) -> State {
        State {
            blocks: level.blocks.iter().zip(&level.start).map(|(b, &pos)| BlockState { pos, alive: true, shell: b.shell.is_some() }).collect(),
            exits: 0,
        }
    }

    pub fn is_won(&self) -> bool {
        self.blocks.iter().all(|b| !b.alive)
    }

    pub fn remaining(&self) -> usize {
        self.blocks.iter().filter(|b| b.alive).count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Move {
    pub block: u8,
    pub dir: Dir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Result {
    /// The block slid and rests. `against` is the door it rests against, if any (a door that did not accept it).
    Stop { against: Option<usize> },
    /// The block left the board through this door.
    Exit { door: usize },
    /// The outer layer came off at this door; the block rests in front of it with its own key.
    Peel { door: usize },
    /// Wrong door in a language level: the block bounced back to where it was (nothing changed).
    Bounce { door: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Cells travelled before reaching the door or the obstacle.
    pub steps: i32,
    pub result: Result,
}

/// Why a block cannot be moved at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stuck {
    Gone,
    /// Frozen: this many more exits needed.
    Ice(u8),
    Locked(u8),
    /// Only moves along this axis.
    Lane(Axis),
}

/// Occupancy grid: 0 for empty, `i + 1` for block `i`.
pub fn occupancy(level: &Level, st: &State) -> Vec<u8> {
    let mut occ = vec![0u8; (level.w * level.h) as usize];
    for (i, (b, s)) in level.blocks.iter().zip(&st.blocks).enumerate() {
        if !s.alive {
            continue;
        }
        for c in &b.shape.cells {
            let q = s.pos.add(*c);
            if level.in_bounds(q) {
                occ[level.idx(q)] = i as u8 + 1;
            }
        }
    }
    occ
}

pub fn ice_left(level: &Level, st: &State, b: usize) -> u8 {
    level.blocks[b].ice.saturating_sub(st.exits.min(255) as u8)
}

pub fn is_locked(level: &Level, st: &State, b: usize) -> bool {
    match level.blocks[b].lock {
        Some(l) => level.blocks.iter().zip(&st.blocks).any(|(o, s)| s.alive && o.carries == Some(l)),
        None => false,
    }
}

/// The key the block needs right now.
pub fn needed_key(level: &Level, st: &State, b: usize) -> Key {
    let def = &level.blocks[b];
    match def.shell {
        Some(k) if st.blocks[b].shell => k,
        _ => def.key,
    }
}

/// Can this block move in this direction at all (ignoring obstacles)?
pub fn stuck(level: &Level, st: &State, b: usize, dir: Option<Dir>) -> Option<Stuck> {
    if !st.blocks[b].alive {
        return Some(Stuck::Gone);
    }
    let ice = ice_left(level, st, b);
    if ice > 0 {
        return Some(Stuck::Ice(ice));
    }
    if is_locked(level, st, b) {
        return Some(Stuck::Locked(level.blocks[b].lock.unwrap_or(0)));
    }
    if let (Some(a), Some(d)) = (level.blocks[b].axis, dir)
        && d.axis() != a
    {
        return Some(Stuck::Lane(a));
    }
    None
}

fn fits(level: &Level, occ: &[u8], b: usize, at: Pos) -> bool {
    level.blocks[b].shape.cells.iter().all(|c| {
        let q = at.add(*c);
        level.is_floor(q) && {
            let o = occ[level.idx(q)];
            o == 0 || o as usize == b + 1
        }
    })
}

/// Doors reached by each line of the block when pushed further in `d` from `at`, or `None` if some line ends at a
/// plain wall or another block.
fn doors_ahead(level: &Level, occ: &[u8], b: usize, at: Pos, d: Dir) -> Option<Vec<usize>> {
    let shape = &level.blocks[b].shape;
    let mut out = Vec::new();
    // Lines: rows for horizontal moves, columns for vertical moves.
    let (lines, along_h) = match d.axis() {
        Axis::H => (shape.h, true),
        Axis::V => (shape.w, false),
    };
    for l in 0..lines {
        // Rear-most own cell of this line (relative to the direction of travel).
        let mut cells: Vec<Pos> = shape.cells.iter().filter(|c| if along_h { c.y == l } else { c.x == l }).map(|c| at.add(*c)).collect();
        if cells.is_empty() {
            continue;
        }
        cells.sort_by_key(|c| match d {
            Dir::E => c.x,
            Dir::W => -c.x,
            Dir::S => c.y,
            Dir::N => -c.y,
        });
        let mut q = cells[0];
        loop {
            let next = q.step(d, 1);
            if !level.is_floor(next) {
                out.push(level.door_at(q, d)?);
                break;
            }
            let o = occ[level.idx(next)];
            if o != 0 && o as usize != b + 1 {
                return None;
            }
            q = next;
        }
    }
    Some(out)
}

/// Plays a move. Returns `None` if nothing would happen (blocked block, or no room and no door).
pub fn play(level: &Level, st: &State, occ: &[u8], mv: Move) -> Option<(State, Outcome)> {
    let b = mv.block as usize;
    if b >= st.blocks.len() || stuck(level, st, b, Some(mv.dir)).is_some() {
        return None;
    }
    let start = st.blocks[b].pos;
    let mut steps = 0;
    while fits(level, occ, b, start.step(mv.dir, steps + 1)) {
        steps += 1;
    }
    let at = start.step(mv.dir, steps);
    let mut next = st.clone();
    let result = match doors_ahead(level, occ, b, at, mv.dir) {
        Some(doors) if !doors.is_empty() => {
            let need = needed_key(level, st, b);
            let door = doors[0];
            let all_accept = doors.iter().all(|&i| level.doors[i].key == need);
            let all_open = doors.iter().all(|&i| level.doors[i].is_open(st.exits));
            if all_accept && all_open {
                if st.blocks[b].shell && level.blocks[b].shell.is_some() {
                    next.blocks[b].pos = at;
                    next.blocks[b].shell = false;
                    Result::Peel { door }
                } else {
                    next.blocks[b].alive = false;
                    next.exits += 1;
                    Result::Exit { door }
                }
            } else if !all_accept && all_open && level.meta.wrong_door == WrongDoor::Bounce {
                return Some((st.clone(), Outcome { steps, result: Result::Bounce { door } }));
            } else {
                next.blocks[b].pos = at;
                Result::Stop { against: Some(door) }
            }
        }
        _ => {
            next.blocks[b].pos = at;
            Result::Stop { against: None }
        }
    };
    if steps == 0 && matches!(result, Result::Stop { .. }) {
        return None;
    }
    Some((next, Outcome { steps, result }))
}

/// All moves that change something (bounces excluded).
pub fn legal_moves(level: &Level, st: &State, occ: &[u8]) -> Vec<(Move, State, Outcome)> {
    let mut v = Vec::new();
    for b in 0..st.blocks.len() {
        if !st.blocks[b].alive {
            continue;
        }
        for d in Dir::ALL {
            let mv = Move { block: b as u8, dir: d };
            if let Some((s, o)) = play(level, st, occ, mv)
                && !matches!(o.result, Result::Bounce { .. })
            {
                v.push((mv, s, o));
            }
        }
    }
    v
}
