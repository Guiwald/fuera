//! The application: screens, buttons, input, saved progress. Platform code only forwards events and shows the
//! picture.

mod learn;

use fuera_core::rules::{self, Result, Stuck};
use fuera_core::{Dir, Game, Level, Move};
use fuera_learn::content::{Band, Content};
use fuera_learn::memory::Memory;
use fuera_learn::plan::Mode;
use tiny_skia::{Color, Pixmap, Transform};

use crate::board::{BoardView, Fx, Skin};
use crate::gfx::{self, fill, solid, stroke};
use crate::i18n::{Lang, T, tr};
use crate::icons::{self, Icon};
use crate::save::Save;
use crate::sfx::Sfx;
use crate::text::{Align, Face, Fonts};
use crate::theme::{self, col, col_a, shade};

/// Services the platform provides.
pub trait Host {
    /// The saved progress, if any.
    fn load(&mut self) -> Option<String>;
    fn store(&mut self, text: &str);
    /// Speaks a text with the system voice (`lang` like "es-ES").
    fn speak(&mut self, _text: &str, _lang: &str) {}
    fn vibrate(&mut self, _ms: u32) {}
    fn open_url(&mut self, _url: &str) {}
    /// Leaves the app (Android back on the home screen).
    fn quit(&mut self) {}
    /// System language, like "fr-FR".
    fn locale(&self) -> String {
        "en".into()
    }
    /// Days since 1970-01-01 (local time), for the daily puzzle and streaks.
    fn today(&self) -> i64 {
        0
    }
    /// Time in days since 1970-01-01 (local), with the fraction of the day.
    fn now_days(&self) -> f64 {
        self.today() as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerEvent {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    Arrow(Dir),
    Undo,
    Restart,
    Hint,
    Back,
    Enter,
    NextBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Continue,
    Levels,
    Learn,
    Settings,
    Back,
    Undo,
    Restart,
    Hint,
    Next,
    Replay,
    Open(usize),
    ToggleSound,
    ToggleVib,
    CycleLang,
    Credits,
    LearnHome,
    LearnSession,
    LearnMode(Mode),
    LearnGo,
    LearnNext,
    Notebook,
    ToggleBase,
    ToggleSayAnswer,
    Placement,
    SayIntro(usize),
    SayMiss(usize),
    Dismiss,
}

#[derive(Clone, Copy)]
enum Style {
    Primary,
    Secondary,
    Ghost,
}

struct Hit {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    a: Action,
}

struct Drag {
    block: usize,
    x0: f32,
    y0: f32,
    done: bool,
}

struct Play {
    game: Game,
    view: BoardView,
    index: usize,
    drag: Option<Drag>,
    won: Option<f32>,
    stars: u8,
    toast: Option<(String, f32)>,
    exits_in_row: u8,
    intro: Option<T>,
    /// Rest of an optimal solution from the current state (filled by the hint button, followed while the player
    /// plays its moves, dropped as soon as they leave it).
    plan: Vec<Move>,
}

enum Screen {
    Home,
    Levels { scroll: f32, drag: Option<(f32, f32, f32)> },
    Play(Box<Play>),
    Settings,
    Credits,
    LearnHome,
    LearnIntro(Box<learn::LearnPlay>),
    LearnPlay(Box<learn::LearnPlay>),
    Notebook { scroll: f32, drag: Option<(f32, f32, f32)> },
    Placed(Band),
}

pub struct App {
    host: Box<dyn Host>,
    fonts: Fonts,
    w: f32,
    h: f32,
    u: f32,
    top: f32,
    bottom: f32,
    pub lang: Lang,
    save: Save,
    campaign: Vec<Level>,
    screen: Screen,
    hits: Vec<Hit>,
    pointer: (f32, f32),
    down: Option<(f32, f32)>,
    sounds: Vec<Sfx>,
    time: f32,
    bg: Option<Pixmap>,
    dirty: bool,
    since_input: f32,
    content: Option<Content>,
    memory: Memory,
    placement: Vec<(Option<Band>, bool)>,
}

static CAMPAIGN: &str = include_str!("../../../assets/levels/campaign.txt");

pub fn parse_pack(text: &str) -> Vec<Level> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for line in text.lines() {
        cur.push_str(line);
        cur.push('\n');
        if line.trim() == "end" {
            match Level::parse(&cur) {
                Ok(l) => out.push(l),
                Err(e) => log::error!("level {}: {e}", out.len() + 1),
            }
            cur.clear();
        }
    }
    out
}

impl App {
    pub fn new(mut host: Box<dyn Host>) -> App {
        let save = host.load().map(|t| Save::parse(&t)).unwrap_or_default();
        let lang = save.get("lang").and_then(Lang::from_code).or_else(|| Lang::from_code(&host.locale())).unwrap_or(Lang::En);
        let memory = Memory::from_pairs(save.keys_with_prefix("m.").chain(save.keys_with_prefix("l.")));
        App {
            host,
            fonts: Fonts::new(),
            w: 412.0,
            h: 915.0,
            u: 1.0,
            top: 0.0,
            bottom: 0.0,
            lang,
            save,
            campaign: parse_pack(CAMPAIGN),
            screen: Screen::Home,
            hits: Vec::new(),
            pointer: (0.0, 0.0),
            down: None,
            sounds: Vec::new(),
            time: 0.0,
            bg: None,
            dirty: true,
            since_input: 0.0,
            content: None,
            memory,
            placement: Vec::new(),
        }
    }

    /// Window size in pixels and safe-area insets (top, bottom) in pixels.
    pub fn resize(&mut self, w: u32, h: u32, top: f32, bottom: f32) {
        self.w = w as f32;
        self.h = h as f32;
        self.top = top;
        self.bottom = bottom;
        // One unit = one dp of a 412×915 phone; the layout always fits the height.
        self.u = (self.w / 412.0).min((self.h - top - bottom) / 880.0).max(0.5);
        self.bg = None;
        self.dirty = true;
    }

    fn t(&self, t: T) -> &'static str {
        tr(self.lang, t)
    }

    pub fn sound_on(&self) -> bool {
        self.save.get_bool("sound", true)
    }

    /// Sounds to play since the last call.
    pub fn take_sounds(&mut self) -> Vec<Sfx> {
        let v = std::mem::take(&mut self.sounds);
        if self.sound_on() { v } else { Vec::new() }
    }

    fn sfx(&mut self, s: Sfx) {
        self.sounds.push(s);
    }

    fn buzz(&mut self, ms: u32) {
        if self.save.get_bool("vib", true) {
            self.host.vibrate(ms);
        }
    }

    pub fn needs_redraw(&self) -> bool {
        if self.dirty {
            return true;
        }
        match &self.screen {
            Screen::Home => self.since_input < 30.0,
            Screen::Play(p) => p.view.animating() || p.toast.is_some() || p.won.is_some_and(|t| t < 3.0) || p.intro.is_some() && self.since_input < 30.0,
            Screen::Levels { .. } => false,
            Screen::LearnPlay(p) => p.view.animating() || p.feedback.is_some() || p.won.is_some_and(|t| t < 1.0),
            _ => false,
        }
    }

    fn persist(&mut self) {
        if self.save.dirty {
            let t = self.save.to_text();
            self.host.store(&t);
            self.save.dirty = false;
        }
    }

    fn stars_of(&self, i: usize) -> u8 {
        self.save.get_u32(&format!("c{i}")) as u8
    }

    /// First level not yet cleared.
    fn next_level(&self) -> usize {
        (0..self.campaign.len()).find(|&i| self.stars_of(i) == 0).unwrap_or(self.campaign.len().saturating_sub(1))
    }

    fn unlocked(&self, i: usize) -> bool {
        // Cleared levels and the next three are open.
        i <= self.next_level() + 2
    }

    fn open_level(&mut self, i: usize) {
        let Some(level) = self.campaign.get(i).cloned() else { return };
        let intro = match level.meta.extra.iter().find(|(k, _)| k == "intro").map(|(_, v)| v.as_str()) {
            Some("ice") => Some(T::IceIntro),
            Some("lane") => Some(T::LaneIntro),
            Some("shell") => Some(T::ShellIntro),
            Some("lock") => Some(T::LockIntro),
            Some("shutter") => Some(T::ShutterIntro),
            Some("wall") => Some(T::WallIntro),
            Some("big") => Some(T::BigIntro),
            Some("t1") => Some(T::Tutorial1),
            Some("t2") => Some(T::Tutorial2),
            Some("t3") => Some(T::Tutorial3),
            _ => None,
        };
        let game = Game::new(level.clone());
        let mut view = BoardView::new(&game, Skin::classic(&level));
        if i == 0 {
            // First level: show the first move.
            view.show_hint(fuera_core::solver::hint(&game.level, &game.state, 50_000));
        }
        self.screen = Screen::Play(Box::new(Play { game, view, index: i, drag: None, won: None, stars: 0, toast: None, exits_in_row: 0, intro, plan: Vec::new() }));
        self.save.set("last", i);
        self.dirty = true;
    }

    pub fn update(&mut self, dt: f32) {
        let dt = dt.min(0.1);
        self.time += dt;
        self.since_input += dt;
        self.learn_update(dt);
        let mut fx = Vec::new();
        let mut won_now = false;
        if let Screen::Play(p) = &mut self.screen {
            p.view.update(dt);
            fx.append(&mut p.view.fx);
            if let Some((_, t)) = &mut p.toast {
                *t -= dt;
                if *t <= 0.0 {
                    p.toast = None;
                    self.dirty = true;
                }
            }
            if p.game.is_won() && !p.view.busy() {
                match &mut p.won {
                    None => {
                        p.won = Some(0.0);
                        p.stars = p.game.stars();
                        p.view.celebrate(&p.game.level);
                        won_now = true;
                    }
                    Some(t) => *t += dt,
                }
            }
        }
        for f in fx {
            match f {
                Fx::Slide => {}
                Fx::Bump => self.sfx(Sfx::Bump),
                Fx::Exit => {
                    let n = if let Screen::Play(p) = &mut self.screen {
                        p.exits_in_row = p.exits_in_row.saturating_add(1);
                        p.exits_in_row - 1
                    } else {
                        0
                    };
                    self.sfx(Sfx::Exit(n));
                    self.buzz(18);
                }
                Fx::Peel => self.sfx(Sfx::Peel),
                Fx::Bounce => {
                    self.sfx(Sfx::Bounce);
                    self.buzz(40);
                }
                Fx::Win => self.sfx(Sfx::Win),
                Fx::Tap => self.sfx(Sfx::Tap),
                Fx::Nope => self.sfx(Sfx::Nope),
            }
        }
        if won_now {
            self.sfx(Sfx::Win);
            self.buzz(30);
            if let Screen::Play(p) = &self.screen {
                let key = format!("c{}", p.index);
                let best = self.save.get_u32(&key).max(p.stars as u32);
                self.save.set(&key, best);
                self.persist();
            }
        }
        if let Screen::Play(p) = &self.screen
            && let Some(t) = p.won
        {
            for k in 0..p.stars {
                let at = 0.45 + k as f32 * 0.22;
                if t - dt < at && t >= at {
                    self.sounds.push(Sfx::Star(k));
                }
            }
        }
    }

    pub fn pointer(&mut self, ev: PointerEvent, x: f32, y: f32) {
        self.pointer = (x, y);
        self.since_input = 0.0;
        self.dirty = true;
        self.learn_pointer(ev, x, y);
        match ev {
            PointerEvent::Down => {
                self.down = Some((x, y));
                if let Screen::Play(p) = &mut self.screen
                    && p.won.is_none()
                    && let Some(b) = p.view.block_at(&p.game, x, y)
                {
                    p.drag = Some(Drag { block: b, x0: x, y0: y, done: false });
                    p.view.select(Some(b));
                    p.intro = p.intro.filter(|_| p.game.moves < 1);
                }
                if let Screen::Levels { scroll, drag } | Screen::Notebook { scroll, drag } = &mut self.screen {
                    *drag = Some((y, *scroll, 0.0));
                }
            }
            PointerEvent::Move => {
                let mut try_move = None;
                if let Screen::Play(p) = &mut self.screen
                    && let Some(d) = &mut p.drag
                    && !d.done
                {
                    let (dx, dy) = (x - d.x0, y - d.y0);
                    let th = p.view.cell * 0.28;
                    if dx.abs().max(dy.abs()) > th {
                        let dir = if dx.abs() > dy.abs() {
                            if dx > 0.0 { Dir::E } else { Dir::W }
                        } else if dy > 0.0 {
                            Dir::S
                        } else {
                            Dir::N
                        };
                        d.done = true;
                        try_move = Some(Move { block: d.block as u8, dir });
                    }
                }
                if let Some(mv) = try_move {
                    self.play_move(mv);
                }
                if let Screen::Levels { scroll, drag: Some((y0, s0, moved)) } | Screen::Notebook { scroll, drag: Some((y0, s0, moved)) } = &mut self.screen {
                    *scroll = (*s0 - (y - *y0)).max(0.0);
                    *moved = moved.max((y - *y0).abs());
                }
            }
            PointerEvent::Up | PointerEvent::Cancel => {
                let mut scrolled = false;
                if let Screen::Levels { drag, .. } | Screen::Notebook { drag, .. } = &mut self.screen {
                    scrolled = drag.is_some_and(|d| d.2 > 12.0 * self.u);
                    *drag = None;
                }
                if let Screen::Play(p) = &mut self.screen
                    && let Some(d) = p.drag.take()
                    && d.done
                {
                    p.view.select(None);
                }
                if ev == PointerEvent::Up
                    && !scrolled
                    && let Some((dx, dy)) = self.down
                {
                    let hit = self.hits.iter().rev().find(|h| inside(h, x, y) && inside(h, dx, dy)).map(|h| h.a);
                    if let Some(a) = hit {
                        self.sfx(Sfx::Tap);
                        self.act(a);
                    }
                }
                self.down = None;
            }
        }
    }

    /// Opens a screen by name (screenshots and tests): `learn`, `session`, `placement`, `notebook`, `mode:<n>`,
    /// `level:<n>`, `go`, `next`.
    #[doc(hidden)]
    pub fn open(&mut self, what: &str) {
        let a = match what {
            "learn" => Action::Learn,
            "session" => Action::LearnSession,
            "placement" => Action::Placement,
            "notebook" => Action::Notebook,
            "go" => Action::LearnGo,
            "next" => Action::LearnNext,
            "home" => Action::LearnHome,
            "settings" => Action::Settings,
            _ => {
                if let Some(n) = what.strip_prefix("mode:").and_then(|n| n.parse::<usize>().ok()) {
                    Action::LearnMode(Mode::MENU[n % Mode::MENU.len()])
                } else if let Some(n) = what.strip_prefix("level:").and_then(|n| n.parse::<usize>().ok()) {
                    Action::Open(n)
                } else {
                    return;
                }
            }
        };
        self.act(a);
    }

    /// Name of the current screen (tests).
    #[doc(hidden)]
    pub fn screen_name(&self) -> &'static str {
        match &self.screen {
            Screen::Home => "home",
            Screen::Levels { .. } => "levels",
            Screen::Play(_) => "play",
            Screen::Settings => "settings",
            Screen::Credits => "credits",
            Screen::LearnHome => "learn",
            Screen::LearnIntro(_) => "intro",
            Screen::LearnPlay(_) => "round",
            Screen::Notebook { .. } => "notebook",
            Screen::Placed(_) => "placed",
        }
    }

    /// Plays the hint move (screenshots and tests).
    #[doc(hidden)]
    pub fn auto_move(&mut self) -> bool {
        let mv = match &self.screen {
            Screen::LearnPlay(p) => fuera_core::solver::hint(&p.game.level, &p.game.state, 200_000),
            Screen::Play(p) => fuera_core::solver::hint(&p.game.level, &p.game.state, 200_000),
            _ => None,
        };
        match (mv, &self.screen) {
            (Some(m), Screen::LearnPlay(_)) => {
                self.learn_move(m);
                true
            }
            (Some(m), Screen::Play(_)) => {
                self.play_move(m);
                true
            }
            _ => false,
        }
    }

    pub fn key(&mut self, k: KeyEvent) {
        self.since_input = 0.0;
        self.dirty = true;
        match k {
            KeyEvent::Back => self.act(Action::Back),
            KeyEvent::Undo => self.act(Action::Undo),
            KeyEvent::Restart => self.act(Action::Restart),
            KeyEvent::Hint => self.act(Action::Hint),
            KeyEvent::Enter => {
                if let Screen::Play(p) = &self.screen {
                    if p.won.is_some() {
                        self.act(Action::Next);
                    }
                } else if let Screen::Home = self.screen {
                    self.act(Action::Continue);
                }
            }
            KeyEvent::NextBlock => {
                if let Screen::Play(p) = &mut self.screen {
                    let n = p.game.state.blocks.len();
                    let start = p.view.selected.map(|s| s + 1).unwrap_or(0);
                    let next = (0..n).map(|i| (start + i) % n).find(|&i| p.game.state.blocks[i].alive);
                    p.view.select(next);
                }
            }
            KeyEvent::Arrow(d) => {
                if let Screen::LearnPlay(p) = &mut self.screen {
                    let sel = p.view.selected.or_else(|| (0..p.game.state.blocks.len()).find(|&i| p.game.state.blocks[i].alive));
                    p.view.select(sel);
                    if let Some(b) = sel {
                        self.learn_move(Move { block: b as u8, dir: d });
                    }
                    return;
                }
                let sel = if let Screen::Play(p) = &mut self.screen {
                    if p.view.selected.is_none() {
                        let first = (0..p.game.state.blocks.len()).find(|&i| p.game.state.blocks[i].alive);
                        p.view.select(first);
                        None
                    } else {
                        p.view.selected
                    }
                } else {
                    None
                };
                if let Some(b) = sel {
                    self.play_move(Move { block: b as u8, dir: d });
                }
            }
        }
    }

    fn play_move(&mut self, mv: Move) {
        let Screen::Play(p) = &mut self.screen else { return };
        if p.won.is_some() {
            return;
        }
        let before = p.game.state.clone();
        match p.game.play(mv) {
            Some(out) => {
                if p.plan.first() == Some(&mv) {
                    p.plan.remove(0);
                } else {
                    p.plan.clear();
                }
                if !matches!(out.result, Result::Exit { .. }) {
                    p.exits_in_row = 0;
                }
                p.view.animate(&p.game, &before, mv, out);
                if p.game.is_won() {
                    p.view.select(None);
                } else if !p.game.state.blocks[mv.block as usize].alive {
                    p.view.select(None);
                }
                if p.game.is_stuck() {
                    p.toast = Some((tr(self.lang, T::Stuck).to_string(), 3.0));
                } else if !p.game.is_won() && p.plan.is_empty() {
                    // Off the known route: look for one from here (small budget, so no hitch). A proof that none
                    // exists is a dead end — say so now rather than let the player wander; a route found serves
                    // the next hint at once.
                    match fuera_core::solver::solve(&p.game.level, &p.game.state, 25_000).result {
                        fuera_core::solver::Solve::Solved(m) => p.plan = m,
                        fuera_core::solver::Solve::Unsolvable => p.toast = Some((tr(self.lang, T::DeadEnd).to_string(), 3.5)),
                        fuera_core::solver::Solve::TooBig => {}
                    }
                }
            }
            None => {
                let msg = match rules::stuck(&p.game.level, &p.game.state, mv.block as usize, Some(mv.dir)) {
                    Some(Stuck::Ice(n)) => Some(tr(self.lang, T::Frozen).replace("{}", &n.to_string())),
                    Some(Stuck::Locked(n)) => Some(tr(self.lang, T::Locked).replace("{}", &n.to_string())),
                    Some(Stuck::Lane(fuera_core::Axis::H)) => Some(tr(self.lang, T::LaneH).to_string()),
                    Some(Stuck::Lane(fuera_core::Axis::V)) => Some(tr(self.lang, T::LaneV).to_string()),
                    _ => None,
                };
                if let Some(m) = msg {
                    p.toast = Some((m, 2.2));
                }
                self.sounds.push(Sfx::Nope);
            }
        }
    }

    fn act(&mut self, a: Action) {
        self.dirty = true;
        match a {
            Action::Continue => {
                let i = self.next_level();
                self.open_level(i);
            }
            Action::Levels => {
                let row = (self.next_level() / 5) as f32;
                let scroll = (row * 76.0 * self.u - self.h * 0.3).max(0.0);
                self.screen = Screen::Levels { scroll, drag: None };
            }
            Action::Learn => self.screen = Screen::LearnHome,
            Action::LearnHome
            | Action::LearnSession
            | Action::LearnMode(_)
            | Action::LearnGo
            | Action::LearnNext
            | Action::Notebook
            | Action::ToggleBase
            | Action::Placement
            | Action::SayIntro(_)
            | Action::SayMiss(_)
            | Action::Dismiss => self.learn_act(a),
            Action::Settings => self.screen = Screen::Settings,
            Action::Credits => self.screen = Screen::Credits,
            Action::Back => {
                self.screen = match &self.screen {
                    Screen::Play(_) => {
                        let row = (self.next_level() / 5) as f32;
                        Screen::Levels { scroll: (row * 76.0 * self.u - self.h * 0.3).max(0.0), drag: None }
                    }
                    Screen::Credits => Screen::Settings,
                    Screen::LearnPlay(_) | Screen::LearnIntro(_) | Screen::Notebook { .. } | Screen::Placed(_) => Screen::LearnHome,
                    Screen::Home => {
                        self.host.quit();
                        Screen::Home
                    }
                    _ => Screen::Home,
                };
            }
            Action::Undo => {
                if let Screen::LearnPlay(p) = &mut self.screen
                    && p.won.is_none()
                    && p.game.undo()
                {
                    p.view.sync(&p.game);
                }
                if let Screen::Play(p) = &mut self.screen
                    && p.won.is_none()
                    && p.game.undo()
                {
                    p.view.sync(&p.game);
                    p.toast = None;
                    p.exits_in_row = 0;
                    p.plan.clear();
                }
            }
            Action::Restart => {
                if let Screen::LearnPlay(p) = &mut self.screen
                    && p.won.is_none()
                {
                    p.game.restart();
                    p.view.sync(&p.game);
                }
                if let Screen::Play(p) = &mut self.screen {
                    p.game.restart();
                    p.view.sync(&p.game);
                    p.won = None;
                    p.toast = None;
                    p.exits_in_row = 0;
                    p.plan.clear();
                }
            }
            Action::Hint => {
                self.learn_hint();
                if let Screen::Play(p) = &mut self.screen
                    && p.won.is_none()
                {
                    if p.plan.is_empty()
                        && let fuera_core::solver::Solve::Solved(m) = fuera_core::solver::solve(&p.game.level, &p.game.state, 400_000).result
                    {
                        p.plan = m;
                    }
                    match p.plan.first().copied() {
                        Some(mv) => {
                            p.game.hints += 1;
                            p.view.show_hint(Some(mv));
                            p.view.select(Some(mv.block as usize));
                        }
                        None => p.toast = Some((tr(self.lang, T::NoHint).to_string(), 2.5)),
                    }
                }
            }
            Action::Next => {
                if let Screen::Play(p) = &self.screen {
                    let n = p.index + 1;
                    if n < self.campaign.len() {
                        self.open_level(n);
                    } else {
                        self.act(Action::Levels);
                    }
                }
            }
            Action::Replay => {
                if let Screen::Play(p) = &self.screen {
                    let i = p.index;
                    self.open_level(i);
                }
            }
            Action::Open(i) => {
                if self.unlocked(i) {
                    self.open_level(i);
                }
            }
            Action::ToggleSound => {
                let v = !self.sound_on();
                self.save.set("sound", v as u8);
                self.persist();
            }
            Action::ToggleSayAnswer => {
                let v = !self.save.get_bool("say_answer", false);
                self.save.set("say_answer", v as u8);
                self.persist();
            }
            Action::ToggleVib => {
                let v = !self.save.get_bool("vib", true);
                self.save.set("vib", v as u8);
                self.persist();
                if v {
                    self.host.vibrate(30);
                }
            }
            Action::CycleLang => {
                self.lang = self.lang.next();
                self.save.set("lang", self.lang.code());
                self.persist();
            }
        }
        self.persist();
    }

    // ---------------------------------------------------------------------------------------------- drawing

    pub fn draw(&mut self, pm: &mut Pixmap) {
        self.dirty = false;
        self.hits.clear();
        if self.bg.as_ref().is_none_or(|b| b.width() != pm.width() || b.height() != pm.height()) {
            let mut bg = Pixmap::new(pm.width(), pm.height()).unwrap();
            let h = pm.height() as f32;
            let g = gfx::vgrad(0.0, h, &[(0.0, col(theme::SEA_TOP)), (1.0, col(theme::SEA_BOTTOM))]);
            gfx::fill_rect(&mut bg, 0.0, 0.0, pm.width() as f32, h, &g);
            // Light ripples.
            let u = self.u;
            for i in 0..14 {
                let y = h * (i as f32 + 0.5) / 14.0;
                let x = (i * 97 % 41) as f32 / 41.0 * pm.width() as f32;
                let r = gfx::rrect(x - 40.0 * u, y, 80.0 * u, 4.0 * u, 2.0 * u);
                fill(&mut bg, &r, &solid(col_a(theme::WHITE, 0.035)), Transform::identity());
            }
            self.bg = Some(bg);
        }
        pm.copy_from_slice_or_draw(self.bg.as_ref().unwrap());
        match &self.screen {
            Screen::Home => self.draw_home(pm),
            Screen::Levels { .. } => self.draw_levels(pm),
            Screen::Play(_) => self.draw_play(pm),
            Screen::Settings => self.draw_settings(pm),
            Screen::Credits => self.draw_credits(pm),
            Screen::LearnHome => self.draw_learn_home(pm),
            Screen::LearnIntro(_) => self.draw_learn_intro(pm),
            Screen::LearnPlay(_) => self.draw_learn_play(pm),
            Screen::Notebook { .. } => self.draw_notebook(pm),
            Screen::Placed(b) => {
                let b = *b;
                self.draw_placed(pm, b)
            }
        }
    }

    fn button(&mut self, pm: &mut Pixmap, a: Action, x: f32, y: f32, w: f32, h: f32, label: &str, icon: Option<Icon>, style: Style) {
        let u = self.u;
        let pressed = self.down.is_some_and(|(dx, dy)| {
            let hh = Hit { x, y, w, h, a };
            inside(&hh, dx, dy) && inside(&hh, self.pointer.0, self.pointer.1)
        });
        let off = if pressed { 3.0 * u } else { 0.0 };
        let r = h * 0.38;
        let (face, ink) = match style {
            Style::Primary => (theme::ACCENT, theme::WHITE),
            Style::Secondary => (theme::BUTTON, theme::INK),
            Style::Ghost => ((0x2B, 0x6A, 0x8C), theme::WHITE),
        };
        let id = Transform::identity();
        fill(pm, &gfx::rrect(x, y + 5.0 * u, w, h, r), &solid(col(shade(face, -0.3))), id);
        fill(pm, &gfx::rrect(x, y + off, w, h, r), &gfx::vgrad(y, y + h, &[(0.0, col(shade(face, 0.12))), (1.0, col(face))]), id);
        let cy = y + off + h / 2.0;
        let px = (h * 0.42).min(24.0 * u);
        if label.is_empty() {
            if let Some(ic) = icon {
                icons::draw(pm, ic, x + w / 2.0, cy, h * 0.62, col(ink));
            }
        } else {
            let mut tx = x + w / 2.0;
            if let Some(ic) = icon {
                let tw = self.fonts.width(Face::Title, label, px);
                let total = tw + h * 0.55;
                let ix = x + (w - total) / 2.0 + h * 0.22;
                icons::draw(pm, ic, ix, cy, h * 0.5, col(ink));
                tx = ix + h * 0.33 + tw / 2.0;
            }
            let fpx = self.fonts.fit(Face::Title, label, px, w - h * 0.6);
            self.fonts.draw(pm, Face::Title, label, tx, cy + fpx * 0.36, fpx, col(ink), Align::Center);
        }
        self.hits.push(Hit { x, y, w, h: h + 5.0 * u, a });
    }

    fn round_button(&mut self, pm: &mut Pixmap, a: Action, cx: f32, cy: f32, d: f32, icon: Icon, label: Option<&str>) {
        self.button(pm, a, cx - d / 2.0, cy - d / 2.0, d, d, "", Some(icon), Style::Secondary);
        if let Some(l) = label {
            let px = 13.0 * self.u;
            self.fonts.draw_shadowed(pm, Face::Bold, l, cx, cy + d / 2.0 + 22.0 * self.u, px, col(theme::WHITE), Align::Center);
        }
    }

    fn draw_logo(&mut self, pm: &mut Pixmap, cx: f32, y: f32, px: f32) {
        // "¡Fuera!" with each letter on its own brick colour.
        let word = "¡Fuera!";
        let total = self.fonts.width(Face::Title, word, px);
        let mut x = cx - total / 2.0;
        for (i, ch) in word.chars().enumerate() {
            let s = ch.to_string();
            let w = self.fonts.width(Face::Title, &s, px);
            let rgb = theme::KEY_COLORS[[0, 7, 2, 3, 1, 4, 5][i % 7]];
            let bob = (self.time * 2.2 + i as f32 * 0.6).sin() * px * 0.03;
            self.fonts.draw(pm, Face::Title, &s, x, y + px * 0.08 + bob, px, col(shade(rgb, -0.55)), Align::Left);
            self.fonts.draw(pm, Face::Title, &s, x, y + bob, px, col(rgb), Align::Left);
            x += w;
        }
    }

    fn draw_home(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let cx = w / 2.0;
        self.draw_logo(pm, cx, self.top + h * 0.2, 92.0 * u);
        let tag = self.t(T::Tagline);
        let lines = self.fonts.wrap(Face::Bold, tag, 18.0 * u, w * 0.8);
        for (i, l) in lines.iter().enumerate() {
            self.fonts.draw_shadowed(pm, Face::Bold, l, cx, self.top + h * 0.2 + 46.0 * u + i as f32 * 24.0 * u, 18.0 * u, col_a(theme::WHITE, 0.9), Align::Center);
        }
        // Decorative bricks sliding out of a door.
        self.draw_home_scene(pm, cx, h * 0.42);
        let bw = (300.0 * u).min(w * 0.84);
        let bh = 62.0 * u;
        let mut y = h * 0.58;
        let n = self.next_level();
        let play = format!("{}  ·  {} {}", self.t(T::Play), self.t(T::Level), n + 1);
        self.button(pm, Action::Continue, cx - bw / 2.0, y, bw, bh * 1.1, &play, Some(Icon::Play), Style::Primary);
        y += bh * 1.1 + 18.0 * u;
        let lv = self.t(T::Levels);
        self.button(pm, Action::Levels, cx - bw / 2.0, y, bw, bh, lv, Some(Icon::Star), Style::Secondary);
        y += bh + 16.0 * u;
        let learn = self.t(T::LearnSpanish);
        self.button(pm, Action::Learn, cx - bw / 2.0, y, bw, bh, learn, Some(Icon::Book), Style::Secondary);
        y += bh + 16.0 * u;
        let st = self.t(T::Settings);
        self.button(pm, Action::Settings, cx - bw / 2.0, y, bw, bh, st, Some(Icon::Gear), Style::Ghost);
        let total: u32 = (0..self.campaign.len()).map(|i| self.stars_of(i) as u32).sum();
        let s = format!("{}  {} / {}", self.t(T::StarsTotal), total, self.campaign.len() * 3);
        self.fonts.draw_shadowed(pm, Face::Bold, &s, cx, h - self.bottom - 24.0 * u, 15.0 * u, col_a(theme::WHITE, 0.8), Align::Center);
    }

    fn draw_home_scene(&mut self, pm: &mut Pixmap, cx: f32, cy: f32) {
        let u = self.u;
        let c = 44.0 * u;
        let id = Transform::identity();
        // A short plaster lane with a door; bricks keep sliding out.
        let lane_w = c * 5.0;
        let x0 = cx - lane_w / 2.0;
        fill(pm, &gfx::rrect(x0 - c * 0.4, cy - c * 0.9, lane_w + c * 0.8, c * 1.8, c * 0.5), &solid(col(theme::PLASTER_SIDE)), id);
        fill(pm, &gfx::rrect(x0 - c * 0.4, cy - c * 0.95, lane_w + c * 0.8, c * 1.75, c * 0.5), &solid(col(theme::PLASTER)), id);
        fill(pm, &gfx::rrect(x0, cy - c * 0.55, lane_w, c, c * 0.2), &solid(col(theme::TILE_A)), id);
        let door = theme::KEY_COLORS[(self.time / 1.6) as usize % 8];
        fill(pm, &gfx::rrect(x0 + lane_w + c * 0.05, cy - c * 0.5, c * 0.32, c * 0.9, c * 0.1), &solid(col(door)), id);
        let ch = gfx::chevron(x0 + lane_w + c * 0.21, cy - c * 0.05, c * 0.22, 1.0, 0.0);
        fill(pm, &ch, &solid(col(theme::WHITE)), id);
        let t = (self.time % 1.6) / 1.6;
        let k = (self.time / 1.6) as usize % 8;
        let x = x0 + gfx::ease_in_out(t) * (lane_w + c * 0.5);
        let a = if x > x0 + lane_w - c { 1.0 - (x - (x0 + lane_w - c)) / (c * 1.5) } else { 1.0 };
        let look = crate::board::Look::key(k as u16);
        let spr = crate::board::render_brick(&[fuera_core::Pos::new(0, 0)], c, &look, None);
        gfx::blit(pm, &spr, (x - c * 0.18).round(), (cy - c * 0.55 - c * 0.18).round(), a.clamp(0.0, 1.0), 1.0);
    }

    fn draw_header(&mut self, pm: &mut Pixmap, title: &str) -> f32 {
        let u = self.u;
        let y = self.top + 40.0 * u;
        self.round_button(pm, Action::Back, 36.0 * u, y, 48.0 * u, Icon::Back, None);
        self.fonts.draw_shadowed(pm, Face::Title, title, self.w / 2.0, y + 11.0 * u, 30.0 * u, col(theme::WHITE), Align::Center);
        y + 40.0 * u
    }

    fn draw_levels(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let scroll = if let Screen::Levels { scroll, .. } = &self.screen { *scroll } else { 0.0 };
        let cols = 5usize;
        let cell = ((self.w - 32.0 * u) / cols as f32).min(80.0 * u);
        let size = cell * 0.82;
        let x0 = (self.w - cell * cols as f32) / 2.0;
        let y0 = self.top + 110.0 * u - scroll;
        let next = self.next_level();
        for i in 0..self.campaign.len() {
            let (cx, cy) = (x0 + (i % cols) as f32 * cell + cell / 2.0, y0 + (i / cols) as f32 * (cell + 10.0 * u) + cell / 2.0);
            if cy < self.top + 60.0 * u || cy > self.h + cell {
                continue;
            }
            let stars = self.stars_of(i);
            let open = self.unlocked(i);
            let rgb = if !open {
                (0x5A, 0x7B, 0x90)
            } else if i == next {
                theme::ACCENT
            } else {
                theme::PLASTER
            };
            let id = Transform::identity();
            fill(pm, &gfx::rrect(cx - size / 2.0, cy - size / 2.0 + 4.0 * u, size, size, size * 0.26), &solid(col(shade(rgb, -0.3))), id);
            fill(pm, &gfx::rrect(cx - size / 2.0, cy - size / 2.0, size, size, size * 0.26), &solid(col(rgb)), id);
            if open {
                let ink = if i == next { theme::WHITE } else { theme::INK };
                self.fonts.draw(pm, Face::Title, &(i + 1).to_string(), cx, cy + size * 0.12, size * 0.38, col(ink), Align::Center);
                for k in 0..3 {
                    let sx = cx + (k as f32 - 1.0) * size * 0.22;
                    let c = if k < stars { theme::GOLD } else { shade(rgb, -0.15) };
                    fill(pm, &icons::star_path(sx, cy + size * 0.3, size * 0.11), &solid(col(c)), id);
                }
            } else {
                icons::draw(pm, Icon::Lock, cx, cy, size * 0.5, col_a(theme::WHITE, 0.6));
            }
            self.hits.push(Hit { x: cx - size / 2.0, y: cy - size / 2.0, w: size, h: size, a: Action::Open(i) });
        }
        // Header over the list.
        let g = gfx::vgrad(0.0, self.top + 100.0 * u, &[(0.0, col(theme::SEA_TOP)), (0.8, col(theme::SEA_TOP)), (1.0, col_a(theme::SEA_TOP, 0.0))]);
        gfx::fill_rect(pm, 0.0, 0.0, self.w, self.top + 100.0 * u, &g);
        let title = self.t(T::Levels);
        self.draw_header(pm, title);
        // Keep the scroll within the list.
        let rows = self.campaign.len().div_ceil(cols) as f32;
        let max = (rows * (cell + 10.0 * u) + 140.0 * u - self.h).max(0.0);
        if let Screen::Levels { scroll, .. } = &mut self.screen {
            *scroll = scroll.min(max);
        }
    }

    fn draw_play(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let top = self.top;
        let bottom = self.bottom;
        let lang = self.lang;
        let Screen::Play(p) = &mut self.screen else { return };
        let board_top = top + 120.0 * u;
        let board_bottom = h - bottom - 130.0 * u;
        let margin = 14.0 * u;
        p.view.layout(&p.game.level, margin, board_top, w - 2.0 * margin, board_bottom - board_top);
        p.view.draw(pm, &p.game, &mut self.fonts);
        // Moves counter.
        let opt = p.game.level.meta.optimal;
        let moves = p.game.moves;
        let idx = p.index;
        let won = p.won;
        let stars = p.stars;
        let toast = p.toast.clone();
        let intro = p.intro;
        let can_undo = p.game.can_undo();
        let title = format!("{} {}", tr(lang, T::Level), idx + 1);
        let y = self.draw_header(pm, &title);
        let line = match opt {
            Some(o) => format!("{} {}  ·  {} {}", tr(lang, T::Moves), moves, tr(lang, T::Goal), o),
            None => format!("{} {}", tr(lang, T::Moves), moves),
        };
        self.fonts.draw_shadowed(pm, Face::Bold, &line, w / 2.0, y + 6.0 * u, 16.0 * u, col_a(theme::WHITE, 0.85), Align::Center);
        if let Some(t) = intro {
            let s = tr(lang, t);
            self.pill(pm, s, w / 2.0, y + 34.0 * u, theme::GOLD, theme::INK);
        }
        // Bottom bar.
        let by = h - bottom - 82.0 * u;
        let d = 58.0 * u;
        let gap = (w / 4.0).min(110.0 * u);
        self.round_button(pm, Action::Undo, w / 2.0 - gap, by, d, Icon::Undo, Some(tr(lang, T::Undo)));
        self.round_button(pm, Action::Restart, w / 2.0, by, d, Icon::Restart, Some(tr(lang, T::Restart)));
        self.round_button(pm, Action::Hint, w / 2.0 + gap, by, d, Icon::Hint, Some(tr(lang, T::Hint)));
        let _ = can_undo;
        if let Some((msg, _)) = toast {
            self.pill(pm, &msg, w / 2.0, board_bottom + 4.0 * u, theme::WHITE, theme::INK);
        }
        if let Some(t) = won {
            self.draw_win(pm, t, stars, moves, opt);
        }
    }

    fn pill(&mut self, pm: &mut Pixmap, text: &str, cx: f32, cy: f32, bg: (u8, u8, u8), ink: (u8, u8, u8)) {
        let u = self.u;
        let px = 15.0 * u;
        let lines = self.fonts.wrap(Face::Bold, text, px, self.w - 60.0 * u);
        let tw = lines.iter().map(|l| self.fonts.width(Face::Bold, l, px)).fold(0.0, f32::max);
        let lh = px * 1.3;
        let (bw, bh) = (tw + 28.0 * u, lines.len() as f32 * lh + 14.0 * u);
        fill(pm, &gfx::rrect(cx - bw / 2.0, cy - bh / 2.0 + 3.0 * u, bw, bh, bh.min(40.0 * u) / 2.0), &solid(col_a((0, 0, 0), 0.25)), Transform::identity());
        fill(pm, &gfx::rrect(cx - bw / 2.0, cy - bh / 2.0, bw, bh, bh.min(40.0 * u) / 2.0), &solid(col(bg)), Transform::identity());
        for (i, l) in lines.iter().enumerate() {
            self.fonts.draw(pm, Face::Bold, l, cx, cy - bh / 2.0 + 7.0 * u + (i as f32 + 0.78) * lh, px, col(ink), Align::Center);
        }
    }

    fn draw_win(&mut self, pm: &mut Pixmap, t: f32, stars: u8, moves: u32, opt: Option<u16>) {
        let u = self.u;
        let (w, h) = (self.w, self.h);
        let a = (t / 0.35).min(1.0);
        gfx::fill_rect(pm, 0.0, 0.0, w, h, &solid(col_a((5, 20, 35), 0.45 * a)));
        let cw = (330.0 * u).min(w * 0.9);
        let ch = 330.0 * u;
        let pop = gfx::ease_back((t / 0.4).min(1.0));
        let cy = h * 0.45 + (1.0 - pop) * 60.0 * u;
        let x = w / 2.0 - cw / 2.0;
        let y = cy - ch / 2.0;
        let id = Transform::identity();
        fill(pm, &gfx::rrect(x, y + 8.0 * u, cw, ch, 28.0 * u), &solid(col_a((0, 0, 0), 0.3 * a)), id);
        fill(pm, &gfx::rrect(x, y, cw, ch, 28.0 * u), &solid(col_a(theme::PLASTER, a)), id);
        let title = self.t(T::Cleared);
        self.fonts.draw(pm, Face::Title, title, w / 2.0, y + 62.0 * u, 44.0 * u, col_a(theme::ACCENT, a), Align::Center);
        for k in 0..3u8 {
            let at = 0.45 + k as f32 * 0.22;
            let sx = w / 2.0 + (k as f32 - 1.0) * 70.0 * u;
            let sy = y + 125.0 * u - if k == 1 { 12.0 * u } else { 0.0 };
            fill(pm, &icons::star_path(sx, sy, 32.0 * u), &solid(col_a(shade(theme::PLASTER, -0.15), a)), id);
            if k < stars && t >= at {
                let s = gfx::ease_back(((t - at) / 0.3).min(1.0));
                let p = icons::star_path(sx, sy, 32.0 * u * s);
                fill(pm, &p, &solid(col(theme::GOLD)), id);
                stroke(pm, &p, &solid(col(shade(theme::GOLD, -0.35))), 3.0 * u, id);
            }
        }
        let line = match opt {
            Some(o) => format!("{} {}  ·  {} {}", self.t(T::Moves), moves, self.t(T::Best), o),
            None => format!("{} {}", self.t(T::Moves), moves),
        };
        self.fonts.draw(pm, Face::Bold, &line, w / 2.0, y + 190.0 * u, 17.0 * u, col_a(theme::INK, a), Align::Center);
        let bw = cw * 0.4;
        let bh = 54.0 * u;
        let next = self.t(T::Next);
        let replay = self.t(T::Replay);
        self.button(pm, Action::Replay, x + cw * 0.07, y + ch - bh - 30.0 * u, bw, bh, replay, Some(Icon::Restart), Style::Secondary);
        self.button(pm, Action::Next, x + cw * 0.53, y + ch - bh - 30.0 * u, bw, bh, next, Some(Icon::Play), Style::Primary);
    }

    fn draw_settings(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let title = self.t(T::Settings);
        let mut y = self.draw_header(pm, title) + 30.0 * u;
        let bw = (320.0 * u).min(self.w * 0.86);
        let bh = 58.0 * u;
        let x = self.w / 2.0 - bw / 2.0;
        let lang = format!("{} : {}", self.t(T::Language), self.lang.name());
        self.button(pm, Action::CycleLang, x, y, bw, bh, &lang, None, Style::Secondary);
        y += bh + 16.0 * u;
        let on = |b: bool, s: &Self| if b { s.t(T::On) } else { s.t(T::Off) };
        let snd = format!("{} : {}", self.t(T::Sound), on(self.sound_on(), self));
        self.button(pm, Action::ToggleSound, x, y, bw, bh, &snd, Some(Icon::Speaker), Style::Secondary);
        y += bh + 16.0 * u;
        let vib = format!("{} : {}", self.t(T::Vibration), on(self.save.get_bool("vib", true), self));
        self.button(pm, Action::ToggleVib, x, y, bw, bh, &vib, None, Style::Secondary);
        y += bh + 16.0 * u;
        let say = format!("{} : {}", self.t(T::SayAnswer), on(self.save.get_bool("say_answer", false), self));
        self.button(pm, Action::ToggleSayAnswer, x, y, bw, bh, &say, Some(Icon::Speaker), Style::Secondary);
        y += bh + 16.0 * u;
        let cr = self.t(T::Credits);
        self.button(pm, Action::Credits, x, y, bw, bh, cr, Some(Icon::Book), Style::Ghost);
        let v = concat!("¡Fuera! ", env!("CARGO_PKG_VERSION"));
        self.fonts.draw_shadowed(pm, Face::Bold, v, self.w / 2.0, self.h - self.bottom - 24.0 * u, 14.0 * u, col_a(theme::WHITE, 0.7), Align::Center);
    }

    fn draw_credits(&mut self, pm: &mut Pixmap) {
        let u = self.u;
        let title = self.t(T::Credits);
        let mut y = self.draw_header(pm, title) + 30.0 * u;
        let text = CREDITS;
        let px = 15.0 * u;
        for para in text.split("\n\n") {
            for l in self.fonts.wrap(Face::Body, para, px, self.w - 48.0 * u) {
                self.fonts.draw(pm, Face::Body, &l, 24.0 * u, y, px, col(theme::WHITE), Align::Left);
                y += px * 1.4;
            }
            y += px * 0.8;
        }
    }
}

const CREDITS: &str = "¡Fuera! — free and open-source game (MIT licence) by Guiwald, written in Rust. github.com/Guiwald/fuera\n\nFonts: Lilita One (Juan Montoreano) and Atkinson Hyperlegible (Braille Institute), SIL Open Font License 1.1.\n\nSpanish content: written for ¡Fuera!, checked against the English Wiktionary (kaikki.org) and ordered with Hermit Dave's FrequencyWords (OpenSubtitles 2018); released under CC BY-SA 4.0.\n\nInspired by colour-sorting block puzzles. All levels, graphics and sounds are original.";

fn inside(h: &Hit, x: f32, y: f32) -> bool {
    x >= h.x && x <= h.x + h.w && y >= h.y && y <= h.y + h.h
}

trait CopyBg {
    fn copy_from_slice_or_draw(&mut self, bg: &Pixmap);
}

impl CopyBg for Pixmap {
    fn copy_from_slice_or_draw(&mut self, bg: &Pixmap) {
        if bg.width() == self.width() && bg.height() == self.height() {
            self.data_mut().copy_from_slice(bg.data());
        } else {
            self.fill(Color::BLACK);
        }
    }
}
