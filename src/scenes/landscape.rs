//! 五光十色 — a colorful Chinese landscape.
//!
//! Clear layered mountain silhouettes, a wide lake, a small boat, geese, and
//! lush jungle at both edges.  Animated ripples, mist and drifting petals add
//! movement while the composition stays readable.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const SKY_TOP: Rgb = Rgb::hex(0x0b1026);
const SKY_MID: Rgb = Rgb::hex(0x1d4ed8);
const SKY_LOW: Rgb = Rgb::hex(0xf59e0b);
const SUN: Rgb = Rgb::hex(0xfbbf24);
const MIST: Rgb = Rgb::hex(0xcffafe);
const LAKE: Rgb = Rgb::hex(0x0e7490);
const LAKE_DARK: Rgb = Rgb::hex(0x083344);
const HULL: Rgb = Rgb::hex(0x78350f);
const LEAF: Rgb = Rgb::hex(0x15803d);
const LEAF_LIGHT: Rgb = Rgb::hex(0x4ade80);
const BLOSSOM: Rgb = Rgb::hex(0xf472b6);
const WHITE: Rgb = Rgb::hex(0xf8fafc);

struct Tree {
    x: f32,
    size: f32,
    phase: f32,
    tint: Rgb,
    blossom: bool,
}

struct Petal {
    x: f32,
    y: f32,
    phase: f32,
    speed: f32,
}

pub struct LandscapeScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    trees: Vec<Tree>,
    petals: Vec<Petal>,
}

impl LandscapeScene {
    pub fn new(seed: u64) -> Self {
        LandscapeScene {
            seed,
            rng: Rng::new(seed ^ 0x4c41_4e44),
            w: 0,
            h: 0,
            trees: Vec::new(),
            petals: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.trees.clear();
        for side in 0..2 {
            for i in 0..4 {
                let x = if side == 0 {
                    self.rng.range(0.02, 0.17) * self.w as f32
                } else {
                    self.rng.range(0.83, 0.98) * self.w as f32
                };
                self.trees.push(Tree {
                    x,
                    size: self.rng.range(2.5, 5.0),
                    phase: self.rng.range(0.0, TAU),
                    tint: if i % 2 == 0 { LEAF } else { LEAF_LIGHT },
                    blossom: i == 1 || i == 3,
                });
            }
        }
        self.petals.clear();
        for _ in 0..36 {
            self.petals.push(Petal {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                phase: self.rng.range(0.0, TAU),
                speed: self.rng.range(0.5, 1.8),
            });
        }
    }

    fn horizon(&self) -> f32 {
        self.h as f32 * 0.50
    }

    fn mountains(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let horizon = self.horizon() as isize;
        let layers = [
            (0.32, Rgb::hex(0x1e3a8a), 0.52, 0.0),
            (0.25, Rgb::hex(0x0f766e), 0.66, 1.0),
            (0.19, Rgb::hex(0x14532d), 0.82, 2.0),
        ];
        for (li, (amp, color, dim, seed_off)) in layers.iter().enumerate() {
            for x in 0..self.w as isize {
                let u = x as f32 / self.w.max(1) as f32;
                let mut ridge = 0.0f32;
                for k in 0..5 {
                    let cx = 0.10
                        + k as f32 * 0.21
                        + 0.05 * ((k as f32 * 1.7 + *seed_off + li as f32).sin());
                    let ph = 0.55 + 0.40 * ((k as f32 * 2.3 + *seed_off * 1.9 + li as f32).cos());
                    let d = (u - cx).abs();
                    ridge = ridge.max((ph - d * 1.8).max(0.0));
                }
                let hgt = (self.h as f32 * amp * ridge).max(1.0) as isize;
                let top = horizon - hgt;
                for y in top..horizon {
                    let depth = (y - top) as f32 / hgt.max(1) as f32;
                    let ch = if y == top {
                        if unicode {
                            '▴'
                        } else {
                            '^'
                        }
                    } else if unicode {
                        '█'
                    } else {
                        '#'
                    };
                    let c = if y <= top + 1 && hgt > 5 {
                        MIST.scale(0.55)
                    } else {
                        color.scale(dim - depth * 0.10)
                    };
                    canvas.set(x, y, ch, c);
                }
                if hgt > 6 && x % 9 == 0 {
                    canvas.set(x, top + 1, if unicode { '·' } else { '.' }, MIST.scale(0.4));
                }
            }
            // a few very high snow peaks
            for k in 0..4 {
                let pk = ((self.w as f32 * (0.18 + k as f32 * 0.20)) as isize)
                    .clamp(1, self.w as isize - 2);
                let ph = (self.h as f32 * amp * 0.95) as isize;
                canvas.set(
                    pk,
                    horizon - ph,
                    if unicode { '✦' } else { '*' },
                    MIST.scale(0.9),
                );
                canvas.set(
                    pk - 1,
                    horizon - ph + 1,
                    if unicode { '·' } else { '.' },
                    MIST.scale(0.6),
                );
            }
        }
        // sun
        let sx = (self.w as f32 * 0.72) as isize;
        let sy = (self.h as f32 * 0.18) as isize;
        for dy in -5..=5 {
            for dx in -9..=9 {
                let d = (dx * dx) as f32 / 81.0 + (dy * dy) as f32 / 25.0;
                if d <= 1.0 {
                    canvas.add_light(
                        sx + dx,
                        sy + dy,
                        SUN,
                        0.06 * (1.0 - d),
                        if unicode { '░' } else { ':' },
                    );
                }
            }
        }
        canvas.set(sx, sy, if unicode { '◉' } else { 'O' }, SUN);
        for k in 0..12 {
            let a = k as f32 / 12.0 * TAU + t * 0.02;
            let x = sx + (a.cos() * 13.0) as isize;
            let y = sy + (a.sin() * 6.0) as isize;
            canvas.set_if_blank(x, y, if unicode { '·' } else { '-' }, SUN.scale(0.42));
        }
    }

    fn lake(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let horizon = self.horizon() as isize;
        let h = self.h as isize;
        for y in horizon..h {
            let v = (y - horizon) as f32 / (h - horizon).max(1) as f32;
            let c = LAKE.lerp(LAKE_DARK, 0.15 + v * 0.75);
            for x in 0..self.w as isize {
                let wave = ((x as f32 * 0.11 + t * 1.8 + y as f32 * 0.55).sin()) > 0.78;
                let ch = if wave {
                    if unicode {
                        '≈'
                    } else {
                        '~'
                    }
                } else if unicode {
                    '░'
                } else {
                    '.'
                };
                canvas.set(
                    x,
                    y,
                    ch,
                    if wave {
                        c.lerp(MIST, 0.45)
                    } else {
                        c.scale(0.45)
                    },
                );
            }
        }
        // sun reflection
        let sx = (self.w as f32 * 0.72) as isize;
        for y in (horizon + 2)..h {
            let half = ((y - horizon) as f32 * 0.28) as isize + 1;
            for dx in -half..=half {
                if ((dx + y) as f32 + t * 2.0).sin() > -0.1 {
                    canvas.set_if_blank(
                        sx + dx,
                        y,
                        if unicode { '≈' } else { '~' },
                        SUN.scale(0.18),
                    );
                }
            }
        }
        // boat
        let bx = (self.w as f32 * 0.56 + (t * 0.5).sin() * 1.2) as isize;
        let by = horizon + (h - horizon) / 2;
        for dx in -4..=4 {
            canvas.set(
                bx + dx,
                by,
                if unicode { '▄' } else { '=' },
                HULL.scale(0.9),
            );
        }
        canvas.set(bx - 4, by - 1, if unicode { '╱' } else { '/' }, HULL);
        canvas.set(bx + 4, by - 1, if unicode { '╲' } else { '\\' }, HULL);
        canvas.set(bx, by - 1, if unicode { '│' } else { '|' }, HULL.scale(0.8));
        for k in 0..4 {
            canvas.set(
                bx + 1 + k / 2,
                by - 2 - k,
                if unicode { '▰' } else { '/' },
                WHITE.scale(0.65),
            );
        }
        canvas.set(
            bx - 1,
            by - 2,
            if unicode { '·' } else { 'o' },
            WHITE.scale(0.8),
        );
        // geese, a small V on the water
        for i in 0..4 {
            let gx =
                (self.w as f32 * 0.65 + i as f32 * 4.0 + (t * 0.3 + i as f32).sin() * 1.0) as isize;
            let gy = by + 2 + (i % 2);
            canvas.set(gx, gy, if unicode { '○' } else { 'o' }, WHITE.scale(0.8));
            canvas.set(
                gx + 1,
                gy - 1,
                if unicode { '│' } else { '|' },
                WHITE.scale(0.65),
            );
            canvas.set(
                gx + 2,
                gy - 1,
                if unicode { '·' } else { 'v' },
                SUN.scale(0.7),
            );
        }
    }

    fn jungle(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let ground = (self.h as f32 * 0.90) as isize;
        for tree in &self.trees {
            let x = tree.x.round() as isize;
            let top = ground - (tree.size * 3.0) as isize;
            for y in top..ground {
                canvas.set(x, y, if unicode { '║' } else { '|' }, HULL.scale(0.65));
            }
            let r = tree.size as isize + 1;
            for dy in -r..=r {
                for dx in -r..=r {
                    let d = dx * dx + dy * dy;
                    if d <= r * r + r {
                        let sway = ((t * 0.4 + tree.phase + dy as f32 * 0.2).sin() * 0.5) as isize;
                        let ch = if tree.blossom && (dx + dy) % 3 == 0 {
                            if unicode {
                                '✿'
                            } else {
                                '*'
                            }
                        } else if unicode {
                            '▓'
                        } else {
                            '#'
                        };
                        let c = if tree.blossom && (dx + dy) % 3 == 0 {
                            BLOSSOM
                        } else {
                            tree.tint
                                .scale(0.55 + (1.0 - d as f32 / (r * r) as f32) * 0.30)
                        };
                        canvas.set(x + dx + sway, top + r + dy, ch, c);
                    }
                }
            }
        }
        for x in 0..self.w as isize {
            if (x * 5 + (t * 1.5) as isize) % 7 == 0 {
                canvas.set(x, ground, if unicode { '▀' } else { '"' }, LEAF.scale(0.45));
            }
        }
    }
}

impl Scene for LandscapeScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        let horizon = self.horizon();
        for y in 0..self.h as isize {
            let v = (y as f32 / horizon.max(1.0)).min(1.0);
            let c = if v < 0.6 {
                SKY_TOP.lerp(SKY_MID, v / 0.6)
            } else {
                SKY_MID.lerp(SKY_LOW, (v - 0.6) / 0.4 * 0.45)
            };
            for x in 0..self.w as isize {
                if (x + y * 4) % 10 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.5));
                } else if (x + y) % 4 == 0 {
                    canvas.set(x, y, if unicode { '░' } else { '.' }, c.scale(0.40));
                }
            }
        }
        // high birds
        for k in 0..4 {
            let x =
                ((t * (1.5 + k as f32) + k as f32 * 23.0) as isize) % (self.w as isize + 12) - 6;
            let y = (2 + k % 3) as isize;
            canvas.set(x, y, 'v', MIST.scale(0.5));
            canvas.set(
                x + 1,
                y + 1,
                if unicode { '·' } else { '.' },
                MIST.scale(0.3),
            );
        }
        self.mountains(canvas, unicode, t);
        self.lake(canvas, unicode, t);
        self.jungle(canvas, unicode, t);
        for p in &self.petals {
            let x = (p.x + (t * p.speed + p.phase).sin() * 2.0) as isize;
            let y = (p.y + t * p.speed * 0.3) as isize % self.h as isize;
            canvas.set(x, y, if unicode { '❀' } else { '*' }, BLOSSOM.scale(0.5));
        }
        let h = self.h as isize;
        let title = if unicode {
            "╡ SHAN SHUI // FIVE COLOURS ╞"
        } else {
            " SHAN SHUI // FIVE COLOURS "
        };
        canvas.put_str(2, 0, title, SUN.scale(0.9));
        canvas.put_str(
            2,
            h - 1,
            "MOUNTAINS · LAKE · BOAT · GEESE · JUNGLE",
            MIST.scale(0.55),
        );
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn has_trees() {
        let mut s = LandscapeScene::new(7);
        s.resize(120, 36);
        assert!(s.trees.len() >= 8);
    }
}
