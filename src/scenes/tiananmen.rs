//! 光芒万丈 — Tiananmen gate tower radiating light.
//!
//! A centered red gate tower with golden tiered roofs sits against a dawn
//! sky; rays and particles fan upward.  ASCII by default; `--unicode` works.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const SKY_TOP: Rgb = Rgb::hex(0x0b1026);
const SKY_LOW: Rgb = Rgb::hex(0x7c2d12);
const RED: Rgb = Rgb::hex(0xb91c1c);
const RED_DARK: Rgb = Rgb::hex(0x7f1d1d);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const GOLD_LIGHT: Rgb = Rgb::hex(0xfef3c7);
const YELLOW: Rgb = Rgb::hex(0xfacc15);
const STONE: Rgb = Rgb::hex(0xd6d3d1);
const DARK: Rgb = Rgb::hex(0x1c1917);

struct Spark {
    a: f32,
    r: f32,
    speed: f32,
    phase: f32,
}

pub struct TiananmenScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    sparks: Vec<Spark>,
}

impl TiananmenScene {
    pub fn new(seed: u64) -> Self {
        TiananmenScene {
            seed,
            rng: Rng::new(seed ^ 0x5449_414e_414e),
            w: 0,
            h: 0,
            sparks: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.sparks.clear();
        for _ in 0..70 {
            self.sparks.push(Spark {
                a: self.rng.range(-2.7, -0.4),
                r: self.rng.range(2.0, 1.0),
                speed: self.rng.range(2.0, 8.0),
                phase: self.rng.range(0.0, TAU),
            });
        }
    }

    fn rays(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.48;
        for k in 0..26 {
            let a = -2.85 + k as f32 / 26.0 * 2.30;
            let len = self.h as f32 * 0.85;
            let x1 = cx + a.cos() * len;
            let y1 = cy + a.sin() * len * 0.9;
            let pulse = 0.18 + 0.12 * (t * 1.2 + k as f32 * 0.5).sin().abs();
            canvas.line(
                cx as isize,
                cy as isize,
                x1 as isize,
                y1 as isize,
                if unicode { '·' } else { '.' },
                GOLD.scale(pulse),
            );
        }
        for k in 0..9 {
            let a = -2.6 + k as f32 * 0.28;
            let len = self.h as f32 * 0.65;
            canvas.line(
                cx as isize,
                cy as isize,
                (cx + a.cos() * len) as isize,
                (cy + a.sin() * len * 0.9) as isize,
                if unicode { '╱' } else { '/' },
                GOLD.scale(0.10),
            );
        }
        for s in &self.sparks {
            let r = ((s.r * self.h as f32 + t * s.speed) % (self.h as f32 * 0.8)) + 2.0;
            let x = cx + s.a.cos() * r;
            let y = cy + s.a.sin() * r * 0.9;
            canvas.set(
                x as isize,
                y as isize,
                if unicode { '✦' } else { '*' },
                GOLD_LIGHT.scale(0.35 + 0.25 * (t + s.phase).sin().abs()),
            );
        }
    }

    fn tower(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let cx = (self.w as f32 * 0.5) as isize;
        let base = (self.h as f32 * 0.82) as isize;
        let half = ((self.w as f32 * 0.20) as isize).clamp(8, (self.w as isize / 2) - 2);
        let wall_top = base - (self.h as f32 * 0.18) as isize;
        // stone terrace
        for y in base..(base + 3) {
            for x in (cx - half - 3)..=(cx + half + 3) {
                canvas.set(x, y, if unicode { '█' } else { '=' }, STONE.scale(0.38));
            }
        }
        // main red wall
        for y in wall_top..base {
            for x in (cx - half)..=(cx + half) {
                let edge = x == cx - half || x == cx + half;
                let ch = if edge {
                    if unicode {
                        '║'
                    } else {
                        '|'
                    }
                } else if unicode {
                    '█'
                } else {
                    '#'
                };
                canvas.set(x, y, ch, RED.scale(0.72));
            }
        }
        // five arches
        let arch_w = (half * 2 / 7).max(2);
        for k in 0..5 {
            let ax = cx - half + 2 + k * (half * 2 - 4) / 5;
            let ah = (base - wall_top) / 2;
            for dy in 0..ah {
                for dx in 0..arch_w {
                    let top = dy < 2 && (dx == 0 || dx == arch_w - 1);
                    if !top {
                        canvas.set(
                            ax + dx,
                            base - 1 - dy,
                            if unicode { '▄' } else { '=' },
                            DARK,
                        );
                    }
                }
            }
        }
        // upper hall
        let top_y = wall_top - (self.h as f32 * 0.12) as isize;
        let hw = half - 2;
        for y in top_y..wall_top {
            for x in (cx - hw)..=(cx + hw) {
                canvas.set(x, y, if unicode { '█' } else { '#' }, RED.scale(0.58));
            }
        }
        // golden double roof with upturned eaves
        for (ry, rw) in [(top_y, hw + 4), (top_y - 4, hw + 1)] {
            for x in (cx - rw)..=(cx + rw) {
                canvas.set(x, ry, if unicode { '▄' } else { '^' }, GOLD);
                canvas.set(x, ry + 1, if unicode { '█' } else { '#' }, GOLD.scale(0.72));
            }
            for e in 0..4 {
                canvas.set(
                    cx - rw - e,
                    ry + 1 + e / 2,
                    if unicode { '╱' } else { '/' },
                    GOLD.scale(0.85),
                );
                canvas.set(
                    cx + rw + e,
                    ry + 1 + e / 2,
                    if unicode { '╲' } else { '\\' },
                    GOLD.scale(0.85),
                );
            }
            for x in (cx - rw + 2..cx + rw - 2).step_by(4) {
                canvas.set(
                    x,
                    ry - 1,
                    if unicode { '·' } else { '.' },
                    YELLOW.scale(0.55),
                );
            }
        }
        // central portrait / emblem on the main red wall
        let py = wall_top + (base - wall_top) / 2;
        for dy in -2isize..=2 {
            for dx in -3isize..=3 {
                let edge = dx.abs() == 3 || dy.abs() == 2;
                canvas.set(
                    cx + dx,
                    py + dy,
                    if edge {
                        if unicode {
                            '█'
                        } else {
                            '#'
                        }
                    } else if unicode {
                        '▒'
                    } else {
                        ':'
                    },
                    if edge { GOLD.scale(0.55) } else { RED_DARK },
                );
            }
        }
        canvas.set(cx, py, if unicode { '★' } else { '*' }, GOLD);
        for dx in -8..=8 {
            canvas.set(
                cx + dx,
                wall_top + 2,
                if unicode { '▀' } else { '=' },
                GOLD.scale(0.28),
            );
        }
        // flagpole and flag
        let fx = cx - half - 4;
        for y in (top_y - 10)..base {
            canvas.set(fx, y, if unicode { '│' } else { '|' }, STONE.scale(0.7));
        }
        let wave = (t * 3.0).sin();
        for k in 0..5 {
            canvas.set(
                fx + 1 + k / 2,
                top_y - 9 + (k as f32 * wave) as isize / 2,
                if unicode { '▰' } else { '=' },
                RED,
            );
        }
        // lanterns
        for k in 0..4 {
            let lx = cx - half + 2 + k * (half * 2 - 4) / 3;
            canvas.set(lx, wall_top + 2, if unicode { '●' } else { 'o' }, GOLD);
            canvas.add_light(lx, wall_top + 2, GOLD, 0.25, '.');
        }
    }
}

impl Scene for TiananmenScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / (self.h as f32 * 0.75).max(1.0);
            let c = SKY_TOP.lerp(SKY_LOW, v.min(1.0));
            for x in 0..self.w as isize {
                if (x + y * 5) % 11 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.5));
                }
            }
        }
        self.rays(canvas, unicode, t);
        // clouds
        for k in 0..4 {
            let x =
                ((t * (2.0 + k as f32) + k as f32 * 27.0) as isize) % (self.w as isize + 20) - 10;
            let y = (3 + k % 3) as isize;
            for i in 0..6 {
                canvas.set_if_blank(x + i, y, if unicode { '≈' } else { '~' }, GOLD.scale(0.20));
            }
        }
        self.tower(canvas, unicode, t);
        // plaza foreground + tiny people
        let base = (self.h as f32 * 0.86) as isize;
        for y in base..self.h as isize {
            for x in 0..self.w as isize {
                canvas.set(
                    x,
                    y,
                    if unicode { '█' } else { '#' },
                    Rgb::hex(0x292524).scale(0.45),
                );
            }
        }
        for k in 0..14 {
            let x = (self.w as f32 * (0.12 + k as f32 * 0.055)) as isize;
            canvas.set(
                x,
                base + 1,
                if unicode { '·' } else { '.' },
                GOLD_LIGHT.scale(0.22),
            );
            canvas.set(
                x + 1,
                base + 1,
                if unicode { '▴' } else { '^' },
                RED.scale(0.35),
            );
        }
        let h = self.h as isize;
        let title = if unicode {
            "╡ TIANANMEN // RADIANT LIGHT ╞"
        } else {
            " TIANANMEN // RADIANT LIGHT "
        };
        canvas.put_str(2, 0, title, GOLD.scale(0.9));
        let hint = "THE EAST IS RED   //   LIGHT OF DAWN";
        canvas.put_str(2, h - 1, hint, GOLD_LIGHT.scale(0.5));
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparks_exist() {
        let mut s = TiananmenScene::new(7);
        s.resize(120, 36);
        assert!(!s.sparks.is_empty());
    }
}
