//! 燃烧火焰 — a heat-diffusion fire with a real flame palette.
//!
//! The simulation runs on a coarse per-cell heat buffer at 30 Hz; rendering
//! maps heat → density glyph → black-red-orange-yellow-white color.  Rising
//! embers and occasional flare-ups keep it alive without much CPU.

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const RAMP: &[u8] = b" .:-=+*#%@";

struct Pile {
    x: f32,
    width: f32,
    heat: f32,
    phase: f32,
    drift: f32,
    rate: f32,
    tilt: f32,
}

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    heat: f32,
}

/// Maximum heat in the classic Doom-fire cellular grid.  Taller than the
/// v0.1 value (36) so the central column of flame can reach the top rows.
const FIRE_MAX: f32 = 60.0;

pub struct FireScene {
    rng: Rng,
    seed: u64,
    w: usize,
    h: usize,
    heat: Vec<u8>,
    sparks: Vec<Spark>,
    piles: Vec<Pile>,
    flare: f32,
    next_flare: f32,
    acc: f32,
}

impl FireScene {
    pub fn new(seed: u64) -> Self {
        FireScene {
            rng: Rng::new(seed ^ 0xf1f1_f1f1),
            seed,
            w: 0,
            h: 0,
            heat: Vec::new(),
            sparks: Vec::new(),
            piles: Vec::new(),
            flare: 0.0,
            next_flare: 2.5,
            acc: 0.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.heat = vec![0; self.w * self.h];
        self.sparks.clear();
        self.acc = 0.0;
        self.piles.clear();
        let unit = (self.w as f32 / 120.0).clamp(0.5, 1.7);
        let n = (self.w / 24).clamp(3, 5);
        let cx = self.w as f32 * 0.5;
        for i in 0..n {
            let slot = (i as f32 - (n as f32 - 1.0) * 0.5) / (n as f32 * 1.5);
            let x = cx + slot * self.w as f32 + self.rng.range(-0.8, 0.8) * unit;
            let is_center = i == n / 2;
            self.piles.push(Pile {
                x: x.clamp(4.0, self.w as f32 - 5.0),
                width: if is_center {
                    7.2
                } else {
                    self.rng.range(5.0, 7.5)
                } * unit,
                heat: if is_center {
                    60.0
                } else {
                    self.rng.range(34.0, 48.0)
                },
                phase: self.rng.range(0.0, std::f32::consts::TAU),
                drift: self.rng.range(1.0, 3.0) * unit,
                rate: self.rng.range(0.6, 1.4),
                tilt: if is_center {
                    0.0
                } else if slot < 0.0 {
                    -self.rng.range(8.0, 15.0)
                } else {
                    self.rng.range(8.0, 15.0)
                },
            });
        }
        // Exactly one pile owns the tall center flame.
        if let Some(center) = self.piles.get_mut(n / 2) {
            center.x = cx;
            center.heat = 60.0;
            center.width = 7.2 * unit;
            center.tilt = 0.0;
        }
        // Warm start: a soft column of heat so the first frame already glows
        // instead of fading in from black.
        for y in 0..self.h {
            let depth = (y + 1) as f32 / self.h as f32;
            for x in 0..self.w {
                let n = (hash2(x as i64, y as i64, self.seed) & 0xff) as f32 / 255.0;
                let mound = (1.0 - (x as f32 - self.w as f32 * 0.5).abs() / (self.w as f32 * 0.50))
                    .max(0.0);
                self.heat[y * self.w + x] =
                    ((depth.powf(3.0) * (8.0 + 26.0 * mound) + n * 3.0).min(36.0)) as u8;
            }
        }
    }

    /// The classic Doom-fire cellular update: every cell copies its lower
    /// neighbour, loses a little energy, and drifts sideways.  It is cheap,
    /// stable at any frame rate, and produces a ragged, living flame front.
    fn simulate(&mut self, t: f32) {
        let (w, h) = (self.w, self.h);
        if w == 0 || h < 2 {
            return;
        }
        let max = FIRE_MAX + self.flare * 8.0;
        // Several independent, slender fuel piles.  Each has its own width,
        // heat and drift, so the scene gets multiple separate flame tongues
        // instead of one central bonfire.
        for x in 0..w {
            let xf = x as f32;
            let center = self.w as f32 * 0.5;
            let dxc = (xf - center) / (self.w as f32 * 0.44);
            let mut v = 5.0 + 22.0 * (-dxc * dxc).exp();
            for pile in &self.piles {
                let cx = pile.x + (t * pile.rate + pile.phase).sin() * pile.drift;
                let d = (xf - cx) / pile.width;
                let pulse = 0.93 + 0.07 * (t * (2.0 + pile.rate) + pile.phase).sin();
                v += pile.heat * pulse * (-d * d).exp();
            }
            let flick =
                ((hash2(x as i64, (t * 30.0) as i64, self.seed ^ 3) & 15) as f32 - 7.5) * 0.35;
            let v = (v + flick).clamp(4.0, max);
            self.heat[(h - 1) * w + x] = v as u8;
        }

        for y in 0..h - 1 {
            let row = y * w;
            let below = (y + 1) * w;
            for x in 0..w {
                let src = self.heat[below + x];
                let decay = self.rng.below(2) as u8;
                let drift = self.rng.below(3) as isize - 1;
                let dx = (x as isize + drift).rem_euclid(w as isize) as usize;
                self.heat[row + dx] = src.saturating_sub(decay);
            }
        }
    }

    fn spawn_sparks(&mut self, dt: f32) {
        if self.sparks.len() > 240 {
            return;
        }
        let n = if self.rng.chance(0.85) { 2 } else { 1 };
        for _ in 0..n {
            let x = self.rng.range(0.0, self.w as f32);
            let y = self.h as f32 - self.rng.range(0.0, 3.0);
            let life = self.rng.range(0.6, 2.2);
            self.sparks.push(Spark {
                x,
                y,
                vx: self.rng.range(-5.0, 5.0),
                vy: self.rng.range(-24.0, -8.0),
                life,
                max_life: life,
                heat: self.rng.range(0.5, 1.0),
            });
        }
        for s in self.sparks.iter_mut() {
            s.life -= dt;
            if s.life <= 0.0 {
                continue;
            }
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.vy += 10.0 * dt; // slow down / fall back
            s.vx += (s.y * 0.16 + s.x * 0.03).sin() * 6.0 * dt;
        }
        self.sparks
            .retain(|s| s.life > 0.0 && s.y > -1.0 && s.x >= 0.0 && s.x < self.w as f32);
    }
}

fn flame_color(theme: &Theme, f: f32) -> Rgb {
    let f = f.clamp(0.0, 1.0);
    if f < 0.22 {
        theme.c(0).lerp(theme.c(1), f / 0.22)
    } else if f < 0.5 {
        theme.c(1).lerp(theme.c(2), (f - 0.22) / 0.28)
    } else if f < 0.76 {
        theme.c(2).lerp(theme.c(3), (f - 0.5) / 0.26)
    } else {
        theme.c(3).lerp(theme.c(4), (f - 0.76) / 0.24)
    }
}

impl Scene for FireScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let t = f.t as f32;
        if t > self.next_flare {
            self.flare = 1.0;
            self.next_flare = t + self.rng.range(3.5, 8.0);
        }
        self.flare = (self.flare - dt * 1.2).max(0.0);
        // Keep the fire simulation at a fixed 30 Hz regardless of render FPS.
        self.acc = (self.acc + dt).min(0.1);
        let mut steps = 0;
        while self.acc >= 1.0 / 30.0 && steps < 3 {
            self.acc -= 1.0 / 30.0;
            self.simulate(t);
            steps += 1;
        }
        self.spawn_sparks(dt);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let w = self.w;
        let h = self.h;
        for y in 0..h {
            for x in 0..w {
                let mut raw = self.heat[y * w + x] as f32 / 36.0;
                if raw < 0.03 {
                    continue;
                }
                // Narrow each pile's plume as it rises: the base can merge,
                // while the tips become thin, separate tongues.
                let depth = y as f32 / (h as f32 - 1.0).max(1.0);
                let xf = x as f32;
                let mut env = 0.0f32;
                for pile in &self.piles {
                    let cx = pile.x + (t * pile.rate + pile.phase).sin() * pile.drift;
                    // Only the bottom footprint is doubled: a wide, overlapping fire bed,
                    // while the tips keep their original separation.
                    let width = pile.width * (0.55 + 0.95 * depth) * (1.0 + depth.powi(6));
                    let d = (xf - cx).abs() / width;
                    let e = 1.0 - ((d - 0.45).max(0.0) / 0.55).clamp(0.0, 1.0);
                    env = env.max(e);
                }
                env = env * 0.95 + 0.04;
                // Bottom bed: the piles merge into one continuous fire line,
                // while the tops above remain separate tongues.
                let dxc = (xf - self.w as f32 * 0.5) / (self.w as f32 * 0.48);
                let bed = 0.98 * depth.powf(5.0) * (-dxc * dxc).exp();
                env = env.max(bed);
                let top_fade = (y as f32 / 1.5).clamp(0.0, 1.0) * 0.25 + 0.75;
                raw *= env * top_fade;
                if raw < 0.03 {
                    continue;
                }
                let flicker = 0.88
                    + 0.12 * (t * 23.0 + x as f32 * 0.9 + y as f32 * 1.7).sin()
                    + ((hash2(x as i64, y as i64, self.seed ^ 7) & 0xff) as f32 / 255.0 - 0.5)
                        * 0.10;
                let v = (raw * flicker).clamp(0.0, 1.0);
                let idx =
                    ((v.powf(0.62) * (RAMP.len() - 1) as f32).round() as usize).min(RAMP.len() - 1);
                if idx == 0 {
                    continue;
                }
                let ch = RAMP[idx] as char;
                let mut color = flame_color(theme, (v * 1.12).min(1.0));
                if v > 0.82 {
                    color = color.lerp(theme.hot(), (v - 0.82) * 3.2);
                }
                canvas.set(x as isize, y as isize, ch, color);
            }
        }

        // Sparks.
        for s in &self.sparks {
            let life = (s.life / s.max_life).clamp(0.0, 1.0);
            let x = s.x as isize;
            let y = s.y as isize;
            let col = flame_color(theme, 0.55 + s.heat * 0.45).scale(0.6 + life * 0.8);
            let ch = if life > 0.7 {
                '@'
            } else if life > 0.35 {
                '*'
            } else {
                '.'
            };
            canvas.set(x, y, ch, col);
            canvas.add_light(x, y, theme.c(3).scale(0.5), 0.35, '.');
        }
    }
}
