//! `fuera-cli`: level tools.
//!
//! ```text
//! fuera-cli solve <file.lvl>              optimal solution and analysis
//! fuera-cli check <pack.txt>              validate and solve every level of a pack
//! fuera-cli gen [k=v ...]                 one random level (w h outline colors fill shapes walls ice lanes
//!                                         shells locks shutters seed attempts target minpark)
//! fuera-cli campaign <out.txt> [seed]     build the 100-level campaign
//! fuera-cli conj-fr < infinitives         French paradigms (infinitive, tense, person, form), for the data checks
//! ```

use std::time::Instant;

use fuera_core::generator::{self, Outline, Params, Rng};
use fuera_core::solver::{self, Solve};
use fuera_core::{Level, State};

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    match a.first().map(|s| s.as_str()) {
        Some("solve") => solve(&a[1]),
        Some("check") => check(&a[1]),
        Some("gen") => gen_one(&a[1..]),
        Some("bench") => bench(&a[1..]),
        Some("hints") => hints(&a[1]),
        Some("campaign") => campaign(&a[1], a.get(2).and_then(|s| s.parse().ok()).unwrap_or(2026)),
        Some("prove") => prove(&a[1..]),
        Some("deadends") => deadends(&a[1]),
        Some("voices") => voices(&a[1], &a[2], &a[3]),
        Some("voice") => voice_wav(&a[1], &a[2], &a[3]),
        Some("conj-fr") => conj_fr(),
        Some("words") => words(&a[1]),
        _ => {
            eprintln!("usage: fuera-cli solve <lvl> | check <pack> | gen [k=v…] | campaign <out> [seed]");
            std::process::exit(2);
        }
    }
}

/// Exhaustive dead-end proofs for the language board layouts (see `fuera_core::lattice`).
/// `fuera-cli prove` checks the layouts the game uses (fails if one has a dead end);
/// `fuera-cli prove compare [max tiles]` also shows the layouts that were rejected, with a trap for each.
fn prove(a: &[String]) {
    use fuera_core::lattice::{End, Geometry, Layout, production, prove};
    let report = |name: &str, g: &Geometry, max: usize| -> bool {
        let t = Instant::now();
        let p = prove(g, max);
        let verdict = match &p.dead {
            None => format!("aucune impasse jusqu'à {max} tuiles"),
            Some(dead) => format!("IMPASSE avec {} tuiles :\n{}", dead.len(), g.draw(dead)),
        };
        println!("{name} ({}x{}) — {verdict}\n    positions examinées {:?}, au plus {} coups avant qu'une tuile puisse sortir, {:.1?}", g.cols, g.rows, p.positions, p.longest, t.elapsed());
        p.dead.is_none()
    };
    if a.first().map(|s| s.as_str()) == Some("compare") {
        let max: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
        let (w, d) = (End::Wall, End::Door);
        let rejected = [
            ("4 portes sur 2 colonnes (première version)", Geometry { cols: 2, rows: 7, top: vec![d(0), d(1)], bottom: vec![d(2), d(3)] }),
            ("5 portes sur 3 colonnes (première version)", Geometry { cols: 3, rows: 7, top: vec![d(0), d(1), d(2)], bottom: vec![d(3), d(4), w] }),
            ("2 catégories, portes sur toute la largeur, tuiles de 2", Geometry { cols: 3, rows: 7, top: vec![d(0); 3], bottom: vec![d(1); 3] }),
            ("2 catégories, portes sur toute la largeur, tuiles de 3", Geometry { cols: 2, rows: 7, top: vec![d(0); 2], bottom: vec![d(1); 2] }),
        ];
        for (name, g) in rejected {
            report(name, &g, max);
        }
        for (name, l, _) in production() {
            report(name, &l.geometry, max);
        }
        return;
    }
    let mut ok = true;
    for (name, l, max) in production() {
        let l: Layout = l;
        ok &= report(name, &l.geometry, max);
    }
    if !ok {
        std::process::exit(1);
    }
}

fn solve(path: &str) {
    let level = Level::parse(&std::fs::read_to_string(path).expect("read")).expect("parse");
    for e in level.validate() {
        println!("invalid: {e}");
    }
    let t = Instant::now();
    let r = solver::solve(&level, &State::initial(&level), 5_000_000);
    println!("explored {} states in {:?}", r.explored, t.elapsed());
    match r.result {
        Solve::Solved(m) => {
            println!("optimal: {} moves", m.len());
            let s: Vec<String> = m.iter().map(|mv| format!("{}{}", level.blocks[mv.block as usize].name, mv.dir.letter())).collect();
            println!("{}", s.join(" "));
        }
        other => println!("{other:?}"),
    }
    if let Some(g) = generator::grade(&level, 2_000_000) {
        println!("{g:#?}");
    }
}

/// Builds the voice pack from a manifest (`text<TAB>speaker<TAB>licence<TAB>commons file<TAB>wav`) and a folder of
/// 16-bit mono WAV files at 16 kHz (see `assets/es/voces/README.md`).
fn voices(manifest: &str, dir: &str, out: &str) {
    use fuera_learn::voice;
    let mut speakers: Vec<String> = Vec::new();
    let mut clips = Vec::new();
    let mut seconds = 0.0;
    for line in std::fs::read_to_string(manifest).expect("manifest").lines().skip(1) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 {
            continue;
        }
        let Some(pcm) = read_wav16(&std::path::Path::new(dir).join(f[4])) else {
            eprintln!("skipped (not 16 kHz mono 16-bit): {}", f[4]);
            continue;
        };
        let sp = match speakers.iter().position(|s| s == f[1]) {
            Some(i) => i,
            None => {
                speakers.push(f[1].to_string());
                speakers.len() - 1
            }
        };
        seconds += pcm.len() as f64 / 16_000.0;
        clips.push((f[0].to_string(), sp as u16, pcm));
    }
    let pack = voice::encode(16_000, &speakers, &clips);
    std::fs::write(out, &pack).expect("write");
    let v = voice::Voices::parse(pack.clone()).expect("reparse");
    println!("{} recordings, {} speakers, {:.0} s of audio, {:.1} MB", v.len(), speakers.len(), seconds, pack.len() as f64 / 1e6);
}

/// Writes one recording of a pack to a WAV file (to listen to it or compare it with its source).
fn voice_wav(pack: &str, text: &str, out: &str) {
    let v = fuera_learn::voice::Voices::parse(std::fs::read(pack).expect("pack")).expect("not a voice pack");
    let Some(pcm) = v.clip(text) else {
        eprintln!("no recording of « {text} »");
        std::process::exit(1);
    };
    let mut w = Vec::new();
    let n = (pcm.len() * 2) as u32;
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + n).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes());
    w.extend_from_slice(&v.rate().to_le_bytes());
    w.extend_from_slice(&(v.rate() * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&n.to_le_bytes());
    for s in pcm {
        w.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(out, w).expect("write");
    println!("{text} — {}", v.speaker(text).unwrap_or("?"));
}

fn read_wav16(path: &std::path::Path) -> Option<Vec<i16>> {
    let d = std::fs::read(path).ok()?;
    if d.get(0..4)? != b"RIFF" || d.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut at = 12;
    let mut ok = false;
    while at + 8 <= d.len() {
        let id = &d[at..at + 4];
        let len = u32::from_le_bytes(d[at + 4..at + 8].try_into().ok()?) as usize;
        let body = d.get(at + 8..(at + 8 + len).min(d.len()))?;
        if id == b"fmt " {
            let ch = u16::from_le_bytes(body.get(2..4)?.try_into().ok()?);
            let rate = u32::from_le_bytes(body.get(4..8)?.try_into().ok()?);
            let bits = u16::from_le_bytes(body.get(14..16)?.try_into().ok()?);
            ok = ch == 1 && rate == 16_000 && bits == 16;
        } else if id == b"data" {
            return ok.then(|| body.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect());
        }
        at += 8 + len + (len & 1);
    }
    None
}

/// Dead ends of every level of a pack: reachable positions from which the level can no longer be won.
fn deadends(path: &str) {
    let text = std::fs::read_to_string(path).expect("read");
    let (mut levels, mut with_dead, mut losing_start, mut incomplete) = (0, 0, 0, 0);
    for chunk in text.split_inclusive("\nend\n") {
        let Ok(level) = Level::parse(chunk) else { continue };
        levels += 1;
        let a = solver::analyze(&level, 120_000);
        if !a.complete {
            incomplete += 1;
        }
        if a.dead_states > 0 {
            with_dead += 1;
        }
        if a.losing_first_moves > 0 {
            losing_start += 1;
        }
        println!(
            "{levels:3} {:>12}  états {:7}{}  impasses {:6} ({:4.1} %)  premiers coups perdants {}/{}",
            level.meta.name,
            a.states,
            if a.complete { " " } else { "+" },
            a.dead_states,
            100.0 * a.dead_states as f64 / a.states.max(1) as f64,
            a.losing_first_moves,
            a.first_moves
        );
    }
    println!("{levels} niveaux : {with_dead} avec des impasses, {losing_start} où un premier coup peut déjà perdre, {incomplete} non explorés en entier");
}

fn check(path: &str) {
    let text = std::fs::read_to_string(path).expect("read");
    let mut cur = String::new();
    let mut n = 0;
    let mut bad = 0;
    for line in text.lines() {
        cur.push_str(line);
        cur.push('\n');
        if line.trim() != "end" {
            continue;
        }
        n += 1;
        let level = match Level::parse(&cur) {
            Ok(l) => l,
            Err(e) => {
                println!("{n}: parse error {e}");
                bad += 1;
                cur.clear();
                continue;
            }
        };
        cur.clear();
        let errs = level.validate();
        let r = solver::solve(&level, &State::initial(&level), 3_000_000);
        let ok = match &r.result {
            Solve::Solved(m) => level.meta.optimal.is_none_or(|o| o as usize == m.len()),
            _ => false,
        };
        let opt = match &r.result {
            Solve::Solved(m) => m.len().to_string(),
            o => format!("{o:?}"),
        };
        println!("{n:3} {:<22} blocks {:2}  optimal {:>4} (file {:?})  states {:7} {}", level.meta.name, level.blocks.len(), opt, level.meta.optimal, r.explored, if ok && errs.is_empty() { "ok" } else { "PROBLEM" });
        if !ok || !errs.is_empty() {
            bad += 1;
            for e in errs {
                println!("     {e}");
            }
        }
    }
    println!("{n} levels, {bad} problem(s)");
    if bad > 0 {
        std::process::exit(1);
    }
}

fn parse_params(args: &[String]) -> (Params, u64, usize, f32, u16) {
    let mut p = Params::default();
    let (mut seed, mut attempts, mut target, mut minpark) = (1u64, 200usize, 4.0f32, 1u16);
    for a in args {
        let Some((k, v)) = a.split_once('=') else { continue };
        match k {
            "w" => p.w = v.parse().unwrap(),
            "h" => p.h = v.parse().unwrap(),
            "outline" => p.outline = outline(v),
            "colors" => p.colors = v.parse().unwrap(),
            "fill" => p.fill = v.parse().unwrap(),
            "shapes" => p.shapes = generator::shapes_by_name(v),
            "walls" => p.walls = v.parse().unwrap(),
            "ice" => p.ice = v.parse().unwrap(),
            "lanes" => p.lanes = v.parse().unwrap(),
            "shells" => p.shells = v.parse().unwrap(),
            "locks" => p.locks = v.parse().unwrap(),
            "shutters" => p.shutters = v.parse().unwrap(),
            "wide" => p.wide_doors = v.parse().unwrap(),
            "blocks" => p.max_blocks = v.parse().unwrap(),
            "seed" => seed = v.parse().unwrap(),
            "attempts" => attempts = v.parse().unwrap(),
            "target" => target = v.parse().unwrap(),
            "minpark" => minpark = v.parse().unwrap(),
            _ => eprintln!("unknown parameter {k}"),
        }
    }
    (p, seed, attempts, target, minpark)
}

fn outline(s: &str) -> Outline {
    match s {
        "u" => Outline::U,
        "l" => Outline::L,
        "plus" => Outline::Plus,
        "ring" => Outline::Ring,
        "h" => Outline::H,
        "random" => Outline::Random,
        _ => Outline::Rect,
    }
}

/// Time of the in-game hint (A*, same budget as the game) from the start of every level of a pack.
fn hints(path: &str) {
    let text = std::fs::read_to_string(path).expect("read");
    let mut cur = String::new();
    let mut worst = (0usize, std::time::Duration::ZERO);
    let mut n = 0;
    for line in text.lines() {
        cur.push_str(line);
        cur.push('\n');
        if line.trim() != "end" {
            continue;
        }
        n += 1;
        let level = Level::parse(&cur).expect("parse");
        cur.clear();
        let t = Instant::now();
        let h = solver::hint(&level, &State::initial(&level), 400_000);
        let e = t.elapsed();
        if h.is_none() {
            println!("level {n}: NO HINT within budget ({e:?})");
        }
        if e > worst.1 {
            worst = (n, e);
        }
    }
    println!("{n} levels, slowest hint: level {} in {:?}", worst.0, worst.1);
}

fn bench(args: &[String]) {
    let (p, seed, attempts, _, _) = parse_params(args);
    let mut rng = Rng(seed);
    for _ in 0..attempts {
        let Some(level) = generator::candidate(&p, &mut rng) else {
            println!("no candidate");
            continue;
        };
        let t = Instant::now();
        let r = solver::solve(&level, &State::initial(&level), 300_000);
        let res = match &r.result {
            Solve::Solved(m) => format!("solved in {}", m.len()),
            o => format!("{o:?}"),
        };
        println!("blocks {:2} doors {} -> {res:16} explored {:7} in {:?}", level.blocks.len(), level.doors.len(), r.explored, t.elapsed());
    }
}

fn gen_one(args: &[String]) {
    let (p, seed, attempts, target, minpark) = parse_params(args);
    let t = Instant::now();
    match generator::generate(&p, seed, attempts, target, minpark, 300_000) {
        Some((l, g)) => {
            print!("{}", l.to_text());
            eprintln!("score {:.2} optimal {} slack {} blocks {} free {} explored {} ({:?})", g.score, g.optimal, g.slack, g.blocks, g.free_at_start, g.explored, t.elapsed());
        }
        None => eprintln!("nothing found ({:?})", t.elapsed()),
    }
}

/// What a campaign slot asks for.
struct Slot {
    intro: Option<&'static str>,
    p: Params,
    target: f32,
    minpark: u16,
}

fn slot(i: usize, rng: &mut Rng) -> Slot {
    let lvl = i + 1;
    let hard = lvl % 5 == 0 && lvl >= 10;
    let super_hard = matches!(lvl, 30 | 50 | 75 | 100);
    let after_hard = lvl % 5 == 1 && lvl > 10;
    let intro = match lvl {
        8 => Some("wall"),
        11 => Some("big"),
        16 => Some("ice"),
        26 => Some("lane"),
        36 => Some("shell"),
        46 => Some("lock"),
        56 => Some("shutter"),
        _ => None,
    };
    let t = (i as f32 / 99.0).clamp(0.0, 1.0);
    let mut p = Params {
        w: 4 + (t * 3.0 + rng.below(2) as f32 * 0.6) as i16,
        h: 5 + (t * 4.0 + rng.below(2) as f32 * 0.6) as i16,
        colors: (2.0 + t * 3.5) as usize + (hard as usize),
        fill: 0.85,
        max_blocks: (4.0 + t * 16.0) as usize + 2 * hard as usize + rng.below(3),
        ..Params::default()
    };
    p.w = p.w.min(7);
    p.h = p.h.min(9);
    p.colors = p.colors.min(6);
    p.outline = if lvl < 6 {
        Outline::Rect
    } else {
        *rng.pick(&[Outline::Rect, Outline::U, Outline::L, Outline::Plus, Outline::H, Outline::Random, Outline::Rect, Outline::Ring])
    };
    p.shapes = generator::shapes_by_name(if lvl < 11 {
        "1,1,1,2h,2v"
    } else if lvl < 30 {
        "1,1,2h,2v,3h,3v,sq"
    } else {
        "1,2h,2v,3h,3v,sq,l1,l2,l3,l4"
    });
    if lvl < 11 {
        p.shapes = generator::shapes_by_name("1");
        if lvl >= 6 {
            p.shapes = generator::shapes_by_name("1,1,1,2h,2v");
        }
    }
    p.wide_doors = if lvl >= 11 { 0.45 } else { 0.15 };
    let since = |at: usize| lvl >= at;
    let maybe = |rng: &mut Rng, at: usize, prob: f32| if since(at) && rng.chance(prob) { 1 } else { 0 };
    p.walls = if since(8) { rng.below(3) } else { 0 };
    p.ice = maybe(rng, 16, 0.4) + if since(40) { maybe(rng, 40, 0.3) } else { 0 };
    p.lanes = maybe(rng, 26, 0.35);
    p.shells = maybe(rng, 36, 0.35);
    p.locks = maybe(rng, 46, 0.3);
    p.shutters = maybe(rng, 56, 0.3);
    if let Some(k) = intro {
        // Intro level: small board, exactly one instance of the new element, gentle.
        p.w = 5;
        p.h = 6;
        p.colors = 3;
        p.fill = 0.6;
        p.max_blocks = 6;
        p.walls = 0;
        p.ice = 0;
        p.lanes = 0;
        p.shells = 0;
        p.locks = 0;
        p.shutters = 0;
        p.outline = Outline::Rect;
        match k {
            "wall" => p.walls = 2,
            "big" => {
                p.shapes = generator::shapes_by_name("1,2h,2v,sq");
                p.wide_doors = 0.8;
            }
            "ice" => p.ice = 1,
            "lane" => p.lanes = 1,
            "shell" => p.shells = 1,
            "lock" => p.locks = 1,
            "shutter" => p.shutters = 1,
            _ => {}
        }
    }
    let mut target = 1.5 + t * 8.0;
    if hard {
        target += 2.5;
    }
    if super_hard {
        target += 2.0;
    }
    if after_hard {
        target -= 1.5;
    }
    if intro.is_some() {
        target = 1.5 + t * 2.0;
    }
    let minpark = if lvl < 6 || intro.is_some() {
        0
    } else {
        (1 + i / 25) as u16 + hard as u16 + super_hard as u16
    };
    Slot { intro, p, target, minpark }
}

fn campaign(out: &str, seed: u64) {
    let mut text = String::new();
    // Hand-made tutorials, with their optimal move count from the solver.
    let mut cur = String::new();
    for line in TUTORIAL.lines() {
        cur.push_str(line);
        cur.push('\n');
        if line.trim() == "end" {
            let mut l = Level::parse(&cur).expect("tutorial");
            if let Solve::Solved(m) = solver::solve(&l, &State::initial(&l), 100_000).result {
                l.meta.optimal = Some(m.len() as u16);
            }
            text.push_str(&l.to_text());
            cur.clear();
        }
    }
    let t0 = Instant::now();
    // Slots are drawn in order from one RNG (stable), then generated in parallel.
    let mut rng = Rng(seed);
    let slots: Vec<(usize, Slot)> = (3..100).map(|i| (i, slot(i, &mut rng))).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(2);
    let results: std::sync::Mutex<Vec<(usize, String)>> = std::sync::Mutex::new(Vec::new());
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some((i, s0)) = slots.get(k) else { break };
                let i = *i;
                let mut s = Slot { intro: s0.intro, p: s0.p.clone(), target: s0.target, minpark: s0.minpark };
                let mut found = None;
                for attempt in 0..30u64 {
                    if let Some(r) = generator::generate(&s.p, seed.wrapping_mul(1000).wrapping_add(i as u64 * 97 + attempt), 60, s.target, s.minpark, 150_000) {
                        found = Some(r);
                        break;
                    }
                    s.minpark = s.minpark.saturating_sub(1);
                    s.p.max_blocks = s.p.max_blocks.saturating_sub(1).max(3);
                }
                let Some((mut l, g)) = found else {
                    eprintln!("level {}: nothing found", i + 1);
                    continue;
                };
                l.meta.name = format!("{}", i + 1);
                if let Some(k) = s.intro {
                    l.meta.extra.push(("intro".into(), k.into()));
                }
                l.meta.extra.push(("score".into(), format!("{:.1}", g.score)));
                eprintln!(
                    "{:3} {:>7} target {:4.1} score {:4.1} optimal {:2} slack {:2} blocks {:2} board {}x{} ({:.0?})",
                    i + 1,
                    s.intro.unwrap_or(""),
                    s.target,
                    g.score,
                    g.optimal,
                    g.slack,
                    g.blocks,
                    l.w,
                    l.h,
                    t0.elapsed()
                );
                results.lock().unwrap().push((i, l.to_text()));
            });
        }
    });
    let mut r = results.into_inner().unwrap();
    r.sort_by_key(|x| x.0);
    for (_, t) in r {
        text.push_str(&t);
    }
    std::fs::write(out, text).expect("write");
}

/// The first three levels, made by hand.
const TUTORIAL: &str = "fuera 1
name 1
intro t1
map
  .....
  A....
  .....
  ...B.
  .....
blocks
  A red
  B blue
doors
  E 4 0 5 red
  S 0 4 5 blue
end
fuera 1
name 2
intro t2
map
  A....
  .....
  ..B..
  .....
  ....C
blocks
  A red
  B blue
  C yellow
doors
  E 4 0 1 blue
  S 0 4 1 red
  W 0 4 1 yellow
end
fuera 1
name 3
intro t3
map
  A..B
  ....
  C...
  ....
blocks
  A blue
  B red
  C yellow
doors
  E 3 0 1 blue
  N 3 0 1 red
  E 3 2 1 yellow
end
";

/// Reads French infinitives (one per line) and prints every form the game's conjugator gives:
/// `infinitive tense person form` (tab-separated); `aux` and `pp` rows give the auxiliary and the participle.
fn conj_fr() {
    use fuera_learn::conj_fr::{Aux, Person, Tense, auxiliary, conjugate, participle};
    use std::io::BufRead;
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let inf = line.trim();
        if inf.is_empty() {
            continue;
        }
        if let Some(pp) = participle(inf) {
            println!("{inf}\tpp\t-\t{pp}");
            println!("{inf}\taux\t-\t{}", if auxiliary(inf) == Aux::Etre { "être" } else { "avoir" });
        }
        for t in Tense::ALL {
            for p in Person::ALL {
                if let Some(f) = conjugate(inf, t, p) {
                    println!("{inf}\t{}\t{}\t{f}", t.name(), p.pronoun());
                }
            }
        }
    }
}

/// Prints every word a course can say alone (lexicon, gender traps, false friends), one per line, for the voice
/// pack scripts: `fuera-cli words es | es-419 | fr`.
fn words(course: &str) {
    use fuera_learn::content::{Content, Course};
    let c = Content::load(match course {
        "fr" => Course::FRENCH,
        "es-419" => Course::LATAM,
        _ => Course::SPAIN,
    });
    let mut out: Vec<String> = c.words.iter().map(|w| w.lemma.clone()).collect();
    out.extend(c.gender_traps.iter().map(|w| w.lemma.clone()));
    out.extend(c.false_friends.iter().map(|f| f.word.clone()));
    out.sort();
    out.dedup();
    for w in out {
        println!("{w}");
    }
}
