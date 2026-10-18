//! 满天星河 — a full-screen Milky Way.
//!
//! Thousands of precomputed stars fill the whole terminal; a dense diagonal
//! band plus a central bulge forms the galaxy.  A handful of shooting stars,
//! faint nebula dust and constellation lines add motion without many cells.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};

const STAR_CHARS: &[u8] = b".'`+*x";
const NEBULA_CHARS: &[u8] = b".:;'";

struct Star {
    x: f32,
    y: f32,
    base: f32,
    phase: f32,
    speed: f32,
    color: Rgb,
    ch: u8,
    constellation: u8,
}

struct Dust {
    x: f32,
    y: f32,
    color: Rgb,
    ch: u8,
    bright: f32,
}

struct Meteor {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    color: Rgb,
}

pub struct GalaxyScene {
    rng: Rng,
    seed: u64,
    w: usize,
    h: usize,
    stars: Vec<Star>,
    dust: Vec<Dust>,
    meteors: Vec<Meteor>,
    next_meteor: f32,
    density: f32,
}

impl GalaxyScene {
    pub fn new(seed: u64) -> Self {
        GalaxyScene {
            rng: Rng::new(seed ^ 0x9a1a_7e11),
            seed,
            w: 0,
            h: 0,
            stars: Vec::new(),
            dust: Vec::new(),
            meteors: Vec::new(),
            next_meteor: 1.0,
            density: 1.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.stars.clear();
        self.dust.clear();
        self.meteors.clear();
        let cells = self.w * self.h;
        let n = ((cells as f32 / 2.0 * self.density).round() as usize).clamp(300, 6800);
        let warm = [
            Rgb::hex(0xffffff),
            Rgb::hex(0xcfe8ff),
            Rgb::hex(0x9cc9ff),
            Rgb::hex(0xffe6a8),
            Rgb::hex(0xffb86b),
            Rgb::hex(0xff7b6b),
        ];
        for i in 0..n {
            let band = i % 20 < 17;
            let (mut x, mut y) = if band {
                // Diagonal Milky Way band, denser near the middle.
                let u = self.rng.range(0.0, 1.0);
                let bulge = self.rng.chance(0.26);
                let xf = if bulge {
                    0.5 + (self.rng.range(0.0, 1.0) - 0.5) * 0.42
                } else {
                    u
                };
                let center = 0.64 - xf * 0.30 + (xf * 5.0).sin() * 0.02;
                let spread = if bulge { 0.055 } else { 0.070 };
                let g = (self.rng.range(0.0, 1.0) + self.rng.range(0.0, 1.0) - 1.0) * spread;
                (xf * self.w as f32, (center + g) * self.h as f32)
            } else {
                (
                    self.rng.range(0.0, self.w as f32),
                    self.rng.range(0.0, self.h as f32),
                )
            };
            x += self.rng.range(-0.5, 0.5);
            y += self.rng.range(-0.5, 0.5);
            if x < 0.0 || x >= self.w as f32 || y < 0.0 || y >= self.h as f32 {
                continue;
            }
            let mut color = warm[self.rng.below(warm.len())];
            if self.rng.chance(0.08) {
                color = Rgb::from_hsv(self.rng.range(0.5, 0.72), 0.55, 1.0);
            }
            let base = if band {
                self.rng.range(0.22, 1.0)
            } else {
                self.rng.range(0.05, 0.35)
            };
            let ch = if base > 0.92 {
                b'*'
            } else if base > 0.7 {
                b'+'
            } else if band {
                b'.'
            } else {
                STAR_CHARS[self.rng.below(5)]
            };
            self.stars.push(Star {
                x,
                y,
                base,
                phase: self.rng.range(0.0, TAU),
                speed: self.rng.range(0.4, 2.2),
                color,
                ch,
                constellation: (self.rng.below(100) < 12) as u8,
            });
        }
        // Dust / faint nebula patches along the band.
        let dn = ((cells as f32 / 12.0 * self.density) as usize).clamp(40, 900);
        for _ in 0..dn {
            let x = self.rng.range(0.0, self.w as f32);
            let center = 0.64 - (x / self.w as f32) * 0.30;
            let y = (center + (self.rng.range(0.0, 1.0) - 0.5) * 0.18) * self.h as f32;
            let color = if self.rng.chance(0.5) {
                Rgb::hex(0x6d5dfc)
            } else {
                Rgb::hex(0x22d3ee)
            };
            self.dust.push(Dust {
                x,
                y,
                color,
                ch: NEBULA_CHARS[self.rng.below(NEBULA_CHARS.len())],
                bright: self.rng.range(0.18, 0.55),
            });
        }
        self.next_meteor = 1.0;
    }

    fn meteor(&mut self) {
        let from_left = self.rng.chance(0.6);
        let color = if self.rng.chance(0.5) {
            Rgb::hex(0xcfe8ff)
        } else {
            Rgb::hex(0xffe6a8)
        };
        let x = if from_left {
            self.rng.range(0.0, self.w as f32 * 0.7)
        } else {
            self.rng.range(self.w as f32 * 0.3, self.w as f32)
        };
        let y = self.rng.range(0.0, self.h as f32 * 0.35);
        let vx = if from_left {
            self.rng.range(18.0, 34.0)
        } else {
            -self.rng.range(18.0, 34.0)
        };
        let vy = self.rng.range(8.0, 16.0);
        self.meteors.push(Meteor {
            x,
            y,
            vx,
            vy,
            life: 1.2,
            max_life: 1.2,
            color,
        });
    }
}

impl Scene for GalaxyScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, _f: &Frame) -> Option<String> {
        Some(format!("STARS {}", self.stars.len()))
    }

    fn update(&mut self, f: &Frame) {
        if (f.tune.density - self.density).abs() > 0.01 {
            self.density = f.tune.density.clamp(0.05, 4.0);
            self.rebuild(self.w, self.h);
        }
        let dt = f.dt.clamp(0.0, 0.1);
        if f.t as f32 > self.next_meteor {
            if self.meteors.len() < f.tune.meteors as usize {
                self.meteor();
            }
            self.next_meteor = f.t as f32 + self.rng.range(1.5, 4.5);
        }
        for m in self.meteors.iter_mut() {
            m.x += m.vx * dt;
            m.y += m.vy * dt;
            m.life -= dt;
        }
        self.meteors
            .retain(|m| m.life > 0.0 && m.x > -20.0 && m.x < self.w as f32 + 20.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let twinkle = f.tune.twinkle.clamp(0.05, 8.0);
        // faint dust
        for d in &self.dust {
            let tw = 0.7 + 0.3 * (t * 0.7 * twinkle + d.x * 0.2).sin();
            canvas.set_if_blank(
                d.x as isize,
                d.y as isize,
                d.ch as char,
                d.color.scale(d.bright * tw),
            );
            if (d.x as i64 + d.y as i64) % 11 == 0 {
                canvas.add_light(d.x as isize, d.y as isize, d.color, 0.12, '.');
            }
        }

        // constellation guide lines between a few brighter stars
        let bright: Vec<&Star> = self
            .stars
            .iter()
            .filter(|s| s.constellation == 1 && s.base > 0.65)
            .take(10)
            .collect();
        for w in bright.windows(2) {
            if w[0].constellation == w[1].constellation {
                canvas.line(
                    w[0].x.round() as isize,
                    w[0].y.round() as isize,
                    w[1].x.round() as isize,
                    w[1].y.round() as isize,
                    '.',
                    Rgb::hex(0x1e4b6b),
                );
            }
        }

        // Draw brighter stars first so dim ones do not overwrite them.
        for s in &self.stars {
            let tw = 0.62 + 0.38 * (t * s.speed * twinkle + s.phase).sin().abs();
            let col = s.color.scale(s.base * tw);
            if col.luminance() < 0.015 {
                continue;
            }
            let sway = (t * 0.12 + s.y * 0.08).sin() * 0.7;
            canvas.set((s.x + sway) as isize, s.y as isize, s.ch as char, col);
            if s.base > 0.9 {
                canvas.add_light(s.x as isize, s.y as isize, s.color, 0.22, '.');
            }
        }

        // galactic core glow
        let cx = (self.w as f32 * 0.5) as isize;
        let cy = (self.h as f32 * 0.5) as isize;
        for r in 1..=(self.h as isize / 3).max(1) {
            for k in 0..6 {
                let a = k as f32 / 6.0 * TAU + t * 0.02;
                let x = cx + (a.cos() * r as f32 * 1.4).round() as isize;
                let y = cy + (a.sin() * r as f32 * 0.55).round() as isize;
                canvas.add_light(x, y, theme.c(2), 0.05, '.');
            }
        }

        // slowly turning galactic arcs
        for ring in 0..3 {
            let rr = (self.h as f32 * 0.10 + ring as f32 * 2.5).min(self.w as f32 * 0.30);
            let n = 90usize;
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU + t * (0.06 + ring as f32 * 0.02);
                let x = cx + (a.cos() * rr * 1.7).round() as isize;
                let y = cy + (a.sin() * rr * 0.55).round() as isize;
                canvas.set(
                    x,
                    y,
                    if k % 4 == 0 { '+' } else { '.' },
                    theme.c(2).scale(0.15),
                );
            }
        }

        // shooting stars
        for m in &self.meteors {
            let fade = (m.life / m.max_life).clamp(0.0, 1.0);
            let head = (m.x.round() as isize, m.y.round() as isize);
            canvas.set(head.0, head.1, '*', m.color.scale(fade));
            for i in 1..=8 {
                let x = head.0 - (m.vx.signum() * i as f32) as isize;
                let y = head.1 - (i as f32 * 0.45) as isize;
                let c = m.color.scale(fade * (1.0 - i as f32 / 9.0) * 0.7);
                canvas.set(x, y, if i < 3 { '-' } else { '.' }, c);
            }
            canvas.add_light(head.0, head.1, m.color, 0.8, '.');
        }

        let _ = self.seed;
        let _ = hash2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stars_fill_screen() {
        let mut s = GalaxyScene::new(3);
        s.resize(80, 24);
        assert!(s.stars.len() > 500);
        for st in &s.stars {
            assert!(st.x >= 0.0 && st.x < 80.0 && st.y >= 0.0 && st.y < 24.0);
        }
    }
}
