//! Pure learning logic for Fuera, with no I/O and no dependencies.
//!
//! * [`conj`]: Spanish (Spain) verb conjugator for the seven simple tenses
//!   drilled by learners, including pronominal verbs and irregular compounds.
//! * [`conj_fr`]: French verb conjugator (six tenses, passé composé with its agreement, pronominal verbs).
//! * [`content`], [`memory`], [`plan`]: the content of each course (Spanish of Spain or Latin America, French), what
//!   the learner knows, and how a round is chosen.
//! * [`fsrs`]: the FSRS-6 spaced-repetition scheduler with its default
//!   parameters, plus a compact text form for card state.

pub mod conj;
pub mod conj_fr;
pub mod fsrs;
pub mod content;
pub mod memory;
pub mod plan;
pub mod voice;
