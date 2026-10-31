//! A level being played: current state, unlimited undo, counters.

use crate::level::Level;
use crate::rules::{self, Move, Outcome, Result, State};

#[derive(Clone, Debug)]
pub struct Game {
    pub level: Level,
    pub state: State,
    history: Vec<State>,
    /// Moves that changed the board (undone moves are removed).
    pub moves: u32,
    pub hints: u32,
    pub undos: u32,
    /// Wrong-door attempts (language levels).
    pub mistakes: u32,
    occ: Vec<u8>,
}

impl Game {
    pub fn new(level: Level) -> Game {
        let state = State::initial(&level);
        let occ = rules::occupancy(&level, &state);
        Game { level, state, history: Vec::new(), moves: 0, hints: 0, undos: 0, mistakes: 0, occ }
    }

    pub fn occupancy(&self) -> &[u8] {
        &self.occ
    }

    /// Would this move do anything? (Does not play it.)
    pub fn preview(&self, mv: Move) -> Option<(State, Outcome)> {
        rules::play(&self.level, &self.state, &self.occ, mv)
    }

    pub fn play(&mut self, mv: Move) -> Option<Outcome> {
        let (next, out) = rules::play(&self.level, &self.state, &self.occ, mv)?;
        if let Result::Bounce { .. } = out.result {
            self.mistakes += 1;
            return Some(out);
        }
        self.history.push(std::mem::replace(&mut self.state, next));
        self.occ = rules::occupancy(&self.level, &self.state);
        self.moves += 1;
        Some(out)
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        match self.history.pop() {
            Some(s) => {
                self.state = s;
                self.occ = rules::occupancy(&self.level, &self.state);
                self.moves = self.moves.saturating_sub(1);
                self.undos += 1;
                true
            }
            None => false,
        }
    }

    pub fn restart(&mut self) {
        self.state = State::initial(&self.level);
        self.occ = rules::occupancy(&self.level, &self.state);
        self.history.clear();
        self.moves = 0;
    }

    pub fn is_won(&self) -> bool {
        self.state.is_won()
    }

    /// No move changes anything: only undo or restart can help.
    pub fn is_stuck(&self) -> bool {
        !self.is_won() && rules::legal_moves(&self.level, &self.state, &self.occ).is_empty()
    }

    /// Stars: 3 at the solver's best (without hints), 2 within 50 % more moves, else 1.
    pub fn stars(&self) -> u8 {
        let Some(opt) = self.level.meta.optimal else { return 3 };
        let opt = opt as u32;
        if self.moves <= opt && self.hints == 0 {
            3
        } else if self.moves * 2 <= opt * 3 {
            2
        } else {
            1
        }
    }
}
