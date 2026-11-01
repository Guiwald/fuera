//! Timing of round preparation (planner + board), the work done when the player taps a mode.
use fuera_learn::content::{Band, Base, Content};
use fuera_learn::memory::Memory;
use fuera_learn::plan::{Mode, Planner};

#[test]
#[ignore]
fn round_preparation_time() {
    let c = Content::load();
    let m = Memory::default();
    for mode in Mode::MENU.into_iter().chain([Mode::Placement]) {
        let mut worst = std::time::Duration::ZERO;
        let mut total = std::time::Duration::ZERO;
        let mut trivial = 0;
        for seed in 1..30u64 {
            let t = std::time::Instant::now();
            let mut p = Planner::new(&c, &m, Base::Fr, Band::B2, 20_400.3, seed * 7919);
            let mut r = p.plan(mode).unwrap();
            if r.gates.len().div_ceil(2) >= 3 {
                for t in &mut r.tiles {
                    t.width = t.width.min(2);
                }
            }
            let tiles: Vec<(i16, u16)> = r.tiles.iter().map(|t| (t.width as i16, t.key)).collect();
            let doors: Vec<u16> = r.gates.iter().map(|g| g.key).collect();
            let b = fuera_core::generator::word_board(&tiles, &doors, seed * 7919, 40, (0, 2));
            assert!(b.is_some(), "{mode:?} seed {seed}");
            let l = b.unwrap();
            if l.meta.optimal == Some(l.blocks.len() as u16) {
                trivial += 1;
            }
            let e = t.elapsed();
            total += e;
            worst = worst.max(e);
        }
        println!("{mode:?}: mean {:?}, worst {:?}, one-move-per-tile boards {trivial}/29", total / 29, worst);
    }
}
