//! Sound output: a tiny mixer of one-shot voices on the default output device.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fuera_ui::sfx::{self, Sfx};

struct Voice {
    samples: Arc<Vec<f32>>,
    pos: usize,
}

pub struct Audio {
    stream: cpal::Stream,
    voices: Arc<Mutex<Vec<Voice>>>,
    rate: u32,
    cache: Mutex<Vec<(String, Arc<Vec<f32>>)>>,
}

impl Audio {
    pub fn open() -> Option<Audio> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        let rate = config.sample_rate();
        let fmt = config.sample_format();
        let sc: cpal::StreamConfig = config.into();
        let voices: Arc<Mutex<Vec<Voice>>> = Arc::new(Mutex::new(Vec::new()));
        let stream = match fmt {
            cpal::SampleFormat::F32 => build::<f32>(&device, sc, voices.clone()),
            cpal::SampleFormat::I16 => build::<i16>(&device, sc, voices.clone()),
            cpal::SampleFormat::U16 => build::<u16>(&device, sc, voices.clone()),
            cpal::SampleFormat::I32 => build::<i32>(&device, sc, voices.clone()),
            _ => None,
        }?;
        stream.play().ok()?;
        Some(Audio { stream, voices, rate, cache: Mutex::new(Vec::new()) })
    }

    pub fn resume(&self) {
        let _ = self.stream.play();
    }

    pub fn play(&self, s: Sfx) {
        let key = format!("{s:?}");
        let samples = {
            let mut c = self.cache.lock().unwrap();
            match c.iter().find(|(k, _)| *k == key) {
                Some((_, v)) => v.clone(),
                None => {
                    let v = Arc::new(sfx::synth(s, self.rate));
                    c.push((key, v.clone()));
                    v
                }
            }
        };
        if let Ok(mut v) = self.voices.lock() {
            if v.len() > 12 {
                v.remove(0);
            }
            v.push(Voice { samples, pos: 0 });
        }
    }
}

impl Audio {
    /// Plays a recording (mono 16-bit at `rate`), resampled to the output rate.
    pub fn play_pcm(&self, pcm: &[i16], rate: u32) {
        if pcm.is_empty() || rate == 0 {
            return;
        }
        let ratio = rate as f64 / self.rate as f64;
        let n = (pcm.len() as f64 / ratio) as usize;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 * ratio;
            let j = t as usize;
            let f = (t - j as f64) as f32;
            let a = pcm[j.min(pcm.len() - 1)] as f32;
            let b = pcm[(j + 1).min(pcm.len() - 1)] as f32;
            out.push((a + (b - a) * f) / 32768.0 * 1.1);
        }
        if let Ok(mut v) = self.voices.lock() {
            v.push(Voice { samples: Arc::new(out), pos: 0 });
        }
    }
}

fn build<T: SizedSample + FromSample<f32>>(device: &cpal::Device, config: cpal::StreamConfig, voices: Arc<Mutex<Vec<Voice>>>) -> Option<cpal::Stream> {
    let ch = config.channels.max(1) as usize;
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                let mut vs = match voices.try_lock() {
                    Ok(v) => v,
                    Err(_) => {
                        for s in data.iter_mut() {
                            *s = T::from_sample(0.0f32);
                        }
                        return;
                    }
                };
                for frame in data.chunks_mut(ch) {
                    let mut x = 0.0f32;
                    for v in vs.iter_mut() {
                        if v.pos < v.samples.len() {
                            x += v.samples[v.pos];
                            v.pos += 1;
                        }
                    }
                    let x = (x * 0.8).tanh();
                    for s in frame.iter_mut() {
                        *s = T::from_sample(x);
                    }
                }
                vs.retain(|v| v.pos < v.samples.len());
            },
            |e| log::warn!("audio: {e}"),
            None,
        )
        .ok()
}
