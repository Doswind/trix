//! 光束 — irregular horizontal light beams in random bright colours.
//!
//! A sparse starfield and a faint scan grid fill the screen; a pool of beams
//! streaks left/right at different heights, widths and speeds, each with a
//! hot head and a fading tail.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};

const NEON: [Rgb; 8] = [
    Rgb::hex(0x22d3ee),
    Rgb::hex(0xff3df2),
    Rgb::hex(0xfacc15),
    Rgb::hex(0x4ade80),
    Rgb::hex(0xfb923c),
    Rgb::hex(0x60a5fa),
    Rgb::hex(0xff4d6d),
    Rgb::hex(0xffffff),
];

struct Beam {
    y: f32,
    x: f32,
    len: f32,
    speed: f32,
    width: f32,
    wobble: f32,
    phase: f32,
    color: Rgb,
    dir: f32,
}

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    color: Rgb,
}

pub struct BeamsScene {
    rng: Rng,
    w: usize,
    h: usize,
    beams: Vec<Beam>,
    sparks: Vec<Spark>,
    grid: Vec<(f32, f32)>,
    density: f32,
}

impl BeamsScene {
    pub fn new(seed: u64) -> Self {
        BeamsScene {
            rng: Rng::new(seed ^ 0xb3a5_5eed),
            w: 0,
            h: 0,
            beams: Vec::new(),
            sparks: Vec::new(),
            grid: Vec::new(),
            density: 1.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.beams.clear();
        self.sparks.clear();
        self.grid.clear();
        let n = ((h as f32 * 0.5 * self.density).round() as usize).clamp(4, 28);
        for _ in 0..n {
            let b = self.make_beam(true);
            self.beams.push(b);
        }
        for y in (2..self.h as isize).step_by(4) {
            for x in (3..self.w as isize).step_by(9) {
                self.grid.push((x as f32, y as f32));
            }
        }
    }

    fn make_beam(&mut self, initial: bool) -> Beam {
        let dir = 1.0;
        let len = self.rng.range(22.0, self.w as f32 * 0.62);
        let speed = self.rng.range(32.0, 110.0);
        let x = if initial {
            self.rng.range(-len, self.w as f32)
        } else {
            -len
        };
        Beam {
            y: self.rng.range(2.0, self.h as f32 - 2.0),
            x,
            len,
            speed,
            width: self.rng.range(0.6, 1.6),
            wobble: self.rng.range(0.0, 1.8),
            phase: self.rng.range(0.0, TAU),
            color: NEON[self.rng.below(NEON.len())],
            dir,
        }
    }

    fn rebuild_beam(&mut self, i: usize) {
        let b = self.make_beam(false);
        self.beams[i] = b;
    }
}

impl Scene for BeamsScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        if (f.tune.density - self.density).abs() > 0.01 {
            self.density = f.tune.density.clamp(0.05, 4.0);
            self.rebuild(self.w, self.h);
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let speed_mul = f.tune.beam_speed.clamp(0.05, 4.0);
        let mut reset = Vec::new();
        let mut spawn = Vec::new();
        for i in 0..self.beams.len() {
            let (x, y, dir, color, speed, len) = {
                let b = &mut self.beams[i];
                b.x += b.dir * b.speed * speed_mul * dt;
                (b.x, b.y, b.dir, b.color, b.speed, b.len)
            };
            let _ = (speed, len);
            if (dir > 0.0 && x > self.w as f32 + 4.0) || (dir < 0.0 && x + 4.0 < -4.0) {
                reset.push(i);
            } else if self.rng.chance(0.12) {
                spawn.push((
                    x,
                    y + self.rng.range(-1.0, 1.0),
                    self.rng.range(6.0, 24.0) * speed_mul,
                    self.rng.range(-4.0, 4.0),
                    self.rng.range(0.25, 0.8),
                    color,
                ));
            }
        }
        for i in reset {
            self.rebuild_beam(i);
        }
        for (x, y, vx, vy, life, color) in spawn {
            self.sparks.push(Spark {
                x,
                y,
                vx,
                vy,
                life,
                color,
            });
        }
        for s in self.sparks.iter_mut() {
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.life -= dt;
        }
        self.sparks
            .retain(|s| s.life > 0.0 && s.x > -4.0 && s.x < self.w as f32 + 4.0);
        if self.sparks.len() > 180 {
            let drop = self.sparks.len() - 180;
            self.sparks.drain(0..drop);
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;

        // Sparse background stars.
        for y in 0..self.h {
            for x in 0..self.w {
                let hsh = hash2(x as i64, y as i64, 0x51ee);
                if hsh % 43 != 0 {
                    continue;
                }
                let tw = 0.5 + 0.5 * (t * 2.0 + (hsh % 100) as f32 * 0.1).sin().abs();
                let col = if hsh & 1 == 0 {
                    theme.c(2).scale(0.3 * tw)
                } else {
                    theme.c(3).scale(0.28 * tw)
                };
                canvas.set(x as isize, y as isize, '.', col);
            }
        }
        // Tech grid nodes.
        for (x, y) in &self.grid {
            let tw = 0.4 + 0.6 * (t * 1.3 + x * 0.02).sin().abs();
            canvas.set(
                *x as isize,
                *y as isize,
                if ((*x as i64 + *y as i64) & 3) == 0 {
                    '+'
                } else {
                    '.'
                },
                theme.c(1).scale(0.4 * tw),
            );
        }

        // Beams.
        for b in &self.beams {
            let color = b.color.hue_rotate(t * 0.012 + b.phase * 0.01);
            let steps = b.len as isize;
            let head_x = b.x.round() as isize;
            for i in 0..steps {
                let x = head_x - (b.dir as isize) * i;
                if x < 0 || x >= self.w as isize {
                    continue;
                }
                let fade = 1.0 - i as f32 / b.len.max(1.0);
                let yy = if f.tune.beam_jump {
                    let amp = 2.5 + b.wobble * 1.6;
                    b.y + (x as f32 * 0.11 + b.phase + t * 2.6).sin() * amp
                } else {
                    b.y
                };
                let y = yy.round().clamp(1.0, self.h as f32 - 2.0) as isize;
                let ch = if i == 0 {
                    '@'
                } else if i < 5 {
                    '*'
                } else if i % 4 == 0 {
                    '='
                } else if i % 7 == 0 {
                    '~'
                } else {
                    '-'
                };
                let col = if i == 0 {
                    color.lerp(Rgb::hex(0xffffff), 0.9)
                } else if i < 5 {
                    color.lerp(Rgb::hex(0xffffff), 0.45)
                } else {
                    color.scale(fade.powf(0.5) * 1.1)
                };
                canvas.set(x, y, ch, col);
                if b.width > 1.2 && i < (b.len * 0.7) as isize {
                    canvas.set(x, y + 1, if i < 5 { '=' } else { '-' }, col.scale(0.45));
                }
                if i < 4 {
                    canvas.add_light(x, y, color, 0.55, '.');
                }
                // irregular split / sparkle
                if (x as i64 + i as i64 * 7) % 23 == 0 {
                    canvas.set(x, y - 1, '\'', color.scale(0.6));
                }
            }
        }

        // Sparks.
        for s in &self.sparks {
            canvas.set(
                s.x as isize,
                s.y as isize,
                if s.life > 0.4 { '*' } else { '.' },
                s.color.scale(s.life.min(1.0)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beams_cover_screen() {
        let mut s = BeamsScene::new(4);
        s.resize(80, 24);
        assert!(s.beams.len() >= 8);
        for b in &s.beams {
            assert!(b.y > 0.0 && b.y < 24.0);
        }
    }
}
