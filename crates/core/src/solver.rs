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

/// Fewest-moves solution by plain breadth-first search (reference implementation for tests).
pub fn solve_bfs(level: &Level, start: &State, limit: usize) -> SolveReport {
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

/// Lower bound of the moves each block still needs, ignoring the other blocks: the fewest straight slides
/// (stopping anywhere) to reach an accepting door, through its outer layer first if it has one.
/// `table[b][cell * 2 + shell]`, `u8::MAX` when the block can never leave.
pub struct Heuristic {
    table: Vec<Vec<u8>>,
    w: i16,
}

impl Heuristic {
    pub fn new(level: &Level) -> Heuristic {
        let n = (level.w * level.h) as usize;
        let empty = vec![0u8; n];
        let mut cache: Vec<(usize, Vec<u8>)> = Vec::new();
        let mut table = Vec::with_capacity(level.blocks.len());
        for b in 0..level.blocks.len() {
            if let Some((_, t)) = cache.iter().find(|(o, _)| level.blocks[*o].same_class(&level.blocks[b])) {
                table.push(t.clone());
                continue;
            }
            let def = &level.blocks[b];
            let mut d = vec![u8::MAX; n * 2];
            // Positions where the block fits on the empty board.
            let fits_at: Vec<bool> = (0..n).map(|i| rules::fits(level, &empty, b, Pos::new((i as i16) % level.w, (i as i16) / level.w))).collect();
            // Value iteration on a tiny graph.
            loop {
                let mut changed = false;
                for i in 0..n {
                    if !fits_at[i] {
                        continue;
                    }
                    let p = Pos::new((i as i16) % level.w, (i as i16) / level.w);
                    for shell in [false, true] {
                        if shell && def.shell.is_none() {
                            continue;
                        }
                        let need = if shell { def.shell.unwrap() } else { def.key };
                        let mut best = d[i * 2 + shell as usize];
                        for dir in crate::geom::Dir::ALL {
                            if let Some(a) = def.axis
                                && dir.axis() != a
                            {
                                continue;
                            }
                            // Slide to every reachable stop.
                            let mut k = 1;
                            let mut last = p;
                            while rules::fits(level, &empty, b, p.step(dir, k)) {
                                let q = p.step(dir, k);
                                last = q;
                                let v = d[level.idx(q) * 2 + shell as usize];
                                if v != u8::MAX {
                                    best = best.min(v.saturating_add(1));
                                }
                                k += 1;
                            }
                            if let Some(doors) = rules::doors_ahead(level, &empty, b, last, dir)
                                && !doors.is_empty()
                                && doors.iter().all(|&di| level.doors[di].key == need)
                            {
                                if shell {
                                    let v = d[level.idx(last) * 2];
                                    if v != u8::MAX {
                                        best = best.min(v.saturating_add(1));
                                    }
                                } else {
                                    best = best.min(1);
                                }
                            }
                        }
                        if best < d[i * 2 + shell as usize] {
                            d[i * 2 + shell as usize] = best;
                            changed = true;
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
            cache.push((b, d.clone()));
            table.push(d);
        }
        Heuristic { table, w: level.w }
    }

    /// Sum over the blocks still on the board, `None` if one of them can never leave.
    pub fn eval(&self, st: &State) -> Option<u32> {
        let mut h = 0u32;
        for (b, s) in st.blocks.iter().enumerate() {
            if !s.alive {
                continue;
            }
            let i = (s.pos.y * self.w + s.pos.x) as usize;
            let v = self.table[b][i * 2 + s.shell as usize];
            if v == u8::MAX {
                return None;
            }
            h += v as u32;
        }
        Some(h)
    }
}

/// Fewest-moves solution from `start` (A* with [`Heuristic`]), exploring at most `limit` states.
pub fn solve(level: &Level, start: &State, limit: usize) -> SolveReport {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, HashMap};
    if start.is_won() {
        return SolveReport { result: Solve::Solved(Vec::new()), explored: 0 };
    }
    let pk = Packer::new(level);
    let heur = Heuristic::new(level);
    let Some(h0) = heur.eval(start) else {
        return SolveReport { result: Solve::Unsolvable, explored: 1 };
    };
    let root = pk.pack(start);
    let mut best: HashMap<u128, u16> = HashMap::new();
    best.insert(hash128(&root), 0);
    let mut nodes: Vec<(Box<[u16]>, u32, Move, u16)> = vec![(root, u32::MAX, Move { block: 0, dir: crate::geom::Dir::N }, 0)];
    let mut open: BinaryHeap<(Reverse<u32>, u16, Reverse<u32>)> = BinaryHeap::new();
    open.push((Reverse(h0), 0, Reverse(0)));
    while let Some((_, g, Reverse(idx))) = open.pop() {
        if nodes.len() > limit {
            return SolveReport { result: Solve::TooBig, explored: nodes.len() };
        }
        let idx = idx as usize;
        if nodes[idx].3 != g {
            continue;
        }
        let st = pk.unpack(&nodes[idx].0);
        if st.is_won() {
            let mut path = Vec::new();
            let mut i = idx;
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
            return SolveReport { result: Solve::Solved(moves), explored: nodes.len() };
        }
        let occ = rules::occupancy(level, &st);
        for (mv, next, _) in rules::legal_moves(level, &st, &occ) {
            let Some(h) = heur.eval(&next) else { continue };
            let packed = pk.pack(&next);
            let key = hash128(&packed);
            let g2 = g + 1;
            if best.get(&key).is_some_and(|&b| b <= g2) {
                continue;
            }
            best.insert(key, g2);
            let j = nodes.len() as u32;
            nodes.push((packed, idx as u32, mv, g2));
            open.push((Reverse(g2 as u32 + h), g2, Reverse(j)));
        }
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
    /// The closest dead state to the start, if any (for diagnostics).
    pub dead_example: Option<State>,
}

/// Explores every reachable state (up to `limit`) and measures how forgiving the level is.
pub fn analyze(level: &Level, limit: usize) -> Analysis {
    let pk = Packer::new(level);
    let start = State::initial(level);
    let mut index: std::collections::HashMap<u128, u32> = std::collections::HashMap::new();
    let mut packed: Vec<Box<[u16]>> = Vec::new();
    let mut edges: Vec<Vec<u32>> = Vec::new();
    let mut won: Vec<bool> = Vec::new();
    let mut expanded: Vec<bool> = Vec::new();
    let root = pk.pack(&start);
    index.insert(hash128(&root), 0);
    packed.push(root);
    edges.push(Vec::new());
    won.push(start.is_won());
    expanded.push(false);
    let mut queue = VecDeque::from([0u32]);
    let mut complete = true;
    while let Some(i) = queue.pop_front() {
        if packed.len() > limit {
            complete = false;
            break;
        }
        let st = pk.unpack(&packed[i as usize]);
        expanded[i as usize] = true;
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
                    expanded.push(false);
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
    // Dead for certain: neither a win nor a state left unexplored (budget) can be reached from it.
    let mut open = vec![false; n];
    let mut q: VecDeque<usize> = (0..n).filter(|&i| won[i] || !expanded[i]).collect();
    for &i in &q {
        open[i] = true;
    }
    while let Some(i) = q.pop_front() {
        for &p in &rev[i] {
            if !open[p as usize] {
                open[p as usize] = true;
                q.push_back(p as usize);
            }
        }
    }
    let is_dead = |i: usize| !open[i];
    let dead_states = (0..n).filter(|&i| is_dead(i)).count();
    let dead_example = (0..n).find(|&i| is_dead(i)).map(|i| pk.unpack(&packed[i]));
    let first = &edges[0];
    let optimal_first_moves = first.iter().filter(|&&j| dist[j as usize] != u32::MAX && dist[j as usize] + 1 == dist[0]).count();
    let losing_first_moves = first.iter().filter(|&&j| is_dead(j as usize)).count();
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
        dead_example,
    }
}
