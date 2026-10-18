//! 蒲公英 — a wind-blown dandelion field.
//!
//! Several dandelions bloom along the left edge while their seeds peel off
//! and drift to the right.  `--seed-count` and `--seed-speed` control the
//! number and drift speed.  Unicode mode uses fluffy `✿ ✦` glyphs; ASCII
//! mode falls back to `* o .`.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const WHITE: Rgb = Rgb::hex(0xfff7d6);
const CREAM: Rgb = Rgb::hex(0xfef3c7);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const GREEN: Rgb = Rgb::hex(0x22c55e);
const GREEN_DARK: Rgb = Rgb::hex(0x166534);
const STEM: Rgb = Rgb::hex(0x4d7c0f);
const SKY_TOP: Rgb = Rgb::hex(0x071a2e);
const SKY_BOTTOM: Rgb = Rgb::hex(0x164e63);
const HAZE: Rgb = Rgb::hex(0xfde68a);

#[derive(Clone)]
struct Flower {
    x: f32,
    head_y: f32,
    size: f32,
    phase: f32,
    lean: f32,
}

#[derive(Clone)]
struct Seed {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    phase: f32,
    size: f32,
    tint: f32,
}

pub struct DandelionScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    ground: f32,
    flowers: Vec<Flower>,
    seeds: Vec<Seed>,
}

impl DandelionScene {
    pub fn new(seed: u64) -> Self {
        DandelionScene {
            seed,
            rng: Rng::new(seed ^ 0x4441_4e44_454c),
            w: 0,
            h: 0,
            ground: 0.0,
            flowers: Vec::new(),
            seeds: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.ground = (self.h as f32 - 4.0).max(6.0);
        self.flowers.clear();
        // Three clumps; each clump has two stems growing from one root so the
        // stems converge at the base and fan out to different heights.
        for c in 0..3 {
            let base = self.w as f32 * (0.06 + c as f32 * 0.105)
                + self.rng.range(-0.01, 0.01) * self.w as f32;
            for st in 0..2 {
                let want = (self.h as f32
                    * (0.16 + c as f32 * 0.095 + st as f32 * 0.16 + self.rng.range(-0.035, 0.035)))
                .max(1.0);
                let max_span = (self.ground - 3.0).max(1.0);
                let head_y = (self.ground - want.min(max_span)).max(1.0);
                let lean = if st == 0 {
                    self.rng.range(-4.6, -2.2)
                } else {
                    self.rng.range(2.2, 4.6)
                };
                self.flowers.push(Flower {
                    x: base,
                    head_y,
                    size: 4.4 + (c * 2 + st) as f32 * 0.22,
                    phase: self.rng.range(0.0, TAU),
                    lean: lean + self.rng.range(-0.35, 0.35),
                });
            }
        }
        self.seeds.clear();
        for _ in 0..90 {
            self.spawn_seed(None);
        }
    }

    fn spawn_seed(&mut self, only: Option<usize>) {
        if self.flowers.is_empty() {
            return;
        }
        let idx = only
            .unwrap_or_else(|| self.rng.below(self.flowers.len()))
            .min(self.flowers.len() - 1);
        let f = &self.flowers[idx];
        let a = self.rng.range(-0.9, 0.9);
        let r = self.rng.range(0.3, f.size + 0.6);
        let x = f.x + a.cos() * r;
        let y = f.head_y + a.sin() * r * 0.75;
        let vx = self.rng.range(3.2, 8.5);
        let vy = self.rng.range(-1.0, 1.2);
        self.seeds.push(Seed {
            x,
            y,
            vx,
            vy,
            phase: self.rng.range(0.0, TAU),
            size: self.rng.range(0.9, 1.7),
            tint: self.rng.f32(),
        });
    }

    fn draw_flower(&self, canvas: &mut Canvas, f: &Flower, unicode: bool, t: f32) {
        let sway = (t * 0.8 + f.phase).sin() * 1.4 + f.lean;
        let base_x = f.x.round() as isize;
        let ground = self.ground.round() as isize;
        let head_x = (f.x + sway).round() as isize;
        let head_y = f.head_y.round() as isize;
        let stem = if unicode { '║' } else { '|' };
        // shared root tuft
        for dx in -3..=3 {
            canvas.set(
                base_x + dx,
                ground,
                if unicode { '▀' } else { '"' },
                GREEN_DARK.scale(0.75),
            );
        }
        let steps = (ground - head_y).max(1);
        for k in 0..steps {
            let p = k as f32 / steps as f32;
            let x = base_x + ((head_x - base_x) as f32 * p) as isize;
            canvas.set(x, ground - k, stem, STEM.scale(0.9));
        }
        // long, narrow leaves fanning from the stem
        let leaf_y = ground - steps / 2;
        for (dir, ly) in [(-1.0, leaf_y), (1.0, leaf_y - steps / 4)] {
            for k in 0..6isize {
                let dx = (dir * (2.0 + k as f32)) as isize;
                let dy = -(k / 2);
                canvas.set(
                    base_x + dx,
                    ly + dy,
                    if unicode { '▰' } else { '#' },
                    GREEN.scale(0.60 - k as f32 * 0.035),
                );
                canvas.set(
                    base_x + dx,
                    ly + dy + 1,
                    if unicode { '█' } else { '=' },
                    GREEN_DARK.scale(0.85),
                );
            }
        }
        // a fluffy filled seed head
        let rx = f.size;
        let ry = f.size * 0.64;
        for dy in -(ry.ceil() as isize)..=(ry.ceil() as isize) {
            for dx in -(rx.ceil() as isize)..=(rx.ceil() as isize) {
                let d = (dx as f32 / rx).powi(2) + (dy as f32 / ry).powi(2);
                if d > 1.0 {
                    continue;
                }
                let ch = if unicode {
                    if d > 0.82 {
                        '·'
                    } else {
                        match (dx * 2 + dy * 3).rem_euclid(7) {
                            0 | 3 => '✿',
                            1 => '✦',
                            2 => '✧',
                            4 => '❀',
                            5 => '·',
                            _ => '✶',
                        }
                    }
                } else if d > 0.82 {
                    '.'
                } else if (dx + dy).rem_euclid(3) == 0 {
                    '*'
                } else {
                    'o'
                };
                let col = if d > 0.72 {
                    WHITE.scale(0.82)
                } else if d > 0.35 {
                    CREAM.scale(1.0)
                } else {
                    WHITE
                };
                canvas.set(head_x + dx, head_y + dy, ch, col);
                if (dx * 3 + dy * 5).rem_euclid(7) == 0 {
                    canvas.set(
                        head_x + dx,
                        head_y + dy,
                        if unicode { '✦' } else { '*' },
                        WHITE,
                    );
                }
            }
        }
        // wispy fibers poking out of the fluff
        let rays = if unicode { 20 } else { 14 };
        for k in 0..rays {
            let a = k as f32 / rays as f32 * TAU + f.phase * 0.2;
            let len = f.size + 1.4 + (k % 3) as f32 * 0.5;
            let tx = head_x + (a.cos() * len).round() as isize;
            let ty = head_y + (a.sin() * len * 0.62).round() as isize;
            let r0 = f.size * 0.92;
            let sx = head_x + (a.cos() * r0).round() as isize;
            let sy = head_y + (a.sin() * r0 * 0.62).round() as isize;
            canvas.line(
                sx,
                sy,
                tx,
                ty,
                if unicode { '·' } else { '.' },
                WHITE.scale(0.48),
            );
            canvas.set(
                tx,
                ty,
                if unicode {
                    if k % 3 == 0 {
                        '✦'
                    } else {
                        '·'
                    }
                } else if k % 3 == 0 {
                    '*'
                } else {
                    '.'
                },
                CREAM.scale(0.95),
            );
            canvas.add_light(tx, ty, WHITE, 0.25, '.');
        }
        // green calyx under the head
        canvas.set(
            head_x,
            head_y + ry.ceil() as isize,
            if unicode { '▄' } else { 'v' },
            GREEN_DARK,
        );
        canvas.set(head_x, head_y, if unicode { '❀' } else { '*' }, GOLD);
        canvas.set(
            head_x - 2,
            head_y - 1,
            if unicode { '✦' } else { '*' },
            WHITE,
        );
        canvas.set(
            head_x + 2,
            head_y + 1,
            if unicode { '✧' } else { '*' },
            WHITE,
        );
        canvas.set(head_x - 1, head_y, if unicode { '·' } else { '.' }, WHITE);
        canvas.set(head_x + 1, head_y, if unicode { '·' } else { '.' }, WHITE);
        canvas.add_light(head_x, head_y, GOLD, 0.4, '.');
    }

    fn draw_seed(&self, canvas: &mut Canvas, s: &Seed, unicode: bool, t: f32) {
        let x = s.x.round() as isize;
        let y = (s.y + (t * 1.3 + s.phase).sin() * 0.8).round() as isize;
        let tw = 0.7 + 0.3 * (t * 2.0 + s.phase).sin().abs();
        let col = WHITE.lerp(CREAM, s.tint).scale(tw);
        let glyph = if unicode {
            if s.size > 1.3 {
                '✦'
            } else {
                '✧'
            }
        } else if s.size > 1.3 {
            '*'
        } else {
            '+'
        };
        canvas.set(x, y, glyph, col);
        canvas.set(x - 1, y, if unicode { '·' } else { '-' }, col.scale(0.55));
        canvas.set(x - 2, y, if unicode { '·' } else { '.' }, col.scale(0.30));
        canvas.add_light(x, y, WHITE, 0.22 * tw, '.');
    }

    fn draw_background(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let h = self.h as isize;
        let ground = self.ground as isize;
        for y in 0..h {
            let v = (y as f32 / ground.max(1) as f32).min(1.0);
            let col = SKY_TOP
                .lerp(SKY_BOTTOM, v)
                .lerp(HAZE, (v * v * 0.10).min(0.10));
            for x in 0..self.w as isize {
                if (x + y * 5) % 7 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col.scale(1.5));
                }
            }
        }
        // low sun / warm haze
        let sx = (self.w as f32 * 0.78) as isize;
        let sy = (self.h as f32 * 0.20) as isize;
        for dy in -3..=3 {
            for dx in -5..=5 {
                if dx * dx + dy * dy * 3 <= 25 {
                    canvas.add_light(
                        sx + dx,
                        sy + dy,
                        GOLD,
                        0.08,
                        if unicode { '░' } else { ':' },
                    );
                }
            }
        }
        canvas.set(sx, sy, if unicode { '◉' } else { 'O' }, GOLD.scale(0.8));
        // distant hills
        for i in 0..3 {
            let hx = (i as f32 * 44.0) as isize - 8;
            for dy in 0..4 {
                for dx in -10isize..=10 {
                    let edge = (dx.abs() as f32 / 10.0 * 4.0) as isize;
                    if dy < 4 - edge {
                        canvas.set(
                            hx + dx,
                            ground - 2 - dy,
                            if unicode { '▒' } else { ':' },
                            GREEN_DARK.scale(0.32),
                        );
                    }
                }
            }
        }
        // grass bed
        for y in ground..h {
            for x in 0..self.w as isize {
                let ch = if y == ground {
                    if unicode {
                        '▀'
                    } else {
                        '"'
                    }
                } else if (x + y) % 3 == 0 {
                    if unicode {
                        '▒'
                    } else {
                        ':'
                    }
                } else if unicode {
                    '█'
                } else {
                    '#'
                };
                canvas.set(
                    x,
                    y,
                    ch,
                    if y == ground {
                        GREEN.scale(0.45)
                    } else {
                        GREEN_DARK.scale(0.55)
                    },
                );
            }
        }
        // wind streaks
        for i in 0..5 {
            let y = 4 + i * (self.h as isize - 10) / 5;
            let x0 =
                ((t * (8.0 + i as f32) + i as f32 * 17.0) as isize) % (self.w as isize + 20) - 10;
            for k in 0..5 {
                canvas.set_if_blank(
                    x0 + k,
                    y,
                    if unicode { '─' } else { '-' },
                    WHITE.scale(0.10),
                );
            }
        }
    }
}

impl Scene for DandelionScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let target = f.tune.seed_count.clamp(1, 400) as usize;
        let speed = f.tune.seed_speed.clamp(0.05, 4.0);
        if self.seeds.len() > target {
            self.seeds.truncate(target);
        }
        let mut made = 0;
        while self.seeds.len() < target && made < 3 {
            self.spawn_seed(None);
            made += 1;
        }
        for s in &mut self.seeds {
            s.x += s.vx * speed * dt;
            s.y += (s.vy + (f.t as f32 * 0.7 + s.phase).sin() * 0.7) * speed * dt;
            s.phase += dt * 2.0;
        }
        let w = self.w as f32;
        self.seeds
            .retain(|s| s.x < w + 6.0 && s.y < self.h as f32 + 3.0);
        // respawn from the heads on the left
        let mut made = 0;
        while self.seeds.len() < target && made < 3 {
            self.spawn_seed(None);
            made += 1;
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_background(canvas, unicode, t);
        for s in &self.seeds {
            self.draw_seed(canvas, s, unicode, t);
        }
        for flower in &self.flowers {
            self.draw_flower(canvas, flower, unicode, t);
        }
        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ DANDELION // WIND FIELD ╞"
        } else {
            " DANDELION // WIND FIELD "
        };
        canvas.put_str(2, 0, title, CREAM.scale(0.9));
        let status = format!(
            "SEEDS {:03}   WIND {:.1}x",
            self.seeds.len(),
            f.tune.seed_speed
        );
        let sx = (w - status.len() as isize - 2).max(2);
        canvas.put_str(sx, 0, &status, GOLD.scale(0.85));
        canvas.put_str(
            2,
            h - 1,
            "SEEDS DRIFT EAST   //   --seed-count / --seed-speed",
            WHITE.scale(0.55),
        );
        let _ = f.theme;
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flowers_and_seeds_exist() {
        let mut s = DandelionScene::new(7);
        s.resize(120, 36);
        assert_eq!(s.flowers.len(), 6);
        assert!(s.seeds.len() > 20);
    }
}
