//! Sound effects, synthesised at run time (no sample files, nothing to license).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sfx {
    Slide,
    Bump,
    /// A block leaves; `n` counts exits in a row (rising pitch).
    Exit(u8),
    Peel,
    /// Wrong door (language levels).
    Bounce,
    Win,
    Tap,
    Nope,
    /// One star appears (0, 1, 2).
    Star(u8),
    /// Right answer chime (language levels).
    Right,
}

fn env(t: f32, attack: f32, decay: f32) -> f32 {
    if t < attack { t / attack } else { (-(t - attack) / decay).exp() }
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn pluck(out: &mut [f32], rate: f32, start: f32, freq: f32, amp: f32, decay: f32) {
    let s0 = (start * rate) as usize;
    for (i, o) in out.iter_mut().enumerate().skip(s0) {
        let t = (i - s0) as f32 / rate;
        let e = env(t, 0.004, decay);
        if e < 0.0005 && t > 0.05 {
            break;
        }
        let ph = std::f32::consts::TAU * freq * t;
        *o += amp * e * (ph.sin() + 0.35 * (2.0 * ph).sin() * (-t * 18.0).exp() + 0.15 * (3.0 * ph).sin() * (-t * 30.0).exp());
    }
}

/// Mono samples in -1..1.
pub fn synth(s: Sfx, rate: u32) -> Vec<f32> {
    let r = rate as f32;
    let len = |sec: f32| vec![0.0f32; (sec * r) as usize];
    let mut rng = Rng(0x9E37_79B9);
    match s {
        Sfx::Slide => {
            let mut v = len(0.12);
            let mut lp = 0.0;
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                let n = rng.next();
                let a = 0.08 + 0.25 * (t / 0.12);
                lp += a * (n - lp);
                *o = lp * 0.22 * env(t, 0.02, 0.05);
            }
            v
        }
        Sfx::Bump => {
            let mut v = len(0.12);
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                let f = 140.0 - 60.0 * (t / 0.12);
                *o = 0.35 * env(t, 0.002, 0.03) * (std::f32::consts::TAU * f * t).sin() + 0.08 * rng.next() * env(t, 0.0, 0.006);
            }
            v
        }
        Sfx::Exit(n) => {
            let mut v = len(0.45);
            let semis = [0.0, 2.0, 4.0, 7.0, 9.0, 12.0, 14.0, 16.0][n.min(7) as usize];
            let base = 523.25 * 2f32.powf(semis / 12.0);
            // Pop: fast upward sweep.
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                if t > 0.07 {
                    break;
                }
                let f = 300.0 + 2400.0 * t;
                *o += 0.25 * env(t, 0.001, 0.02) * (std::f32::consts::TAU * f * t).sin();
            }
            pluck(&mut v, r, 0.03, base, 0.22, 0.12);
            pluck(&mut v, r, 0.08, base * 1.5, 0.12, 0.10);
            v
        }
        Sfx::Peel => {
            let mut v = len(0.3);
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                *o = 0.25 * rng.next() * env(t, 0.001, 0.04);
            }
            pluck(&mut v, r, 0.02, 880.0, 0.15, 0.08);
            v
        }
        Sfx::Bounce => {
            let mut v = len(0.35);
            for (k, (st, f)) in [(0.0, 220.0f32), (0.13, 175.0)].iter().enumerate() {
                let s0 = (st * r) as usize;
                for (i, o) in v.iter_mut().enumerate().skip(s0) {
                    let t = (i - s0) as f32 / r;
                    let e = env(t, 0.005, 0.07);
                    let ph = std::f32::consts::TAU * f * t;
                    // Soft square-ish tone.
                    *o += 0.16 * e * (ph.sin() + 0.3 * (3.0 * ph).sin()) * if k == 0 { 1.0 } else { 0.9 };
                }
            }
            v
        }
        Sfx::Win => {
            let mut v = len(1.4);
            let notes = [523.25, 659.25, 783.99, 1046.5, 1318.5];
            for (i, f) in notes.iter().enumerate() {
                pluck(&mut v, r, i as f32 * 0.085, *f, 0.18, 0.35);
            }
            pluck(&mut v, r, 0.5, 1046.5, 0.1, 0.6);
            pluck(&mut v, r, 0.5, 1567.98, 0.08, 0.6);
            v
        }
        Sfx::Tap => {
            let mut v = len(0.04);
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                *o = 0.2 * env(t, 0.0005, 0.008) * (std::f32::consts::TAU * 1800.0 * t).sin();
            }
            v
        }
        Sfx::Nope => {
            let mut v = len(0.16);
            for (i, o) in v.iter_mut().enumerate() {
                let t = i as f32 / r;
                let ph = std::f32::consts::TAU * 110.0 * t;
                *o = 0.18 * env(t, 0.003, 0.05) * (ph.sin() + 0.5 * (2.0 * ph).sin());
            }
            v
        }
        Sfx::Star(k) => {
            let mut v = len(0.5);
            let f = [880.0, 1108.73, 1318.51][k.min(2) as usize];
            pluck(&mut v, r, 0.0, f, 0.2, 0.25);
            pluck(&mut v, r, 0.0, f * 2.0, 0.06, 0.15);
            v
        }
        Sfx::Right => {
            let mut v = len(0.5);
            pluck(&mut v, r, 0.0, 783.99, 0.18, 0.2);
            pluck(&mut v, r, 0.09, 1174.66, 0.16, 0.3);
            v
        }
    }
}
