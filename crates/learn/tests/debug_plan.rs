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
            let wide = if r.gates.len() == 2 { r.tiles.iter().map(|t| t.width).max().unwrap_or(2).clamp(2, 3) } else { 2 };
            for t in &mut r.tiles {
                t.width = wide;
            }
            let tiles: Vec<(i16, u16)> = r.tiles.iter().map(|t| (t.width as i16, t.key)).collect();
            let doors: Vec<u16> = r.gates.iter().map(|g| g.key).collect();
            let slack = [(0, 0), (0, 2), (1, 3)][seed as usize % 3];
            let b = fuera_core::generator::word_board(&tiles, &doors, seed * 7919, 40, slack);
            assert!(b.is_some(), "{mode:?} seed {seed}");
            let l = b.unwrap();
            if l.meta.optimal == Some(l.blocks.len() as u16) {
                trivial += 1;
            }
            let proven = fuera_core::lattice::Layout::of_level(&l).map(|x| x.proven()).unwrap_or(0);
            assert!(proven >= l.blocks.len(), "{mode:?}: {} tiles on a layout proven for {proven}", l.blocks.len());
            let e = t.elapsed();
            total += e;
            worst = worst.max(e);
        }
        println!("{mode:?}: mean {:?}, worst {:?}, one-move-per-tile boards {trivial}/29", total / 29, worst);
    }
}
