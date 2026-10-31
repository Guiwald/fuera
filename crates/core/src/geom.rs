//! Directions, axes and grid positions.

/// One of the four slide directions. Also used for the side of a cell a door sits on
/// (a door on side `E` lets blocks leave by moving east).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::E => (1, 0),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
        }
    }

    pub fn axis(self) -> Axis {
        match self {
            Dir::E | Dir::W => Axis::H,
            Dir::N | Dir::S => Axis::V,
        }
    }

    pub fn opposite(self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::E => Dir::W,
            Dir::S => Dir::N,
            Dir::W => Dir::E,
        }
    }

    /// Rotates a quarter turn clockwise.
    pub fn cw(self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::S => Dir::W,
            Dir::W => Dir::N,
        }
    }

    pub fn letter(self) -> char {
        match self {
            Dir::N => 'N',
            Dir::E => 'E',
            Dir::S => 'S',
            Dir::W => 'W',
        }
    }

    pub fn from_letter(c: char) -> Option<Dir> {
        Some(match c.to_ascii_uppercase() {
            'N' => Dir::N,
            'E' => Dir::E,
            'S' => Dir::S,
            'W' => Dir::W,
            _ => return None,
        })
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Movement axis (for blocks that may only move along one axis).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    /// East–west.
    H,
    /// North–south.
    V,
}

/// A cell coordinate on the board (x to the right, y down).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Pos {
    pub x: i16,
    pub y: i16,
}

impl Pos {
    pub const fn new(x: i16, y: i16) -> Pos {
        Pos { x, y }
    }

    pub fn step(self, d: Dir, n: i32) -> Pos {
        let (dx, dy) = d.delta();
        Pos { x: self.x + (dx * n) as i16, y: self.y + (dy * n) as i16 }
    }

    pub fn add(self, o: Pos) -> Pos {
        Pos { x: self.x + o.x, y: self.y + o.y }
    }
}
