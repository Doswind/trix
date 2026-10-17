//! 超级马里奥 — an auto-running mushroom level.
//!
//! Mario runs across a scrolling 1-1 style level, jumps under question blocks,
//! bumps mushrooms out of them, and collects coins.  Unicode mode draws solid
//! block sprites; `--ascii` uses a classic ASCII platformer palette.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const RED: Rgb = Rgb::hex(0xef4444);
const RED_DARK: Rgb = Rgb::hex(0x991b1b);
const SKIN: Rgb = Rgb::hex(0xfbbf24);
const BLUE: Rgb = Rgb::hex(0x2563eb);
const BROWN: Rgb = Rgb::hex(0x92400e);
const BROWN_LIGHT: Rgb = Rgb::hex(0xf59e0b);
const GOLD: Rgb = Rgb::hex(0xfacc15);
const GREEN: Rgb = Rgb::hex(0x22c55e);
const GREEN_DARK: Rgb = Rgb::hex(0x166534);
const WHITE: Rgb = Rgb::hex(0xf8fafc);
const SKY: Rgb = Rgb::hex(0x0c4a6e);
const SKY_LIGHT: Rgb = Rgb::hex(0x38bdf8);
const INK: Rgb = Rgb::hex(0x020617);

#[derive(Clone)]
struct Block {
    x: f32,
    y: f32,
    question: bool,
    used: bool,
    bump: f32,
}

#[derive(Clone)]
struct Shroom {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

#[derive(Clone)]
struct Coin {
    x: f32,
    y: f32,
    phase: f32,
}

#[derive(Clone)]
struct Goomba {
    x: f32,
    phase: f32,
}

pub struct MarioScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    ground: f32,
    blocks: Vec<Block>,
    shrooms: Vec<Shroom>,
    coins: Vec<Coin>,
    goombas: Vec<Goomba>,
    mario_y: f32,
    vy: f32,
    on_ground: bool,
    jump_lock: f32,
    scroll: f32,
    time: f32,
    score: u32,
    coin_count: u32,
}

impl MarioScene {
    pub fn new(seed: u64) -> Self {
        MarioScene {
            seed,
            rng: Rng::new(seed ^ 0x004d_4152_494f),
            w: 0,
            h: 0,
            ground: 0.0,
            blocks: Vec::new(),
            shrooms: Vec::new(),
            coins: Vec::new(),
            goombas: Vec::new(),
            mario_y: 0.0,
            vy: 0.0,
            on_ground: true,
            jump_lock: 0.0,
            scroll: 0.0,
            time: 0.0,
            score: 0,
            coin_count: 0,
        }
    }

    fn mario_x(&self) -> f32 {
        (self.w as f32 * 0.28).clamp(4.0, (self.w as f32 - 12.0).max(4.0))
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.ground = (self.h as f32 - 5.0).max(8.0);
        self.mario_y = self.ground;
        self.vy = 0.0;
        self.on_ground = true;
        self.scroll = 0.0;
        self.time = 0.0;
        self.score = 0;
        self.coin_count = 0;
        self.blocks.clear();
        self.shrooms.clear();
        self.coins.clear();
        self.goombas.clear();

        let start = self.w as f32 * 0.55;
        for i in 0..7 {
            let question = i % 3 != 1;
            self.blocks.push(Block {
                x: start + i as f32 * 10.0,
                y: self.ground - 10.0,
                question,
                used: false,
                bump: 0.0,
            });
            if i == 2 || i == 5 {
                self.coins.push(Coin {
                    x: start + i as f32 * 10.0 + 1.0,
                    y: self.ground - 14.0,
                    phase: self.rng.range(0.0, TAU),
                });
            }
        }
        for i in 0..3 {
            self.goombas.push(Goomba {
                x: self.w as f32 * 0.65 + i as f32 * 18.0,
                phase: self.rng.range(0.0, TAU),
            });
        }
    }

    fn spawn_shroom(&mut self, x: f32, y: f32) {
        self.shrooms.push(Shroom {
            x,
            y,
            vx: 3.2,
            vy: -7.5,
            life: 7.0,
        });
        self.coins.push(Coin {
            x: x + 1.0,
            y: y - 2.0,
            phase: 0.0,
        });
        self.coin_count = self.coin_count.saturating_add(1);
        self.score = self.score.saturating_add(200);
    }

    fn update_blocks(&mut self, dt: f32) {
        let speed = 9.5;
        let mx = self.mario_x();
        for b in &mut self.blocks {
            b.x -= speed * dt;
            b.bump = (b.bump - dt * 3.2).max(0.0);
        }
        // recycle any block that leaves the left edge
        for i in 0..self.blocks.len() {
            if self.blocks[i].x < -4.0 {
                let right = self
                    .blocks
                    .iter()
                    .map(|b| b.x)
                    .fold(self.w as f32, f32::max);
                let question = self.rng.chance(0.66);
                self.blocks[i] = Block {
                    x: right + self.rng.range(9.0, 14.0),
                    y: self.ground - 10.0,
                    question,
                    used: false,
                    bump: 0.0,
                };
            }
        }

        // Auto-jump when a fresh block is just ahead.
        let ahead = self
            .blocks
            .iter()
            .filter(|b| !b.used && b.x > mx - 1.0 && b.x < mx + 7.5)
            .min_by(|a, b| {
                (a.x - mx)
                    .abs()
                    .partial_cmp(&(b.x - mx).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        self.jump_lock = (self.jump_lock - dt).max(0.0);
        if self.on_ground && ahead.is_some() && self.jump_lock <= 0.0 {
            self.vy = -16.5;
            self.on_ground = false;
            self.jump_lock = 0.55;
        }

        if !self.on_ground {
            self.vy += 34.0 * dt;
            self.mario_y += self.vy * dt;
            if self.mario_y >= self.ground {
                self.mario_y = self.ground;
                self.vy = 0.0;
                self.on_ground = true;
            }
        }

        // The auto-jump lands under the next block; bump it on contact.
        if !self.on_ground {
            for b in &mut self.blocks {
                if b.used || (b.x + 1.0 - mx).abs() >= 3.6 {
                    continue;
                }
                b.bump = 0.38;
                b.used = true;
                let (bx, by) = (b.x, b.y);
                if b.question {
                    self.spawn_shroom(bx, by - 1.0);
                } else {
                    self.score = self.score.saturating_add(50);
                }
                self.vy = 1.0;
                break;
            }
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for s in &mut self.shrooms {
            s.x += s.vx * dt;
            s.vy += 18.0 * dt;
            s.y += s.vy * dt;
            if s.y > self.ground {
                s.y = self.ground;
                s.vy = -s.vy * 0.55;
            }
            s.life -= dt;
        }
        self.shrooms
            .retain(|s| s.life > 0.0 && s.x < self.w as f32 + 8.0 && s.x > -8.0);
        for c in &mut self.coins {
            c.phase += dt * 4.0;
        }
        for g in &mut self.goombas {
            g.x -= 2.8 * dt;
            g.phase += dt * 7.0;
            if g.x < -6.0 {
                g.x = self.w as f32 + self.rng.range(2.0, 18.0);
            }
        }
    }

    fn draw_sky(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let h = self.h as isize;
        let ground = self.ground as isize;
        for y in 0..ground {
            let v = y as f32 / ground.max(1) as f32;
            let col = INK.lerp(SKY, 0.28 + v * 0.40);
            for x in 0..self.w as isize {
                if (x + y * 3) % 5 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col.scale(1.7));
                }
            }
        }
        // clouds, parallax with the scroll
        for i in 0..4 {
            let cx = ((i as f32 * 31.0 - self.scroll * 0.18) % (self.w as f32 + 24.0)
                + self.w as f32
                + 24.0)
                % (self.w as f32 + 24.0)
                - 12.0;
            let cy = (3 + (i * 3) % 6) as isize;
            let c = if unicode { '≈' } else { '~' };
            let col = SKY_LIGHT.scale(0.35 + 0.08 * (t + i as f32).sin());
            for k in 0..7 {
                canvas.set(cx as isize + k, cy, c, col);
                if k > 1 && k < 5 {
                    canvas.set(cx as isize + k, cy + 1, c, col.scale(0.7));
                }
            }
        }
        // hills
        for i in 0..3 {
            let hx = ((i as f32 * 43.0 - self.scroll * 0.3) % (self.w as f32 + 30.0)
                + self.w as f32
                + 30.0)
                % (self.w as f32 + 30.0)
                - 15.0;
            let r = 6 + (i % 2) * 2;
            for dy in 0..r {
                let half = ((r - dy) as f32 * 0.9) as isize;
                for dx in -half..=half {
                    canvas.set(
                        hx as isize + dx,
                        ground - 1 - dy as isize,
                        if unicode { '▓' } else { '#' },
                        GREEN_DARK.scale(0.55 + dy as f32 * 0.03),
                    );
                }
            }
        }
        // ground bed
        for y in ground..(h - 1) {
            for x in 0..self.w as isize {
                let ch = if y == ground {
                    if unicode {
                        '▀'
                    } else {
                        '#'
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
                let col = if y == ground {
                    GREEN.scale(0.56)
                } else if (x / 3 + y) % 2 == 0 {
                    BROWN.scale(0.52)
                } else {
                    BROWN.scale(0.62)
                };
                canvas.set(x, y, ch, col);
            }
        }
    }

    fn draw_block(&self, canvas: &mut Canvas, b: &Block, unicode: bool) {
        let off = if b.bump > 0.0 {
            (b.bump * 8.0).round() as isize
        } else {
            0
        };
        let x = b.x.round() as isize;
        let y = b.y.round() as isize - off;
        let color = if b.used {
            BROWN.scale(0.55)
        } else if b.question {
            GOLD
        } else {
            BROWN_LIGHT.scale(0.85)
        };
        for dy in 0..3 {
            for dx in 0..3 {
                let edge = dx == 0 || dx == 2 || dy == 0 || dy == 2;
                let ch = if unicode {
                    if b.question && !b.used && dx == 1 && dy == 1 {
                        '?'
                    } else if b.used && dx == 1 && dy == 1 {
                        '·'
                    } else if edge {
                        '█'
                    } else {
                        '▓'
                    }
                } else if b.question && !b.used && dx == 1 && dy == 1 {
                    '?'
                } else if b.used && dx == 1 && dy == 1 {
                    '.'
                } else {
                    '#'
                };
                let c = if b.question && !b.used && dx == 1 && dy == 1 {
                    INK
                } else if edge {
                    color
                } else {
                    color.scale(0.58)
                };
                canvas.set(x + dx, y + dy, ch, c);
            }
        }
        if b.question && !b.used {
            canvas.add_light(x + 1, y + 1, GOLD, 0.35, '.');
        }
    }

    fn draw_mushroom(&self, canvas: &mut Canvas, s: &Shroom, unicode: bool) {
        let x = s.x.round() as isize;
        let y = s.y.round() as isize;
        // cap
        for dx in -1..=1 {
            canvas.set(x + dx, y - 1, if unicode { '▄' } else { '*' }, RED);
        }
        for dx in -2..=2 {
            canvas.set(x + dx, y - 2, if unicode { '█' } else { '*' }, RED);
        }
        canvas.set(x - 2, y - 2, if unicode { '▟' } else { '*' }, RED_DARK);
        canvas.set(x + 2, y - 2, if unicode { '▙' } else { '*' }, RED_DARK);
        // spots
        canvas.set(x - 1, y - 2, if unicode { '·' } else { 'o' }, WHITE);
        canvas.set(x + 1, y - 2, if unicode { '·' } else { 'o' }, WHITE);
        // stem
        for dx in -1..=1 {
            canvas.set(
                x + dx,
                y - 1,
                if unicode { '█' } else { 'o' },
                WHITE.scale(0.85),
            );
            canvas.set(x + dx, y, if unicode { '█' } else { 'o' }, WHITE.scale(0.7));
        }
        canvas.add_light(x, y - 2, RED, 0.35, '.');
    }

    fn draw_mario(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        const SPRITE: &[&str] = &[
            "...RRR..", "..RRRRR.", "..RRRRR.", "..SSKS..", ".SKKSS..", ".RRBRR..", "RBBBBBR.",
            "BB...BB.", "KK...KK.",
        ];
        let running = self.on_ground;
        let leg = if running { (t * 14.0).sin() } else { 0.0 };
        let x = self.mario_x().round() as isize;
        let feet = self.mario_y.round() as isize;
        let y0 = feet - SPRITE.len() as isize + 1;
        for (r, row) in SPRITE.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                let col = match ch {
                    'R' => RED,
                    'S' => SKIN,
                    'B' => BLUE,
                    'K' => BROWN,
                    _ => continue,
                };
                let mut dx = c as isize;
                let mut dy = r as isize;
                if running && r >= 7 {
                    if leg > 0.2 {
                        dx += if c < 3 { -1 } else { 1 };
                    } else if leg < -0.2 {
                        dx += if c > 4 { 1 } else { -1 };
                    }
                }
                if !running {
                    dy -= 1;
                    if r >= 7 {
                        dx += if c < 4 { -1 } else { 1 };
                    }
                }
                let glyph = if unicode {
                    '█'
                } else if ch == 'K' {
                    '@'
                } else {
                    '#'
                };
                canvas.set(x + dx, y0 + dy, glyph, col);
                if unicode {
                    canvas.add_light(x + dx, y0 + dy, col.scale(0.45), 0.10, '.');
                }
            }
        }
        // eye
        canvas.set(x + 4, y0 + 3, if unicode { '·' } else { 'o' }, INK);
        canvas.add_light(x + 3, y0, RED, 0.3, '.');
        // ground shadow
        for dx in 0..6 {
            canvas.add_light(x + dx, feet, INK, 0.6, '.');
        }
    }

    fn draw_goomba(&self, canvas: &mut Canvas, g: &Goomba, unicode: bool) {
        let x = g.x.round() as isize;
        let y = (self.ground - 2.0) as isize;
        for dx in 0..4 {
            canvas.set(x + dx, y, if unicode { '█' } else { 'M' }, BROWN);
            canvas.set(
                x + dx,
                y + 1,
                if unicode { '▓' } else { 'M' },
                BROWN.scale(0.8),
            );
        }
        canvas.set(x + 1, y, if unicode { '·' } else { 'o' }, WHITE);
        canvas.set(x + 2, y, if unicode { '·' } else { 'o' }, WHITE);
        canvas.set(x, y + 2, if unicode { '▄' } else { '=' }, INK);
        canvas.set(x + 3, y + 2, if unicode { '▄' } else { '=' }, INK);
        let step = if g.phase.sin() > 0.0 { 1 } else { 0 };
        canvas.set(
            x + step,
            y + 2,
            if unicode { '▄' } else { '=' },
            BROWN_LIGHT.scale(0.5),
        );
    }

    fn draw_coin(&self, canvas: &mut Canvas, c: &Coin, unicode: bool) {
        let x = c.x.round() as isize;
        let y = (c.y + (c.phase.sin() * 0.7)).round() as isize;
        canvas.set(x, y, if unicode { '●' } else { 'o' }, GOLD);
        canvas.set(x, y - 1, if unicode { '·' } else { '.' }, GOLD.scale(0.5));
        canvas.add_light(x, y, GOLD, 0.3, '.');
    }
}

impl Scene for MarioScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        self.time += dt;
        self.scroll += 9.5 * dt;
        self.update_blocks(dt);
        self.update_particles(dt);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_sky(canvas, unicode, t);

        for b in &self.blocks {
            self.draw_block(canvas, b, unicode);
        }
        for c in &self.coins {
            self.draw_coin(canvas, c, unicode);
        }
        for g in &self.goombas {
            self.draw_goomba(canvas, g, unicode);
        }
        for s in &self.shrooms {
            self.draw_mushroom(canvas, s, unicode);
        }
        self.draw_mario(canvas, unicode, t);

        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ SUPER MARIO // MUSHROOM RUN ╞"
        } else {
            " SUPER MARIO // MUSHROOM RUN "
        };
        canvas.put_str(2, 0, title, GOLD.scale(0.9));
        let status = format!("SCORE {:06}  COINS {:02}", self.score, self.coin_count);
        let sx = (w - status.len() as isize - 2).max(2);
        canvas.put_str(sx, 0, &status, WHITE.scale(0.85));
        let bottom = "WORLD 1-1   //   PRESS JUMP";
        canvas.put_str(2, h - 1, bottom, theme.c(2).scale(0.85));
        let badge = if unicode { "UNICODE" } else { "ASCII" };
        let bx = (w - badge.len() as isize - 2).max(2);
        canvas.put_str(bx, h - 1, badge, GOLD.scale(0.7));
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn has_mushroom_blocks() {
        let mut s = MarioScene::new(7);
        s.resize(120, 36);
        assert!(s.blocks.iter().any(|b| b.question));
        assert!(s.ground > 10.0);
    }
}
