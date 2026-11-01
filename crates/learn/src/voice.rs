//! Recorded pronunciations (human voices from Lingua Libre, see `assets/es/voces/README.md`).
//!
//! The pack is one file: a header, an index of the spoken texts, and IMA ADPCM audio (mono, 4 bits a sample, about
//! 8 KB a second at 16 kHz). Lookups ignore case, surrounding spaces and final punctuation, so `"Casa."` finds the
//! recording of `casa`. Anything without a recording goes to the system voice instead.
//!
//! Layout (little endian):
//! ```text
//! "FVOX" u8 version=1 u32 sample_rate u32 count u32 speakers
//! speakers × { u8 len, name utf-8 }
//! count × { u8 len, text utf-8, u16 speaker, u32 offset, u32 samples }
//! data: per clip, i16 first sample, u8 step index, then (samples - 1) nibbles, low nibble first
//! ```

use std::collections::HashMap;

/// An index over a voice pack held in memory.
pub struct Voices {
    data: Vec<u8>,
    rate: u32,
    speakers: Vec<String>,
    index: HashMap<String, (u16, u32, u32)>,
}

const STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230,
    253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272, 2499, 2749, 3024, 3327,
    3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
    32767,
];
const INDEX: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

/// One IMA ADPCM step: updates the predictor and the step index from a 4-bit code.
pub fn adpcm_step(pred: &mut i32, idx: &mut i32, code: u8) -> i16 {
    let step = STEPS[*idx as usize];
    let mut diff = step >> 3;
    if code & 4 != 0 {
        diff += step;
    }
    if code & 2 != 0 {
        diff += step >> 1;
    }
    if code & 1 != 0 {
        diff += step >> 2;
    }
    if code & 8 != 0 {
        *pred -= diff;
    } else {
        *pred += diff;
    }
    *pred = (*pred).clamp(-32768, 32767);
    *idx = (*idx + INDEX[code as usize & 15]).clamp(0, 88);
    *pred as i16
}

/// The key a text is looked up under.
pub fn key(text: &str) -> String {
    text.trim().trim_end_matches(['.', '!', '?', ',', ';', ':', '…']).trim().to_lowercase()
}

impl Voices {
    /// Reads a pack; `None` if it is not one.
    pub fn parse(data: Vec<u8>) -> Option<Voices> {
        let mut r = Reader { d: &data, at: 0 };
        if r.take(4)? != b"FVOX" || r.u8()? != 1 {
            return None;
        }
        let rate = r.u32()?;
        let count = r.u32()? as usize;
        let n_speakers = r.u32()? as usize;
        let mut speakers = Vec::with_capacity(n_speakers);
        for _ in 0..n_speakers {
            let len = r.u8()? as usize;
            speakers.push(String::from_utf8(r.take(len)?.to_vec()).ok()?);
        }
        let mut index = HashMap::with_capacity(count);
        for _ in 0..count {
            let len = r.u8()? as usize;
            let text = std::str::from_utf8(r.take(len)?).ok()?.to_string();
            let speaker = r.u16()?;
            let offset = r.u32()?;
            let samples = r.u32()?;
            index.insert(key(&text), (speaker, offset, samples));
        }
        let base = r.at as u32;
        for v in index.values_mut() {
            v.1 += base;
        }
        Some(Voices { data, rate, speakers, index })
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub fn has(&self, text: &str) -> bool {
        self.index.contains_key(&key(text))
    }

    /// Who recorded `text`.
    pub fn speaker(&self, text: &str) -> Option<&str> {
        self.index.get(&key(text)).and_then(|v| self.speakers.get(v.0 as usize)).map(|s| s.as_str())
    }

    /// Every speaker in the pack (for the credits).
    pub fn speakers(&self) -> &[String] {
        &self.speakers
    }

    /// The recording of `text`, decoded (mono, at [`Voices::rate`]).
    pub fn clip(&self, text: &str) -> Option<Vec<i16>> {
        let &(_, offset, samples) = self.index.get(&key(text))?;
        let d = self.data.get(offset as usize..)?;
        if samples == 0 || d.len() < 3 {
            return None;
        }
        let mut pred = i16::from_le_bytes([d[0], d[1]]) as i32;
        let mut idx = (d[2] as i32).clamp(0, 88);
        let mut out = Vec::with_capacity(samples as usize);
        out.push(pred as i16);
        let nibbles = samples as usize - 1;
        let bytes = d.get(3..3 + nibbles.div_ceil(2))?;
        for i in 0..nibbles {
            let b = bytes[i / 2];
            let code = if i % 2 == 0 { b & 15 } else { b >> 4 };
            out.push(adpcm_step(&mut pred, &mut idx, code));
        }
        Some(out)
    }
}

struct Reader<'a> {
    d: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let s = self.d.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }
    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }
    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}

/// Encodes clips into a pack (used by `fuera-cli voices`; the decoder above is the reference).
pub fn encode(rate: u32, speakers: &[String], clips: &[(String, u16, Vec<i16>)]) -> Vec<u8> {
    let mut head = Vec::new();
    head.extend_from_slice(b"FVOX");
    head.push(1);
    head.extend_from_slice(&rate.to_le_bytes());
    head.extend_from_slice(&(clips.len() as u32).to_le_bytes());
    head.extend_from_slice(&(speakers.len() as u32).to_le_bytes());
    for s in speakers {
        let b = &s.as_bytes()[..s.len().min(255)];
        head.push(b.len() as u8);
        head.extend_from_slice(b);
    }
    let mut data = Vec::new();
    for (text, speaker, pcm) in clips {
        let b = &text.as_bytes()[..text.len().min(255)];
        head.push(b.len() as u8);
        head.extend_from_slice(b);
        head.extend_from_slice(&speaker.to_le_bytes());
        head.extend_from_slice(&(data.len() as u32).to_le_bytes());
        head.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        if pcm.is_empty() {
            continue;
        }
        let mut pred = pcm[0] as i32;
        // Start with a step that suits the opening of the clip.
        let mut idx = 0i32;
        let first_diff = pcm.iter().take(32).zip(pcm.iter().skip(1)).map(|(a, b)| (*b as i32 - *a as i32).abs()).max().unwrap_or(0);
        while idx < 88 && STEPS[idx as usize] < first_diff {
            idx += 1;
        }
        data.extend_from_slice(&pcm[0].to_le_bytes());
        data.push(idx as u8);
        let mut byte = 0u8;
        for (i, &s) in pcm[1..].iter().enumerate() {
            let step = STEPS[idx as usize];
            let mut diff = s as i32 - pred;
            let mut code = 0u8;
            if diff < 0 {
                code = 8;
                diff = -diff;
            }
            let mut tmp = step;
            if diff >= tmp {
                code |= 4;
                diff -= tmp;
            }
            tmp >>= 1;
            if diff >= tmp {
                code |= 2;
                diff -= tmp;
            }
            tmp >>= 1;
            if diff >= tmp {
                code |= 1;
            }
            adpcm_step(&mut pred, &mut idx, code);
            if i % 2 == 0 {
                byte = code;
            } else {
                data.push(byte | (code << 4));
            }
        }
        if (pcm.len() - 1) % 2 == 1 {
            data.push(byte);
        }
    }
    head.extend_from_slice(&data);
    head
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_close() {
        let rate = 16_000;
        let tone: Vec<i16> = (0..8000).map(|i| ((i as f32 * 0.07).sin() * 9000.0 * (1.0 - i as f32 / 8000.0)) as i16).collect();
        let pack = encode(rate, &["Ana".into()], &[("Casa".into(), 0, tone.clone()), ("vacío".into(), 0, Vec::new())]);
        let v = Voices::parse(pack).unwrap();
        assert!(v.has("casa.") && v.has(" CASA ") && !v.has("perro"));
        assert_eq!(v.speaker("casa"), Some("Ana"));
        let out = v.clip("casa").unwrap();
        assert_eq!(out.len(), tone.len());
        let err: f64 = tone.iter().zip(&out).map(|(a, b)| ((*a as f64) - (*b as f64)).powi(2)).sum::<f64>() / tone.len() as f64;
        let sig: f64 = tone.iter().map(|a| (*a as f64).powi(2)).sum::<f64>() / tone.len() as f64;
        assert!(10.0 * (sig / err).log10() > 20.0, "SNR too low");
        assert!(v.clip("vacío").is_none());
    }
}
