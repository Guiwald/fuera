//! Breadth-first solver (optimal in number of moves) and level analysis.
//!
//! States are packed into `u16` per block (cell index, shell bit, or `GONE`). Interchangeable blocks (same shape,
//! key and modifiers) are sorted so that permutations of identical blocks count once. Visited states are kept as a
//! 128-bit hash.

use std::collections::{HashSet, VecDeque};

use crate::geom::Pos;
use crate::level::Level;
use crate::rules::{self, BlockState, Move, State};

const GONE: u16 = 0xFFFF;
const SHELL: u16 = 0x8000;

pub struct Packer {
    w: i16,
    /// Groups of interchangeable block indices (each sorted).
    groups: Vec<Vec<usize>>,
    n: usize,
}

impl Packer {
    pub fn new(level: &Level) -> Packer {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for (i, b) in level.blocks.iter().enumerate() {
            match groups.iter_mut().find(|g| level.blocks[g[0]].same_class(b)) {
                Some(g) => g.push(i),
                None => groups.push(vec![i]),
            }
        }
        Packer { w: level.w, groups, n: level.blocks.len() }
    }

    fn code(&self, b: &BlockState) -> u16 {
        if !b.alive {
            return GONE;
        }
        let i = (b.pos.y * self.w + b.pos.x) as u16;
        if b.shell { i | SHELL } else { i }
    }

    /// Canonical packing: identical blocks sorted by position.
    pub fn pack(&self, st: &State) -> Box<[u16]> {
        let mut v = vec![0u16; self.n];
        for g in &self.groups {
            let mut codes: Vec<u16> = g.iter().map(|&i| self.code(&st.blocks[i])).collect();
            codes.sort_unstable();
            for (&i, c) in g.iter().zip(codes) {
                v[i] = c;
            }
        }
        v.into_boxed_slice()
    }

    pub fn unpack(&self, v: &[u16]) -> State {
        let mut exits = 0;
        let blocks = v
            .iter()
            .map(|&c| {
                if c == GONE {
                    exits += 1;
                    BlockState { pos: Pos::default(), alive: false, shell: false }
                } else {
                    let i = (c & !SHELL) as i16;
                    BlockState { pos: Pos::new(i % self.w, i / self.w), alive: true, shell: c & SHELL != 0 }
                }
            })
            .collect();
        State { blocks, exits }
    }
}

fn hash128(v: &[u16]) -> u128 {
    // Two independent 64-bit FNV-style hashes.
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    for &x in v {
        a = (a ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3);
        b = (b ^ (x as u64).wrapping_mul(0xff51_afd7_ed55_8ccd)).rotate_left(29).wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    }
    ((a as u128) << 64) | b as u128
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Solve {
    /// Optimal move list from the given state.
    Solved(Vec<Move>),
    Unsolvable,
    /// Search budget exhausted.
    TooBig,
}

pub struct SolveReport {
    pub result: Solve,
    pub explored: usize,
}

/// Maps a move found on a canonical state to the same physical block in the real state (identical blocks may have
/// been permuted by packing): the block of the same class at the same position.
fn real_move(level: &Level, canon: &State, real: &State, mv: Move) -> Move {
    let cb = mv.block as usize;
    let pos = canon.blocks[cb].pos;
    let found = real.blocks.iter().enumerate().position(|(i, b)| b.alive && b.pos == pos && level.blocks[i].same_class(&level.blocks[cb]));
    Move { block: found.unwrap_or(cb) as u8, dir: mv.dir }
}

/// Fewest-moves solution from `start`, exploring at most `limit` states.
pub fn solve(level: &Level, start: &State, limit: usize) -> SolveReport {
    let pk = Packer::new(level);
    if start.is_won() {
        return SolveReport { result: Solve::Solved(Vec::new()), explored: 0 };
    }
    let root = pk.pack(start);
    let mut seen: HashSet<u128> = HashSet::new();
    seen.insert(hash128(&root));
    // (packed state, parent index, move from parent)
    let mut nodes: Vec<(Box<[u16]>, u32, Move)> = vec![(root, u32::MAX, Move { block: 0, dir: crate::geom::Dir::N })];
    let mut head = 0usize;
    while head < nodes.len() {
        if nodes.len() > limit {
            return SolveReport { result: Solve::TooBig, explored: nodes.len() };
        }
        let st = pk.unpack(&nodes[head].0);
        let occ = rules::occupancy(level, &st);
        for (mv, next, _) in rules::legal_moves(level, &st, &occ) {
            let packed = pk.pack(&next);
            if !seen.insert(hash128(&packed)) {
                continue;
            }
            let won = next.is_won();
            nodes.push((packed, head as u32, mv));
            if won {
                // Rebuild the canonical path, then replay it on the real state.
                let mut path = Vec::new();
                let mut i = nodes.len() - 1;
                while nodes[i].1 != u32::MAX {
                    path.push((nodes[i].1 as usize, nodes[i].2));
                    i = nodes[i].1 as usize;
                }
                path.reverse();
                let mut real = start.clone();
                let mut moves = Vec::new();
                for (parent, mv) in path {
                    let canon = pk.unpack(&nodes[parent].0);
                    let rm = real_move(level, &canon, &real, mv);
                    let occ = rules::occupancy(level, &real);
                    match rules::play(level, &real, &occ, rm) {
                        Some((s, _)) => real = s,
                        None => break,
                    }
                    moves.push(rm);
                }
                let explored = nodes.len();
                return SolveReport { result: Solve::Solved(moves), explored };
            }
        }
        head += 1;
    }
    SolveReport { result: Solve::Unsolvable, explored: nodes.len() }
}

/// Next move of an optimal solution, for the hint button.
pub fn hint(level: &Level, st: &State, limit: usize) -> Option<Move> {
    match solve(level, st, limit).result {
        Solve::Solved(m) => m.first().copied(),
        _ => None,
    }
}

/// Whole-graph statistics used to grade difficulty.
#[derive(Debug, Clone, Default)]
pub struct Analysis {
    pub states: usize,
    pub optimal: Option<u16>,
    /// Reachable states from which the level can no longer be won.
    pub dead_states: usize,
    /// Moves from the start that keep an optimal solution.
    pub optimal_first_moves: usize,
    /// Moves from the start that lead to a dead state.
    pub losing_first_moves: usize,
    pub first_moves: usize,
    /// Average number of distinct moves per state along the optimal path.
    pub mean_branching: f32,
    /// Exhaustive (false: budget exceeded, numbers partial).
    pub complete: bool,
}

/// Explores every reachable state (up to `limit`) and measures how forgiving the level is.
pub fn analyze(level: &Level, limit: usize) -> Analysis {
    let pk = Packer::new(level);
    let start = State::initial(level);
    let mut index: std::collections::HashMap<u128, u32> = std::collections::HashMap::new();
    let mut packed: Vec<Box<[u16]>> = Vec::new();
    let mut edges: Vec<Vec<u32>> = Vec::new();
    let mut won: Vec<bool> = Vec::new();
    let root = pk.pack(&start);
    index.insert(hash128(&root), 0);
    packed.push(root);
    edges.push(Vec::new());
    won.push(start.is_won());
    let mut queue = VecDeque::from([0u32]);
    let mut complete = true;
    while let Some(i) = queue.pop_front() {
        if packed.len() > limit {
            complete = false;
            break;
        }
        let st = pk.unpack(&packed[i as usize]);
        if st.is_won() {
            continue;
        }
        let occ = rules::occupancy(level, &st);
        let mut out = Vec::new();
        for (_, next, _) in rules::legal_moves(level, &st, &occ) {
            let p = pk.pack(&next);
            let h = hash128(&p);
            let j = match index.get(&h) {
                Some(&j) => j,
                None => {
                    let j = packed.len() as u32;
                    index.insert(h, j);
                    won.push(next.is_won());
                    packed.push(p);
                    edges.push(Vec::new());
                    queue.push_back(j);
                    j
                }
            };
            if !out.contains(&j) {
                out.push(j);
            }
        }
        edges[i as usize] = out;
    }
    let n = packed.len();
    // Distance to a goal, by backward BFS over reversed edges.
    let mut rev: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, es) in edges.iter().enumerate() {
        for &j in es {
            rev[j as usize].push(i as u32);
        }
    }
    let mut dist = vec![u32::MAX; n];
    let mut q = VecDeque::new();
    for i in 0..n {
        if won[i] {
            dist[i] = 0;
            q.push_back(i);
        }
    }
    while let Some(i) = q.pop_front() {
        for &p in &rev[i] {
            if dist[p as usize] == u32::MAX {
                dist[p as usize] = dist[i] + 1;
                q.push_back(p as usize);
            }
        }
    }
    let optimal = (dist[0] != u32::MAX).then_some(dist[0] as u16);
    // Only states whose successors were all expanded can be called dead with certainty.
    let dead_states = (0..n).filter(|&i| dist[i] == u32::MAX && (complete || !edges[i].is_empty())).count();
    let first = &edges[0];
    let optimal_first_moves = first.iter().filter(|&&j| dist[j as usize] != u32::MAX && dist[j as usize] + 1 == dist[0]).count();
    let losing_first_moves = first.iter().filter(|&&j| dist[j as usize] == u32::MAX).count();
    // Branching along one optimal path.
    let mut branching = Vec::new();
    let mut cur = 0usize;
    if optimal.is_some() {
        while dist[cur] > 0 {
            branching.push(edges[cur].len() as f32);
            match edges[cur].iter().find(|&&j| dist[j as usize] + 1 == dist[cur]) {
                Some(&j) => cur = j as usize,
                None => break,
            }
        }
    }
    let mean_branching = if branching.is_empty() { 0.0 } else { branching.iter().sum::<f32>() / branching.len() as f32 };
    Analysis {
        states: n,
        optimal,
        dead_states,
        optimal_first_moves,
        losing_first_moves,
        first_moves: first.len(),
        mean_branching,
        complete,
    }
}
