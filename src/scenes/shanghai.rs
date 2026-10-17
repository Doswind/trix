//! 东方明珠 — Shanghai Bund at night.
//!
//! The Oriental Pearl tower, Pudong skyline, neon Bund buildings and the
//! Huangpu river.  ASCII by default; `--unicode` uses box drawing and blocks.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};

const NIGHT: Rgb = Rgb::hex(0x070b1f);
const DEEP: Rgb = Rgb::hex(0x111827);
const CYAN: Rgb = Rgb::hex(0x22d3ee);
const PINK: Rgb = Rgb::hex(0xf472b6);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const WHITE: Rgb = Rgb::hex(0xe0f2fe);
const WATER: Rgb = Rgb::hex(0x083344);

pub struct ShanghaiScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    buildings: Vec<(f32, f32, f32, Rgb)>,
    stars: Vec<(f32, f32, f32)>,
}

impl ShanghaiScene {
    pub fn new(seed: u64) -> Self {
        ShanghaiScene {
            seed,
            rng: Rng::new(seed ^ 0x5348_414e_4748),
            w: 0,
            h: 0,
            buildings: Vec::new(),
            stars: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.buildings.clear();
        let palette = [CYAN, PINK, GOLD, Rgb::hex(0x60a5fa), Rgb::hex(0xa78bfa)];
        let base = self.h as f32 * 0.72;
        let mut x = 0.0;
        while x < self.w as f32 {
            let bw = self.rng.range(3.0, 7.0);
            let bh = self.rng.range(self.h as f32 * 0.12, self.h as f32 * 0.34);
            self.buildings
                .push((x, base - bh, bw, palette[self.rng.below(palette.len())]));
            x += bw + self.rng.range(0.4, 1.8);
        }
        self.stars.clear();
        for _ in 0..(w * h / 90).max(30) {
            self.stars.push((
                self.rng.range(0.0, w as f32),
                self.rng.range(0.0, h as f32 * 0.55),
                self.rng.range(0.0, TAU),
            ));
        }
    }

    fn tower(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let cx = (self.w as f32 * 0.62) as isize;
        let base = (self.h as f32 * 0.72) as isize;
        let top = (self.h as f32 * 0.10) as isize;
        let vert = if unicode { '│' } else { '|' };
        let dia = if unicode { '╱' } else { '/' };
        let di2 = if unicode { '╲' } else { '\\' };
        // three columns / lattice shaft
        for y in top..base {
            let w = if y > base - (self.h as f32 * 0.18) as isize {
                2
            } else {
                1
            };
            for dx in -w..=w {
                let ch = if dx == 0 {
                    vert
                } else if (y / 2) % 2 == 0 {
                    dia
                } else {
                    di2
                };
                canvas.set(cx + dx, y, ch, WHITE.scale(0.42));
            }
            if y % 4 == 0 {
                canvas.set(cx - 2, y, if unicode { '·' } else { '.' }, GOLD.scale(0.55));
                canvas.set(cx + 2, y, if unicode { '·' } else { '.' }, PINK.scale(0.55));
            }
        }
        // antenna
        canvas.set(cx, top - 1, if unicode { '▴' } else { '^' }, GOLD);
        canvas.set(
            cx,
            top - 2,
            if unicode { '·' } else { '.' },
            GOLD.scale(0.8),
        );
        // three spheres: lower, middle, upper
        let spheres = [
            (base - (self.h as f32 * 0.18) as isize, 4.2, PINK),
            (base - (self.h as f32 * 0.33) as isize, 3.4, CYAN),
            (base - (self.h as f32 * 0.46) as isize, 2.3, GOLD),
        ];
        for (cy, r, col) in spheres {
            let rr = r as isize;
            for dy in -rr..=rr {
                for dx in -rr..=rr {
                    let d = dx * dx + dy * dy;
                    if d <= rr * rr {
                        let edge = d > (rr * rr * 3) / 4;
                        let ch = if unicode {
                            if edge {
                                '░'
                            } else {
                                '█'
                            }
                        } else if edge {
                            '.'
                        } else if dx == 0 || dy == 0 {
                            'O'
                        } else {
                            '#'
                        };
                        let shimmer = 0.82 + 0.18 * (t * 2.2 + cy as f32 * 0.2).sin();
                        canvas.set(
                            cx + dx,
                            cy + dy,
                            ch,
                            col.scale(if edge { 0.55 } else { 0.95 } * shimmer),
                        );
                    }
                }
            }
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU + t * 0.6;
                canvas.set(
                    cx + (a.cos() * (r + 1.2)) as isize,
                    cy + (a.sin() * (r + 1.2)) as isize,
                    if unicode { '·' } else { '*' },
                    GOLD.scale(0.55),
                );
            }
        }
        // splayed base legs and pedestal
        for dx in -9..=9 {
            canvas.set(
                cx + dx,
                base - 1,
                if unicode { '▄' } else { '_' },
                WHITE.scale(0.38),
            );
        }
        for k in 0..8 {
            canvas.set(cx - 8 + k, base - 2 - k / 2, dia, WHITE.scale(0.35));
            canvas.set(cx + 8 - k, base - 2 - k / 2, di2, WHITE.scale(0.35));
        }
        for y in (base - 5)..base {
            canvas.set(
                cx - 5,
                y,
                if unicode { '║' } else { '|' },
                WHITE.scale(0.40),
            );
            canvas.set(
                cx + 5,
                y,
                if unicode { '║' } else { '|' },
                WHITE.scale(0.40),
            );
        }
    }

    fn skyline(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        for (bx, by, bw, color) in &self.buildings {
            let x = *bx as isize;
            let y = *by as isize;
            let w = *bw as isize;
            let bottom = (self.h as f32 * 0.72) as isize;
            for yy in y..bottom {
                for xx in x..x + w {
                    let window =
                        (xx - x) > 0 && (xx - x) < w - 1 && yy > y + 1 && (xx + yy) % 3 != 0;
                    let on =
                        hash01((xx / 2) as i64, (yy / 2) as i64, self.seed.wrapping_add(9)) > 0.52;
                    let ch = if window && on {
                        if unicode {
                            '▦'
                        } else {
                            '#'
                        }
                    } else if unicode {
                        '▒'
                    } else {
                        ':'
                    };
                    let c = if window && on {
                        color.lerp(GOLD, 0.25).scale(0.9)
                    } else {
                        color.scale(0.20)
                    };
                    canvas.set(xx, yy, ch, c);
                }
            }
            // roof antenna
            if (*bx as i64) % 3 == 0 {
                canvas.set(
                    x + w / 2,
                    y - 1,
                    if unicode { '│' } else { '|' },
                    color.scale(0.7),
                );
                canvas.set(
                    x + w / 2,
                    y - 2,
                    if unicode { '·' } else { '.' },
                    PINK.scale(0.8),
                );
            }
        }
        // a few moving window glints
        for k in 0..8 {
            let bx = (self.w as f32 * (0.05 + k as f32 * 0.12)) as isize;
            let yy = (self.h as f32 * 0.42 + ((t + k as f32) % 3.0) * 2.0) as isize;
            canvas.set_if_blank(bx, yy, if unicode { '✦' } else { '*' }, GOLD.scale(0.45));
        }
    }

    fn river(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let base = (self.h as f32 * 0.72) as isize;
        let h = self.h as isize;
        for y in base..h {
            let v = (y - base) as f32 / (h - base).max(1) as f32;
            let c = WATER.lerp(NIGHT, 0.15 + v * 0.6);
            for x in 0..self.w as isize {
                let wave = ((x as f32 * 0.12 + t * 1.8 + y as f32 * 0.5).sin()) > 0.80;
                if wave {
                    let tint = if x % 17 == 0 {
                        GOLD
                    } else if x % 11 == 0 {
                        PINK
                    } else {
                        CYAN
                    };
                    canvas.set(x, y, if unicode { '≈' } else { '~' }, tint.scale(0.42));
                } else if (x + y * 3) % 5 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.6));
                }
            }
        }
        // neon reflections under the tower and major buildings
        for (i, col) in [PINK, CYAN, GOLD].iter().enumerate() {
            let x0 = (self.w as f32 * (0.62 + i as f32 * 0.07)) as isize;
            for y in base..h {
                let w = ((y - base) as f32 * 0.2) as isize + 1;
                for dx in -w..=w {
                    if ((dx + y + i as isize * 3) as f32 + t * 2.0).sin() > 0.0 {
                        canvas.set_if_blank(
                            x0 + dx,
                            y,
                            if unicode { '≈' } else { '~' },
                            col.scale(0.22),
                        );
                    }
                }
            }
        }
        // a small boat
        let bx = (self.w as f32 * 0.28 + (t * 0.4).sin() * 2.0) as isize;
        let by = base + (h - base) / 2;
        canvas.set(bx, by, if unicode { '▄' } else { '=' }, GOLD.scale(0.6));
        canvas.set(bx + 1, by, if unicode { '▄' } else { '=' }, GOLD.scale(0.5));
        canvas.set(
            bx + 2,
            by - 1,
            if unicode { '│' } else { '|' },
            WHITE.scale(0.4),
        );
    }
}

impl Scene for ShanghaiScene {
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
            let v = y as f32 / self.h.max(1) as f32;
            let c = NIGHT.lerp(DEEP, v * 0.8);
            for x in 0..self.w as isize {
                if (x + y * 3) % 8 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.8));
                }
            }
        }
        for (sx, sy, ph) in &self.stars {
            let tw = 0.5 + 0.5 * (t * 1.4 + ph).sin().abs();
            canvas.set(
                *sx as isize,
                *sy as isize,
                if unicode { '·' } else { '.' },
                WHITE.scale(tw * 0.6),
            );
        }
        // moon
        let mx = (self.w as f32 * 0.18) as isize;
        let my = (self.h as f32 * 0.16) as isize;
        for dy in -2..=2 {
            for dx in -3..=3 {
                if dx * dx + dy * dy * 2 <= 9 {
                    canvas.set(
                        mx + dx,
                        my + dy,
                        if unicode { '◉' } else { 'O' },
                        WHITE.scale(0.72),
                    );
                }
            }
        }
        self.skyline(canvas, unicode, t);
        self.tower(canvas, unicode, t);
        self.river(canvas, unicode, t);
        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ ORIENTAL PEARL // THE BUND ╞"
        } else {
            " ORIENTAL PEARL // THE BUND "
        };
        canvas.put_str(2, 0, title, CYAN.scale(0.9));
        let hint = if unicode {
            "SHANGHAI · HUANGPU RIVER · NIGHT"
        } else {
            "SHANGHAI // HUANGPU RIVER // NIGHT"
        };
        canvas.put_str(2, h - 1, hint, PINK.scale(0.55));
        let mode = if unicode { "UNICODE" } else { "ASCII" };
        canvas.put_str(
            (w - mode.len() as isize - 2).max(2),
            h - 1,
            mode,
            GOLD.scale(0.6),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn skyline_exists() {
        let mut s = ShanghaiScene::new(7);
        s.resize(120, 36);
        assert!(s.buildings.len() > 10);
    }
}
