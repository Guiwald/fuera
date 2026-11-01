//! The Spanish mode: home, new-word cards, word rounds on the board, feedback, notebook, placement test.

use std::collections::HashMap;

use fuera_core::rules::Result as MoveResult;
use fuera_core::{Dir, Game, Move};
use fuera_learn::content::{Band, Base, Content, Course, Target};
use fuera_learn::fsrs::Rating;
use fuera_learn::plan::{self, Mode, Planner, Round};
use tiny_skia::{Pixmap, Transform};

use super::{Action, App, Drag, Hit, Screen, Style};
use crate::board::{BoardView, Look, Skin};
use crate::gfx::{self, fill, solid};
use crate::i18n::{T, tr, tr_course};
use crate::icons::{self, Icon};
use crate::sfx::Sfx;
use crate::text::{Align, Face};
use crate::theme::{self, col, col_a, shade};

/// Rounds in the daily session.
pub const DAILY_GOAL: u32 = 5;
/// Placement test: number of "¿Existe?" rounds.
const PLACEMENT_ROUNDS: usize = 4;
/// Tiles per placement round (four real words, one per band, and two invented ones).
const PLACEMENT_TILES: usize = 6;
const DOOR: (u8, u8, u8) = (0xD9, 0x82, 0x5B);

pub(super) struct LearnPlay {
    pub round: Round,
    pub game: Game,
    pub view: BoardView,
    pub drag: Option<Drag>,
    /// Tile answered already (first answer counts).
    answered: Vec<bool>,
    hinted: Vec<bool>,
    pub right: u32,
    pub wrong: u32,
    /// Worst rating per item this round, applied when the round ends.
    ratings: HashMap<String, Rating>,
    pub feedback: Option<(String, f32)>,
    /// What the correction card is about (item id, or the tile's text for a distractor), for reporting it.
    pub feedback_item: Option<String>,
    pub won: Option<f32>,
    pub session: bool,
    /// Legend: (mark, text) for doors with long texts.
    pub legend: Vec<(String, String)>,
    /// (tile text, right?, item id or text) in answer order, for the summary.
    pub results: Vec<(String, bool, String)>,
}

impl App {
    /// The language of the meanings for the current course (its own setting; by default the interface language
    /// when the course offers it).
    pub(super) fn base(&self) -> Base {
        let target = self.course().target;
        let key = if target == Target::Es { "base" } else { "fr.base" };
        let bases = target.bases();
        if let Some(b) = self.save.get(key).and_then(Base::from_code).filter(|b| bases.contains(b)) {
            return b;
        }
        let ui = match self.lang {
            crate::i18n::Lang::Fr => Base::Fr,
            crate::i18n::Lang::En => Base::En,
            crate::i18n::Lang::Es => Base::Es,
        };
        if bases.contains(&ui) { ui } else { Base::En }
    }

    fn content(&mut self) -> &Content {
        let course = self.course();
        if self.content.as_ref().is_none_or(|c| c.course != course) {
            self.content = Some(Content::load(course));
        }
        self.content.as_ref().unwrap()
    }

    pub(super) fn save_memory(&mut self) {
        let p = super::mem_prefix(self.memory_of);
        for (k, v) in self.memory.to_pairs() {
            self.save.set(&format!("{p}{k}"), v);
        }
        self.persist();
    }

    /// Switches course: the progress of the one left is saved, the other one's loaded.
    fn set_course(&mut self, course: Course) {
        self.save_memory();
        self.save.set("course", course.target.code());
        if course.target == Target::Es {
            self.save.set("es_variant", if course.latam { "419" } else { "es" });
        }
        if self.memory_of != course.target {
            self.memory = super::load_memory(&self.save, course.target);
            self.memory_of = course.target;
        }
        self.placement.clear();
        self.persist();
    }

    fn band(&self) -> Band {
        self.memory.placed.unwrap_or(Band::A1)
    }

    /// Next mode of the daily session: words and grammar alternate, grammar topics rotate.
    fn session_mode(&self) -> Mode {
        let day = self.host.today();
        let done = if self.memory.today.0 == day { self.memory.today.1 } else { 0 };
        let band = self.band();
        let target = self.course().target;
        let grammar: Vec<Mode> = Mode::grammar(target).iter().copied().filter(|m| m.min_band(target) <= band).collect();
        let g = |k: i64| grammar[((day + k).rem_euclid(grammar.len() as i64)) as usize];
        match done % 5 {
            0 => Mode::Words,
            1 => g(0),
            2 => Mode::Words,
            3 => {
                if self.memory.cards.keys().filter(|k| k.starts_with("w:")).count() >= 8 {
                    Mode::Listening
                } else {
                    Mode::Words
                }
            }
            _ => g(3),
        }
    }

    pub(super) fn start_round(&mut self, mode: Mode, session: bool) {
        let now = self.host.now_days();
        let base = self.base();
        let latam = self.latam();
        let band = if mode == Mode::Placement { Band::B2 } else { self.band() };
        let seed = (now * 86_400.0) as u64 ^ (self.time * 1000.0) as u64;
        self.content();
        let content = self.content.as_ref().unwrap();
        let round = {
            let mut p = Planner::new(content, &self.memory, base, band, now, seed).with_latam(latam);
            p.plan(mode)
        };
        let Some(mut round) = round else { return };
        // Boards are six cells wide (see `word_board`): with two categories, two- or three-cell tiles; with three or
        // four doors (in the corners), two-cell ones. All tiles of a round are as wide as the doors.
        let wide = if round.gates.len() == 2 { round.tiles.iter().map(|t| t.width).max().unwrap_or(2).clamp(2, 3) } else { 2 };
        for t in &mut round.tiles {
            t.width = wide;
        }
        let tiles: Vec<(i16, u16)> = round.tiles.iter().map(|t| (t.width as i16, t.key)).collect();
        let doors: Vec<u16> = round.gates.iter().map(|g| g.key).collect();
        // How much of a puzzle: grows with the rounds played (none at first).
        let slack = if mode == Mode::Placement { (0, 0) } else { self.memory.board_slack() };
        let level = fuera_core::generator::word_board(&tiles, &doors, seed, 40, slack);
        let Some(mut level) = level else {
            log::warn!("no board for {mode:?}");
            return;
        };
        level.meta.name = mode.name(self.course().target).to_string();
        // Looks: pastel tiles (colour unrelated to the answer), terracotta doors with signs.
        let mut legend = Vec::new();
        let marks = ["A", "B", "C", "D", "E"];
        let mut gate_look: HashMap<u16, Look> = HashMap::new();
        for g in &round.gates {
            if gate_look.contains_key(&g.key) {
                continue;
            }
            let mut look = Look::text(DOOR, &g.text);
            if g.long {
                let m = marks[legend.len().min(marks.len() - 1)].to_string();
                legend.push((m.clone(), g.text.clone()));
                look.label = None;
                look.mark = Some(m);
            }
            gate_look.insert(g.key, look);
        }
        let blocks: Vec<Look> = round
            .tiles
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let mut l = Look::text(shade(theme::KEY_COLORS[(i * 3 + seed as usize) % 8], 0.62), &t.text);
                l.speaker = t.hidden;
                l
            })
            .collect();
        let skin = Skin {
            shells: vec![None; blocks.len()],
            blocks,
            doors: level.doors.iter().map(|d| gate_look.get(&d.key).cloned().unwrap_or_else(|| Look::text(DOOR, "?"))).collect(),
        };
        let game = Game::new(level);
        let view = BoardView::new(&game, skin);
        let n = round.tiles.len();
        let has_intro = !round.intro.is_empty();
        let lp = LearnPlay {
            round,
            game,
            view,
            drag: None,
            answered: vec![false; n],
            hinted: vec![false; n],
            right: 0,
            wrong: 0,
            ratings: HashMap::new(),
            feedback: None,
            feedback_item: None,
            won: None,
            session,
            legend,
            results: Vec::new(),
        };
        self.screen = if has_intro { Screen::LearnIntro(Box::new(lp)) } else { Screen::LearnPlay(Box::new(lp)) };
        self.dirty = true;
    }

    pub(super) fn learn_move(&mut self, mv: Move) {
        let Screen::LearnPlay(p) = &mut self.screen else { return };
        if p.won.is_some() {
            return;
        }
        let before = p.game.state.clone();
        let Some(out) = p.game.play(mv) else {
            self.sounds.push(Sfx::Nope);
            return;
        };
        p.view.animate(&p.game, &before, mv, out);
        let i = mv.block as usize;
        let tile = p.round.tiles[i].clone();
        let mut say = None;
        match out.result {
            MoveResult::Exit { .. } => {
                if !p.answered[i] {
                    p.answered[i] = true;
                    p.right += 1;
                    p.results.push((tile.text.clone(), true, tile.item.clone().unwrap_or_else(|| format!("t:{}", tile.text))));
                    if let Some(item) = &tile.item {
                        let r = if p.hinted[i] { Rating::Hard } else { Rating::Good };
                        let e = p.ratings.entry(item.clone()).or_insert(r);
                        if (r as u8) < (*e as u8) {
                            *e = r;
                        }
                    }
                    if p.round.mode == Mode::Placement {
                        self.placement.push((tile.band, true));
                    }
                }
                say = tile.say.clone();
                self.sounds.push(Sfx::Right);
                p.feedback = None;
            }
            MoveResult::Bounce { door } => {
                if !p.answered[i] {
                    p.answered[i] = true;
                    p.wrong += 1;
                    p.results.push((tile.text.clone(), false, tile.item.clone().unwrap_or_else(|| format!("t:{}", tile.text))));
                    let gate_item = p.round.gates.iter().find(|g| g.key == p.game.level.doors[door].key).and_then(|g| g.item.clone());
                    if let Some(item) = tile.item.clone().or(gate_item) {
                        p.ratings.insert(item, Rating::Again);
                    }
                    if p.round.mode == Mode::Placement {
                        self.placement.push((tile.band, false));
                    }
                }
                p.feedback = Some((tile.explain.clone(), 5.0));
                p.feedback_item = Some(tile.item.clone().unwrap_or_else(|| format!("t:{}", tile.text)));
                say = tile.say.clone();
            }
            _ => {}
        }
        if let Some(s) = say {
            self.say(&s);
        }
    }

    /// Called when the last tile has left.
    fn finish_round(&mut self) {
        let now = self.host.now_days();
        let day = self.host.today();
        let Screen::LearnPlay(p) = &mut self.screen else { return };
        let ratings: Vec<(String, Rating)> = p.ratings.drain().collect();
        let session = p.session;
        let mode = p.round.mode;
        for (item, r) in ratings {
            self.memory.rate(&item, r, now);
        }
        if mode != Mode::Placement {
            let done = self.memory.round_done(day);
            if session && done >= DAILY_GOAL {
                self.memory.goal_reached(day);
            }
        }
        self.save_memory();
    }

    fn finish_placement(&mut self) {
        let band = plan::place(&self.placement);
        let now = self.host.now_days();
        self.memory.placed = Some(band);
        self.content();
        let ids: Vec<String> = self.content.as_ref().unwrap().words.iter().filter(|w| w.band < band).map(|w| w.id()).collect();
        for (k, id) in ids.iter().enumerate() {
            // Spread their next appearance over the coming weeks.
            let days = 14.0 + (k % 45) as f64;
            self.memory.mark_known(id, now, days);
        }
        self.placement.clear();
        self.save_memory();
        self.screen = Screen::Placed(band);
    }

    pub(super) fn learn_act(&mut self, a: Action) {
        match a {
            Action::LearnSession => {
                let m = self.session_mode();
                self.start_round(m, true);
            }
            Action::LearnMode(m) => self.start_round(m, false),
            Action::Placement => {
                self.placement.clear();
                self.start_round(Mode::Placement, false);
            }
            Action::LearnGo => {
                if let Screen::LearnIntro(_) = &self.screen {
                    let s = std::mem::replace(&mut self.screen, Screen::Home);
                    if let Screen::LearnIntro(p) = s {
                        self.screen = Screen::LearnPlay(p);
                    }
                }
            }
            Action::LearnNext => {
                let (session, mode) = match &self.screen {
                    Screen::LearnPlay(p) => (p.session, p.round.mode),
                    _ => (false, Mode::Words),
                };
                if mode == Mode::Placement {
                    let rounds = self.placement.len() / PLACEMENT_TILES;
                    if rounds >= PLACEMENT_ROUNDS {
                        self.finish_placement();
                    } else {
                        self.start_round(Mode::Placement, false);
                    }
                } else if session {
                    let day = self.host.today();
                    if self.memory.today.0 == day && self.memory.today.1 >= DAILY_GOAL && self.memory.today.1 % DAILY_GOAL == 0 {
                        self.screen = Screen::LearnHome;
                    } else {
                        let m = self.session_mode();
                        self.start_round(m, true);
                    }
                } else {
                    self.start_round(mode, false);
                }
            }
            Action::Notebook => self.screen = Screen::Notebook { scroll: 0.0, drag: None },
            Action::ToggleBase => {
                let target = self.course().target;
                let bases = target.bases();
                let next = if self.base() == bases[0] { bases[1] } else { bases[0] };
                self.save.set(if target == Target::Es { "base" } else { "fr.base" }, next.code());
                self.persist();
            }
            Action::Courses => self.screen = Screen::Courses,
            Action::PickCourse(c) => {
                self.set_course(c);
                self.screen = Screen::LearnHome;
            }
            Action::ToggleVariant => {
                let v = if self.latam() { "es" } else { "419" };
                self.save.set("es_variant", v);
                self.persist();
            }
            Action::Feedback => self.send_feedback(),
            Action::FlagFeedback => {
                if let Screen::LearnPlay(p) = &self.screen
                    && let (Some(item), Some((text, _))) = (p.feedback_item.clone(), p.feedback.clone())
                {
                    self.flagging = Some((item, text));
                }
            }
            Action::FlagResult(i) => {
                if let Screen::LearnPlay(p) = &self.screen
                    && let Some((text, _, item)) = p.results.get(i).cloned()
                {
                    self.flagging = Some((item, text));
                }
            }
            Action::FlagMiss(i) => {
                let base = self.base();
                let latam = self.latam();
                if let Some(id) = self.memory.misses.get(i).cloned() {
                    self.content();
                    let text = plan::describe(self.content.as_ref().unwrap(), &id, base, latam).map(|d| d.0).unwrap_or_else(|| id.clone());
                    self.flagging = Some((id, text));
                }
            }
            Action::FlagReason(r) => {
                if let Some((item, text)) = self.flagging.take() {
                    let reason = ["voice", "meaning", "other"][r.min(2) as usize];
                    let course = self.course();
                    // Fields separated by U+001E (item ids contain `|`), entries by U+001F.
                    let day = self.host.today();
                    let entry = format!("{}\u{1e}{}\u{1e}{}\u{1e}{}\u{1e}{day}", course.voice_pack(), item, reason, text.replace(['\u{1e}', '\u{1f}', '\n'], " "));
                    let mut flags: Vec<String> = self.save.get("fb.flags").unwrap_or("").split('\u{1f}').filter(|x| !x.is_empty()).map(|x| x.to_string()).collect();
                    flags.retain(|f| f.split('\u{1e}').nth(1) != Some(item.as_str()));
                    flags.push(entry);
                    // The 60 most recent reports.
                    let n = flags.len();
                    if n > 60 {
                        flags.drain(..n - 60);
                    }
                    self.save.set("fb.flags", flags.join("\u{1f}"));
                    self.persist();
                    self.toast = Some((tr(self.lang, T::Reported).to_string(), 3.0));
                }
            }
            Action::FlagCancel => self.flagging = None,
            Action::SayIntro(i) => {
                if let Screen::LearnIntro(p) = &self.screen
                    && let Some(w) = p.round.intro.get(i)
                {
                    let s = w.say.clone();
                    self.say(&s);
                }
            }
            Action::SayMiss(i) => {
                let base = self.base();
                let latam = self.latam();
                if let Some(id) = self.memory.misses.get(i).cloned() {
                    self.content();
                    if let Some((_, say)) = plan::describe(self.content.as_ref().unwrap(), &id, base, latam) {
                        self.say(&say);
                    }
                }
            }
            Action::Dismiss => {
                if let Screen::LearnPlay(p) = &mut self.screen {
                    p.feedback = None;
                }
            }
            Action::LearnHome => self.screen = Screen::LearnHome,
            _ => {}
        }
        self.dirty = true;
    }

    /// Per-frame update of a round; returns sounds to play.
    pub(super) fn learn_update(&mut self, dt: f32) {
        let mut finished = false;
        let mut fx = Vec::new();
        if let Screen::LearnPlay(p) = &mut self.screen {
            p.view.update(dt);
            fx.append(&mut p.view.fx);
            if let Some((_, t)) = &mut p.feedback {
                *t -= dt;
                if *t <= 0.0 {
                    p.feedback = None;
                }
            }
            if p.game.is_won() && !p.view.busy() {
                match &mut p.won {
                    None => {
                        p.won = Some(0.0);
                        finished = true;
                    }
                    Some(t) => *t += dt,
                }
            }
        }
        for f in fx {
            match f {
                crate::board::Fx::Bounce => {
                    self.sounds.push(Sfx::Bounce);
                    self.buzz(40);
                }
                crate::board::Fx::Bump => self.sounds.push(Sfx::Bump),
                crate::board::Fx::Exit => self.buzz(15),
                _ => {}
            }
        }
        if finished {
            self.sounds.push(Sfx::Win);
            self.finish_round();
        }
    }

    /// Pointer handling on the board of a round; true if it consumed the event.
    pub(super) fn learn_pointer(&mut self, ev: super::PointerEvent, x: f32, y: f32) -> bool {
        use super::PointerEvent as P;
        let mut mv = None;
        let mut tap = None;
        // Tapping a tile reads it aloud without its answer, unless the player chose to hear the answer (counted as
        // a hint for that tile, like the light bulb).
        let say_answer = self.save.get_bool("say_answer", false);
        if self.flagging.is_some() {
            return false;
        }
        {
            let Screen::LearnPlay(p) = &mut self.screen else { return false };
            if p.won.is_some() {
                return false;
            }
            match ev {
                P::Down => {
                    if let Some(b) = p.view.block_at(&p.game, x, y) {
                        p.drag = Some(Drag { block: b, x0: x, y0: y, done: false });
                        p.view.select(Some(b));
                    }
                }
                P::Move => {
                    if let Some(d) = &mut p.drag
                        && !d.done
                    {
                        let (dx, dy) = (x - d.x0, y - d.y0);
                        if dx.abs().max(dy.abs()) > p.view.cell * 0.28 {
                            let dir = if dx.abs() > dy.abs() {
                                if dx > 0.0 { Dir::E } else { Dir::W }
                            } else if dy > 0.0 {
                                Dir::S
                            } else {
                                Dir::N
                            };
                            d.done = true;
                            mv = Some(Move { block: d.block as u8, dir });
                        }
                    }
                }
                P::Up | P::Cancel => {
                    if let Some(d) = p.drag.take() {
                        p.view.select(None);
                        if !d.done && ev == P::Up {
                            if let Some(t) = p.round.tiles.get(d.block) {
                                let answered = p.answered.get(d.block).copied().unwrap_or(true);
                                if say_answer && !answered && t.say.is_some() && t.say != t.ask {
                                    p.hinted[d.block] = true;
                                }
                                tap = if say_answer || answered { t.say.clone() } else { t.ask.clone() };
                            }
                        }
                    }
                }
            }
        }
        if let Some(m) = mv {
            self.learn_move(m);
        }
        if let Some(s) = tap {
            self.say(&s);
        }
        false
    }

    pub(super) fn learn_hint(&mut self) {
        if let Screen::LearnPlay(p) = &mut self.screen
            && p.won.is_none()
            && let Some(mv) = fuera_core::solver::hint(&p.game.level, &p.game.state, 200_000)
        {
            p.hinted[mv.block as usize] = true;
            p.view.show_hint(Some(mv));
        }
    }

    // ------------------------------------------------------------------------------------------ drawing

    pub(super) fn draw_learn_home(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let lang = self.lang;
        let course = self.course();
        let target = course.target;
        let title = match target {
            Target::Es => tr(lang, T::Spanish),
            Target::Fr => tr(lang, T::French),
        };
        let mut y = self.draw_header(pm, title) + 4.0 * u;
        // Feedback, top right.
        let fy = self.top + 40.0 * u;
        self.round_button(pm, Action::Feedback, self.w - 36.0 * u, fy, 48.0 * u, Icon::Bubble, None);
        let day = self.host.today();
        let w = self.w;
        let cx = w / 2.0;
        let streak = self.memory.current_streak(day);
        let today = if self.memory.today.0 == day { self.memory.today.1 } else { 0 };
        let line = format!("{} {} {}  ·  {} {}/{}", tr(lang, T::Streak), streak, tr(lang, T::Days), tr(lang, T::Rounds), today.min(DAILY_GOAL), DAILY_GOAL);
        self.fonts.draw_shadowed(pm, Face::Bold, &line, cx, y + 8.0 * u, 16.0 * u, col_a(theme::WHITE, 0.9), Align::Center);
        y += 24.0 * u;
        // Daily progress bar.
        let bw = (330.0 * u).min(w * 0.88);
        let x0 = cx - bw / 2.0;
        fill(pm, &gfx::rrect(x0, y, bw, 10.0 * u, 5.0 * u), &solid(col_a(theme::WHITE, 0.2)), Transform::identity());
        let f = (today.min(DAILY_GOAL) as f32 / DAILY_GOAL as f32).max(0.02);
        fill(pm, &gfx::rrect(x0, y, bw * f, 10.0 * u, 5.0 * u), &solid(col(theme::GOLD)), Transform::identity());
        y += 24.0 * u;
        if self.memory.placed.is_none() {
            let ask = tr_course(lang, T::PlacementAsk, target);
            self.button(pm, Action::Placement, x0, y, bw, 52.0 * u, &ask, None, Style::Primary);
            y += 52.0 * u + 14.0 * u;
            let s = tr(lang, T::TodaySession);
            self.button(pm, Action::LearnSession, x0, y, bw, 56.0 * u, s, Some(Icon::Play), Style::Secondary);
        } else {
            let s = tr(lang, T::TodaySession);
            self.button(pm, Action::LearnSession, x0, y, bw, 64.0 * u, s, Some(Icon::Play), Style::Primary);
        }
        y += 64.0 * u + 18.0 * u;
        // Modes, two per row (names in the language learnt).
        let gap = 10.0 * u;
        let mw = (bw - gap) / 2.0;
        let mh = 46.0 * u;
        let band = self.band();
        let menu = Mode::menu(target);
        for (i, m) in menu.iter().enumerate() {
            let (col_i, row) = (i % 2, i / 2);
            let bx = x0 + col_i as f32 * (mw + gap);
            let by = y + row as f32 * (mh + gap);
            let style = if m.min_band(target) <= band { Style::Secondary } else { Style::Ghost };
            self.button(pm, Action::LearnMode(*m), bx, by, mw, mh, m.name(target), None, style);
        }
        y += menu.len().div_ceil(2) as f32 * (mh + gap) + 6.0 * u;
        // Notebook, meanings, course.
        let sq = mh;
        let w2 = (bw - 2.0 * gap - sq) / 2.0;
        let nb = tr(lang, T::Notebook);
        let n_miss = self.memory.misses.len();
        let label = if n_miss > 0 { format!("{nb} ({n_miss})") } else { nb.to_string() };
        self.button(pm, Action::Notebook, x0, y, w2, mh, &label, Some(Icon::Book), Style::Ghost);
        let base = self.base().code().to_uppercase();
        let meanings = format!("{} {base}", tr(lang, T::Meanings));
        self.button(pm, Action::ToggleBase, x0 + w2 + gap, y, w2, mh, &meanings, None, Style::Ghost);
        self.button(pm, Action::Courses, x0 + 2.0 * (w2 + gap), y, sq, mh, "", Some(Icon::Globe), Style::Ghost);
        y += mh + 16.0 * u;
        let (seen, _, solid_n) = self.memory.counts("w:");
        let lvl = match self.memory.placed {
            Some(b) => format!("{} {}  ·  ", tr(lang, T::YourLevel), b.name()),
            None => String::new(),
        };
        let stats = format!("{lvl}{} {seen}  ·  {} {solid_n}", tr(lang, T::Words), tr(lang, T::Known));
        self.fonts.draw_shadowed(pm, Face::Bold, &stats, cx, y + 8.0 * u, 14.0 * u, col_a(theme::WHITE, 0.8), Align::Center);
        if self.memory.placed.is_some() {
            self.hits.push(Hit { x: x0, y: y - 10.0 * u, w: bw, h: 26.0 * u, a: Action::Placement });
        }
    }

    /// The course picker: three courses, the one matching the device's language first in colour.
    pub(super) fn draw_courses(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let lang = self.lang;
        let title = tr(lang, T::LearnLanguage);
        let mut y = self.draw_header(pm, title) + 24.0 * u;
        let cx = self.w / 2.0;
        let q = tr(lang, T::WhichLanguage);
        for l in self.fonts.wrap(Face::Bold, q, 19.0 * u, self.w - 48.0 * u) {
            self.fonts.draw_shadowed(pm, Face::Bold, &l, cx, y + 8.0 * u, 19.0 * u, col(theme::WHITE), Align::Center);
            y += 26.0 * u;
        }
        y += 18.0 * u;
        let current = if self.save.get("course").is_some() {
            Some(self.course())
        } else {
            // Nothing chosen yet: Spanish-speaking devices are offered French, the others Spanish.
            Some(match self.lang {
                crate::i18n::Lang::Es => Course::FRENCH,
                _ => {
                    if self.latam() {
                        Course::LATAM
                    } else {
                        Course::SPAIN
                    }
                }
            })
        };
        let bw = (330.0 * u).min(self.w * 0.88);
        let bh = 76.0 * u;
        let x = cx - bw / 2.0;
        let opts = [(Course::SPAIN, T::CourseSpain, T::CourseSpainNote), (Course::LATAM, T::CourseLatam, T::CourseLatamNote), (Course::FRENCH, T::CourseFrench, T::CourseFrenchNote)];
        for (c, name, note) in opts {
            let style = if Some(c) == current { Style::Primary } else { Style::Secondary };
            self.button(pm, Action::PickCourse(c), x, y, bw, bh * 0.72, tr(lang, name), Some(Icon::Globe), style);
            let n = tr(lang, note);
            self.fonts.draw_shadowed(pm, Face::Bold, n, cx, y + bh * 0.72 + 22.0 * u, 14.0 * u, col_a(theme::WHITE, 0.8), Align::Center);
            y += bh + 22.0 * u;
        }
        let kept = tr(lang, T::ProgressKept);
        self.fonts.draw_shadowed(pm, Face::Bold, kept, cx, y + 10.0 * u, 15.0 * u, col_a(theme::WHITE, 0.85), Align::Center);
    }

    /// The "report this word" card, over any screen.
    pub(super) fn draw_flagging(&mut self, pm: &mut Pixmap) {
        let Some((_, text)) = self.flagging.clone() else { return };
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let lang = self.lang;
        gfx::fill_rect(pm, 0.0, 0.0, w, h, &solid(col_a((5, 20, 35), 0.55)));
        self.hits.push(Hit { x: 0.0, y: 0.0, w, h, a: Action::FlagCancel });
        let cw = (340.0 * u).min(w * 0.92);
        let px = 16.0 * u;
        let lines = self.fonts.wrap(Face::Bold, &text, px, cw - 40.0 * u);
        let lines: Vec<String> = lines.into_iter().take(3).collect();
        let bh = 50.0 * u;
        let ch = 70.0 * u + lines.len() as f32 * px * 1.35 + 4.0 * (bh + 10.0 * u) + 10.0 * u;
        let x = w / 2.0 - cw / 2.0;
        let y = h / 2.0 - ch / 2.0;
        fill(pm, &gfx::rrect(x, y + 6.0 * u, cw, ch, 24.0 * u), &solid(col_a((0, 0, 0), 0.3)), Transform::identity());
        fill(pm, &gfx::rrect(x, y, cw, ch, 24.0 * u), &solid(col(theme::PLASTER)), Transform::identity());
        self.hits.push(Hit { x, y, w: cw, h: ch, a: Action::Noop });
        let t = tr(lang, T::Report);
        icons::draw(pm, Icon::Flag, x + 34.0 * u, y + 36.0 * u, 28.0 * u, col(theme::ACCENT));
        self.fonts.draw(pm, Face::Title, t, x + 56.0 * u, y + 44.0 * u, 24.0 * u, col(theme::INK), Align::Left);
        let mut ty = y + 70.0 * u;
        for l in &lines {
            self.fonts.draw(pm, Face::Bold, l, x + 20.0 * u, ty + px, px, col(shade(theme::INK, 0.3)), Align::Left);
            ty += px * 1.35;
        }
        ty += 14.0 * u;
        let bw = cw - 40.0 * u;
        for (i, r) in [T::ReportVoice, T::ReportMeaning, T::ReportOther].into_iter().enumerate() {
            self.button(pm, Action::FlagReason(i as u8), x + 20.0 * u, ty, bw, bh, tr(lang, r), None, Style::Secondary);
            ty += bh + 10.0 * u;
        }
        self.button(pm, Action::FlagCancel, x + 20.0 * u, ty, bw, bh, tr(lang, T::Cancel), None, Style::Ghost);
    }

    pub(super) fn draw_toast(&mut self, pm: &mut Pixmap, text: &str, t: f32) {
        let u = self.u;
        let a = t.min(0.4) / 0.4;
        let cw = (360.0 * u).min(self.w - 24.0 * u);
        let px = 16.0 * u;
        let lines = self.fonts.wrap(Face::Bold, text, px, cw - 32.0 * u);
        let ch = lines.len() as f32 * px * 1.35 + 24.0 * u;
        let x = self.w / 2.0 - cw / 2.0;
        let y = self.h - self.bottom - 150.0 * u - ch;
        fill(pm, &gfx::rrect(x, y, cw, ch, 16.0 * u), &solid(col_a((0x1C, 0x3B, 0x52), 0.95 * a)), Transform::identity());
        for (i, l) in lines.iter().enumerate() {
            self.fonts.draw(pm, Face::Bold, l, self.w / 2.0, y + 12.0 * u + (i as f32 + 0.8) * px * 1.35, px, col_a(theme::WHITE, a), Align::Center);
        }
    }

    /// Opens the system's share sheet (or a mail to the project) with the reported words and some context, for the
    /// player to add their own words — typed or spoken in their messaging app.
    fn send_feedback(&mut self) {
        let lang = self.lang;
        let course = self.course();
        let base = self.base();
        let band = self.memory.placed.map(|b| b.name()).unwrap_or("-");
        let mut body = format!(
            "¡Fuera! {} · {} · {}→{} · {} · {}\n",
            env!("CARGO_PKG_VERSION"),
            self.host.platform(),
            base.code(),
            course.voice_pack(),
            band,
            lang.code()
        );
        // Reports of the last two weeks (they stay saved: a share sheet closed without sending loses nothing).
        let today = self.host.today();
        let flags: Vec<String> = self
            .save
            .get("fb.flags")
            .unwrap_or("")
            .split('\u{1f}')
            .filter(|x| !x.is_empty())
            .filter(|x| x.rsplit('\u{1e}').next().and_then(|d| d.parse::<i64>().ok()).is_none_or(|d| today - d <= 14))
            .map(|x| x.to_string())
            .collect();
        if !flags.is_empty() {
            body.push_str(&format!("\n{} :\n", tr(lang, T::FeedbackFlags)));
            for f in &flags {
                let c: Vec<&str> = f.splitn(5, '\u{1e}').collect();
                if c.len() >= 4 {
                    let reason = match c[2] {
                        "voice" => tr(lang, T::ReportVoice),
                        "meaning" => tr(lang, T::ReportMeaning),
                        _ => tr(lang, T::ReportOther),
                    };
                    body.push_str(&format!("- {} — {} [{} {}]\n", c[3], reason, c[0], c[1]));
                }
            }
        }
        body.push_str(&format!("\n{}\n\n", tr(lang, T::FeedbackBody)));
        let subject = format!("¡Fuera! — {}", tr(lang, T::Feedback));
        if !self.host.share(&subject, &body) {
            let enc = |s: &str| -> String {
                s.bytes()
                    .map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
                        _ => format!("%{b:02X}"),
                    })
                    .collect()
            };
            let url = format!("mailto:fuera@librejeu.fr?subject={}&body={}", enc(&subject), enc(&body));
            self.host.open_url(&url);
        }
        self.save.set("fb.asked", 1);
        self.persist();
    }

    pub(super) fn draw_learn_intro(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let lang = self.lang;
        let title = tr(lang, T::NewWords);
        let mut y = self.draw_header(pm, title) + 12.0 * u;
        let rows: Vec<(String, String, String)> = match &self.screen {
            Screen::LearnIntro(p) => p.round.intro.iter().map(|w| (w.es.clone(), w.meaning.clone(), w.note.clone())).collect(),
            _ => return,
        };
        let bw = (340.0 * u).min(self.w * 0.9);
        let x0 = self.w / 2.0 - bw / 2.0;
        for (i, (es, meaning, note)) in rows.iter().enumerate() {
            let h = if note.is_empty() { 74.0 * u } else { 96.0 * u };
            fill(pm, &gfx::rrect(x0, y + 4.0 * u, bw, h, 18.0 * u), &solid(col_a((0, 0, 0), 0.25)), Transform::identity());
            fill(pm, &gfx::rrect(x0, y, bw, h, 18.0 * u), &solid(col(theme::PLASTER)), Transform::identity());
            let px = self.fonts.fit(Face::Title, es, 28.0 * u, bw - 90.0 * u);
            self.fonts.draw(pm, Face::Title, es, x0 + 18.0 * u, y + 36.0 * u, px, col(theme::INK), Align::Left);
            self.fonts.draw(pm, Face::Bold, meaning, x0 + 18.0 * u, y + 60.0 * u, 16.0 * u, col(shade(theme::INK, 0.3)), Align::Left);
            if !note.is_empty() {
                let n = self.fonts.wrap(Face::Body, note, 13.0 * u, bw - 90.0 * u).into_iter().next().unwrap_or_default();
                self.fonts.draw(pm, Face::Body, &n, x0 + 18.0 * u, y + 82.0 * u, 13.0 * u, col(shade(theme::INK, 0.45)), Align::Left);
            }
            icons::draw(pm, Icon::Speaker, x0 + bw - 36.0 * u, y + h / 2.0, 30.0 * u, col(theme::ACCENT));
            self.hits.push(Hit { x: x0, y, w: bw, h, a: Action::SayIntro(i) });
            y += h + 12.0 * u;
        }
        let tip = tr(lang, T::TapToHear);
        self.fonts.draw_shadowed(pm, Face::Bold, tip, self.w / 2.0, y + 18.0 * u, 14.0 * u, col_a(theme::WHITE, 0.8), Align::Center);
        let go = tr(lang, T::Go);
        let by = (y + 40.0 * u).max(self.h - self.bottom - 110.0 * u);
        self.button(pm, Action::LearnGo, x0, by, bw, 60.0 * u, go, Some(Icon::Play), Style::Primary);
    }

    pub(super) fn draw_learn_play(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let lang = self.lang;
        let (top, bottom) = (self.top, self.bottom);
        let (mode, heading, production, legend) = match &self.screen {
            Screen::LearnPlay(p) => (p.round.mode, p.round.heading.clone(), p.round.production, p.legend.clone()),
            _ => return,
        };
        let target = self.course().target;
        let title = if heading.is_empty() { mode.name(target).to_string() } else { heading };
        let y = self.draw_header(pm, &title);
        let ins = match mode {
            Mode::Words if production => T::InsProduction,
            Mode::Words => T::InsWords,
            Mode::Listening => T::InsListening,
            Mode::Gender if target == Target::Fr => T::InsGenderFr,
            Mode::Gender => T::InsGender,
            Mode::Verbs => T::InsVerbs,
            Mode::SerEstar | Mode::PorPara => T::InsBinary,
            Mode::Auxiliary => T::InsAuxiliary,
            Mode::Past | Mode::Subjunctive | Mode::Partitive => T::InsSentences,
            Mode::FalseFriends => T::InsFalse,
            Mode::Placement => T::InsPlacement,
        };
        let ins = tr_course(lang, ins, target);
        let px = self.fonts.fit(Face::Bold, &ins, 15.0 * u, w - 24.0 * u);
        self.fonts.draw_shadowed(pm, Face::Bold, &ins, w / 2.0, y + 6.0 * u, px, col_a(theme::WHITE, 0.9), Align::Center);
        let mut board_top = y + 22.0 * u;
        // Legend for long door texts.
        if !legend.is_empty() {
            let lw = (w - 32.0 * u).min(420.0 * u);
            let lx = w / 2.0 - lw / 2.0;
            let px = 15.0 * u;
            let mut lines = Vec::new();
            for (m, t) in &legend {
                for (k, l) in self.fonts.wrap(Face::Bold, t, px, lw - 54.0 * u).into_iter().enumerate() {
                    lines.push((if k == 0 { m.clone() } else { String::new() }, l));
                }
            }
            let lh = px * 1.45;
            let ph = lines.len() as f32 * lh + 16.0 * u;
            fill(pm, &gfx::rrect(lx, board_top, lw, ph, 14.0 * u), &solid(col_a(theme::PLASTER, 0.96)), Transform::identity());
            for (i, (m, l)) in lines.iter().enumerate() {
                let ly = board_top + 8.0 * u + (i as f32 + 0.78) * lh;
                if !m.is_empty() {
                    fill(pm, &gfx::circle(lx + 22.0 * u, ly - px * 0.33, 11.0 * u), &solid(col(theme::GOLD)), Transform::identity());
                    self.fonts.draw(pm, Face::Title, m, lx + 22.0 * u, ly - px * 0.33 + 6.0 * u, 15.0 * u, col(theme::INK), Align::Center);
                }
                self.fonts.draw(pm, Face::Bold, l, lx + 44.0 * u, ly, px, col(theme::INK), Align::Left);
            }
            board_top += ph + 8.0 * u;
        }
        let board_bottom = h - bottom - 130.0 * u;
        let margin = 12.0 * u;
        let (won, feedback, right, wrong, results) = {
            let Screen::LearnPlay(p) = &mut self.screen else { return };
            p.view.layout(&p.game.level, margin, board_top, w - 2.0 * margin, board_bottom - board_top);
            p.view.draw(pm, &p.game, &mut self.fonts);
            (p.won, p.feedback.clone(), p.right, p.wrong, p.results.clone())
        };
        let _ = top;
        let by = h - bottom - 82.0 * u;
        let d = 58.0 * u;
        let gap = (w / 4.0).min(110.0 * u);
        self.round_button(pm, Action::Undo, w / 2.0 - gap, by, d, Icon::Undo, Some(tr(lang, T::Undo)));
        self.round_button(pm, Action::Restart, w / 2.0, by, d, Icon::Restart, Some(tr(lang, T::Restart)));
        self.round_button(pm, Action::Hint, w / 2.0 + gap, by, d, Icon::Hint, Some(tr(lang, T::Hint)));
        if let Some((text, _)) = feedback {
            self.feedback_card(pm, &text, board_bottom);
        }
        if let Some(t) = won {
            self.draw_round_done(pm, t, right, wrong, &results);
        }
    }

    fn feedback_card(&mut self, pm: &mut Pixmap, text: &str, bottom_y: f32) {
        let u = self.u;
        let w = self.w;
        let cw = (360.0 * u).min(w - 24.0 * u);
        let px = 18.0 * u;
        let lines = self.fonts.wrap(Face::Bold, text, px, cw - 70.0 * u);
        let ch = (lines.len() as f32 * px * 1.35 + 28.0 * u).max(64.0 * u);
        let x = w / 2.0 - cw / 2.0;
        let y = bottom_y - ch + 10.0 * u;
        fill(pm, &gfx::rrect(x, y + 5.0 * u, cw, ch, 18.0 * u), &solid(col_a((0, 0, 0), 0.3)), Transform::identity());
        fill(pm, &gfx::rrect(x, y, cw, ch, 18.0 * u), &solid(col(theme::PLASTER)), Transform::identity());
        fill(pm, &gfx::rrect(x, y, 10.0 * u, ch, 5.0 * u), &solid(col(theme::ACCENT)), Transform::identity());
        for (i, l) in lines.iter().enumerate() {
            self.fonts.draw(pm, Face::Bold, l, x + 22.0 * u, y + 14.0 * u + (i as f32 + 0.8) * px * 1.35, px, col(theme::INK), Align::Left);
        }
        icons::draw(pm, Icon::Close, x + cw - 26.0 * u, y + 22.0 * u, 22.0 * u, col(shade(theme::INK, 0.4)));
        self.hits.push(Hit { x, y, w: cw, h: ch, a: Action::Dismiss });
        // Report this word (bottom right of the card).
        icons::draw(pm, Icon::Flag, x + cw - 26.0 * u, y + ch - 20.0 * u, 20.0 * u, col(shade(theme::INK, 0.45)));
        self.hits.push(Hit { x: x + cw - 50.0 * u, y: y + ch - 44.0 * u, w: 50.0 * u, h: 44.0 * u, a: Action::FlagFeedback });
    }

    fn draw_round_done(&mut self, pm: &mut Pixmap, t: f32, right: u32, wrong: u32, results: &[(String, bool, String)]) {
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let lang = self.lang;
        let a = (t / 0.3).min(1.0);
        gfx::fill_rect(pm, 0.0, 0.0, w, h, &solid(col_a((5, 20, 35), 0.5 * a)));
        let cw = (340.0 * u).min(w * 0.92);
        let rows = results.len().min(8) as f32;
        let day = self.host.today();
        let ask = self.memory.today.0 == day && self.memory.today.1 == DAILY_GOAL && !self.save.get_bool("fb.asked", false);
        let ch = 190.0 * u + rows * 26.0 * u + if ask { 52.0 * u } else { 0.0 };
        let pop = gfx::ease_back((t / 0.35).min(1.0));
        let x = w / 2.0 - cw / 2.0;
        let y = h * 0.45 - ch / 2.0 + (1.0 - pop) * 50.0 * u;
        fill(pm, &gfx::rrect(x, y + 8.0 * u, cw, ch, 26.0 * u), &solid(col_a((0, 0, 0), 0.3 * a)), Transform::identity());
        fill(pm, &gfx::rrect(x, y, cw, ch, 26.0 * u), &solid(col_a(theme::PLASTER, a)), Transform::identity());
        let total = right + wrong;
        let perfect = if self.course().target == Target::Fr { "Parfait !" } else { "¡Perfecto!" };
        let title = if wrong == 0 { perfect } else { tr(lang, T::RoundDone) };
        self.fonts.draw(pm, Face::Title, title, w / 2.0, y + 50.0 * u, 34.0 * u, col_a(theme::ACCENT, a), Align::Center);
        let score = format!("{right} / {total} {}", tr(lang, T::Right));
        self.fonts.draw(pm, Face::Bold, &score, w / 2.0, y + 80.0 * u, 18.0 * u, col_a(theme::INK, a), Align::Center);
        let mut ry = y + 112.0 * u;
        for (i, (text, ok, _)) in results.iter().take(8).enumerate() {
            let c = if *ok { (0x2E, 0x9E, 0x5B) } else { (0xD0, 0x45, 0x45) };
            fill(pm, &gfx::circle(x + 30.0 * u, ry - 6.0 * u, 7.0 * u), &solid(col_a(c, a)), Transform::identity());
            let px = self.fonts.fit(Face::Bold, text, 16.0 * u, cw - 100.0 * u);
            self.fonts.draw(pm, Face::Bold, text, x + 48.0 * u, ry, px, col_a(theme::INK, a), Align::Left);
            icons::draw(pm, Icon::Flag, x + cw - 30.0 * u, ry - 6.0 * u, 16.0 * u, col_a(shade(theme::INK, 0.55), a * 0.8));
            self.hits.push(Hit { x: x + cw - 54.0 * u, y: ry - 22.0 * u, w: 48.0 * u, h: 26.0 * u, a: Action::FlagResult(i) });
            ry += 26.0 * u;
        }
        let day = self.host.today();
        if self.memory.today.0 == day && self.memory.today.1 == DAILY_GOAL {
            let g = tr(lang, T::GoalDone);
            self.fonts.draw(pm, Face::Bold, g, w / 2.0, ry + 4.0 * u, 15.0 * u, col_a((0x2E, 0x9E, 0x5B), a), Align::Center);
            // Once, after the first full session: an invitation to give an opinion.
            if !self.save.get_bool("fb.asked", false) {
                let ask = tr(lang, T::FeedbackAsk);
                let bwide = cw - 40.0 * u;
                self.button(pm, Action::Feedback, x + 20.0 * u, ry + 18.0 * u, bwide, 40.0 * u, ask, Some(Icon::Bubble), Style::Ghost);
            }
        }
        let bh = 52.0 * u;
        let bw = cw * 0.42;
        let back = tr(lang, T::Back);
        let next = tr(lang, T::Continue);
        self.button(pm, Action::LearnHome, x + cw * 0.06, y + ch - bh - 20.0 * u, bw, bh, back, None, Style::Secondary);
        self.button(pm, Action::LearnNext, x + cw * 0.52, y + ch - bh - 20.0 * u, bw, bh, next, Some(Icon::Play), Style::Primary);
    }

    pub(super) fn draw_notebook(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let lang = self.lang;
        let title = tr(lang, T::Notebook);
        let y0 = self.draw_header(pm, title) + 16.0 * u;
        let base = self.base();
        let latam = self.latam();
        let misses = self.memory.misses.clone();
        if misses.is_empty() {
            let t = tr(lang, T::NotebookEmpty);
            for (i, l) in self.fonts.wrap(Face::Bold, t, 16.0 * u, self.w - 60.0 * u).iter().enumerate() {
                self.fonts.draw_shadowed(pm, Face::Bold, l, self.w / 2.0, y0 + 30.0 * u + i as f32 * 22.0 * u, 16.0 * u, col(theme::WHITE), Align::Center);
            }
            return;
        }
        self.content();
        let scroll = if let Screen::Notebook { scroll, .. } = &self.screen { *scroll } else { 0.0 };
        let bw = (360.0 * u).min(self.w * 0.92);
        let x0 = self.w / 2.0 - bw / 2.0;
        let mut y = y0 - scroll;
        for (i, id) in misses.iter().enumerate() {
            let Some((text, _)) = plan::describe(self.content.as_ref().unwrap(), id, base, latam) else { continue };
            let px = 15.0 * u;
            let lines = self.fonts.wrap(Face::Bold, &text, px, bw - 96.0 * u);
            let rh = lines.len() as f32 * px * 1.35 + 20.0 * u;
            if y + rh > y0 - 10.0 * u && y < self.h {
                fill(pm, &gfx::rrect(x0, y, bw, rh, 14.0 * u), &solid(col(theme::PLASTER)), Transform::identity());
                for (k, l) in lines.iter().enumerate() {
                    self.fonts.draw(pm, Face::Bold, l, x0 + 14.0 * u, y + 10.0 * u + (k as f32 + 0.8) * px * 1.35, px, col(theme::INK), Align::Left);
                }
                icons::draw(pm, Icon::Speaker, x0 + bw - 26.0 * u, y + rh / 2.0, 24.0 * u, col(theme::ACCENT));
                self.hits.push(Hit { x: x0, y, w: bw, h: rh, a: Action::SayMiss(i) });
                icons::draw(pm, Icon::Flag, x0 + bw - 58.0 * u, y + rh / 2.0, 18.0 * u, col(shade(theme::INK, 0.55)));
                self.hits.push(Hit { x: x0 + bw - 76.0 * u, y, w: 34.0 * u, h: rh, a: Action::FlagMiss(i) });
            }
            y += rh + 8.0 * u;
        }
        let max = (y + scroll - y0 - (self.h - y0 - 40.0 * u)).max(0.0);
        if let Screen::Notebook { scroll, .. } = &mut self.screen {
            *scroll = scroll.min(max);
        }
        // Keep the header readable over the list.
        let g = gfx::vgrad(0.0, self.top + 96.0 * u, &[(0.0, col(theme::SEA_TOP)), (0.8, col(theme::SEA_TOP)), (1.0, col_a(theme::SEA_TOP, 0.0))]);
        gfx::fill_rect(pm, 0.0, 0.0, self.w, self.top + 96.0 * u, &g);
        self.draw_header(pm, title);
    }

    pub(super) fn draw_placed(&mut self, pm: &mut Pixmap, band: Band) {
        let u = self.u;
        let lang = self.lang;
        let title = tr(lang, T::PlacementDone);
        let y = self.draw_header(pm, title);
        let cx = self.w / 2.0;
        let lv = tr(lang, T::YourLevel);
        self.fonts.draw_shadowed(pm, Face::Bold, lv, cx, y + 80.0 * u, 20.0 * u, col(theme::WHITE), Align::Center);
        self.fonts.draw_shadowed(pm, Face::Title, band.name(), cx, y + 190.0 * u, 110.0 * u, col(theme::GOLD), Align::Center);
        let note = tr(lang, T::PlacementNote);
        for (i, l) in self.fonts.wrap(Face::Bold, note, 16.0 * u, self.w - 70.0 * u).iter().enumerate() {
            self.fonts.draw_shadowed(pm, Face::Bold, l, cx, y + 250.0 * u + i as f32 * 22.0 * u, 16.0 * u, col_a(theme::WHITE, 0.9), Align::Center);
        }
        let bw = (300.0 * u).min(self.w * 0.84);
        let go = tr(lang, T::TodaySession);
        self.button(pm, Action::LearnSession, cx - bw / 2.0, self.h - self.bottom - 130.0 * u, bw, 62.0 * u, go, Some(Icon::Play), Style::Primary);
    }
}
