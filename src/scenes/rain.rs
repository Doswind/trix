//! 代码雨 — cyan code rain.
//!
//! Falling columns of source-code glyphs with a bright white head, a
//! cyan→violet trail, sparse binary mist, and occasional horizontal data
//! packets crossing the stream.

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const GLYPHS: &[u8] = b"01{}[]()<>/\\=+-*&%$#@!?;:|^~01abcdefnrtu";

fn pick_glyph(rng: &mut Rng) -> u8 {
    GLYPHS[rng.below(GLYPHS.len())]
}

fn trail_color(theme: &Theme, p: f32) -> Rgb {
    let p = p.rem_euclid(1.0);
    if p < 0.5 {
        theme.c(1).lerp(theme.c(2), p * 2.0)
    } else {
        theme.c(2).lerp(theme.c(3), (p - 0.5) * 2.0)
    }
}

struct Column {
    head: f32,
    speed: f32,
    len: usize,
    phase: f32,
    depth: f32,
    flicker: f32,
    glyphs: Vec<u8>,
}

impl Column {
    fn spawn(&mut self, rng: &mut Rng, h: usize, initial: bool) {
        self.len = (rng.below(15) + 8).min(h.saturating_sub(1).max(2));
        self.speed = rng.range(5.0, 19.0);
        self.phase = rng.f32();
        self.depth = rng.f32();
        self.flicker = rng.range(0.8, 1.15);
        self.head = if initial {
            rng.range(-(self.len as f32), h as f32)
        } else {
            rng.range(-(self.len as f32) * 1.5, -(self.len as f32) * 0.3)
        };
        if self.glyphs.len() != h {
            self.glyphs.resize(h, b'0');
        }
        for g in self.glyphs.iter_mut() {
            *g = pick_glyph(rng);
        }
    }
}

struct Backdrop {
    x: usize,
    y: usize,
    glyph: u8,
    phase: f32,
    bright: f32,
}

struct Streak {
    x: f32,
    y: usize,
    speed: f32,
    len: usize,
    life: f32,
    max_life: f32,
    dir: f32,
}

pub struct RainScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    cols: Vec<Column>,
    backdrop: Vec<Backdrop>,
    streaks: Vec<Streak>,
    next_streak: f64,
}

impl RainScene {
    pub fn new(seed: u64) -> Self {
        RainScene {
            seed,
            rng: Rng::new(seed),
            w: 0,
            h: 0,
            cols: Vec::new(),
            backdrop: Vec::new(),
            streaks: Vec::new(),
            next_streak: 1.5,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.cols.clear();
        self.cols.reserve(w);
        for _ in 0..self.w {
            let mut c = Column {
                head: 0.0,
                speed: 10.0,
                len: 12,
                phase: 0.0,
                depth: 0.5,
                flicker: 1.0,
                glyphs: vec![b'0'; self.h],
            };
            c.spawn(&mut self.rng, self.h, true);
            self.cols.push(c);
        }

        self.backdrop.clear();
        let count = ((w * h) as f32 * 0.007) as usize;
        for _ in 0..count {
            self.backdrop.push(Backdrop {
                x: self.rng.below(self.w),
                y: self.rng.below(self.h),
                glyph: pick_glyph(&mut self.rng),
                phase: self.rng.range(0.0, std::f32::consts::TAU),
                bright: self.rng.range(0.08, 0.28),
            });
        }
        self.streaks.clear();
    }
}

impl Scene for RainScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let hf = self.h as f32;

        for col in self.cols.iter_mut() {
            col.head += col.speed * dt;
            let head_y = col.head as isize;
            if head_y >= 0 && head_y < self.h as isize && self.rng.chance(0.7) {
                let idx = head_y as usize;
                col.glyphs[idx] = pick_glyph(&mut self.rng);
            }
            // A little glyph mutation in the visible trail keeps it alive.
            if self.rng.chance(0.35) {
                let idx = self.rng.below(self.h);
                col.glyphs[idx] = pick_glyph(&mut self.rng);
            }
            if col.head - col.len as f32 > hf {
                col.spawn(&mut self.rng, self.h, false);
            }
        }

        // Horizontal data packets.
        if f.t > self.next_streak && self.streaks.len() < 3 {
            let dir = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
            let len = self.rng.below(18) + 8;
            let life = self.rng.range(0.6, 1.4);
            self.streaks.push(Streak {
                x: if dir > 0.0 {
                    -(len as f32)
                } else {
                    self.w as f32 + len as f32
                },
                y: 1 + self.rng.below(self.h.saturating_sub(2).max(1)),
                speed: self.rng.range(50.0, 120.0) * dir,
                len,
                life,
                max_life: life,
                dir,
            });
            self.next_streak = f.t + self.rng.range(2.0, 6.0) as f64;
        }
        for s in self.streaks.iter_mut() {
            s.x += s.speed * dt;
            s.life -= dt;
        }
        self.streaks
            .retain(|s| s.life > 0.0 && s.x + s.len as f32 > 0.0 && s.x < self.w as f32);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;

        // Binary mist in the background.
        for b in &self.backdrop {
            let tw = 0.65 + 0.35 * (t * 1.7 + b.phase).sin();
            let col = theme.c(1).scale(b.bright * tw);
            canvas.set(b.x as isize, b.y as isize, b.glyph as char, col);
        }

        for (x, col) in self.cols.iter().enumerate() {
            let depth_mul = 0.45 + col.depth * 0.55;
            let head_y = col.head.floor() as isize;
            for d in 0..col.len {
                let y = head_y - d as isize;
                if y < 0 || y >= self.h as isize {
                    continue;
                }
                let glyph = col.glyphs[y as usize] as char;
                let fade = 1.0 - (d as f32 / col.len as f32);
                let p = 0.05 + col.phase * 0.55 + t * 0.013 + d as f32 * 0.008;
                let mut color = if d == 0 {
                    theme.hot()
                } else {
                    trail_color(theme, p)
                };
                let lum = fade.powf(1.8) * depth_mul * col.flicker;
                color = color.scale(lum);
                if color.luminance() < 0.025 {
                    continue;
                }
                canvas.set(x as isize, y, glyph, color);
                if d == 0 {
                    // A one-cell halo under the head.
                    let halo = theme.c(2).scale(0.32 * depth_mul);
                    canvas.add_light(x as isize, y + 1, halo, 0.7, ':');
                    canvas.add_light(x as isize, y - 1, halo.scale(0.5), 0.4, '.');
                }
            }
        }

        // Data packets.
        for s in &self.streaks {
            let fade = (s.life / s.max_life).clamp(0.0, 1.0);
            for i in 0..s.len {
                let x = s.x - i as f32 * s.dir;
                if x < 0.0 || x >= self.w as f32 {
                    continue;
                }
                let f2 = 1.0 - i as f32 / s.len as f32;
                let ch = if i == 0 {
                    '@'
                } else if i % 4 == 0 {
                    '='
                } else if i % 2 == 0 {
                    '-'
                } else {
                    '.'
                };
                let col = if i == 0 {
                    theme.hot()
                } else {
                    theme.c(3).lerp(theme.c(2), 1.0 - f2)
                }
                .scale(f2.powf(1.4) * fade * 0.9);
                canvas.set(x as isize, s.y as isize, ch, col);
                if i == 0 {
                    canvas.add_light(x as isize, s.y as isize, theme.c(2), 0.35, '*');
                    canvas.add_light(x as isize, s.y as isize - 1, theme.c(2), 0.15, '.');
                }
            }
        }

        // A faint horizontal CRT-style scan every few rows, only where blank.
        if (t * 0.6).fract() < 0.35 {
            let band = ((t * 12.0) as usize) % self.h.max(1);
            for x in (0..self.w).step_by(3) {
                let h = hash01(x as i64, band as i64, self.seed);
                if h > 0.6 {
                    canvas.set_if_blank(x as isize, band as isize, '.', theme.c(1).scale(0.12));
                }
            }
        }
    }
}
