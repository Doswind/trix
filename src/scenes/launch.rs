//! launch — a stylized rocket launch, from pad to orbit.
//!
//! The twenty-two second loop covers countdown, ignition, ascent, booster
//! separation and an Earth-limb view.  Everything is glyphs: a tiny rocket
//! sprite, ballistic flame particles, pad smoke and a curved horizon.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const CYCLE: f32 = 22.0;
const FLAME: [Rgb; 4] = [
    Rgb::hex(0xfff6d8),
    Rgb::hex(0xffd166),
    Rgb::hex(0xff8a3d),
    Rgb::hex(0xd63b0a),
];

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    smoke: bool,
}

struct Booster {
    x: f32,
    y: f32,
    vy: f32,
    life: f32,
}

pub struct LaunchScene {
    w: usize,
    h: usize,
    rng: Rng,
    stars: Vec<(f32, f32, f32)>,
    particles: Vec<Particle>,
    booster: Option<Booster>,
    spawn: f32,
    smoke_spawn: f32,
    last_phase: f32,
}

impl LaunchScene {
    pub fn new(seed: u64) -> Self {
        LaunchScene {
            w: 0,
            h: 0,
            rng: Rng::new(seed ^ 0x4c41_554e_4348),
            stars: Vec::new(),
            particles: Vec::new(),
            booster: None,
            spawn: 0.0,
            smoke_spawn: 0.0,
            last_phase: 0.0,
        }
    }

    fn ascent(p: f32) -> f32 {
        let x = ((p - 4.5) / 9.0).clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    }

    fn spawn_flame(&mut self, rx: f32, ry: f32, pad: bool) {
        let life = self.rng.range(0.35, 0.95);
        let (vx, vy) = if pad {
            (self.rng.range(-4.0, 4.0), self.rng.range(2.0, 7.0))
        } else {
            (self.rng.range(-3.0, 3.0), self.rng.range(7.0, 18.0))
        };
        self.particles.push(Particle {
            x: rx + self.rng.range(-1.0, 1.0),
            y: ry,
            vx,
            vy,
            life,
            max_life: life,
            smoke: false,
        });
    }

    fn spawn_smoke(&mut self, x: f32, y: f32) {
        let life = self.rng.range(1.6, 4.0);
        self.particles.push(Particle {
            x,
            y,
            vx: self.rng.range(-4.5, 4.5),
            vy: self.rng.range(-1.8, 0.5),
            life,
            max_life: life,
            smoke: true,
        });
    }
}

impl Scene for LaunchScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let mut rng = Rng::new(0x5354_4152_4c41);
        self.stars.clear();
        for _ in 0..70 {
            self.stars.push((
                rng.range(0.0, w as f32),
                rng.range(1.0, h as f32 * 0.72),
                rng.range(0.0, TAU),
            ));
        }
        self.particles.clear();
        self.booster = None;
        self.spawn = 0.0;
        self.smoke_spawn = 0.0;
        self.last_phase = 0.0;
    }

    fn update(&mut self, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let p = (f.t as f32) % CYCLE;
        if p < self.last_phase {
            self.particles.clear();
            self.booster = None;
            self.spawn = 0.0;
            self.smoke_spawn = 0.0;
        }
        self.last_phase = p;

        let ascent = Self::ascent(p);
        let pad_y = self.h as f32 - 5.0;
        let ry = pad_y - 5.0 - ascent * (pad_y - 7.0);
        let rx = self.w as f32 * 0.5 + (p * 0.6).sin() * self.w as f32 * 0.025 * ascent;
        let pad_phase = (2.6..5.2).contains(&p);

        if (3.0..16.0).contains(&p) {
            let rate = if p < 5.0 { 80.0 } else { 45.0 };
            self.spawn += dt * rate;
            while self.spawn >= 1.0 {
                self.spawn -= 1.0;
                if self.particles.len() < 300 {
                    self.spawn_flame(rx, ry + 4.5, pad_phase);
                }
            }
        }
        if (2.6..7.5).contains(&p) {
            self.smoke_spawn += dt * 18.0;
            while self.smoke_spawn >= 1.0 {
                self.smoke_spawn -= 1.0;
                if self.particles.len() < 320 {
                    let sx = self.w as f32 * 0.5 + self.rng.range(-7.0, 7.0);
                    self.spawn_smoke(sx, pad_y);
                }
            }
        }

        if p >= 10.2 && self.booster.is_none() {
            self.booster = Some(Booster {
                x: rx,
                y: ry + 4.0,
                vy: 1.5,
                life: 1.0,
            });
        }
        if let Some(b) = self.booster.as_mut() {
            b.vy += 3.0 * dt;
            b.y += b.vy * dt;
            b.x += (p * 4.0).sin() * 4.0 * dt;
            b.life -= dt;
            if b.y > self.h as f32 - 2.0 || b.life <= 0.0 {
                self.booster = None;
            }
        }

        for p in &mut self.particles {
            p.vy += if p.smoke { -0.6 * dt } else { 7.0 * dt };
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| {
            p.life > 0.0 && p.y < self.h as f32 - 2.0 && p.x > -4.0 && p.x < self.w as f32 + 4.0
        });
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let p = (f.t as f32) % CYCLE;
        if p < 4.0 {
            Some(format!("T-{:02} | PAD", (4.0 - p).ceil() as u32))
        } else {
            let alt = Self::ascent(p) * 320.0;
            Some(format!(
                "T+{:02} | ALT {:03}KM",
                (p - 4.0) as u32,
                alt as u32
            ))
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let t = f.t as f32;
        let p = t % CYCLE;
        let max_y = self.h as isize - 3;
        if max_y < 2 {
            return;
        }
        let theme = f.theme;
        let w = self.w as f32;
        let h = self.h as f32;
        let cx = (w * 0.5) as isize;
        let ascent = Self::ascent(p);
        let pad_y = h - 5.0;
        let ry = pad_y - 5.0 - ascent * (pad_y - 7.0);
        let rx = w * 0.5 + (p * 0.6).sin() * w * 0.025 * ascent;

        // Stars fade in as the rocket climbs.
        for (sx, sy, phase) in &self.stars {
            let visible = if ascent > 0.35 { 0.35 + ascent } else { 0.25 };
            let tw = 0.5 + 0.5 * (t * 1.3 + phase).sin().abs();
            let y = *sy as isize;
            if y >= 1 && y <= max_y {
                canvas.set(
                    *sx as isize,
                    y,
                    '.',
                    theme.hot().scale(0.12 + 0.30 * tw * visible),
                );
            }
        }

        // Pad and gantry while the ground is still visible.
        let ground_y = pad_y + ascent * (h * 0.55);
        if ground_y > 1.0 && ground_y < max_y as f32 {
            let gy = ground_y.round() as isize;
            for x in 0..self.w as isize {
                canvas.set(x, gy, '#', theme.c(2).scale(0.35));
                if gy < max_y {
                    canvas.set(x, gy + 1, ':', theme.c(2).scale(0.20));
                }
            }
            for dx in [-8isize, 8] {
                let gx = cx + dx;
                if gx > 1 && gx < self.w as isize - 1 {
                    for y in (gy - 6).max(1)..gy {
                        canvas.set(gx, y, '|', theme.c(2).scale(0.30));
                    }
                    canvas.set(gx, (gy - 6).max(1), 'H', theme.c(2).scale(0.45));
                }
            }
            // Launch platform under the rocket.
            if gy > 1 {
                for x in (cx - 12).max(1)..=(cx + 12).min(self.w as isize - 1) {
                    canvas.set(x, gy - 1, '=', theme.c(1).scale(0.45));
                }
            }
        }

        // A low cloud deck rushes down and out during the ascent.
        if p > 5.0 && p < 14.0 {
            let phase = (p - 5.0) / 9.0;
            let deck = (self.h as f32 * 0.80 + phase * self.h as f32 * 0.35) as isize;
            for x in 0..self.w as isize {
                if deck >= 1 && deck <= max_y {
                    canvas.set(
                        x,
                        deck,
                        if (x as f32 * 0.2 + t).sin() > 0.2 {
                            '~'
                        } else {
                            '-'
                        },
                        theme.c(2).scale(0.20),
                    );
                }
                for k in 1..=2 {
                    let y = deck + k;
                    if y <= max_y && (x + y) % 3 == 0 {
                        canvas.add_light(x, y, theme.c(2), 0.16, ':');
                    }
                }
            }
        }

        // Earth limb in the orbital phase.
        if p > 9.0 {
            let k = 0.0017;
            for x in 0..self.w as isize {
                let dx = (x as f32 - cx as f32) * 0.5;
                let y = h - 3.0 + dx * dx * k;
                let yi = y.round() as isize;
                if yi >= 1 && yi <= max_y {
                    canvas.set(x, yi, '#', theme.c(2).scale(0.45));
                    canvas.add_light(x, (yi - 1).max(1), theme.c(3), 0.18, '.');
                    canvas.add_light(x, (yi - 2).max(1), theme.c(3), 0.10, '.');
                }
                for yy in (yi + 1).max(1)..=max_y {
                    canvas.set(x, yy, ':', theme.c(2).scale(0.18));
                }
            }
        }

        if p >= 2.5 {
            // Rocket sprite: 5 rows, 5 columns.
            let sprite = ["  ^  ", " /#\\ ", "|#o#|", "|###|", "/   \\"];
            let sx = rx.round() as isize;
            let sy = ry.round() as isize;
            for (r, line) in sprite.iter().enumerate() {
                for (c, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let x = sx + c as isize - 2;
                    let y = sy + r as isize;
                    if y < 1 || y > max_y {
                        continue;
                    }
                    let col = match ch {
                        '^' => Rgb::hex(0xf8fafc),
                        'o' => Rgb::hex(0x93c5fd),
                        '/' | '\\' => theme.c(3),
                        _ => theme.c(2).lerp(Rgb::WHITE, 0.25),
                    };
                    canvas.set(x, y, ch, col);
                }
            }
            // A bright core in the exhaust sells the thrust.
            for k in 0..7 {
                let y = sy + 5 + k;
                if y > max_y {
                    break;
                }
                let fade = 1.0 - k as f32 / 7.0;
                canvas.add_light(sx, y, FLAME[1], 0.65 * fade, if k < 2 { '*' } else { '.' });
            }
        }

        for p in &self.particles {
            if p.y < 1.0 || p.y > max_y as f32 {
                continue;
            }
            let fade = (p.life / p.max_life.max(0.01)).clamp(0.0, 1.0);
            let x = p.x.round() as isize;
            let y = p.y.round() as isize;
            if p.smoke {
                let ch = if fade > 0.5 { '%' } else { ':' };
                canvas.set(x, y, ch, theme.c(1).scale(0.20 + 0.28 * fade));
            } else {
                let idx = ((1.0 - fade) * 3.0).round() as usize;
                let col = FLAME[idx.min(3)].scale(0.55 + 0.45 * fade);
                canvas.set(x, y, if fade > 0.55 { '*' } else { '.' }, col);
            }
        }

        if let Some(b) = &self.booster {
            let bx = b.x.round() as isize;
            let by = b.y.round() as isize;
            for (r, line) in ["|#|", "|#|", " + "].iter().enumerate() {
                for (c, ch) in line.chars().enumerate() {
                    if ch == ' ' {
                        continue;
                    }
                    let y = by + r as isize;
                    if y < 1 || y > max_y {
                        continue;
                    }
                    canvas.set(bx + c as isize - 1, y, ch, theme.c(2).scale(0.55));
                }
            }
            canvas.add_light(bx, by + 3, FLAME[3], 0.4, '.');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_draws_pad_and_rocket() {
        let mut scene = LaunchScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 3.5,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("ember").unwrap(),
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('#'), "no pad or rocket");
        assert!(text.contains('^'), "no nose cone");
        assert!(canvas.ink() > 20);
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
