//! 超级马里奥 — an auto-running mushroom level.
//!
//! The whole screen is a tiny SMB-style platformer: a solid sky, tiled ground,
//! question blocks floating two tiles above the path, a green pipe, goombas,
//! spinning coins and an SMB HUD.  Mario runs on his own: he jumps before a
//! block or a pipe, bumps the block, pops a coin or a mushroom and hops over
//! enemies.
//!
//! Two views of the same level:
//!
//! * ASCII (default) — hand drawn char sprites, pure ASCII glyphs.
//! * `--unicode` — the same sprites drawn with half blocks, one cell per pixel
//!   column and two pixels per row, so Mario is a real pixel sprite.

use std::f32::consts::TAU;

use super::mario_art as art;
use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

/// Downward acceleration, in rows per second squared.
const GRAVITY: f32 = 46.0;
/// Initial jump speed, in rows per second.  Peaks about 9 rows up, just
/// enough to clear a pipe and bump the block row.
const JUMP_V: f32 = -28.8;
/// World scroll speed, in columns per second.
const RUN_SPEED: f32 = 10.0;
const MARIO_H: f32 = 7.0;
const BLOCK_W: isize = 12;
const BLOCK_H: isize = 5;
const PIPE_W: isize = 12;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Question,
    Brick,
    Used,
}

impl BlockKind {
    fn face(self) -> Rgb {
        match self {
            BlockKind::Question => art::GOLD,
            BlockKind::Brick => art::BRICK,
            BlockKind::Used => art::BRICK_DK,
        }
    }
    fn edge(self) -> Rgb {
        match self {
            BlockKind::Question => art::GOLD_DK,
            BlockKind::Brick => art::BRICK_DK,
            BlockKind::Used => art::BROWN,
        }
    }
}

#[derive(Clone)]
struct Block {
    x: f32,
    y: f32,
    kind: BlockKind,
    bump: f32,
}

#[derive(Clone)]
struct Pipe {
    x: f32,
    height: isize,
}

#[derive(Clone)]
struct Goomba {
    x: f32,
    phase: f32,
    squash: f32,
    /// Flattened by a stomp: plays the squash for a moment, then goes away.
    dead: bool,
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
struct PopCoin {
    x: f32,
    y: f32,
    vy: f32,
    life: f32,
}

#[derive(Clone)]
struct PopText {
    x: f32,
    y: f32,
    text: &'static str,
    life: f32,
}

#[derive(Clone)]
struct Dust {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

pub struct MarioScene {
    rng: Rng,
    w: usize,
    h: usize,
    ground: f32,
    mario_x: f32,
    y: f32,
    vy: f32,
    on_ground: bool,
    jump_lock: f32,
    scroll: f32,
    time: f32,
    score: u32,
    coins: u32,
    time_left: f32,
    cursor: f32,
    blocks: Vec<Block>,
    pipes: Vec<Pipe>,
    goombas: Vec<Goomba>,
    shrooms: Vec<Shroom>,
    coins_free: Vec<Coin>,
    coin_pops: Vec<PopCoin>,
    texts: Vec<PopText>,
    dust: Vec<Dust>,
}

impl MarioScene {
    pub fn new(seed: u64) -> Self {
        MarioScene {
            rng: Rng::new(seed ^ 0x004d_4152_494f),
            w: 0,
            h: 0,
            ground: 0.0,
            mario_x: 0.0,
            y: 0.0,
            vy: 0.0,
            on_ground: true,
            jump_lock: 0.0,
            scroll: 0.0,
            time: 0.0,
            score: 0,
            coins: 0,
            time_left: 380.0,
            cursor: 0.0,
            blocks: Vec::new(),
            pipes: Vec::new(),
            goombas: Vec::new(),
            shrooms: Vec::new(),
            coins_free: Vec::new(),
            coin_pops: Vec::new(),
            texts: Vec::new(),
            dust: Vec::new(),
        }
    }

    fn mario_head(&self) -> f32 {
        self.y - (MARIO_H - 1.0)
    }

    fn block_top(&self) -> f32 {
        // Two tiles above the path, but always inside the playable area.
        let lift = (self.ground * 0.42).clamp(9.0, 13.0);
        (self.ground - lift - (BLOCK_H - 1) as f32).max(2.0)
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let wf = self.w as f32;
        let hf = self.h as f32;
        self.ground = (hf - 5.0).max(5.0);
        self.mario_x = (wf * 0.34).clamp(3.0, (wf - 14.0).max(3.0));
        self.y = self.ground - 1.0;
        self.vy = 0.0;
        self.on_ground = true;
        self.jump_lock = 0.0;
        self.scroll = 0.0;
        self.time = 0.0;
        self.score = 0;
        self.coins = 0;
        self.time_left = 380.0;
        self.blocks.clear();
        self.pipes.clear();
        self.goombas.clear();
        self.shrooms.clear();
        self.coins_free.clear();
        self.coin_pops.clear();
        self.texts.clear();
        self.dust.clear();

        // Seed the level with a couple of screens worth of scenery.
        self.cursor = wf * 0.55;
        while self.cursor < wf + 34.0 {
            self.spawn_next();
        }
        for i in 0..2 {
            self.goombas.push(Goomba {
                x: wf + 6.0 + i as f32 * 24.0,
                phase: self.rng.range(0.0, TAU),
                squash: 0.0,
                dead: false,
            });
        }
    }

    fn spawn_next(&mut self) {
        let x = self.cursor;
        if self.rng.below(10) < 3 {
            self.spawn_pipe(x);
            self.cursor = x + PIPE_W as f32 + self.rng.range(10.0, 18.0);
        } else {
            let used = self.spawn_cluster(x);
            self.cursor = x + used + self.rng.range(10.0, 18.0);
        }
    }

    fn spawn_cluster(&mut self, start: f32) -> f32 {
        let top = self.block_top();
        let count = 2 + self.rng.below(2) as isize;
        for i in 0..count {
            let kind = if i == 1 || self.rng.chance(0.35) {
                BlockKind::Question
            } else {
                BlockKind::Brick
            };
            self.blocks.push(Block {
                x: start + i as f32 * (BLOCK_W as f32 + 1.0),
                y: top,
                kind,
                bump: 0.0,
            });
        }
        let width = count as f32 * (BLOCK_W as f32 + 1.0);
        if self.rng.chance(0.3) {
            // A higher brick shelf, with a visible gap under it.
            let high = (top - (BLOCK_H as f32 + 3.0)).max(2.0);
            let hx = start + self.rng.range(0.0, 1.0) * (BLOCK_W as f32 + 1.0);
            for i in 0..2 {
                self.blocks.push(Block {
                    x: hx + i as f32 * (BLOCK_W as f32 + 1.0),
                    y: high,
                    kind: BlockKind::Brick,
                    bump: 0.0,
                });
            }
        }
        // Coins float in the open gap after the cluster.
        if self.rng.chance(0.85) {
            let gap_x = start + width + 3.0;
            for i in 0..2 {
                self.coins_free.push(Coin {
                    x: gap_x + i as f32 * 8.0,
                    y: self.ground - 8.0 - i as f32 * 0.5,
                    phase: self.rng.range(0.0, TAU),
                });
            }
        }
        width
    }

    fn spawn_pipe(&mut self, x: f32) {
        let height = if self.rng.chance(0.35) { 5 } else { 7 };
        self.pipes.push(Pipe { x, height });
    }

    fn spawn_goomba(&mut self) {
        let x = self.w as f32 + self.rng.range(2.0, 14.0);
        self.goombas.push(Goomba {
            x,
            phase: self.rng.range(0.0, TAU),
            squash: 0.0,
            dead: false,
        });
    }

    fn pop_coin(&mut self, x: f32, y: f32) {
        self.coin_pops.push(PopCoin {
            x,
            y,
            vy: -22.0,
            life: 0.7,
        });
        self.coins = self.coins.saturating_add(1);
        self.score = self.score.saturating_add(200);
        self.texts.push(PopText {
            x,
            y: y - 2.0,
            text: "200",
            life: 0.8,
        });
    }

    fn spawn_shroom(&mut self, x: f32, y: f32) {
        self.shrooms.push(Shroom {
            x,
            y,
            vx: 6.0,
            vy: -12.0,
            life: 9.0,
        });
    }

    fn spawn_dust(&mut self, n: usize) {
        for i in 0..n {
            let dir = if i % 2 == 0 { -1.0 } else { 1.0 };
            self.dust.push(Dust {
                x: self.mario_x + 3.0 + i as f32 * 1.6,
                y: self.y,
                vx: dir * self.rng.range(1.5, 3.5),
                vy: -self.rng.range(2.0, 6.0),
                life: self.rng.range(0.25, 0.5),
            });
        }
    }

    fn update_blocks(&mut self, dt: f32) {
        for b in &mut self.blocks {
            b.x -= RUN_SPEED * dt;
            b.bump = (b.bump - dt * 3.0).max(0.0);
        }
        self.blocks.retain(|b| b.x + BLOCK_W as f32 > -2.0);
    }

    fn update_physics(&mut self, dt: f32) {
        self.jump_lock = (self.jump_lock - dt).max(0.0);

        // Auto-jump: launch when a block, pipe or goomba is about to slide
        // under the jump arc (apex is roughly half a second, five columns).
        if self.on_ground && self.jump_lock <= 0.0 {
            let centre = self.mario_x + 6.0;
            let mut jump = false;
            for b in &self.blocks {
                let d = b.x + 6.0 - centre;
                if (5.0..=7.0).contains(&d) && b.y < self.mario_head() + 2.0 {
                    jump = true;
                }
            }
            for p in &self.pipes {
                let d = p.x - (self.mario_x + 12.0);
                if (1.0..=4.5).contains(&d) {
                    jump = true;
                }
            }
            for g in &self.goombas {
                let d = g.x + 5.0 - centre;
                if (3.0..=5.0).contains(&d) && g.squash <= 0.0 {
                    jump = true;
                }
            }
            if jump {
                self.vy = JUMP_V;
                self.on_ground = false;
                self.jump_lock = 0.7;
            }
        }

        // Support surfaces: the ground, or the top of a pipe Mario is above.
        let mut floor = self.ground - 1.0;
        for p in &self.pipes {
            let surf = self.ground - p.height as f32 - 1.0;
            let overlap = self.mario_x + 11.0 > p.x && self.mario_x < p.x + PIPE_W as f32;
            if overlap && self.y <= surf + 1.5 {
                floor = floor.min(surf);
            }
        }

        if !self.on_ground {
            self.vy = (self.vy + GRAVITY * dt).clamp(-40.0, 40.0);
            self.y += self.vy * dt;
        }
        // Hard safety net: never let the runner leave the screen for good.
        if !self.y.is_finite() || self.y < -1.0 {
            self.y = self.ground - 1.0;
            self.vy = 0.0;
            self.on_ground = true;
        }
        if self.y >= floor {
            if !self.on_ground {
                self.spawn_dust(4);
            }
            self.y = floor;
            self.vy = 0.0;
            self.on_ground = true;
        } else if self.on_ground {
            // Walked off the end of a pipe.
            self.on_ground = false;
            self.vy = 0.0;
        }

        // Head bumps: only when rising and actually under a block.
        if self.vy < 0.0 {
            let head = self.mario_head();
            let centre = self.mario_x + 6.0;
            let mut hit: Option<(f32, f32, BlockKind)> = None;
            for b in &self.blocks {
                let bottom = b.y + (BLOCK_H - 1) as f32;
                if head <= bottom
                    && head >= b.y - 1.0
                    && (b.x + 6.0 - centre).abs() < 6.5
                    && b.kind != BlockKind::Used
                {
                    hit = Some((b.x, b.y, b.kind));
                    break;
                }
            }
            if let Some((bx, by, kind)) = hit {
                if let Some(b) = self
                    .blocks
                    .iter_mut()
                    .find(|b| (b.x - bx).abs() < 0.01 && (b.y - by).abs() < 0.01)
                {
                    b.bump = 0.35;
                    if kind == BlockKind::Question {
                        b.kind = BlockKind::Used;
                    }
                }
                self.vy = 2.0;
                if kind == BlockKind::Question {
                    if self.rng.chance(0.3) {
                        self.spawn_shroom(bx + 3.0, by - 6.0);
                    } else {
                        self.pop_coin(bx + 6.0, by - 2.0);
                    }
                }
            }
        }

        // Land on a pipe: keep the runner on top of it for a beat.
        if self.y > self.ground - 1.0 {
            self.y = self.ground - 1.0;
        }
    }

    fn update_actors(&mut self, dt: f32) {
        let wf = self.w as f32;
        let ground = self.ground;
        let mx = self.mario_x;
        let feet = self.y;

        // Goombas march left; Mario squashes them only when he is actually
        // falling onto one.  Without the descent + vertical checks a goomba
        // walking under a high-flying Mario would bounce him again and again.
        for i in 0..self.goombas.len() {
            if self.goombas[i].dead {
                self.goombas[i].squash -= dt;
                continue;
            }
            self.goombas[i].x -= 3.2 * dt;
            self.goombas[i].phase += dt * 6.0;
            let gx = self.goombas[i].x;
            let close = (gx + 5.0 - (mx + 6.0)).abs() < 6.0;
            let falling = self.vy > 0.0;
            let near = (ground - 8.0..=ground).contains(&self.y);
            if close && falling && near {
                self.goombas[i].squash = 0.35;
                self.goombas[i].dead = true;
                self.score = self.score.saturating_add(100);
                self.texts.push(PopText {
                    x: gx,
                    y: ground - 10.0,
                    text: "100",
                    life: 0.8,
                });
                self.vy = -15.0;
                self.on_ground = false;
            }
        }
        self.goombas
            .retain(|g| g.x > -14.0 && !(g.dead && g.squash <= 0.0));

        // Mushrooms: pop out, fall, bounce along the ground, get eaten.
        let mut i = 0;
        while i < self.shrooms.len() {
            let s = &mut self.shrooms[i];
            s.x += (s.vx - RUN_SPEED) * dt;
            s.vy += 18.0 * dt;
            s.y += s.vy * dt;
            if s.y > ground - 1.0 {
                s.y = ground - 1.0;
                s.vy = -s.vy * 0.55;
            }
            s.life -= dt;
            let overlap = (s.x + 4.0 - (mx + 6.0)).abs() < 9.0 && (s.y - feet).abs() < 7.0;
            let gone = s.life <= 0.0 || s.x > wf + 10.0 || s.x < -14.0;
            if overlap {
                self.shrooms.remove(i);
                self.score = self.score.saturating_add(1000);
                self.texts.push(PopText {
                    x: mx + 2.0,
                    y: feet - 8.0,
                    text: "1000",
                    life: 0.9,
                });
                continue;
            }
            if gone {
                self.shrooms.remove(i);
                continue;
            }
            i += 1;
        }

        // Free coins: scroll with the level, collect on touch.
        for c in &mut self.coins_free {
            c.x -= RUN_SPEED * dt;
        }
        let mut got = 0u32;
        self.coins_free.retain(|c| {
            let hit = (c.x + 2.0 - (mx + 6.0)).abs() < 8.0 && (c.y - (feet - 3.0)).abs() < 6.5;
            if hit {
                got += 1;
            }
            !hit
        });
        if got > 0 {
            self.coins = self.coins.saturating_add(got);
            self.score = self.score.saturating_add(200 * got);
        }

        for c in &mut self.coin_pops {
            c.vy += 42.0 * dt;
            c.y += c.vy * dt;
            c.life -= dt;
        }
        self.coin_pops.retain(|c| c.life > 0.0);

        for t in &mut self.texts {
            t.y -= 4.0 * dt;
            t.life -= dt;
        }
        self.texts.retain(|t| t.life > 0.0);

        for d in &mut self.dust {
            d.x += d.vx * dt;
            d.y += d.vy * dt;
            d.vy += 20.0 * dt;
            d.life -= dt;
        }
        self.dust.retain(|d| d.life > 0.0);
    }

    fn update_spawner(&mut self, dt: f32) {
        self.scroll += RUN_SPEED * dt;
        self.time_left = (self.time_left - dt * 2.5).max(0.0);
        if self.time_left <= 0.0 {
            self.time_left = 380.0;
        }
        let wf = self.w as f32;
        self.cursor -= RUN_SPEED * dt;
        while self.cursor < wf + 24.0 {
            self.spawn_next();
        }
        if self.goombas.len() < 2 {
            self.spawn_goomba();
        }
    }

    // ------------------------------------------------------------ drawing ---

    fn draw_sky(&self, canvas: &mut Canvas) {
        let ground = self.ground as isize;
        let w = self.w as isize;

        // The terminal background is the sky.  A sparse star field gives it
        // depth without the old wall of dots.
        for y in 1..ground {
            for x in 0..w {
                if (x * 7 + y * 13).rem_euclid(43) == 0 {
                    canvas.set(x, y, '.', art::STAR);
                }
            }
        }

        // Clouds, parallax 0.18.
        let span = w as f32 + 34.0;
        let clouds = (w / 26).clamp(4, 12);
        let sky_h = (self.ground as isize - 10).max(6);
        for i in 0..clouds {
            let base = i as f32 * (span / clouds as f32) - self.scroll * 0.18;
            let x = wrap(base, span) - 16.0;
            let y = 2 + ((i * 5) % sky_h.min(9));
            self.cloud(canvas, x.round() as isize, y);
        }

        // Hills, parallax 0.32.
        let hspan = w as f32 + 64.0;
        for i in 0..3 {
            let base = i as f32 * 71.0 - self.scroll * 0.32;
            let x = wrap(base, hspan) - 28.0;
            let h = if i % 2 == 0 { 8 } else { 6 };
            self.hill(canvas, x.round() as isize, h);
        }

        // Bushes, parallax 0.55.
        let bspan = w as f32 + 40.0;
        for i in 0..5 {
            let base = i as f32 * 37.0 - self.scroll * 0.55;
            let x = wrap(base, bspan) - 18.0;
            self.bush(canvas, x.round() as isize);
        }
    }

    fn cloud(&self, canvas: &mut Canvas, x: isize, y: isize) {
        for dx in 2..9 {
            canvas.set(x + dx, y, '~', art::CLOUD);
        }
        canvas.set(x, y + 1, '(', art::CLOUD);
        for dx in 1..10 {
            canvas.set(x + dx, y + 1, '~', art::CLOUD);
        }
        canvas.set(x + 10, y + 1, ')', art::CLOUD);
        for dx in 2..9 {
            canvas.set(x + dx, y + 2, '~', art::CLOUD_SH);
        }
    }

    fn hill(&self, canvas: &mut Canvas, cx: isize, height: isize) {
        let base = self.ground as isize - 1;
        // Glyph-only scene: a solid mound would be a wall of `#`.  Draw the
        // dome as a silhouette with two dotted shading bands instead.
        let mut last = 1;
        for dy in 0..height {
            let t = dy as f32 / height as f32;
            let span = (height as f32 * (1.0 - t * t).powf(0.55)).round() as isize;
            canvas.set(cx - span, base - dy, '#', art::HILL);
            canvas.set(cx + span, base - dy, '#', art::HILL_DK);
            if dy % 3 == 1 {
                let mut dx = -span + 2;
                while dx < span - 1 {
                    canvas.set(cx + dx, base - dy, '.', art::HILL_DK);
                    dx += 3;
                }
            }
            last = span;
        }
        for dx in -last..=last {
            canvas.set(cx + dx, base - height + 1, '#', art::HILL_HI);
        }
    }

    fn bush(&self, canvas: &mut Canvas, cx: isize) {
        let base = self.ground as isize - 1;
        for dy in 0..2isize {
            let span = 5 - dy;
            for dx in -span..=span {
                let edge = dx.abs() >= span - 1;
                let (ch, col) = if edge {
                    ('#', art::HILL)
                } else if (dx + dy).rem_euclid(4) == 0 {
                    ('.', art::HILL_HI)
                } else {
                    ('#', art::HILL_DK)
                };
                canvas.set(cx + dx, base - dy, ch, col);
            }
        }
        for dx in -3..=3 {
            canvas.set(cx + dx, base - 2, '#', art::HILL);
        }
    }

    fn draw_ground(&self, canvas: &mut Canvas) {
        let g = self.ground as isize;
        let w = self.w as isize;
        let h = self.h as isize;
        if g >= h {
            return;
        }
        for x in 0..w {
            canvas.set(x, g, '#', art::BRICK_HI);
            if (g + 1) < h {
                canvas.set(x, g + 1, '#', art::BRICK);
            }
            if (g + 2) < h {
                canvas.set(x, g + 2, '-', art::BRICK_DK);
            }
            for y in g + 3..h {
                let speck = (x * 7 + y * 3).rem_euclid(13) == 0;
                let (ch, col) = if speck {
                    ('.', art::SOIL_HI)
                } else {
                    ('#', art::SOIL)
                };
                canvas.set(x, y, ch, col);
            }
        }
        // Staggered brick seams: top row on 12s, lower row on 6s.
        for x in (0..w).step_by(12) {
            if (g + 1) < h {
                canvas.set(x, g + 1, '|', art::BRICK_DK);
            }
        }
        for x in (6..w).step_by(12) {
            if (g + 2) < h {
                canvas.set(x, g + 2, '+', art::BRICK_DK);
            }
        }
    }

    fn draw_block(&self, canvas: &mut Canvas, b: &Block) {
        let offset = if b.bump > 0.0 {
            (b.bump * 9.0).sin().abs() * 2.0
        } else {
            0.0
        };
        let x = b.x.round() as isize;
        let y = b.y.round() as isize - offset.round() as isize;
        let face = b.kind.face();
        let edge = b.kind.edge();
        for dy in 0..BLOCK_H {
            for dx in 0..BLOCK_W {
                let border = dx == 0 || dx == BLOCK_W - 1 || dy == 0 || dy == BLOCK_H - 1;
                if border {
                    let ch = if (dx == 0 || dx == BLOCK_W - 1) && (dy == 0 || dy == BLOCK_H - 1) {
                        '+'
                    } else if dy == 0 || dy == BLOCK_H - 1 {
                        '-'
                    } else {
                        '|'
                    };
                    canvas.set(x + dx, y + dy, ch, edge);
                } else {
                    canvas.set(x + dx, y + dy, '#', face);
                }
            }
        }
        match b.kind {
            BlockKind::Question => {
                canvas.set(x + 5, y + 2, '?', art::INK);
                for (rx, ry) in [(2, 1), (9, 1), (2, 3), (9, 3)] {
                    canvas.set(x + rx, y + ry, '.', art::GOLD_DK);
                }
            }
            BlockKind::Brick => {
                // One seam is enough at this size; a full grid reads as noise.
                for dx in 6..BLOCK_W - 1 {
                    canvas.set(x + dx, y + 2, '-', art::BRICK_DK);
                }
            }
            BlockKind::Used => {}
        }
    }

    fn draw_pipe(&self, canvas: &mut Canvas, p: &Pipe) {
        let x = p.x.round() as isize;
        let ground = self.ground as isize;
        let top = ground - p.height;
        let body_top = top + 2;
        if body_top < ground {
            for y in body_top..ground {
                for dx in 0..PIPE_W {
                    let (ch, col) = if dx < 2 {
                        ('#', art::PIPE_HI)
                    } else if dx >= PIPE_W - 3 {
                        ('#', art::PIPE_DK)
                    } else {
                        ('#', art::PIPE)
                    };
                    canvas.set(x + dx, y, ch, col);
                }
                canvas.set(x, y, '|', art::PIPE_DK);
                canvas.set(x + PIPE_W - 1, y, '|', art::PIPE_DK);
            }
        }
        // Mouth: a lip that overhangs the body by one column.
        for dy in 0..2 {
            for dx in -1..=PIPE_W {
                let col = if dx < 2 {
                    art::PIPE_HI
                } else if dx > PIPE_W - 4 {
                    art::PIPE_DK
                } else {
                    art::PIPE
                };
                canvas.set(x + dx, top + dy, '#', col);
            }
        }
        for dx in -1..=PIPE_W {
            canvas.set(x + dx, top, '-', art::PIPE_DK);
        }
        canvas.set(x - 1, top, '+', art::PIPE_DK);
        canvas.set(x + PIPE_W, top, '+', art::PIPE_DK);
        for dx in -1..=PIPE_W {
            canvas.set(x + dx, top + 2, '-', art::PIPE_DK);
        }
    }

    fn draw_actor(
        &self,
        canvas: &mut Canvas,
        rows: &'static [&'static str],
        x: f32,
        feet: f32,
        unicode: bool,
    ) {
        let height = rows.len() as isize;
        let top = feet.round() as isize - (height - 1);
        let left = x.round() as isize;
        for (dy, row) in rows.iter().enumerate() {
            for (dx, key) in row.chars().enumerate() {
                if let Some((ascii, uni, color)) = art::material(key) {
                    let ch = if unicode { uni } else { ascii };
                    canvas.set(left + dx as isize, top + dy as isize, ch, color);
                }
            }
        }
    }

    fn draw_mario(&self, canvas: &mut Canvas, unicode: bool) {
        let rows = if !self.on_ground {
            art::MARIO_JUMP
        } else {
            match self.run_frame() {
                0 => art::MARIO_STAND,
                1 => art::MARIO_RUN_A,
                _ => art::MARIO_RUN_B,
            }
        };
        self.draw_actor(canvas, rows, self.mario_x, self.y, unicode);
    }

    fn run_frame(&self) -> usize {
        // Three-frame shuffle driven by the distance travelled.
        ((self.scroll * 0.45) as i64 % 3).max(0) as usize
    }

    fn draw_goombas(&self, canvas: &mut Canvas, unicode: bool) {
        for g in &self.goombas {
            let rows = if g.dead {
                art::GOOMBA_FLAT
            } else if g.phase.sin() > 0.0 {
                art::GOOMBA_A
            } else {
                art::GOOMBA_B
            };
            self.draw_actor(canvas, rows, g.x, self.ground - 1.0, unicode);
        }
    }

    fn draw_shrooms(&self, canvas: &mut Canvas, unicode: bool) {
        for s in &self.shrooms {
            self.draw_actor(canvas, art::MUSHROOM, s.x, s.y, unicode);
        }
    }

    fn draw_coins(&self, canvas: &mut Canvas, unicode: bool) {
        let mut draw = |x: f32, feet: f32, phase: f32| {
            let spin = ((phase + self.time * 5.0) * 0.9).sin();
            let rows = if spin.abs() > 0.66 {
                art::COIN_WIDE
            } else if spin.abs() > 0.2 {
                art::COIN_MID
            } else {
                art::COIN_THIN
            };
            self.draw_actor(canvas, rows, x, feet, unicode);
        };
        for c in &self.coins_free {
            draw(c.x, c.y, c.phase);
        }
        for c in &self.coin_pops {
            draw(c.x, c.y, c.life * 12.0);
        }
    }

    fn draw_effects(&self, canvas: &mut Canvas, theme: &crate::theme::Theme) {
        for t in &self.texts {
            let col = if t.life > 0.4 {
                art::WHITE
            } else {
                theme.c(4).scale(0.9)
            };
            canvas.put_str(t.x.round() as isize, t.y.round() as isize, t.text, col);
        }
        for d in &self.dust {
            let col = art::CLOUD_SH.scale(0.9);
            let ch = if d.life > 0.25 { 'o' } else { '.' };
            canvas.set(d.x.round() as isize, d.y.round() as isize, ch, col);
        }
    }

    fn draw_hud(&self, canvas: &mut Canvas, unicode: bool) {
        let w = self.w as isize;
        let white = art::WHITE;
        canvas.put_str(2, 1, "MARIO", white);
        canvas.put_str(9, 1, &format!("{:06}", self.score), white);
        canvas.put_str(18, 1, if unicode { "◉" } else { "o" }, art::GOLD);
        canvas.put_str(20, 1, &format!("x{:02}", self.coins.min(99)), white);
        // On narrow terminals the right-hand world/time block would collide
        // with the score, so keep only the essential counters.
        if w >= 60 {
            let world = "WORLD 1-1";
            canvas.put_str((w - world.len() as isize - 22).max(34), 1, world, white);
            let time = format!("TIME {:03}", self.time_left as u32);
            canvas.put_str((w - time.len() as isize - 2).max(2), 1, &time, white);
        }
    }
}

fn wrap(v: f32, span: f32) -> f32 {
    let s = span.max(1.0);
    ((v % s) + s) % s
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
        self.update_spawner(dt);
        self.update_blocks(dt);
        self.update_physics(dt);
        self.update_actors(dt);
        for p in &mut self.pipes {
            p.x -= RUN_SPEED * dt;
        }
        self.pipes.retain(|p| p.x + PIPE_W as f32 > -4.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let unicode = f.charset == Charset::Unicode;
        self.draw_sky(canvas);
        self.draw_ground(canvas);
        for p in &self.pipes {
            self.draw_pipe(canvas, p);
        }
        for b in &self.blocks {
            self.draw_block(canvas, b);
        }
        self.draw_coins(canvas, unicode);
        self.draw_shrooms(canvas, unicode);
        self.draw_goombas(canvas, unicode);
        self.draw_mario(canvas, unicode);
        self.draw_effects(canvas, theme);
        self.draw_hud(canvas, unicode);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme;

    fn frame() -> Frame<'static> {
        Frame {
            t: 0.0,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: theme::find("gold").unwrap(),
            charset: Charset::Ascii,
            seed: 3,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    #[test]
    fn has_mushroom_blocks() {
        let mut s = MarioScene::new(7);
        s.resize(120, 36);
        assert!(s.blocks.iter().any(|b| b.kind == BlockKind::Question));
        assert!(s.ground > 10.0);
        assert!(s.blocks.iter().all(|b| b.y >= 0.0 && b.y + 4.0 < s.ground));
    }

    #[test]
    fn no_stomp_from_far_above() {
        // A goomba walking under a high-flying Mario must not bounce him.
        let mut s = MarioScene::new(3);
        s.resize(120, 36);
        s.goombas.clear();
        s.goombas.push(Goomba {
            x: s.mario_x,
            phase: 0.0,
            squash: 0.0,
            dead: false,
        });
        s.y = 6.0;
        s.vy = 1.0;
        s.on_ground = false;
        for _ in 0..30 {
            s.update_actors(1.0 / 30.0);
        }
        assert!(s.vy > 0.0, "Mario must keep falling, vy = {}", s.vy);
        assert_eq!(s.score, 0);
        assert!(!s.goombas[0].dead);
    }

    #[test]
    fn stomp_needs_a_descent() {
        let mut s = MarioScene::new(3);
        s.resize(120, 36);
        s.goombas.clear();
        s.goombas.push(Goomba {
            x: s.mario_x,
            phase: 0.0,
            squash: 0.0,
            dead: false,
        });
        // Rising over the goomba: no stomp.
        s.y = s.ground - 2.0;
        s.vy = -5.0;
        s.update_actors(1.0 / 30.0);
        assert_eq!(s.score, 0);
        assert!(!s.goombas[0].dead);

        // Falling onto it: stomp.
        s.vy = 10.0;
        s.update_actors(1.0 / 30.0);
        assert_eq!(s.score, 100);
        assert!(s.goombas[0].dead);
        assert!(s.vy < 0.0);
    }

    #[test]
    fn stomped_goomba_disappears() {
        let mut s = MarioScene::new(3);
        s.resize(120, 36);
        s.goombas.clear();
        s.goombas.push(Goomba {
            x: s.mario_x,
            phase: 0.0,
            squash: 0.0,
            dead: false,
        });
        s.y = s.ground - 2.0;
        s.vy = 10.0;
        s.update_actors(1.0 / 30.0);
        assert_eq!(s.goombas.len(), 1);
        for _ in 0..15 {
            s.update_actors(1.0 / 30.0);
        }
        assert!(s.goombas.is_empty(), "flattened goomba must go away");
    }

    #[test]
    fn runner_stays_near_the_ground() {
        // 30 simulated seconds: no phantom stomp may launch Mario into orbit.
        let mut s = MarioScene::new(11);
        s.resize(120, 36);
        let mut f = frame();
        let mut highest = 0.0f32;
        for i in 0..900 {
            f.t = i as f64 / 30.0;
            s.update(&f);
            highest = highest.max(s.ground - 1.0 - s.y);
            assert!(s.y < f32::INFINITY);
        }
        assert!(
            highest <= 12.0,
            "Mario rose {highest} rows above the ground"
        );
    }

    #[test]
    fn runner_never_leaves_the_screen() {
        let mut s = MarioScene::new(3);
        s.resize(120, 36);
        s.y = -25.0;
        s.vy = -10.0;
        s.on_ground = false;
        s.update(&frame());
        assert!(s.y >= -1.0, "y = {}", s.y);
        assert!(s.y <= s.ground - 1.0);
    }

    #[test]
    fn sprite_rows_are_rectangular() {
        for sprite in [
            art::MARIO_STAND,
            art::MARIO_RUN_A,
            art::MARIO_RUN_B,
            art::MARIO_JUMP,
            art::GOOMBA_A,
            art::GOOMBA_B,
            art::MUSHROOM,
        ] {
            let width = sprite[0].chars().count();
            assert!(width > 0);
            for row in sprite {
                assert_eq!(row.chars().count(), width);
            }
        }
    }
}
