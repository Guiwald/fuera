//! FSRS-6 spaced-repetition scheduler (open-spaced-repetition).
//!
//! Implements the FSRS-6 memory model as in py-fsrs 6 / fsrs-rs / the
//! fsrs4anki scheduler, with the 21 default parameters:
//!
//! * forgetting curve `R(t, S) = (1 + F·t/S)^C` with `C = −w20` and
//!   `F = 0.9^(1/C) − 1`, so that `R(S, S) = 0.9`;
//! * interval `I = S/F · (r^(1/C) − 1)` for the desired retention `r`;
//! * initial stability `S0(G) = w[G−1]`, initial difficulty
//!   `D0(G) = w4 − e^(w5·(G−1)) + 1`;
//! * difficulty update with linear damping and mean reversion to `D0(Easy)`;
//! * recall stability `S·(1 + e^w8·(11−D)·S^−w9·(e^(w10·(1−R))−1)·hard·easy)`
//!   (`hard = w15`, `easy = w16`);
//! * lapse stability `min(w11·D^−w12·((S+1)^w13−1)·e^(w14·(1−R)), S/e^(w17·w18))`;
//! * same-day (short-term) stability `S·e^(w17·(G−3+w18))·S^−w19`, never
//!   decreasing for Good/Easy.
//!
//! Times are in days (`f64`, fractional allowed); a review less than one day
//! after the previous one uses the short-term formula. There is no learning
//! state machine: *Again* schedules the card [`RELEARN_STEP`] later (so it comes
//! back in the same session) and the other ratings schedule it
//! [`Fsrs::interval_days`] later.

/// FSRS-6 default parameters `w0..w20`.
pub const DEFAULT_PARAMETERS: [f32; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835,
    0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

/// Delay before a failed card is shown again: 10 minutes, in days.
pub const RELEARN_STEP: f64 = 10.0 / 1440.0;

/// Longest interval ever scheduled, in days.
pub const MAX_INTERVAL: f64 = 36500.0;

const S_MIN: f32 = 0.001;
const S_MAX: f32 = 36500.0;
const D_MIN: f32 = 1.0;
const D_MAX: f32 = 10.0;

/// Answer grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    fn grade(self) -> f32 {
        self as u8 as f32
    }
}

/// Memory state of one card. `last` and `due` are absolute times in days.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Card {
    pub stability: f32,
    pub difficulty: f32,
    pub last: f64,
    pub due: f64,
    pub reps: u16,
    pub lapses: u16,
}

impl Card {
    /// `stability,difficulty,last,due,reps,lapses`, numbers with at most
    /// four decimals (trailing zeros trimmed), e.g. `2.3065,2.1181,20000,20002,1,0`.
    pub fn to_compact(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            num(f64::from(self.stability)),
            num(f64::from(self.difficulty)),
            num(self.last),
            num(self.due),
            self.reps,
            self.lapses
        )
    }

    /// Parses [`Card::to_compact`] output; `None` if malformed.
    pub fn from_compact(s: &str) -> Option<Card> {
        let mut it = s.trim().split(',').map(str::trim);
        let stability: f32 = it.next()?.parse().ok()?;
        let difficulty: f32 = it.next()?.parse().ok()?;
        let last: f64 = it.next()?.parse().ok()?;
        let due: f64 = it.next()?.parse().ok()?;
        let reps: u16 = it.next()?.parse().ok()?;
        let lapses: u16 = it.next()?.parse().ok()?;
        let finite =
            stability.is_finite() && difficulty.is_finite() && last.is_finite() && due.is_finite();
        (it.next().is_none() && finite).then_some(Card {
            stability,
            difficulty,
            last,
            due,
            reps,
            lapses,
        })
    }
}

fn num(x: f64) -> String {
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

/// FSRS-6 scheduler: parameters and desired retention.
#[derive(Clone, Debug, PartialEq)]
pub struct Fsrs {
    pub w: [f32; 21],
    pub retention: f32,
}

impl Default for Fsrs {
    fn default() -> Self {
        Fsrs {
            w: DEFAULT_PARAMETERS,
            retention: 0.9,
        }
    }
}

impl Fsrs {
    fn decay(&self) -> f32 {
        -self.w[20]
    }

    fn factor(&self) -> f32 {
        0.9f32.powf(1.0 / self.decay()) - 1.0
    }

    /// First review of a new card at time `now`.
    pub fn first(&self, r: Rating, now: f64) -> Card {
        let stability = self.w[r as usize - 1].clamp(S_MIN, S_MAX);
        let difficulty = self.init_difficulty(r).clamp(D_MIN, D_MAX);
        Card {
            stability,
            difficulty,
            last: now,
            due: self.due(r, stability, now),
            reps: 1,
            lapses: 0,
        }
    }

    /// Review of `c` at time `now`. Reviews less than a day after the previous
    /// one use the FSRS-6 short-term formula (w17..w19). A lapse is counted when
    /// a card that had an interval of at least a day is failed.
    pub fn review(&self, c: &Card, r: Rating, now: f64) -> Card {
        let s = c.stability.clamp(S_MIN, S_MAX);
        let d = c.difficulty.clamp(D_MIN, D_MAX);
        let elapsed = (now - c.last).max(0.0);
        let stability = if elapsed < 1.0 {
            self.short_term_stability(s, r)
        } else {
            let ret = self.forgetting_curve(elapsed, s);
            match r {
                Rating::Again => self.forget_stability(d, s, ret),
                _ => self.recall_stability(d, s, ret, r),
            }
        }
        .clamp(S_MIN, S_MAX);
        let lapse = r == Rating::Again && c.due - c.last >= 1.0;
        Card {
            stability,
            difficulty: self.next_difficulty(d, r),
            last: now,
            due: self.due(r, stability, now),
            reps: c.reps.saturating_add(1),
            lapses: c.lapses.saturating_add(u16::from(lapse)),
        }
    }

    /// Probability of recall of `c` at time `now`.
    pub fn retrievability(&self, c: &Card, now: f64) -> f32 {
        self.forgetting_curve((now - c.last).max(0.0), c.stability.clamp(S_MIN, S_MAX))
    }

    /// Interval in whole days after which recall probability falls to the
    /// desired retention; at least 1 day, at most [`MAX_INTERVAL`].
    pub fn interval_days(&self, stability: f32) -> f64 {
        let s = f64::from(stability.clamp(S_MIN, S_MAX));
        let decay = f64::from(self.decay());
        let factor = f64::from(self.factor());
        let retention = f64::from(self.retention.clamp(0.01, 0.99));
        let raw = s / factor * (retention.powf(1.0 / decay) - 1.0);
        raw.round().clamp(1.0, MAX_INTERVAL)
    }

    fn due(&self, r: Rating, stability: f32, now: f64) -> f64 {
        match r {
            Rating::Again => now + RELEARN_STEP,
            _ => now + self.interval_days(stability),
        }
    }

    fn forgetting_curve(&self, elapsed: f64, stability: f32) -> f32 {
        let t = elapsed as f32;
        (1.0 + self.factor() * t / stability).powf(self.decay())
    }

    fn init_difficulty(&self, r: Rating) -> f32 {
        self.w[4] - (self.w[5] * (r.grade() - 1.0)).exp() + 1.0
    }

    fn next_difficulty(&self, d: f32, r: Rating) -> f32 {
        let delta = -self.w[6] * (r.grade() - 3.0);
        let damped = d + delta * (10.0 - d) / 9.0;
        let reverted = self.w[7] * self.init_difficulty(Rating::Easy) + (1.0 - self.w[7]) * damped;
        reverted.clamp(D_MIN, D_MAX)
    }

    fn recall_stability(&self, d: f32, s: f32, ret: f32, r: Rating) -> f32 {
        let w = &self.w;
        let hard_penalty = if r == Rating::Hard { w[15] } else { 1.0 };
        let easy_bonus = if r == Rating::Easy { w[16] } else { 1.0 };
        s * (1.0
            + w[8].exp()
                * (11.0 - d)
                * s.powf(-w[9])
                * ((w[10] * (1.0 - ret)).exp() - 1.0)
                * hard_penalty
                * easy_bonus)
    }

    fn forget_stability(&self, d: f32, s: f32, ret: f32) -> f32 {
        let w = &self.w;
        let long_term =
            w[11] * d.powf(-w[12]) * ((s + 1.0).powf(w[13]) - 1.0) * (w[14] * (1.0 - ret)).exp();
        let short_term_cap = s / (w[17] * w[18]).exp();
        long_term.min(short_term_cap)
    }

    fn short_term_stability(&self, s: f32, r: Rating) -> f32 {
        let w = &self.w;
        let mut sinc = (w[17] * (r.grade() - 3.0 + w[18])).exp() * s.powf(-w[19]);
        if matches!(r, Rating::Good | Rating::Easy) {
            sinc = sinc.max(1.0);
        }
        s * sinc
    }
}
