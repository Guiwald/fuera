//! Property tests of the FSRS-6 scheduler.

use fuera_learn::fsrs::{Card, DEFAULT_PARAMETERS, Fsrs, RELEARN_STEP, Rating};

const T0: f64 = 20_000.0;

#[test]
fn default_parameters() {
    let f = Fsrs::default();
    assert_eq!(f.w, DEFAULT_PARAMETERS);
    assert_eq!(f.retention, 0.9);
}

#[test]
fn first_review_uses_initial_stability() {
    let f = Fsrs::default();
    for (r, i) in [
        (Rating::Again, 0),
        (Rating::Hard, 1),
        (Rating::Good, 2),
        (Rating::Easy, 3),
    ] {
        let c = f.first(r, T0);
        assert_eq!(c.stability, f.w[i]);
        assert!((1.0..=10.0).contains(&c.difficulty));
        assert_eq!((c.reps, c.lapses, c.last), (1, 0, T0));
    }
    let good = f.first(Rating::Good, T0);
    assert_eq!(good.stability, f.w[2]);
    assert_eq!(good.due, T0 + f.interval_days(f.w[2]));
    // harder first answers mean higher difficulty
    assert!(f.first(Rating::Again, T0).difficulty > good.difficulty);
    assert!(f.first(Rating::Easy, T0).difficulty < good.difficulty);
    // Again comes back within the session
    assert_eq!(f.first(Rating::Again, T0).due, T0 + RELEARN_STEP);
}

#[test]
fn retrievability_at_stability_is_ninety_percent() {
    let f = Fsrs::default();
    for s in [0.5f32, 2.3065, 10.0, 123.4] {
        let c = Card {
            stability: s,
            difficulty: 5.0,
            last: T0,
            due: T0,
            reps: 1,
            lapses: 0,
        };
        assert!((f.retrievability(&c, T0 + f64::from(s)) - 0.9).abs() < 1e-3);
        assert!((f.retrievability(&c, T0) - 1.0).abs() < 1e-6);
        assert!(f.retrievability(&c, T0 + 2.0 * f64::from(s)) < 0.9);
    }
}

#[test]
fn interval_matches_stability_at_ninety_percent() {
    let f = Fsrs::default();
    assert_eq!(f.interval_days(10.0), 10.0);
    assert_eq!(f.interval_days(0.2), 1.0); // never less than a day
    let strict = Fsrs {
        retention: 0.95,
        ..Fsrs::default()
    };
    assert!(strict.interval_days(100.0) < f.interval_days(100.0));
}

#[test]
fn good_after_interval_increases_stability() {
    let f = Fsrs::default();
    let c = f.first(Rating::Good, T0);
    let next = f.review(&c, Rating::Good, c.due);
    assert!(next.stability > c.stability);
    assert_eq!(next.reps, 2);
    assert_eq!(next.lapses, 0);
    // rating order is reflected in stability
    let hard = f.review(&c, Rating::Hard, c.due);
    let easy = f.review(&c, Rating::Easy, c.due);
    assert!(hard.stability < next.stability && next.stability < easy.stability);
}

#[test]
fn again_lowers_stability_and_counts_a_lapse() {
    let f = Fsrs::default();
    let mut c = f.first(Rating::Good, T0);
    for _ in 0..3 {
        c = f.review(&c, Rating::Good, c.due);
    }
    let failed = f.review(&c, Rating::Again, c.due);
    assert!(failed.stability < c.stability);
    assert!(failed.difficulty > c.difficulty);
    assert_eq!(failed.lapses, c.lapses + 1);
    assert_eq!(failed.due, c.due + RELEARN_STEP);
    // failing it again in the same session is not another lapse
    let again = f.review(&failed, Rating::Again, failed.due);
    assert_eq!(again.lapses, failed.lapses);
}

#[test]
fn same_day_reviews_use_short_term_stability() {
    let f = Fsrs::default();
    let c = f.first(Rating::Again, T0);
    let later = T0 + RELEARN_STEP;
    let good = f.review(&c, Rating::Good, later);
    let again = f.review(&c, Rating::Again, later);
    let w = &f.w;
    let expected = c.stability * (w[17] * w[18]).exp() * c.stability.powf(-w[19]);
    assert!((good.stability - expected).abs() < 1e-5);
    assert!(again.stability < c.stability);
    assert!(good.due - later >= 1.0);
    // Good never lowers stability, even for an already stable card
    let stable = Card {
        stability: 50.0,
        difficulty: 5.0,
        last: T0,
        due: T0 + 50.0,
        reps: 5,
        lapses: 0,
    };
    assert!(f.review(&stable, Rating::Good, T0 + 0.1).stability >= 50.0);
}

#[test]
fn intervals_grow_over_successive_good_reviews() {
    let f = Fsrs::default();
    let mut c = f.first(Rating::Good, T0);
    let mut prev = c.due - c.last;
    for _ in 0..8 {
        c = f.review(&c, Rating::Good, c.due);
        let interval = c.due - c.last;
        assert!(interval > prev, "{interval} <= {prev}");
        prev = interval;
    }
    assert!(prev > 100.0);
}

#[test]
fn compact_round_trip() {
    let f = Fsrs::default();
    let mut c = f.first(Rating::Hard, T0 + 0.375);
    c = f.review(&c, Rating::Again, c.due + 0.5);
    let text = c.to_compact();
    assert_eq!(text.split(',').count(), 6);
    let back = Card::from_compact(&text).expect("parses");
    assert!((back.stability - c.stability).abs() < 1e-4);
    assert!((back.difficulty - c.difficulty).abs() < 1e-4);
    assert!((back.last - c.last).abs() < 1e-4);
    assert!((back.due - c.due).abs() < 1e-4);
    assert_eq!((back.reps, back.lapses), (c.reps, c.lapses));
    assert_eq!(back.to_compact(), text);

    let simple = Card {
        stability: 2.5,
        difficulty: 3.0,
        last: 20000.0,
        due: 20002.0,
        reps: 1,
        lapses: 0,
    };
    assert_eq!(simple.to_compact(), "2.5,3,20000,20002,1,0");
    assert_eq!(Card::from_compact("2.5,3,20000,20002,1,0"), Some(simple));

    for bad in [
        "",
        "1,2,3",
        "1,2,3,4,5,6,7",
        "a,2,3,4,5,6",
        "1,2,3,4,-1,0",
        "NaN,2,3,4,5,6",
    ] {
        assert_eq!(Card::from_compact(bad), None, "{bad:?}");
    }
}

/// First steps of the review trace in the py-fsrs 6.3.0 reference fixture
/// (default parameters): Good on a new card, Again two minutes later, Good
/// three days after that.
#[test]
fn matches_py_fsrs_reference_trace() {
    let f = Fsrs::default();
    let close = |a: f32, b: f64| (f64::from(a) - b).abs() < 1e-4 * b.max(1.0);
    let c = f.first(Rating::Good, T0);
    assert!(
        close(c.stability, 2.3065) && close(c.difficulty, 2.118103970459016),
        "{c:?}"
    );
    let t1 = T0 + 2.0 / 1440.0;
    let c = f.review(&c, Rating::Again, t1);
    assert!(
        close(c.stability, 0.7750839828558984) && close(c.difficulty, 7.394502741279718),
        "{c:?}"
    );
    let c = f.review(&c, Rating::Good, t1 + 3.0);
    assert!(
        close(c.stability, 4.308886950836501) && close(c.difficulty, 7.382336607835277),
        "{c:?}"
    );
}
