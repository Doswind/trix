//! 仙女散花 — a hand-drawn apsara drifting across the night sky.
//!
//! The figure is a 30x34 pixel-art sprite drawn cell by cell
//! (`src/data/fairy_sprite.txt`): a low-bunned beauty in a white robe with a
//! blue sash, one arm raised to a basket of peonies, the other folded across
//! her chest, the skirt sweeping into a train.
//!
//! She orbits slowly through the upper sky and turns around her own axis
//! (a `scaleX` turn, never a head-over-heels spin), shedding petals from the
//! basket that follow her wake and then scatter downwards.
//!
//! `--petal-count` and `--petal-speed` control the shower.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};

const NIGHT: Rgb = Rgb::hex(0x070b1c);
const SKY: Rgb = Rgb::hex(0x272761);
const MOON: Rgb = Rgb::hex(0xfff3d0);
const GOLD: Rgb = Rgb::hex(0xf5c96b);
const WHITE: Rgb = Rgb::hex(0xfff6ea);
const PETAL_COLORS: [Rgb; 4] = [
    Rgb::hex(0xf7a8c8),
    Rgb::hex(0xfb7185),
    Rgb::hex(0xffe1ec),
    Rgb::hex(0xfacc15),
];

const BLANK: u8 = 0xff;

struct Ink {
    ascii: char,
    uni: char,
    color: Rgb,
}

/// Palette order matches `ink_index`.
const INK: [Ink; 17] = [
    Ink {
        ascii: '#',
        uni: '█',
        color: Rgb::hex(0x2a2232),
    }, // 1 hair
    Ink {
        ascii: '%',
        uni: '█',
        color: Rgb::hex(0x453a52),
    }, // 2 hair highlight
    Ink {
        ascii: '+',
        uni: '+',
        color: Rgb::hex(0x6f6a78),
    }, // 3 ink line
    Ink {
        ascii: '8',
        uni: '▒',
        color: Rgb::hex(0xc08a62),
    }, // 4 skin shadow
    Ink {
        ascii: '@',
        uni: '▒',
        color: Rgb::hex(0xe0b088),
    }, // 5 skin mid
    Ink {
        ascii: '@',
        uni: '░',
        color: Rgb::hex(0xf6d0a8),
    }, // 6 skin light
    Ink {
        ascii: '#',
        uni: '░',
        color: Rgb::hex(0xcfc6b8),
    }, // 7 robe shadow
    Ink {
        ascii: '#',
        uni: '░',
        color: Rgb::hex(0xe9e2d6),
    }, // 8 robe
    Ink {
        ascii: '#',
        uni: '░',
        color: Rgb::hex(0xfffbf2),
    }, // 9 robe light
    Ink {
        ascii: '%',
        uni: '▓',
        color: Rgb::hex(0x24395f),
    }, // a blue dark
    Ink {
        ascii: '%',
        uni: '▓',
        color: Rgb::hex(0x35548a),
    }, // b blue mid
    Ink {
        ascii: '=',
        uni: '▒',
        color: Rgb::hex(0x6f93c4),
    }, // c blue light
    Ink {
        ascii: 'n',
        uni: '▒',
        color: Rgb::hex(0xb07a42),
    }, // d basket
    Ink {
        ascii: '&',
        uni: '▒',
        color: Rgb::hex(0x4f7d4a),
    }, // e leaf
    Ink {
        ascii: '*',
        uni: '❀',
        color: Rgb::hex(0xe08fb4),
    }, // f flower
    Ink {
        ascii: 'o',
        uni: '·',
        color: Rgb::hex(0xf0c060),
    }, // g gold
    Ink {
        ascii: 'f',
        uni: '•',
        color: Rgb::hex(0xc0505a),
    }, // L lips
];

fn ink_index(ch: char) -> Option<u8> {
    let i = match ch {
        '1' => 0,
        '2' => 1,
        '3' => 2,
        '4' => 3,
        '5' => 4,
        '6' => 5,
        '7' => 6,
        '8' => 7,
        '9' => 8,
        'a' => 9,
        'b' => 10,
        'c' => 11,
        'd' => 12,
        'e' => 13,
        'f' => 14,
        'g' => 15,
        'L' => 16,
        _ => return None,
    };
    Some(i)
}

struct Sprite {
    w: isize,
    h: isize,
    cells: Vec<u8>,
}

impl Sprite {
    fn parse(text: &str) -> Self {
        let rows: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let h = rows.len();
        let mut cells = vec![BLANK; w * h];
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(i) = ink_index(ch) {
                    cells[y * w + x] = i;
                }
            }
        }
        Sprite {
            w: w as isize,
            h: h as isize,
            cells,
        }
    }

    fn at(&self, col: isize, row: isize) -> Option<u8> {
        if col < 0 || row < 0 || col >= self.w || row >= self.h {
            return None;
        }
        let v = self.cells[row as usize * self.w as usize + col as usize];
        (v != BLANK).then_some(v)
    }
}

#[derive(Clone)]
struct Petal {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    phase: f32,
    spin: f32,
    color: Rgb,
    glyph: u8,
    life: f32,
}

#[derive(Clone, Copy)]
struct Pose {
    cx: f32,
    cy: f32,
    scale: f32,
    /// `cos` of the turn angle: +1 faces us, 0 is edge-on, -1 is the back.
    turn: f32,
}

impl Pose {
    fn mirror(&self) -> bool {
        self.turn < 0.0
    }

    fn ax(&self) -> f32 {
        self.turn.abs().max(0.08)
    }

    /// Sprite cell -> screen cell.
    fn project(&self, sprite: &Sprite, col: f32, row: f32) -> (f32, f32) {
        let u = (col - sprite.w as f32 * 0.5) * self.scale * self.ax();
        let u = if self.mirror() { -u } else { u };
        let v = (row - sprite.h as f32 * 0.5) * self.scale;
        (self.cx + u, self.cy + v)
    }
}

pub struct FairyScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    sprite: Sprite,
    petals: Vec<Petal>,
    pose: Pose,
    drift_x: f32,
    spawn_acc: f32,
}

impl FairyScene {
    pub fn new(seed: u64) -> Self {
        FairyScene {
            seed,
            rng: Rng::new(seed ^ 0x0046_4149_5259),
            w: 0,
            h: 0,
            sprite: Sprite::parse(include_str!("../data/fairy_sprite.txt")),
            petals: Vec::new(),
            pose: Pose {
                cx: 0.0,
                cy: 0.0,
                scale: 1.0,
                turn: 1.0,
            },
            drift_x: 0.0,
            spawn_acc: 0.0,
        }
    }

    /// A slow elliptical orbit through the upper sky plus a gentle turn.
    fn layout(&self, f: &Frame) -> Pose {
        let t = f.t as f32;
        let w = self.w.max(1) as f32;
        let h = self.h.max(1) as f32;
        let scale = (h * 0.70 / self.sprite.h as f32).clamp(0.45, 2.0);
        let half_h = self.sprite.h as f32 * 0.5 * scale;

        let cx = w * 0.50 + w * 0.20 * (t * 0.13).cos();
        let cy = h * 0.40 + h * 0.09 * (t * 0.19 + 0.7).sin();
        let lo = half_h + 1.0;
        let hi = h - half_h - 3.0;
        let cy = if lo <= hi { cy.clamp(lo, hi) } else { h * 0.5 };

        let turn = turn_angle(t).cos();
        Pose {
            cx,
            cy,
            scale,
            turn,
        }
    }

    fn spawn_petal(&mut self, ambient: bool) {
        let glyph = self.rng.below(4) as u8;
        let color = PETAL_COLORS[self.rng.below(PETAL_COLORS.len())];
        if ambient || self.rng.chance(0.30) {
            let x = self.rng.range(-2.0, self.w as f32 + 2.0);
            let y = self.rng.range(-4.0, self.h as f32 * 0.20);
            self.petals.push(Petal {
                x,
                y,
                vx: self.rng.range(-0.5, 0.8),
                vy: self.rng.range(0.4, 1.4),
                phase: self.rng.range(0.0, TAU),
                spin: self.rng.range(0.5, 1.8),
                color,
                glyph,
                life: 0.0,
            });
            return;
        }
        // basket mouth
        let (bx, by) = self.pose.project(&self.sprite, 24.5, 4.0);
        let a = self.rng.range(-TAU * 0.55, -TAU * 0.05);
        let out = self.rng.range(0.8, 2.4);
        self.petals.push(Petal {
            x: bx + a.cos() * self.rng.range(0.0, 1.8),
            y: by + a.sin() * self.rng.range(0.0, 1.8),
            vx: a.cos() * out + self.drift_x * 0.5,
            vy: a.sin() * out * 0.4 + 0.7,
            phase: self.rng.range(0.0, TAU),
            spin: self.rng.range(0.6, 2.0),
            color,
            glyph,
            life: 0.0,
        });
    }

    fn draw_sky(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let w = self.w as isize;
        let h = self.h as isize;
        // faint gradient dots
        for y in (1..h).step_by(2) {
            let v = y as f32 / h.max(1) as f32;
            let c = NIGHT.lerp(SKY, v.min(1.0));
            for x in ((y * 7) % 11..w).step_by(11) {
                canvas.set(x, y, '.', c.scale(0.30));
            }
        }
        // stars
        let n = ((self.w * self.h) / 320).clamp(24, 120);
        for i in 0..n {
            let fx = hash01(i as i64, 11, self.seed);
            let fy = hash01(i as i64, 5, self.seed ^ 0x9e37_79b9);
            let x = (fx * self.w as f32) as isize;
            let y = (fy * self.h as f32 * 0.9) as isize;
            let tw = 0.4 + 0.6 * (t * (0.5 + fx) + i as f32).sin().abs();
            if tw > 0.72 {
                canvas.set(x, y, '.', WHITE.scale(0.12 + tw * 0.28));
            }
        }
        // moon
        let mx = (self.w as f32 * 0.82) as isize;
        let my = (self.h as f32 * 0.15) as isize;
        for dy in -3..=3 {
            for dx in -5..=5 {
                let e = (dx * dx) as f32 / 25.0 + (dy * dy) as f32 / 9.0;
                if e <= 1.0 {
                    let rim = e > 0.6;
                    canvas.set(
                        mx + dx,
                        my + dy,
                        if rim {
                            if unicode {
                                '◯'
                            } else {
                                'O'
                            }
                        } else {
                            '.'
                        },
                        MOON.scale(if rim { 0.55 } else { 0.28 }),
                    );
                }
            }
        }
    }

    fn draw_figure(&self, canvas: &mut Canvas, unicode: bool) {
        let p = self.pose;
        let ax = p.ax();
        let half_w = self.sprite.w as f32 * 0.5 * p.scale * ax;
        let half_h = self.sprite.h as f32 * 0.5 * p.scale;
        let x0 = (p.cx - half_w).floor() as isize;
        let x1 = (p.cx + half_w).ceil() as isize;
        let y0 = (p.cy - half_h).floor() as isize;
        let y1 = (p.cy + half_h).ceil() as isize;

        for y in y0..=y1 {
            for x in x0..=x1 {
                let u = (x as f32 + 0.5 - p.cx) / (p.scale * ax);
                let u = if p.mirror() { -u } else { u };
                let v = (y as f32 + 0.5 - p.cy) / p.scale;
                let col = self.sprite.w as f32 * 0.5 + u;
                let row = self.sprite.h as f32 * 0.5 + v;
                let hit = self
                    .sprite
                    .at(col.floor() as isize, row.floor() as isize)
                    .or_else(|| {
                        self.sprite
                            .at(col.floor() as isize, (row + 0.5).floor() as isize)
                    })
                    .or_else(|| {
                        self.sprite
                            .at((col + 0.5).floor() as isize, row.floor() as isize)
                    });
                if let Some(i) = hit {
                    let ink = &INK[i as usize];
                    canvas.set(x, y, if unicode { ink.uni } else { ink.ascii }, ink.color);
                }
            }
        }
        // a soft line of light when the figure is edge-on
        if ax < 0.22 {
            let len = (self.sprite.h as f32 * 0.35 * p.scale) as isize;
            let cy = p.cy as isize;
            let cx = p.cx as isize;
            for dy in -len..=len {
                canvas.add_light(cx, cy + dy, WHITE, 0.05, '.');
            }
        }
    }

    fn draw_petals(&self, canvas: &mut Canvas, unicode: bool) {
        for p in &self.petals {
            let glyph = match (unicode, p.glyph) {
                (true, 0) => '❀',
                (true, 1) => '✿',
                (true, 2) => '❁',
                (true, _) => '·',
                (false, 0) => '*',
                (false, 1) => '+',
                (false, 2) => 'x',
                (false, _) => '.',
            };
            let x = p.x.round() as isize;
            let y = p.y.round() as isize;
            let fade = (1.0 - p.life * 0.12).clamp(0.4, 1.0);
            canvas.set(x, y, glyph, p.color.scale(0.95 * fade));
            if p.glyph == 0 {
                canvas.set(
                    x - 1,
                    y,
                    if unicode { '·' } else { '.' },
                    p.color.scale(0.3 * fade),
                );
            }
        }
    }
}

impl Scene for FairyScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w == self.w && h == self.h {
            return;
        }
        self.w = w.max(1);
        self.h = h.max(1);
        self.petals.clear();
        self.pose = Pose {
            cx: self.w as f32 * 0.5,
            cy: self.h as f32 * 0.40,
            scale: (self.h as f32 * 0.70 / self.sprite.h as f32).clamp(0.45, 2.0),
            turn: 1.0,
        };
        let n = 50.min(self.w * self.h / 24).max(20);
        for _ in 0..n {
            self.spawn_petal(true);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let target = f.tune.petal_count.clamp(1, 300) as usize;
        let speed = f.tune.petal_speed.clamp(0.05, 4.0);
        let prev_x = self.pose.cx;
        self.pose = self.layout(f);
        if dt > 0.0 {
            self.drift_x = (self.pose.cx - prev_x) / dt;
        }

        self.spawn_acc += dt;
        let mut made = 0;
        while self.spawn_acc > 0.05 && self.petals.len() < target && made < 3 {
            self.spawn_acc -= 0.05;
            self.spawn_petal(false);
            made += 1;
        }
        if self.spawn_acc > 0.5 {
            self.spawn_acc = 0.5;
        }
        while self.petals.len() < target / 3 {
            self.spawn_petal(true);
        }

        for p in &mut self.petals {
            p.life += dt;
            p.vy = (p.vy + 2.6 * dt).min(6.5);
            p.x += (p.vx + (p.life * 2.0 + p.phase).sin() * 1.0) * dt;
            p.y += p.vy * speed * dt;
            p.phase += p.spin * dt;
            p.vx *= 1.0 - 0.3 * dt;
        }
        self.petals
            .retain(|p| p.y < self.h as f32 + 3.0 && p.x > -4.0 && p.x < self.w as f32 + 4.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_sky(canvas, unicode, t);
        self.draw_figure(canvas, unicode);
        self.draw_petals(canvas, unicode);

        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ FAIRY // CELESTIAL SHOWER ╞"
        } else {
            " FAIRY // CELESTIAL SHOWER "
        };
        canvas.put_str(2, 0, title, Rgb::hex(0x9fd2f2).scale(0.9));
        let status = format!(
            "PETALS {:03}  {:.1}x",
            self.petals.len(),
            f.tune.petal_speed
        );
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            GOLD.scale(0.8),
        );
        canvas.put_str(
            2,
            h - 1,
            "--petal-count / --petal-speed",
            Rgb::hex(0x9fd2f2).scale(0.45),
        );
    }
}

/// One full turn every 24 s, six seconds long, plus a gentle idle sway.
fn turn_angle(t: f32) -> f32 {
    const PERIOD: f32 = 24.0;
    const TIME: f32 = 6.0;
    const START: f32 = 8.0;
    let sway = 0.32 * (t * 0.31).sin();
    let u = ((t + PERIOD - START).rem_euclid(PERIOD)) / TIME;
    if u >= 1.0 {
        return sway;
    }
    let e = u * u * (3.0 - 2.0 * u);
    sway + TAU * e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_is_present() {
        let s = Sprite::parse(include_str!("../data/fairy_sprite.txt"));
        assert_eq!(s.w, 30);
        assert_eq!(s.h, 34);
        let ink = s.cells.iter().filter(|c| **c != BLANK).count();
        assert!(ink > 300, "sprite only has {ink} inked cells");
    }

    #[test]
    fn turn_returns_to_front() {
        assert!((turn_angle(0.0).cos() - 1.0).abs() < 0.06);
        assert!(
            turn_angle(11.0).cos() < -0.8,
            "should be back view mid-turn"
        );
        assert!(turn_angle(14.0).cos() > 0.8, "should face us again");
    }

    #[test]
    fn petals_exist() {
        let mut s = FairyScene::new(7);
        s.resize(120, 36);
        assert!(s.petals.len() > 15);
    }
}
