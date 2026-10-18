//! Tom & Jerry — a zero-gravity chase through a neon space station.
//!
//! The global `--speed` scale (and the interactive +/- keys) directly change
//! how fast Jerry runs and Tom follows.  The corridor, window, floor grid and
//! props keep the whole screen populated between the two characters.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const TOM_A: &[&str] = &[
    "~  /\\_/\\       ",
    "  ( o.o )>      ",
    "   > ^ <        ",
    "  /|   |\\__    ",
    " /_|___|___\\   ",
    "    |   |       ",
];
const TOM_B: &[&str] = &[
    "~  /\\_/\\       ",
    "  ( o.o )>      ",
    "   > ^ <        ",
    "  /|   |\\__    ",
    " |_|_____|__|   ",
    "   /|   |\\     ",
];
const JERRY_A: &[&str] = &[" ~(\\_/) ", "  (o.o)>", "   > ^ < ", "  /|_|\\ "];
const JERRY_B: &[&str] = &[" ~(\\_/) ", "  (o.o)>", "   > ^ < ", "  _| |_  "];

struct Dust {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
}

pub struct TomJerryScene {
    rng: Rng,
    w: usize,
    h: usize,
    dust: Vec<Dust>,
    stars: Vec<(f32, f32, f32, f32)>,
    next_dust: f32,
    tom_phase: f32,
}

impl TomJerryScene {
    pub fn new(seed: u64) -> Self {
        TomJerryScene {
            rng: Rng::new(seed ^ 0x70a1_1e22),
            w: 0,
            h: 0,
            dust: Vec::new(),
            stars: Vec::new(),
            next_dust: 0.0,
            tom_phase: 0.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.dust.clear();
        self.stars.clear();
        let n = (self.w * self.h / 40).clamp(60, 240);
        for _ in 0..n {
            self.stars.push((
                self.rng.range(0.0, self.w as f32),
                self.rng.range(0.0, self.h as f32 * 0.55),
                self.rng.range(0.0, TAU),
                self.rng.range(0.15, 0.7),
            ));
        }
    }

    fn draw_sprite(
        &self,
        canvas: &mut Canvas,
        art: &[&str],
        x: f32,
        y: f32,
        scale: usize,
        tom: bool,
    ) {
        for (row, line) in art.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let color = sprite_color(ch, tom);
                for sy in 0..scale {
                    for sx in 0..scale {
                        canvas.set(
                            x as isize + (col * scale + sx) as isize,
                            y as isize + (row * scale + sy) as isize,
                            ch,
                            color,
                        );
                    }
                }
                if ch == 'o' {
                    canvas.add_light(
                        x as isize + (col * scale) as isize,
                        y as isize + (row * scale) as isize,
                        Rgb::hex(0xffffff),
                        0.0,
                        '.',
                    );
                }
            }
        }
    }

    fn draw_props(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        // On very short terminals the holograms would land in the HUD rows;
        // the chase still reads without them.
        if self.h < 14 {
            return;
        }
        let ground = (self.h as f32 * 0.72) as isize;
        // left wall terminal
        canvas.put_str(3, ground - 7, "+--------+", theme.c(2).scale(0.8));
        canvas.put_str(3, ground - 6, "| > _    |", theme.c(2).scale(0.7));
        canvas.put_str(3, ground - 5, "| NASA-7 |", theme.c(2).scale(0.65));
        canvas.put_str(3, ground - 4, "+--------+", theme.c(2).scale(0.8));
        // right-hand cheese hologram
        let cx = self.w as isize - 16;
        canvas.put_str(cx, ground - 8, "  ___  ", Rgb::hex(0xfacc15));
        canvas.put_str(cx, ground - 7, " /   \\ ", Rgb::hex(0xfacc15));
        canvas.put_str(cx, ground - 6, "/_____\\", Rgb::hex(0xeab308));
        canvas.put_str(cx + 1, ground - 9, "CHEESE", Rgb::hex(0xfde68a).scale(0.8));
        // floating dust / station motes
        for i in 0..18 {
            let x = (i as f32 * 7.0 + t * 3.0 + (i as f32 * 1.7).sin() * 4.0) % self.w as f32;
            let y = self.h as f32 * 0.25 + (i as f32 * 2.3 + t).sin() * self.h as f32 * 0.18;
            canvas.set_if_blank(x as isize, y as isize, '.', theme.c(2).scale(0.2));
        }
    }
}

fn sprite_color(ch: char, tom: bool) -> Rgb {
    if tom {
        match ch {
            '#' | '%' => Rgb::hex(0x94a3b8),
            'o' => Rgb::hex(0x0f172a),
            '^' => Rgb::hex(0xf9a8d4),
            '_' => Rgb::hex(0xf8fafc),
            '~' => Rgb::hex(0xf9a8d4),
            '>' => Rgb::hex(0xf8fafc),
            _ => Rgb::hex(0x64748b),
        }
    } else {
        match ch {
            '#' | '%' => Rgb::hex(0xd97706),
            'o' => Rgb::hex(0x111827),
            '^' => Rgb::hex(0xf9a8d4),
            '_' => Rgb::hex(0xfde68a),
            '>' => Rgb::hex(0xfde68a),
            _ => Rgb::hex(0xb45309),
        }
    }
}

impl Scene for TomJerryScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!("CHASE x{:.1}", 0.55 * f.speed))
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        self.tom_phase += dt;
        if f.t as f32 > self.next_dust {
            for _ in 0..2 {
                self.dust.push(Dust {
                    x: self.w as f32 * self.rng.range(0.2, 0.8),
                    y: self.h as f32 * 0.70,
                    vx: self.rng.range(-6.0, 2.0),
                    vy: self.rng.range(-7.0, -2.0),
                    life: self.rng.range(0.4, 1.1),
                });
            }
            self.next_dust = f.t as f32 + self.rng.range(0.12, 0.3);
        }
        for d in self.dust.iter_mut() {
            d.x += d.vx * dt;
            d.y += d.vy * dt;
            d.vy += 3.0 * dt;
            d.life -= dt;
        }
        self.dust.retain(|d| d.life > 0.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let w = self.w as f32;
        let h = self.h as f32;
        let scale = 1usize;
        let ground = (h * 0.72) as isize;
        let tom_art_h = TOM_A.len() * scale;
        let jerry_art_h = JERRY_A.len() * scale;

        // window / starfield in the upper wall
        let win_top = 1isize;
        let win_bot = (h * 0.43) as isize;
        for y in win_top..win_bot {
            for x in 1..self.w as isize - 1 {
                let hsh = hash2(x as i64, y as i64, 0x7a11);
                if hsh % 31 != 0 {
                    continue;
                }
                canvas.set(
                    x,
                    y,
                    if hsh & 1 == 0 { '.' } else { '+' },
                    theme
                        .c(2)
                        .scale(0.22 + 0.25 * ((t * 1.3 + x as f32 * 0.1).sin().abs())),
                );
            }
        }
        canvas.hline(1, self.w as isize - 2, win_top, '-', theme.c(2).scale(0.4));
        canvas.hline(1, self.w as isize - 2, win_bot, '-', theme.c(2).scale(0.4));
        canvas.vline(1, win_top, win_bot, '|', theme.c(2).scale(0.4));
        canvas.vline(
            self.w as isize - 2,
            win_top,
            win_bot,
            '|',
            theme.c(2).scale(0.4),
        );

        // wall panels / neon strips
        for x in (6..self.w as isize - 4).step_by(14) {
            canvas.vline(x, win_bot + 1, ground - 1, '|', theme.c(1).scale(0.55));
            canvas.set(x, win_bot + 2, '+', theme.c(2).scale(0.7));
            canvas.set(x, ground - 2, '+', theme.c(2).scale(0.7));
        }
        for y in (win_bot + 3..ground - 1).step_by(3) {
            for x in (2..self.w as isize - 2).step_by(2) {
                if hash2(x as i64, y as i64, 0x5a1) % 19 == 0 {
                    canvas.set(x, y, '.', theme.c(2).scale(0.2));
                }
            }
        }

        // perspective floor
        let vp_x = w * 0.5;
        let vp_y = ground as f32 - 2.0;
        for i in 0..=12 {
            let x_bottom = i as f32 / 12.0 * w;
            canvas.line(
                vp_x as isize,
                vp_y as isize,
                x_bottom as isize,
                self.h as isize - 2,
                '/',
                theme.c(2).scale(0.35),
            );
        }
        for i in 0..4 {
            let y = ground + 1 + i * 2;
            if y < self.h as isize - 1 {
                canvas.hline(1, self.w as isize - 2, y, '-', theme.c(1).scale(0.5));
            }
        }
        canvas.hline(1, self.w as isize - 2, ground, '=', theme.c(2).scale(0.8));

        // dust
        for d in &self.dust {
            canvas.set(
                d.x as isize,
                d.y as isize,
                if d.life > 0.5 { '*' } else { '.' },
                theme.c(3).scale(0.35),
            );
        }

        // chase positions: deliberately slower than the old default
        let chase = 0.55;
        let tt = t * chase;
        let jumping = f.tune.jump;
        let track = (w - 20.0).max(18.0);
        let jerry_x = 6.0 + (tt * 32.0 + 60.0) % track;
        let jump = if jumping {
            ((tt * 4.0).sin().max(0.0)).powf(2.0) * 3.0 * scale as f32
        } else {
            0.0
        };
        let jerry_y = (ground - jerry_art_h as isize) as f32 - jump;
        let gap = 22.0 + 8.0 * scale as f32;
        let pounce = if jumping {
            ((tt * 4.0).sin().max(0.0)).powf(2.0) * 4.0
        } else {
            0.0
        };
        let tom_x = (jerry_x - gap + (tt * 1.7).sin() * 2.0 + pounce).max(2.0);
        let tom_y = (ground - tom_art_h as isize) as f32
            - if jumping {
                ((tt * 3.8 + 0.7).sin().max(0.0)).powf(2.0) * 2.0 * scale as f32
            } else {
                0.0
            };

        // shadows and motion lines
        for (x, size) in [(jerry_x, 3.0), (tom_x, 5.0)] {
            let sx = x.round() as isize;
            for i in -size as isize..=size as isize {
                canvas.set(sx + i, ground, '-', theme.c(1).scale(0.7));
            }
        }
        for i in 0..5 {
            let x = jerry_x - 5.0 - i as f32 * 1.5;
            canvas.set(
                x as isize,
                jerry_y as isize + 3 * scale as isize,
                '-',
                theme.c(2).scale(0.4),
            );
        }
        for i in 0..4 {
            let x = tom_x - 5.0 - i as f32 * 1.5;
            canvas.set(
                x as isize,
                tom_y as isize + 4 * scale as isize,
                '-',
                theme.c(2).scale(0.3),
            );
        }

        // sprites
        let frame = ((tt * 8.0) as usize) % 2;
        let tom = if frame == 0 { TOM_A } else { TOM_B };
        let jerry = if frame == 0 { JERRY_A } else { JERRY_B };
        self.draw_sprite(canvas, tom, tom_x, tom_y, scale, true);
        self.draw_sprite(canvas, jerry, jerry_x, jerry_y, scale, false);

        // labels
        if self.w >= 70 {
            canvas.put_str(
                tom_x as isize + 2,
                tom_y as isize - 2,
                "TOM",
                Rgb::hex(0xcbd5e1).scale(0.8),
            );
            canvas.put_str(
                jerry_x as isize + 1,
                jerry_y as isize - 2,
                "JERRY",
                Rgb::hex(0xfbbf24).scale(0.8),
            );
        }
        self.draw_props(canvas, theme, t);
        let _ = (tom_art_h, tom_x, tom_y, chase);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sprites_are_nonempty() {
        assert!(TOM_A.iter().any(|l| !l.trim().is_empty()));
        assert!(JERRY_A.iter().any(|l| !l.trim().is_empty()));
    }
}
