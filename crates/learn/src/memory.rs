//! What the learner knows: one FSRS card per item, the "Mi cuaderno" list of misses, the placement result and the
//! daily streak. Stored as `key=value` pairs by the app's save file.

use std::collections::HashMap;

use crate::content::Band;
use crate::fsrs::{Card, Fsrs, Rating};

#[derive(Clone, Debug, Default)]
pub struct Memory {
    pub cards: HashMap<String, Card>,
    /// Missed items, most recent first.
    pub misses: Vec<String>,
    /// Band from the placement test (words below it start as known).
    pub placed: Option<Band>,
    /// Last day a session was completed, and the streak length.
    pub last_day: i64,
    pub streak: u32,
    /// Rounds played today (day, count), for the daily goal.
    pub today: (i64, u32),
    fsrs: Fsrs,
}

/// Item ids may contain spaces and punctuation; keys of the save file are `m.` + id with `=` escaped.
fn key_of(id: &str) -> String {
    format!("m.{}", id.replace('=', "%3D"))
}

impl Memory {
    pub fn from_pairs<'a>(pairs: impl Iterator<Item = (&'a str, &'a str)>) -> Memory {
        let mut m = Memory::default();
        for (k, v) in pairs {
            if let Some(id) = k.strip_prefix("m.") {
                if let Some(c) = Card::from_compact(v) {
                    m.cards.insert(id.replace("%3D", "="), c);
                }
            } else if k == "l.miss" {
                m.misses = v.split('\u{1f}').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
            } else if k == "l.placed" {
                m.placed = Some(Band::parse(v));
            } else if k == "l.streak" {
                let mut it = v.split(',');
                m.last_day = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
                m.streak = it.next().and_then(|x| x.parse().ok()).unwrap_or(0);
            } else if k == "l.today" {
                let mut it = v.split(',');
                m.today = (it.next().and_then(|x| x.parse().ok()).unwrap_or(0), it.next().and_then(|x| x.parse().ok()).unwrap_or(0));
            }
        }
        m
    }

    /// All pairs to save (cards and settings).
    pub fn to_pairs(&self) -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = self.cards.iter().map(|(id, c)| (key_of(id), c.to_compact())).collect();
        v.push(("l.miss".into(), self.misses.join("\u{1f}")));
        if let Some(b) = self.placed {
            v.push(("l.placed".into(), b.name().into()));
        }
        v.push(("l.streak".into(), format!("{},{}", self.last_day, self.streak)));
        v.push(("l.today".into(), format!("{},{}", self.today.0, self.today.1)));
        v
    }

    pub fn card(&self, id: &str) -> Option<&Card> {
        self.cards.get(id)
    }

    pub fn rate(&mut self, id: &str, r: Rating, now: f64) {
        let c = match self.cards.get(id) {
            Some(c) => self.fsrs.review(c, r, now),
            None => self.fsrs.first(r, now),
        };
        self.cards.insert(id.to_string(), c);
        if r == Rating::Again {
            self.misses.retain(|m| m != id);
            self.misses.insert(0, id.to_string());
            self.misses.truncate(200);
        }
    }

    /// Marks an item as already known (placement), due again in `days`.
    pub fn mark_known(&mut self, id: &str, now: f64, days: f64) {
        if self.cards.contains_key(id) {
            return;
        }
        let mut c = self.fsrs.first(Rating::Easy, now);
        c.stability = days as f32;
        c.due = now + days;
        self.cards.insert(id.to_string(), c);
    }

    /// Items due now (`prefix` filters the kind, e.g. `w:`), most overdue first.
    pub fn due(&self, prefix: &str, now: f64) -> Vec<String> {
        let mut v: Vec<(&String, f32)> = self.cards.iter().filter(|(id, c)| id.starts_with(prefix) && c.due <= now).map(|(id, c)| (id, self.fsrs.retrievability(c, now))).collect();
        v.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(b.0)));
        v.into_iter().map(|(id, _)| id.clone()).collect()
    }

    pub fn retrievability(&self, id: &str, now: f64) -> Option<f32> {
        self.cards.get(id).map(|c| self.fsrs.retrievability(c, now))
    }

    /// Counts for the progress screen: (seen, learning, solid) among items starting with `prefix`.
    pub fn counts(&self, prefix: &str) -> (usize, usize, usize) {
        let mut seen = 0;
        let mut solid = 0;
        for (id, c) in &self.cards {
            if id.starts_with(prefix) {
                seen += 1;
                if c.stability >= 21.0 {
                    solid += 1;
                }
            }
        }
        (seen, seen - solid, solid)
    }

    /// Records a finished round; returns the rounds done today.
    pub fn round_done(&mut self, day: i64) -> u32 {
        if self.today.0 != day {
            self.today = (day, 0);
        }
        self.today.1 += 1;
        self.today.1
    }

    /// Updates the streak when the daily goal is reached. One missed day is forgiven (repairable streak).
    pub fn goal_reached(&mut self, day: i64) {
        if self.last_day == day {
            return;
        }
        let gap = day - self.last_day;
        self.streak = if gap == 1 || gap == 2 && self.streak > 0 { self.streak + 1 } else { 1 };
        self.last_day = day;
    }

    /// Streak as shown today (0 if it has lapsed).
    pub fn current_streak(&self, day: i64) -> u32 {
        if day - self.last_day <= 2 { self.streak } else { 0 }
    }
}
