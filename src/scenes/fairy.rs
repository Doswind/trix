//! 仙女散花 — a flying apsara scattering petals.
//!
//! The fairy floats in a moonlit sky with flowing ribbons and a flower basket;
//! petals continuously fall across the whole screen.  `--petal-count` and
//! `--petal-speed` control the shower.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const NIGHT: Rgb = Rgb::hex(0x0b1026);
const SKY: Rgb = Rgb::hex(0x312e81);
const MOON: Rgb = Rgb::hex(0xfef3c7);
const SKIN: Rgb = Rgb::hex(0xfbbf24);
const HAIR: Rgb = Rgb::hex(0x1e1b4b);
const ROBE: Rgb = Rgb::hex(0xf9a8d4);
const RIBBON: Rgb = Rgb::hex(0x22d3ee);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const BASKET: Rgb = Rgb::hex(0xb45309);
const WHITE: Rgb = Rgb::hex(0xfff7fb);
const PETAL_COLORS: [Rgb; 4] = [
    Rgb::hex(0xf472b6),
    Rgb::hex(0xfb7185),
    Rgb::hex(0xfacc15),
    Rgb::hex(0xfda4af),
];

#[derive(Clone)]
struct Petal {
    x: f32,
    y: f32,
    vy: f32,
    vx: f32,
    phase: f32,
    color: Rgb,
    ch: char,
}

pub struct FairyScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    petals: Vec<Petal>,
}

impl FairyScene {
    pub fn new(seed: u64) -> Self {
        FairyScene {
            seed,
            rng: Rng::new(seed ^ 0x0046_4149_5259),
            w: 0,
            h: 0,
            petals: Vec::new(),
        }
    }

    fn basket(&self) -> (f32, f32) {
        (self.w as f32 * 0.47, self.h as f32 * 0.42)
    }

    fn spawn_petal(&mut self) {
        let (bx, by) = self.basket();
        let a = self.rng.range(-2.5, -0.7);
        let r = self.rng.range(0.0, 5.0);
        let ch = if self.rng.chance(0.5) { '❀' } else { '✿' };
        self.petals.push(Petal {
            x: bx + a.cos() * r,
            y: by + a.sin() * r,
            vy: self.rng.range(1.2, 3.8),
            vx: self.rng.range(-0.8, 1.4),
            phase: self.rng.range(0.0, TAU),
            color: PETAL_COLORS[self.rng.below(PETAL_COLORS.len())],
            ch,
        });
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.petals.clear();
        for _ in 0..80 {
            self.spawn_petal();
        }
    }

    fn draw_fairy(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let cx = (self.w as f32 * 0.38 + (t * 0.5).sin() * 1.2) as isize;
        let cy = (self.h as f32 * 0.33) as isize;
        let sway = ((t * 1.1).sin() * 2.0) as isize;
        let hair = if unicode { '█' } else { '#' };
        let skin = if unicode { '█' } else { '@' };
        let robe = if unicode { '▓' } else { '=' };
        // ribbons streaming to the right
        for k in 0..18 {
            let x = cx + 3 + k;
            let y = cy
                + 6
                + ((t * 1.5 + k as f32 * 0.4).sin() * 3.0) as isize
                + (k as f32 * 0.35) as isize;
            canvas.set(
                x,
                y,
                if unicode { '·' } else { '~' },
                RIBBON.scale(0.42 - k as f32 * 0.012),
            );
        }
        // hair bun + head
        for dx in -3..=3 {
            canvas.set(cx + dx + sway / 2, cy - 5, hair, HAIR);
        }
        for dy in -4..=1 {
            for dx in -3..=3 {
                if dx * dx + (dy + 1) * (dy + 1) <= 10 {
                    canvas.set(cx + dx + sway / 2, cy + dy, skin, SKIN.scale(0.95));
                }
            }
        }
        canvas.set(
            cx - 1 + sway / 2,
            cy - 2,
            if unicode { '·' } else { 'o' },
            HAIR,
        );
        canvas.set(
            cx + 1 + sway / 2,
            cy - 2,
            if unicode { '·' } else { 'o' },
            HAIR,
        );
        // robe / body
        for dy in 2..12 {
            let half = 2 + dy / 3;
            for dx in -half..=half {
                canvas.set(
                    cx + dx + sway,
                    cy + dy,
                    robe,
                    ROBE.scale(0.75 + dy as f32 * 0.01),
                );
            }
        }
        // arms and basket
        canvas.line(
            cx + sway,
            cy + 3,
            cx + 7,
            cy + 1,
            if unicode { '╱' } else { '/' },
            SKIN.scale(0.9),
        );
        canvas.line(
            cx + sway,
            cy + 3,
            cx - 6,
            cy + 5,
            if unicode { '╲' } else { '\\' },
            SKIN.scale(0.9),
        );
        let bx = cx + 8;
        let by = cy + 2;
        for dx in 0..5 {
            canvas.set(bx + dx, by, if unicode { '▄' } else { '\\' }, BASKET);
        }
        canvas.set(
            bx + 2,
            by - 1,
            if unicode { '❀' } else { '*' },
            PETAL_COLORS[0],
        );
        canvas.set(
            bx + 1,
            by - 1,
            if unicode { '✿' } else { '*' },
            PETAL_COLORS[2],
        );
        canvas.add_light(bx + 2, by - 1, GOLD, 0.3, '.');
        // lotus base / train
        for k in 0..5 {
            canvas.set(
                cx - 5 + k * 2 + sway,
                cy + 13,
                if unicode { '▰' } else { '=' },
                ROBE.scale(0.55),
            );
        }
        canvas.add_light(cx, cy, WHITE, 0.18, '.');
    }
}

impl Scene for FairyScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let target = f.tune.petal_count.clamp(1, 300) as usize;
        let speed = f.tune.petal_speed.clamp(0.05, 4.0);
        if self.petals.len() > target {
            self.petals.truncate(target);
        }
        let mut made = 0;
        while self.petals.len() < target && made < 3 {
            self.spawn_petal();
            made += 1;
        }
        for p in &mut self.petals {
            p.y += p.vy * speed * dt;
            p.x += (p.vx + (f.t as f32 * 1.3 + p.phase).sin() * 0.8) * dt;
            p.phase += dt * 2.0;
        }
        self.petals
            .retain(|p| p.y < self.h as f32 + 2.0 && p.x > -2.0 && p.x < self.w as f32 + 2.0);
        let mut made = 0;
        while self.petals.len() < target && made < 3 {
            self.spawn_petal();
            made += 1;
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / self.h.max(1) as f32;
            let c = NIGHT.lerp(SKY, v * 0.8);
            for x in 0..self.w as isize {
                if (x + y * 4) % 10 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.7));
                }
            }
        }
        // moon
        let mx = (self.w as f32 * 0.76) as isize;
        let my = (self.h as f32 * 0.18) as isize;
        for dy in -4..=4 {
            for dx in -6..=6 {
                if dx * dx + dy * dy * 2 <= 36 {
                    canvas.set(
                        mx + dx,
                        my + dy,
                        if unicode { '░' } else { '.' },
                        MOON.scale(0.18),
                    );
                }
            }
        }
        canvas.set(mx, my, if unicode { '◉' } else { 'O' }, MOON.scale(0.8));
        // cloud bands
        for k in 0..5 {
            let y = (self.h as f32 * (0.62 + k as f32 * 0.07)) as isize;
            let x0 = ((t * 3.0 + k as f32 * 23.0) as isize) % (self.w as isize + 30) - 15;
            for i in 0..12 {
                canvas.set_if_blank(x0 + i, y, if unicode { '≈' } else { '~' }, ROBE.scale(0.16));
            }
        }
        self.draw_fairy(canvas, unicode, t);
        for p in &self.petals {
            let x = p.x.round() as isize;
            let y = (p.y + (t * 1.5 + p.phase).sin()).round() as isize;
            canvas.set(x, y, if unicode { p.ch } else { '*' }, p.color.scale(0.85));
            if unicode {
                canvas.set(x - 1, y, '·', p.color.scale(0.35));
            }
        }
        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ FAIRY // CELESTIAL SHOWER ╞"
        } else {
            " FAIRY // CELESTIAL SHOWER "
        };
        canvas.put_str(2, 0, title, ROBE.scale(0.95));
        let status = format!(
            "PETALS {:03}  {:.1}x",
            self.petals.len(),
            f.tune.petal_speed
        );
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            GOLD.scale(0.75),
        );
        canvas.put_str(2, h - 1, "--petal-count / --petal-speed", RIBBON.scale(0.5));
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn petals_exist() {
        let mut s = FairyScene::new(7);
        s.resize(120, 36);
        assert!(s.petals.len() > 20);
    }
}
