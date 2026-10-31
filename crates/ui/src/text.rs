//! Text with fontdue: glyph cache, measuring, drawing, wrapping.

use std::collections::HashMap;

use tiny_skia::{Color, Pixmap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    /// Lilita One: titles, numbers, buttons.
    Title,
    /// Atkinson Hyperlegible: body text and words.
    Body,
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

pub struct Fonts {
    faces: [fontdue::Font; 3],
    cache: HashMap<(Face, char, u16), (fontdue::Metrics, Vec<u8>)>,
}

static TITLE: &[u8] = include_bytes!("../../../assets/fonts/LilitaOne-Regular.ttf");
static BODY: &[u8] = include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Regular.ttf");
static BOLD: &[u8] = include_bytes!("../../../assets/fonts/AtkinsonHyperlegible-Bold.ttf");

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    pub fn new() -> Fonts {
        let load = |b: &[u8]| fontdue::Font::from_bytes(b, fontdue::FontSettings::default()).expect("font");
        Fonts { faces: [load(TITLE), load(BODY), load(BOLD)], cache: HashMap::new() }
    }

    fn font(&self, f: Face) -> &fontdue::Font {
        &self.faces[f as usize]
    }

    fn glyph(&mut self, f: Face, c: char, px: f32) -> &(fontdue::Metrics, Vec<u8>) {
        let key = (f, c, (px * 4.0).round() as u16);
        if !self.cache.contains_key(&key) {
            if self.cache.len() > 4000 {
                self.cache.clear();
            }
            let g = self.faces[f as usize].rasterize(c, px);
            self.cache.insert(key, g);
        }
        &self.cache[&key]
    }

    pub fn width(&self, f: Face, s: &str, px: f32) -> f32 {
        let font = self.font(f);
        let mut w = 0.0;
        let mut prev: Option<char> = None;
        for c in s.chars() {
            if let Some(p) = prev {
                w += font.horizontal_kern(p, c, px).unwrap_or(0.0);
            }
            w += font.metrics(c, px).advance_width;
            prev = Some(c);
        }
        w
    }

    pub fn ascent(&self, f: Face, px: f32) -> f32 {
        self.font(f).horizontal_line_metrics(px).map(|m| m.ascent).unwrap_or(px * 0.8)
    }

    pub fn descent(&self, f: Face, px: f32) -> f32 {
        self.font(f).horizontal_line_metrics(px).map(|m| -m.descent).unwrap_or(px * 0.2)
    }

    /// Draws `s` with its baseline at `y`. Returns the width.
    pub fn draw(&mut self, pm: &mut Pixmap, f: Face, s: &str, x: f32, y: f32, px: f32, color: Color, align: Align) -> f32 {
        let w = self.width(f, s, px);
        let mut pen = match align {
            Align::Left => x,
            Align::Center => x - w / 2.0,
            Align::Right => x - w,
        };
        let (pw, ph) = (pm.width() as i32, pm.height() as i32);
        let c = color.premultiply().to_color_u8();
        let (cr, cg, cb, ca) = (c.red() as u32, c.green() as u32, c.blue() as u32, c.alpha() as u32);
        let mut prev: Option<char> = None;
        for ch in s.chars() {
            if let Some(p) = prev {
                pen += self.font(f).horizontal_kern(p, ch, px).unwrap_or(0.0);
            }
            prev = Some(ch);
            let (m, bmp) = {
                let g = self.glyph(f, ch, px);
                (g.0, g.1.clone())
            };
            let gx = (pen + m.xmin as f32).round() as i32;
            let gy = (y - m.height as f32 - m.ymin as f32).round() as i32;
            let data = pm.data_mut();
            for row in 0..m.height as i32 {
                let yy = gy + row;
                if yy < 0 || yy >= ph {
                    continue;
                }
                for col in 0..m.width as i32 {
                    let xx = gx + col;
                    if xx < 0 || xx >= pw {
                        continue;
                    }
                    let cov = bmp[(row * m.width as i32 + col) as usize] as u32;
                    if cov == 0 {
                        continue;
                    }
                    let a = cov * ca / 255;
                    let i = ((yy * pw + xx) * 4) as usize;
                    let inv = 255 - a;
                    data[i] = ((cr * cov / 255) + data[i] as u32 * inv / 255).min(255) as u8;
                    data[i + 1] = ((cg * cov / 255) + data[i + 1] as u32 * inv / 255).min(255) as u8;
                    data[i + 2] = ((cb * cov / 255) + data[i + 2] as u32 * inv / 255).min(255) as u8;
                    data[i + 3] = (a + data[i + 3] as u32 * inv / 255).min(255) as u8;
                }
            }
            pen += m.advance_width;
        }
        w
    }

    /// Text with a soft dark shadow below (titles on the sea background).
    pub fn draw_shadowed(&mut self, pm: &mut Pixmap, f: Face, s: &str, x: f32, y: f32, px: f32, color: Color, align: Align) -> f32 {
        let off = (px * 0.06).max(1.0);
        self.draw(pm, f, s, x, y + off, px, Color::from_rgba8(0, 0, 0, 90), align);
        self.draw(pm, f, s, x, y, px, color, align)
    }

    /// Splits `s` into lines no wider than `max_w`.
    pub fn wrap(&self, f: Face, s: &str, px: f32, max_w: f32) -> Vec<String> {
        let mut lines = Vec::new();
        for para in s.split('\n') {
            let mut line = String::new();
            for word in para.split(' ') {
                let cand = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
                if self.width(f, &cand, px) <= max_w || line.is_empty() {
                    line = cand;
                } else {
                    lines.push(std::mem::take(&mut line));
                    line = word.to_string();
                }
            }
            lines.push(line);
        }
        lines
    }

    /// Largest size ≤ `px` at which `s` fits in `max_w`.
    pub fn fit(&self, f: Face, s: &str, px: f32, max_w: f32) -> f32 {
        let w = self.width(f, s, px);
        if w <= max_w { px } else { (px * max_w / w).max(6.0) }
    }
}
