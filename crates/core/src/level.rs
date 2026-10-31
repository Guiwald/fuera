//! Level description: board cells, doors, blocks, and a small human-readable text format.
//!
//! ```text
//! fuera 1
//! name Primeros pasos
//! map
//!   AAB.
//!   C.#.
//!   ~~..
//! blocks
//!   A pink lane=h
//!   B blue ice=2
//!   C yellow shell=blue key=1
//! doors
//!   E 3 0 2 yellow
//!   W 0 1 1 pink shutter=closed
//! end
//! ```
//!
//! Map: `.` floor, `#` inner wall, `~` (or a missing cell) outside the board, any letter or digit is a cell of the
//! block of that name. Doors: side, x, y of the first cell (top-most for E/W, left-most for N/S), length, key.

use std::fmt::Write as _;

use crate::geom::{Axis, Dir, Pos};

/// What a door accepts and what a block needs. In the classic mode it is a colour index (see [`COLOR_NAMES`]); the
/// language modes use their own numbering (meanings, categories…).
pub type Key = u16;

/// Classic colours, in palette order. The UI pairs each with a symbol for colour-blind players.
pub const COLOR_NAMES: [&str; 8] = ["red", "blue", "yellow", "green", "purple", "orange", "cyan", "pink"];

pub fn color_key(name: &str) -> Option<Key> {
    COLOR_NAMES.iter().position(|c| *c == name).map(|i| i as Key)
}

pub fn key_name(k: Key) -> String {
    COLOR_NAMES.get(k as usize).map(|s| s.to_string()).unwrap_or_else(|| format!("k{k}"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    /// Outside the board.
    Void,
    Floor,
    /// An inner wall: part of the tray, blocks cannot enter it.
    Wall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorKind {
    Plain,
    /// Opens and closes after every exit (any door). `open` is the state at the start of the level.
    Shutter { open: bool },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Door {
    /// Side of the cells the door is on, which is also the direction a block must move to go through it.
    pub side: Dir,
    /// First cell: top-most for E/W doors, left-most for N/S doors.
    pub at: Pos,
    pub len: u8,
    pub key: Key,
    pub kind: DoorKind,
}

impl Door {
    /// The floor cells whose `side` edge this door covers.
    pub fn cells(&self) -> impl Iterator<Item = Pos> + '_ {
        let along = match self.side.axis() {
            Axis::H => Dir::S,
            Axis::V => Dir::E,
        };
        (0..self.len as i32).map(move |i| self.at.step(along, i))
    }

    pub fn covers(&self, p: Pos, side: Dir) -> bool {
        if side != self.side {
            return false;
        }
        match side.axis() {
            Axis::H => p.x == self.at.x && p.y >= self.at.y && p.y < self.at.y + self.len as i16,
            Axis::V => p.y == self.at.y && p.x >= self.at.x && p.x < self.at.x + self.len as i16,
        }
    }

    pub fn is_open(&self, exits: u16) -> bool {
        match self.kind {
            DoorKind::Plain => true,
            DoorKind::Shutter { open } => open ^ (exits % 2 == 1),
        }
    }
}

/// A rigid polyomino, as offsets from its top-left bounding-box corner. Blocks never rotate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shape {
    pub cells: Vec<Pos>,
    pub w: i16,
    pub h: i16,
}

impl Shape {
    pub fn new(mut cells: Vec<Pos>) -> Shape {
        let minx = cells.iter().map(|c| c.x).min().unwrap_or(0);
        let miny = cells.iter().map(|c| c.y).min().unwrap_or(0);
        for c in &mut cells {
            c.x -= minx;
            c.y -= miny;
        }
        cells.sort_by_key(|c| (c.y, c.x));
        cells.dedup();
        let w = cells.iter().map(|c| c.x).max().unwrap_or(0) + 1;
        let h = cells.iter().map(|c| c.y).max().unwrap_or(0) + 1;
        Shape { cells, w, h }
    }

    pub fn rect(w: i16, h: i16) -> Shape {
        let mut v = Vec::new();
        for y in 0..h {
            for x in 0..w {
                v.push(Pos::new(x, y));
            }
        }
        Shape::new(v)
    }

    pub fn contains(&self, p: Pos) -> bool {
        self.cells.contains(&p)
    }

    /// Width of the block across direction `d` (the door length it needs at least).
    pub fn width_across(&self, d: Dir) -> i16 {
        match d.axis() {
            Axis::H => self.h,
            Axis::V => self.w,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockDef {
    pub name: char,
    pub shape: Shape,
    pub key: Key,
    /// Outer layer: while present, the block needs a door of this key; going through it peels the layer off and the
    /// block stays on the board with its own key.
    pub shell: Option<Key>,
    /// Allowed movement axis (`None`: any direction).
    pub axis: Option<Axis>,
    /// Frozen until this many exits have happened.
    pub ice: u8,
    /// This block carries key `n`: when it leaves, blocks locked with `n` are freed.
    pub carries: Option<u8>,
    /// Chained until the block carrying this key has left.
    pub lock: Option<u8>,
}

impl BlockDef {
    pub fn plain(name: char, shape: Shape, key: Key) -> BlockDef {
        BlockDef { name, shape, key, shell: None, axis: None, ice: 0, carries: None, lock: None }
    }

    /// Two blocks of the same class are interchangeable for the solver.
    pub fn same_class(&self, o: &BlockDef) -> bool {
        self.shape == o.shape && self.key == o.key && self.shell == o.shell && self.axis == o.axis && self.ice == o.ice && self.carries == o.carries && self.lock == o.lock
    }
}

/// What happens when a block slides into a door that does not accept it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum WrongDoor {
    /// The door acts as a wall and the block rests against it (classic mode).
    #[default]
    Rest,
    /// The block bounces back to where it started: the attempt is an answer (language modes).
    Bounce,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Meta {
    pub name: String,
    /// Fewest moves, as found by the solver.
    pub optimal: Option<u16>,
    /// Challenge levels only: seconds available.
    pub timer: Option<u16>,
    pub wrong_door: WrongDoor,
    /// Free-form `key value` lines kept as is (difficulty, tags…).
    pub extra: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Level {
    pub w: i16,
    pub h: i16,
    pub cells: Vec<Cell>,
    pub doors: Vec<Door>,
    pub blocks: Vec<BlockDef>,
    /// Starting position (top-left of the bounding box) of each block.
    pub start: Vec<Pos>,
    pub meta: Meta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.msg)
    }
}

impl std::error::Error for ParseError {}

impl Level {
    pub fn in_bounds(&self, p: Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.w && p.y < self.h
    }

    pub fn cell(&self, p: Pos) -> Cell {
        if self.in_bounds(p) { self.cells[(p.y * self.w + p.x) as usize] } else { Cell::Void }
    }

    pub fn is_floor(&self, p: Pos) -> bool {
        self.cell(p) == Cell::Floor
    }

    pub fn idx(&self, p: Pos) -> usize {
        (p.y * self.w + p.x) as usize
    }

    /// The door on `side` of floor cell `p`, if any.
    pub fn door_at(&self, p: Pos, side: Dir) -> Option<usize> {
        self.doors.iter().position(|d| d.covers(p, side))
    }

    /// Checks that the level is well formed; returns human-readable problems.
    pub fn validate(&self) -> Vec<String> {
        let mut errs = Vec::new();
        if self.start.len() != self.blocks.len() {
            errs.push("start positions and blocks differ in number".into());
        }
        let mut occ = vec![usize::MAX; (self.w * self.h) as usize];
        for (i, (b, &p)) in self.blocks.iter().zip(&self.start).enumerate() {
            if b.shape.cells.is_empty() {
                errs.push(format!("block {} has no cell", b.name));
            }
            for &c in &b.shape.cells {
                let q = p.add(c);
                if !self.is_floor(q) {
                    errs.push(format!("block {} is off the floor at {},{}", b.name, q.x, q.y));
                    continue;
                }
                let k = self.idx(q);
                if occ[k] != usize::MAX {
                    errs.push(format!("blocks {} and {} overlap", self.blocks[occ[k]].name, b.name));
                }
                occ[k] = i;
            }
        }
        for d in &self.doors {
            if d.len == 0 {
                errs.push("door of length 0".into());
            }
            for c in d.cells() {
                if !self.is_floor(c) {
                    errs.push(format!("door {:?} at {},{}: cell is not floor", d.side, c.x, c.y));
                } else if self.is_floor(c.step(d.side, 1)) {
                    errs.push(format!("door {:?} at {},{}: not on a wall edge", d.side, c.x, c.y));
                }
            }
        }
        for (i, d) in self.doors.iter().enumerate() {
            for e in &self.doors[i + 1..] {
                if d.side == e.side && d.cells().any(|c| e.covers(c, d.side)) {
                    errs.push("two doors overlap".into());
                }
            }
        }
        // Every key needed must have a door wide enough somewhere.
        for b in &self.blocks {
            for k in b.shell.iter().chain(std::iter::once(&b.key)) {
                let fits = self.doors.iter().any(|d| d.key == *k && d.len as i16 >= b.shape.width_across(d.side));
                if !fits {
                    errs.push(format!("block {} has no door of key {} wide enough", b.name, key_name(*k)));
                }
            }
            if let Some(l) = b.lock
                && !self.blocks.iter().any(|o| o.carries == Some(l))
            {
                errs.push(format!("block {} is locked with key {l} that nobody carries", b.name));
            }
        }
        errs
    }

    pub fn parse(text: &str) -> Result<Level, ParseError> {
        let err = |line: usize, msg: &str| ParseError { line: line + 1, msg: msg.to_string() };
        #[derive(PartialEq)]
        enum Sec {
            Head,
            Map,
            Blocks,
            Doors,
            End,
        }
        let mut sec = Sec::Head;
        let mut meta = Meta::default();
        let mut rows: Vec<Vec<char>> = Vec::new();
        let mut legend: Vec<(char, Vec<String>, usize)> = Vec::new();
        let mut doors = Vec::new();
        let mut saw_magic = false;
        for (ln, raw) in text.lines().enumerate() {
            let line = raw;
            let trimmed = line.trim();
            if sec != Sec::Map && (trimmed.is_empty() || trimmed.starts_with("//")) {
                continue;
            }
            if !saw_magic {
                if !trimmed.starts_with("fuera") {
                    return Err(err(ln, "expected `fuera 1`"));
                }
                saw_magic = true;
                continue;
            }
            match trimmed {
                "map" => {
                    sec = Sec::Map;
                    continue;
                }
                "blocks" => {
                    sec = Sec::Blocks;
                    continue;
                }
                "doors" => {
                    sec = Sec::Doors;
                    continue;
                }
                "end" => {
                    sec = Sec::End;
                    continue;
                }
                _ => {}
            }
            match sec {
                Sec::Head => {
                    let (k, v) = trimmed.split_once(' ').unwrap_or((trimmed, ""));
                    let v = v.trim();
                    match k {
                        "name" => meta.name = v.to_string(),
                        "optimal" => meta.optimal = Some(v.parse().map_err(|_| err(ln, "bad optimal"))?),
                        "timer" => meta.timer = Some(v.parse().map_err(|_| err(ln, "bad timer"))?),
                        "wrong" => {
                            meta.wrong_door = match v {
                                "rest" => WrongDoor::Rest,
                                "bounce" => WrongDoor::Bounce,
                                _ => return Err(err(ln, "wrong: rest or bounce")),
                            }
                        }
                        _ => meta.extra.push((k.to_string(), v.to_string())),
                    }
                }
                Sec::Map => {
                    if trimmed.is_empty() {
                        continue;
                    }
                    let body = line.strip_prefix("  ").unwrap_or(line);
                    rows.push(body.trim_end().chars().collect());
                }
                Sec::Blocks => {
                    let mut it = trimmed.split_whitespace();
                    let name = it.next().and_then(|s| s.chars().next()).ok_or_else(|| err(ln, "block name"))?;
                    legend.push((name, it.map(|s| s.to_string()).collect(), ln));
                }
                Sec::Doors => {
                    let f: Vec<&str> = trimmed.split_whitespace().collect();
                    if f.len() < 5 {
                        return Err(err(ln, "door: side x y len key [shutter=open|closed]"));
                    }
                    let side = f[0].chars().next().and_then(Dir::from_letter).ok_or_else(|| err(ln, "door side N/E/S/W"))?;
                    let x: i16 = f[1].parse().map_err(|_| err(ln, "door x"))?;
                    let y: i16 = f[2].parse().map_err(|_| err(ln, "door y"))?;
                    let len: u8 = f[3].parse().map_err(|_| err(ln, "door len"))?;
                    let key = parse_key(f[4]).ok_or_else(|| err(ln, "door key"))?;
                    let mut kind = DoorKind::Plain;
                    for opt in &f[5..] {
                        match *opt {
                            "shutter=open" => kind = DoorKind::Shutter { open: true },
                            "shutter=closed" => kind = DoorKind::Shutter { open: false },
                            _ => return Err(err(ln, "unknown door option")),
                        }
                    }
                    doors.push(Door { side, at: Pos::new(x, y), len, key, kind });
                }
                Sec::End => {}
            }
        }
        if rows.is_empty() {
            return Err(err(0, "no map"));
        }
        let h = rows.len() as i16;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i16;
        let mut cells = vec![Cell::Void; (w * h) as usize];
        let mut block_cells: Vec<(char, Vec<Pos>)> = Vec::new();
        for (y, r) in rows.iter().enumerate() {
            for (x, &c) in r.iter().enumerate() {
                let i = y * w as usize + x;
                cells[i] = match c {
                    '~' | ' ' => Cell::Void,
                    '#' => Cell::Wall,
                    '.' => Cell::Floor,
                    c if c.is_ascii_alphanumeric() => {
                        let p = Pos::new(x as i16, y as i16);
                        match block_cells.iter_mut().find(|(n, _)| *n == c) {
                            Some((_, v)) => v.push(p),
                            None => block_cells.push((c, vec![p])),
                        }
                        Cell::Floor
                    }
                    _ => return Err(err(0, &format!("unknown map character {c:?}"))),
                };
            }
        }
        let mut blocks = Vec::new();
        let mut start = Vec::new();
        // Blocks in legend order (stable indices), then any block of the map missing from the legend is an error.
        for (name, opts, ln) in &legend {
            let Some((_, ps)) = block_cells.iter().find(|(n, _)| n == name) else {
                return Err(err(*ln, &format!("block {name} is not on the map")));
            };
            let minx = ps.iter().map(|p| p.x).min().unwrap_or(0);
            let miny = ps.iter().map(|p| p.y).min().unwrap_or(0);
            let shape = Shape::new(ps.clone());
            let key = opts.first().and_then(|k| parse_key(k)).ok_or_else(|| err(*ln, "block key"))?;
            let mut b = BlockDef::plain(*name, shape, key);
            for o in &opts[1..] {
                let (k, v) = o.split_once('=').ok_or_else(|| err(*ln, "option k=v"))?;
                match k {
                    "lane" => {
                        b.axis = Some(match v {
                            "h" => Axis::H,
                            "v" => Axis::V,
                            _ => return Err(err(*ln, "lane=h|v")),
                        })
                    }
                    "ice" => b.ice = v.parse().map_err(|_| err(*ln, "ice=n"))?,
                    "shell" => b.shell = Some(parse_key(v).ok_or_else(|| err(*ln, "shell key"))?),
                    "key" => b.carries = Some(v.parse().map_err(|_| err(*ln, "key=n"))?),
                    "lock" => b.lock = Some(v.parse().map_err(|_| err(*ln, "lock=n"))?),
                    _ => return Err(err(*ln, "unknown block option")),
                }
            }
            blocks.push(b);
            start.push(Pos::new(minx, miny));
        }
        if let Some((n, _)) = block_cells.iter().find(|(n, _)| !legend.iter().any(|(l, _, _)| l == n)) {
            return Err(err(0, &format!("block {n} is on the map but not in `blocks`")));
        }
        Ok(Level { w, h, cells, doors, blocks, start, meta })
    }

    pub fn to_text(&self) -> String {
        let mut s = String::from("fuera 1\n");
        if !self.meta.name.is_empty() {
            let _ = writeln!(s, "name {}", self.meta.name);
        }
        if let Some(o) = self.meta.optimal {
            let _ = writeln!(s, "optimal {o}");
        }
        if let Some(t) = self.meta.timer {
            let _ = writeln!(s, "timer {t}");
        }
        if self.meta.wrong_door == WrongDoor::Bounce {
            s.push_str("wrong bounce\n");
        }
        for (k, v) in &self.meta.extra {
            let _ = writeln!(s, "{k} {v}");
        }
        s.push_str("map\n");
        let mut grid: Vec<Vec<char>> = (0..self.h)
            .map(|y| {
                (0..self.w)
                    .map(|x| match self.cell(Pos::new(x, y)) {
                        Cell::Void => '~',
                        Cell::Floor => '.',
                        Cell::Wall => '#',
                    })
                    .collect()
            })
            .collect();
        for (b, p) in self.blocks.iter().zip(&self.start) {
            for c in &b.shape.cells {
                let q = p.add(*c);
                if self.in_bounds(q) {
                    grid[q.y as usize][q.x as usize] = b.name;
                }
            }
        }
        for r in grid {
            let line: String = r.into_iter().collect();
            let _ = writeln!(s, "  {}", line.trim_end_matches('~'));
        }
        s.push_str("blocks\n");
        for b in &self.blocks {
            let _ = write!(s, "  {} {}", b.name, key_name(b.key));
            if let Some(a) = b.axis {
                let _ = write!(s, " lane={}", if a == Axis::H { "h" } else { "v" });
            }
            if b.ice > 0 {
                let _ = write!(s, " ice={}", b.ice);
            }
            if let Some(k) = b.shell {
                let _ = write!(s, " shell={}", key_name(k));
            }
            if let Some(k) = b.carries {
                let _ = write!(s, " key={k}");
            }
            if let Some(k) = b.lock {
                let _ = write!(s, " lock={k}");
            }
            s.push('\n');
        }
        s.push_str("doors\n");
        for d in &self.doors {
            let _ = write!(s, "  {} {} {} {} {}", d.side.letter(), d.at.x, d.at.y, d.len, key_name(d.key));
            if let DoorKind::Shutter { open } = d.kind {
                let _ = write!(s, " shutter={}", if open { "open" } else { "closed" });
            }
            s.push('\n');
        }
        s.push_str("end\n");
        s
    }

    /// Block names available for new blocks, in the order they are handed out.
    pub const NAMES: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
}

fn parse_key(s: &str) -> Option<Key> {
    color_key(s).or_else(|| s.strip_prefix('k').and_then(|n| n.parse().ok()))
}
