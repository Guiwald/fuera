use crate::geom::{Dir, Pos};
use crate::level::Level;
use crate::rules::{Move, Result};
use crate::solver::{self, Solve};
use crate::Game;

fn lvl(s: &str) -> Level {
    let l = Level::parse(s).unwrap_or_else(|e| panic!("{e}"));
    let errs = l.validate();
    assert!(errs.is_empty(), "{errs:?}");
    l
}

fn mv(g: &Game, name: char, d: Dir) -> Move {
    let b = g.level.blocks.iter().position(|b| b.name == name).unwrap();
    Move { block: b as u8, dir: d }
}

const BASIC: &str = "fuera 1
name basic
map
  A...
  ....
  ..B.
blocks
  A red
  B blue
doors
  E 3 0 1 red
  W 0 2 1 blue
end
";

#[test]
fn roundtrip() {
    let l = lvl(BASIC);
    let again = lvl(&l.to_text());
    assert_eq!(l, again);
}

#[test]
fn slide_and_exit() {
    let mut g = Game::new(lvl(BASIC));
    // A slides east along row 0 into the red door.
    let o = g.play(mv(&g, 'A', Dir::E)).unwrap();
    assert_eq!(o.result, Result::Exit { door: 0 });
    assert_eq!(o.steps, 3);
    // B slides south: one cell? No — B is on the last row, cannot move south.
    assert!(g.play(mv(&g, 'B', Dir::S)).is_none());
    // B slides north to row 0 (A is gone), resting against the top wall.
    let o = g.play(mv(&g, 'B', Dir::N)).unwrap();
    assert_eq!(o.result, Result::Stop { against: None });
    assert_eq!(g.state.blocks[1].pos, Pos::new(2, 0));
    // B east: reaches the red door: wrong colour, rests against it.
    let o = g.play(mv(&g, 'B', Dir::E)).unwrap();
    assert_eq!(o.result, Result::Stop { against: Some(0) });
    assert!(!g.is_won());
    let sol = solver::solve(&g.level, &crate::State::initial(&g.level), 10_000);
    assert_eq!(sol.result.clone(), Solve::Solved(sol_moves(&sol.result)));
    assert_eq!(sol_moves(&sol.result).len(), 2);
}

fn sol_moves(s: &Solve) -> Vec<Move> {
    match s {
        Solve::Solved(m) => m.clone(),
        _ => panic!("not solved: {s:?}"),
    }
}

#[test]
fn door_must_be_wide_enough() {
    let l = lvl("fuera 1
map
  AA.
  AA.
  ...
blocks
  A red
doors
  N 0 0 2 red
  W 0 2 1 red
end
");
    let mut g = Game::new(l);
    // 2x2 block going north: door covers both columns: exits.
    assert!(matches!(g.play(mv(&g, 'A', Dir::N)).unwrap().result, Result::Exit { .. }));
    // Second level: the 1-wide west door cannot take the 2-tall block.
    let l = lvl("fuera 1
map
  ...
  .AA
  .AA
blocks
  A red
doors
  W 0 2 1 red
  E 2 1 2 red
end
");
    let mut g = Game::new(l);
    let o = g.play(mv(&g, 'A', Dir::W)).unwrap();
    assert_eq!(o.result, Result::Stop { against: None });
    let o = g.play(mv(&g, 'A', Dir::E)).unwrap();
    assert!(matches!(o.result, Result::Exit { .. }));
}

#[test]
fn l_shape_exit_needs_clear_lines() {
    // An L whose upper arm is one cell shorter: both rows must reach the 2-long east door.
    let l = lvl("fuera 1
map
  A...
  AA..
blocks
  A green
doors
  E 3 0 2 green
end
");
    let mut g = Game::new(l.clone());
    assert!(matches!(g.play(mv(&g, 'A', Dir::E)).unwrap().result, Result::Exit { .. }));
    // Same with a blocker in the upper row in front: no exit.
    let l = lvl("fuera 1
map
  A..B
  AA..
blocks
  A green
  B red
doors
  E 3 0 2 green
  N 3 0 1 red
end
");
    let mut g = Game::new(l);
    let o = g.play(mv(&g, 'A', Dir::E)).unwrap();
    assert_eq!(o.result, Result::Stop { against: None });
    assert_eq!(g.state.blocks[0].pos, Pos::new(2, 0));
}

fn l_text_without_e() -> String {
    "fuera 1
map
  A..
  B..
  C..
  D.E
blocks
  A red
  B blue ice=1
  C yellow lane=v
  D green lock=1
  E cyan key=1
doors
  E 2 0 1 red
  E 2 1 1 blue
  E 2 2 1 yellow
  N 2 0 1 cyan
  W 0 3 1 green
end
"
    .to_string()
}

#[test]
fn specials_behave() {
    let mut g = Game::new(Level::parse(&l_text_without_e()).unwrap());
    // B frozen until one exit.
    assert!(g.play(mv(&g, 'B', Dir::E)).is_none());
    // C only moves vertically.
    assert!(g.play(mv(&g, 'C', Dir::E)).is_none());
    // D locked until E (carrying key 1) leaves.
    assert!(g.play(mv(&g, 'D', Dir::W)).is_none());
    assert!(matches!(g.play(mv(&g, 'A', Dir::E)).unwrap().result, Result::Exit { .. }));
    // B has thawed.
    assert!(matches!(g.play(mv(&g, 'B', Dir::E)).unwrap().result, Result::Exit { .. }));
    // E goes north along column 2 to the cyan door on top.
    assert!(matches!(g.play(mv(&g, 'E', Dir::N)).unwrap().result, Result::Exit { .. }));
    // D is free now; west door is green: it exits with 0 steps.
    let o = g.play(mv(&g, 'D', Dir::W)).unwrap();
    assert!(matches!(o.result, Result::Exit { .. }));
    assert_eq!(o.steps, 0);
    // Undo works and restores the counters.
    assert!(g.undo());
    assert_eq!(g.state.exits, 3);
}

#[test]
fn shell_peels_then_exits() {
    let l = lvl("fuera 1
map
  ...A
blocks
  A red shell=blue
doors
  W 0 0 1 blue
  E 3 0 1 red
end
");
    let mut g = Game::new(l);
    // Red door ignored while the blue layer is on: A rests (0 steps → no move).
    assert!(g.play(mv(&g, 'A', Dir::E)).is_none());
    let o = g.play(mv(&g, 'A', Dir::W)).unwrap();
    assert!(matches!(o.result, Result::Peel { .. }));
    assert_eq!(g.state.blocks[0].pos, Pos::new(0, 0));
    let o = g.play(mv(&g, 'A', Dir::E)).unwrap();
    assert!(matches!(o.result, Result::Exit { .. }));
    assert!(g.is_won());
    let sol = solver::solve(&g.level, &crate::State::initial(&g.level), 1000);
    assert_eq!(sol_moves(&sol.result).len(), 2);
}

#[test]
fn shutter_and_bounce() {
    let l = lvl("fuera 1
wrong bounce
map
  A.
  B.
blocks
  A red
  B blue
doors
  E 1 0 1 red
  E 1 1 1 blue shutter=closed
  W 0 1 1 green
end
");
    let mut g = Game::new(l);
    // Closed shutter: B rests (one step east, against the closed door).
    let o = g.play(mv(&g, 'B', Dir::E)).unwrap();
    assert_eq!(o.result, Result::Stop { against: Some(1) });
    // Bounce: B west into the green door is a wrong answer; nothing changes.
    let before = g.state.clone();
    let o = g.play(mv(&g, 'B', Dir::W)).unwrap();
    assert!(matches!(o.result, Result::Bounce { door: 2 }));
    assert_eq!(g.state, before);
    assert_eq!(g.mistakes, 1);
    // A exits: the shutter opens.
    assert!(matches!(g.play(mv(&g, 'A', Dir::E)).unwrap().result, Result::Exit { .. }));
    assert!(matches!(g.play(mv(&g, 'B', Dir::E)).unwrap().result, Result::Exit { .. }));
}

#[test]
fn identical_blocks_and_dead_ends() {
    // Ten 1x1 reds in a corridor: identical blocks must not blow up the search.
    let l = lvl("fuera 1
map
  ABCDEFGHIJ.
blocks
  A red
  B red
  C red
  D red
  E red
  F red
  G red
  H red
  I red
  J red
doors
  E 10 0 1 red
end
");
    let r = solver::solve(&l, &crate::State::initial(&l), 100_000);
    assert_eq!(sol_moves(&r.result).len(), 10);
    assert!(r.explored < 200);
    // A trap: B must leave west before A blocks the corridor; moving A first kills the level.
    let l = lvl("fuera 1
map
  .A
  B.
blocks
  A red
  B blue
doors
  W 0 1 1 blue
  N 1 0 1 red
end
");
    let a = solver::analyze(&l, 10_000);
    assert!(a.complete);
    assert_eq!(a.optimal, Some(2));
}

#[test]
fn analysis_counts_dead_states() {
    // Moving A down first blocks B's only way out.
    let l = lvl("fuera 1
map
  A.~
  ..B
blocks
  A red
  B blue
doors
  N 0 0 1 red
  W 0 1 1 blue
end
");
    let a = solver::analyze(&l, 10_000);
    assert_eq!(a.optimal, Some(2));
    assert!(a.complete);
}

#[test]
fn astar_matches_bfs() {
    use crate::generator::{Params, Rng, candidate, shapes_by_name};
    let mut rng = Rng(42);
    let mut checked = 0;
    for i in 0..160 {
        let p = Params {
            w: 4 + (i % 2),
            h: 4 + (i % 3) as i16,
            colors: 2 + (i % 3) as usize,
            fill: 0.35 + (i % 4) as f32 * 0.1,
            shapes: shapes_by_name("1,2h,2v,l1"),
            walls: (i % 2) as usize,
            ice: (i % 5 == 0) as usize,
            lanes: (i % 7 == 0) as usize,
            shells: (i % 6 == 0) as usize,
            locks: (i % 9 == 0) as usize,
            shutters: (i % 8 == 0) as usize,
            ..Params::default()
        };
        let Some(level) = candidate(&p, &mut rng) else { continue };
        let st = crate::State::initial(&level);
        let a = solver::solve(&level, &st, 60_000);
        let b = solver::solve_bfs(&level, &st, 60_000);
        match (&a.result, &b.result) {
            (Solve::Solved(x), Solve::Solved(y)) => {
                assert_eq!(x.len(), y.len(), "A* not optimal on\n{}", level.to_text());
                // The A* path really solves the level.
                let mut g = Game::new(level.clone());
                for m in x {
                    assert!(g.play(*m).is_some());
                }
                assert!(g.is_won());
                checked += 1;
            }
            (Solve::Unsolvable, Solve::Unsolvable) => checked += 1,
            (Solve::TooBig, _) | (_, Solve::TooBig) => {}
            (x, y) => panic!("A* {x:?} vs BFS {y:?} on\n{}", level.to_text()),
        }
    }
    assert!(checked > 40, "only {checked} levels checked");
}

#[test]
fn word_boards_are_solvable() {
    use crate::generator::word_board;
    use crate::lattice::Layout;
    // (tiles, doors) as the language rounds use them: meanings, persons, false friends, two categories (short words
    // or sentences), two sentences with a "none" bin.
    let cases: [(&[(i16, u16)], &[u16]); 7] = [
        (&[(2, 0), (2, 1), (2, 2)], &[0, 1, 2, 3]),
        (&[(2, 0), (2, 1)], &[0, 100, 1, 101]),
        (&[(2, 0), (2, 1), (2, 0), (2, 1), (2, 0), (2, 0)], &[0, 1]),
        (&[(3, 0), (3, 1), (3, 1), (3, 0), (3, 1)], &[0, 1]),
        (&[(3, 0), (3, 1), (3, 1), (3, 0), (3, 1), (3, 0)], &[0, 1]),
        (&[(2, 0), (2, 99), (2, 1), (2, 99)], &[0, 1, 99]),
        (&[(2, 0), (2, 99), (2, 1)], &[0, 1, 99]),
    ];
    for (tiles, doors) in cases {
        for seed in 0..10u64 {
            for slack in [(0, 0), (0, 2), (1, 3)] {
                let l = word_board(tiles, doors, seed, 40, slack).expect("board");
                assert_eq!(l.meta.wrong_door, crate::WrongDoor::Bounce);
                assert_eq!((l.w, l.h), (Layout::WIDTH, Layout::ROWS as i16));
                // A layout proven free of dead ends for this many tiles.
                let layout = Layout::of_level(&l).expect("lattice board");
                assert!(layout.proven() >= tiles.len(), "{}", l.to_text());
                assert_eq!(l.doors.len(), doors.len(), "one door per key\n{}", l.to_text());
                if doors.len() == 2 {
                    // First category on top, second at the bottom, every round.
                    assert!(l.doors.iter().any(|d| d.side == crate::Dir::N && d.key == doors[0]));
                    assert!(l.doors.iter().any(|d| d.side == crate::Dir::S && d.key == doors[1]));
                }
                let r = solver::solve(&l, &crate::State::initial(&l), 400_000);
                assert!(matches!(r.result, Solve::Solved(ref m) if m.len() as u16 == l.meta.optimal.unwrap()), "{}", l.to_text());
            }
        }
    }
}

#[test]
fn lattice_model_agrees_with_the_rules() {
    // The proof works on an abstract model of the board; the real rules must agree with it. On small boards, the
    // complete state graph of the real game (solver::analyze) has no dead end either, and the model's own check of
    // a trap layout finds what the real rules find.
    use crate::generator::{Rng, word_candidate};
    for (tiles, doors) in [
        (&[(2, 0), (2, 1), (2, 2)][..], &[0, 1, 2, 3][..]),
        (&[(3, 0), (3, 1), (3, 1), (3, 0)][..], &[0, 1][..]),
        (&[(2, 0), (2, 1), (2, 0), (2, 1)][..], &[0, 1][..]),
        (&[(2, 0), (2, 99), (2, 1)][..], &[0, 1, 99][..]),
    ] {
        let mut rng = Rng(11);
        for _ in 0..15 {
            let l = word_candidate(tiles, doors, &mut rng).expect("candidate");
            let a = solver::analyze(&l, 300_000);
            assert!(a.complete && a.dead_states == 0, "{}", l.to_text());
        }
    }
    // The first version's trap: a door over the middle column can never be reached by a lone tile.
    let lvl = crate::Level::parse(
        "fuera 1\nwrong bounce\nmap\n  ......\n  AA....\n  ......\n  ......\n  ......\n  ......\n  ......\nblocks\n  A red\ndoors\n  N 0 0 2 blue\n  N 2 0 2 red\n  N 4 0 2 yellow\n  S 0 6 2 green\n  S 2 6 2 purple\nend\n",
    );
    let lvl = lvl.expect("parse");
    let a = solver::analyze(&lvl, 10_000);
    assert!(a.optimal.is_none(), "a lone tile cannot stop under the middle door");
}

/// The level drawn with the blocks where `st` has them (exited blocks dropped).
fn show_state(level: &crate::Level, st: &crate::State) -> String {
    let mut l = level.clone();
    let keep: Vec<usize> = (0..l.blocks.len()).filter(|&i| st.blocks[i].alive).collect();
    l.blocks = keep.iter().map(|&i| level.blocks[i].clone()).collect();
    l.start = keep.iter().map(|&i| st.blocks[i].pos).collect();
    l.to_text()
}
