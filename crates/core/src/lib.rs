//! ¡Fuera! — game rules, level format, solver and level generator.
//!
//! The crate has no dependency and draws nothing: the same code runs in the game (all platforms), in the level
//! tools (`fuera-cli`) and in the tests.

pub mod game;
pub mod geom;
pub mod level;
pub mod rules;
pub mod solver;

pub use game::Game;
pub use geom::{Axis, Dir, Pos};
pub use level::{BlockDef, Cell, Door, DoorKind, Key, Level, Meta, Shape, WrongDoor};
pub use rules::{Move, Outcome, State};

#[cfg(test)]
mod tests;
