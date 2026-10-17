//! 黑洞 — a black hole endlessly feeding on stars.
//!
//! A dark event horizon with a hot photon ring sits at the centre; an
//! accretion disk, spiralling dust and infalling stars rotate around it while
//! relativistic jets and a lensed starfield fill the rest of the screen.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

struct Disk {
    r: f32,
    a: f32,
    speed: f32,
    size: u8,
    hue: u8,
}

struct Star {
    x: f32,
    y: f32,
    base: f32,
    phase: f32,
}

struct Infall {
    r: f32,
    a: f32,
    speed: f32,
    life: f32,
    color: Rgb,
}

pub struct BlackHoleScene {
    rng: Rng,
    w: usize,
    h: usize,
    disk: Vec<Disk>,
    dust: Vec<Disk>,
    stars: Vec<Star>,
    infall: Vec<Infall>,
    cx: f32,
    cy: f32,
    radius: f32,
    consumed: u32,
    flash: f32,
}

impl BlackHoleScene {
    pub fn new(seed: u64) -> Self {
        BlackHoleScene {
            rng: Rng::new(seed ^ 0xb1ac_4011),
            w: 0,
            h: 0,
            disk: Vec::new(),
            dust: Vec::new(),
            stars: Vec::new(),
            infall: Vec::new(),
            cx: 0.0,
            cy: 0.0,
            radius: 4.0,
            consumed: 0,
            flash: 0.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.cx = self.w as f32 * 0.5;
        self.cy = self.h as f32 * 0.52;
        self.radius = (self.h as f32 * 0.22).clamp(3.0, 8.5);
        self.disk.clear();
        self.dust.clear();
        self.stars.clear();
        self.infall.clear();
        let n = (self.w * self.h / 15).clamp(120, 380);
        for _ in 0..n {
            let r = self.radius * self.rng.range(1.15, 4.6);
            let d = self.make_disk(r);
            self.disk.push(d);
        }
        let dust_n = (self.w * self.h / 28).clamp(60, 220);
        for _ in 0..dust_n {
            let r = self.radius * self.rng.range(1.1, 6.0);
            let d = self.make_disk(r);
            self.dust.push(d);
        }
        let star_n = (self.w * self.h / 26).clamp(80, 340);
        for _ in 0..star_n {
            self.stars.push(Star {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                base: self.rng.range(0.15, 0.9),
                phase: self.rng.range(0.0, TAU),
            });
        }
        for _ in 0..9 {
            let p = self.make_infall();
            self.infall.push(p);
        }
    }

    fn make_disk(&mut self, r: f32) -> Disk {
        Disk {
            r,
            a: self.rng.range(0.0, TAU),
            speed: self.rng.range(0.5, 2.0),
            size: self.rng.below(4) as u8,
            hue: self.rng.below(6) as u8,
        }
    }

    fn make_infall(&mut self) -> Infall {
        Infall {
            r: self.radius * self.rng.range(4.0, 7.0),
            a: self.rng.range(0.0, TAU),
            speed: self.rng.range(0.4, 1.2),
            life: self.rng.range(2.5, 5.0),
            color: if self.rng.chance(0.5) {
                Rgb::hex(0x9be7ff)
            } else {
                Rgb::hex(0xffe3a3)
            },
        }
    }

    fn disk_color(&self, r: f32, hue: u8) -> Rgb {
        let t = ((r - self.radius) / (self.radius * 3.5)).clamp(0.0, 1.0);
        let hot = Rgb::hex(0xfff7d6);
        let gold = Rgb::hex(0xfbbf24);
        let orange = Rgb::hex(0xfb923c);
        let red = Rgb::hex(0xef4444);
        let magenta = Rgb::hex(0xd946ef);
        let blue = Rgb::hex(0x38bdf8);
        let c = match hue % 6 {
            0 => hot,
            1 => gold,
            2 => orange,
            3 => red,
            4 => magenta,
            _ => blue,
        };
        hot.lerp(c, 0.35 + t * 0.65).lerp(red, t * 0.25)
    }

    fn draw_disk(&self, canvas: &mut Canvas, theme: &Theme, t: f32, dust: bool) {
        let list = if dust { &self.dust } else { &self.disk };
        for p in list {
            let a = p.a + t * p.speed * (self.radius / p.r.max(0.1)).powf(0.9);
            let r = p.r;
            let x = self.cx + a.cos() * r;
            let y = self.cy + a.sin() * r * 0.53;
            let col = self.disk_color(r, p.hue);
            let ch = if dust {
                if p.size == 0 {
                    '.'
                } else {
                    ':'
                }
            } else {
                match p.size {
                    0 => '.',
                    1 => '+',
                    2 => '*',
                    _ => '#',
                }
            };
            let bright = if dust { 0.28 } else { 0.72 };
            canvas.set(
                x as isize,
                y as isize,
                ch,
                col.scale(bright + 0.25 * (t * 2.0 + p.a).sin().abs()),
            );
            if !dust && p.size >= 3 && r < self.radius * 2.2 {
                canvas.add_light(x as isize, y as isize, col, 0.4, '.');
            }
        }
        let _ = theme;
    }

    fn draw_ring(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        let cx = self.cx;
        let cy = self.cy;
        let r = self.radius;
        // bright photon ring (elliptical to compensate terminal cell aspect)
        let mut prev: Option<(isize, isize)> = None;
        for k in 0..=220 {
            let a = k as f32 / 220.0 * TAU;
            let rr = r * (1.0 + 0.02 * (k as f32 * 0.7 + t * 3.0).sin());
            let x = (cx + a.cos() * rr).round() as isize;
            let y = (cy + a.sin() * rr * 0.5).round() as isize;
            if let Some((px, py)) = prev {
                canvas.line(px, py, x, y, '@', Rgb::hex(0xfff7d6));
            }
            prev = Some((x, y));
            if k % 8 == 0 {
                canvas.add_light(x, y, Rgb::hex(0xfbbf24), 0.6, '.');
            }
        }
        // four bright polar flares
        for k in 0..4 {
            let a = k as f32 / 4.0 * TAU + t * 0.05;
            let sx = (cx + a.cos() * r).round() as isize;
            let sy = (cy + a.sin() * r * 0.5).round() as isize;
            for d in 1..=4 {
                let x = (cx + a.cos() * (r + d as f32)).round() as isize;
                let y = (cy + a.sin() * (r + d as f32) * 0.5).round() as isize;
                canvas.set(
                    x,
                    y,
                    if d == 1 { '*' } else { '.' },
                    Rgb::hex(0xffffff).scale(1.0 - d as f32 / 6.0),
                );
                canvas.add_light(x, y, Rgb::hex(0xfbbf24), 0.4, '.');
            }
            let _ = (sx, sy);
        }
        // outer lensed ring
        for k in 0..140 {
            let a = k as f32 / 140.0 * TAU + t * 0.15;
            let rr = r * 1.12;
            let x = (cx + a.cos() * rr).round() as isize;
            let y = (cy + a.sin() * rr * 0.55).round() as isize;
            canvas.set(
                x,
                y,
                if k % 4 == 0 { '+' } else { '.' },
                Rgb::hex(0x67e8f9).scale(0.35),
            );
        }
        // accretion bands: three tilted rings of hot matter
        for band in 0..3 {
            let rr = r * (1.45 + band as f32 * 0.55);
            let n = 90 + band * 20;
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU + t * (0.35 - band as f32 * 0.07);
                let x = (cx + a.cos() * rr).round() as isize;
                let y = (cy + a.sin() * rr * 0.42).round() as isize;
                if (k + band) % 3 == 0 {
                    continue;
                }
                let col = match band {
                    0 => Rgb::hex(0xfbbf24),
                    1 => Rgb::hex(0xfb923c),
                    _ => Rgb::hex(0x9be7ff),
                };
                canvas.set(
                    x,
                    y,
                    if band == 0 { '=' } else { '-' },
                    col.scale(0.35 + 0.25 * (k as f32 * 0.4 + t).sin().abs()),
                );
            }
        }
        // gravitational arcs above and below
        for sign in [-1.0f32, 1.0] {
            for k in 0..50 {
                let a = k as f32 / 50.0 * std::f32::consts::PI;
                let x = (cx + a.cos() * r * 1.45).round() as isize;
                let y = (cy + sign * (r * 0.55 + a.sin() * r * 0.28)).round() as isize;
                canvas.set(x, y, '.', Rgb::hex(0x9be7ff).scale(0.45));
            }
        }
        let _ = theme;
    }

    fn draw_jets(&self, canvas: &mut Canvas, t: f32) {
        for dir in [-1.0f32, 1.0] {
            for i in 1..(self.h as isize / 2) {
                let yy = self.cy + dir * (self.radius * 1.2 + i as f32 * 1.8);
                if yy < 1.0 || yy >= self.h as f32 - 1.0 {
                    break;
                }
                if (i * 3 + (t * 12.0) as isize) % 7 != 0 {
                    continue;
                }
                let x = self.cx + ((i as f32 * 0.7 + t * 2.0).sin() * 2.2).round();
                let col = if dir < 0.0 {
                    Rgb::hex(0x67e8f9)
                } else {
                    Rgb::hex(0xd946ef)
                };
                canvas.set(
                    x as isize,
                    yy as isize,
                    if i % 3 == 0 { '|' } else { '.' },
                    col.scale(0.55 * (1.0 - i as f32 / self.h as f32)),
                );
            }
        }
    }
}

impl Scene for BlackHoleScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let t = f.t as f32;
        let mut respawn_disk = Vec::new();
        for (i, p) in self.disk.iter_mut().enumerate() {
            p.r -= dt * (0.25 + 0.9 * self.radius / p.r.max(0.3));
            p.a += dt * p.speed * 3.0;
            if p.r <= self.radius * 0.95 {
                self.consumed += 1;
                self.flash = 1.0;
                respawn_disk.push(i);
            }
        }
        for i in respawn_disk {
            let nr = self.radius * self.rng.range(2.5, 5.0);
            let p = self.make_disk(nr);
            self.disk[i] = p;
        }
        for p in self.dust.iter_mut() {
            p.a += dt * p.speed * 0.7;
        }
        let mut respawn_infall = Vec::new();
        for (i, p) in self.infall.iter_mut().enumerate() {
            p.life -= dt;
            p.r -= dt * (1.0 + 2.0 * self.radius / p.r.max(0.3));
            p.a += dt * p.speed;
            if p.r <= self.radius || p.life <= 0.0 {
                self.consumed += 1;
                self.flash = 1.0;
                respawn_infall.push(i);
            }
        }
        for i in respawn_infall {
            let p = self.make_infall();
            self.infall[i] = p;
        }
        self.flash = (self.flash - dt * 3.0).max(0.0);
        let _ = t;
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        // Background stars.
        for s in &self.stars {
            let tw = 0.45 + 0.55 * (t * 1.3 + s.phase).sin().abs();
            canvas.set(
                s.x as isize,
                s.y as isize,
                if s.base > 0.7 { '+' } else { '.' },
                Rgb::hex(0xbfd7ff).scale(s.base * tw * 0.55),
            );
        }
        self.draw_jets(canvas, t);
        self.draw_disk(canvas, theme, t, true);
        self.draw_disk(canvas, theme, t, false);

        // Event horizon: erase everything inside, then draw the photon ring.
        let rx = self.radius.ceil() as isize + 1;
        let ry = (self.radius * 0.5).ceil() as isize + 1;
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                let nx = dx as f32 / self.radius.max(0.1);
                let ny = dy as f32 / (self.radius * 0.5).max(0.1);
                if nx * nx + ny * ny <= 1.0 {
                    canvas.set(
                        self.cx as isize + dx,
                        self.cy as isize + dy,
                        ' ',
                        Rgb::BLACK,
                    );
                }
            }
        }
        self.draw_ring(canvas, theme, t);

        // Infalling stars, with a bright trail pointing at the horizon.
        for p in &self.infall {
            let x = self.cx + p.a.cos() * p.r;
            let y = self.cy + p.a.sin() * p.r * 0.7;
            for i in 0..5 {
                let rr = p.r + i as f32 * 0.45;
                let xx = self.cx + p.a.cos() * rr;
                let yy = self.cy + p.a.sin() * rr * 0.7;
                canvas.set(
                    xx as isize,
                    yy as isize,
                    if i == 0 { '*' } else { '.' },
                    p.color.scale((1.0 - i as f32 / 6.0) * p.life.min(1.0)),
                );
            }
            canvas.add_light(x as isize, y as isize, p.color, 0.5, '.');
        }

        if self.flash > 0.0 {
            for k in 0..20 {
                let a = k as f32 / 20.0 * TAU;
                let x = (self.cx + a.cos() * self.radius * 1.3) as isize;
                let y = (self.cy + a.sin() * self.radius * 1.3) as isize;
                canvas.add_light(x, y, Rgb::hex(0xffffff), self.flash * 0.35, '.');
            }
        }

        // HUD
        let title = ":: SAGITTARIUS A* // EVENT HORIZON ::";
        if self.w >= title.len() + 4 {
            canvas.put_str_gradient(2, 0, title, &[theme.c(2), theme.c(3), theme.c(4)]);
        }
        let right = format!("CONSUMED {}", self.consumed);
        if self.w >= right.len() + 10 {
            canvas.put_str(
                self.w as isize - right.len() as isize - 2,
                0,
                &right,
                theme.c(2).scale(0.8),
            );
        }
        let info = "MASS 4.3M SOL // LENSING ACTIVE";
        if self.h > 12 && self.w > info.len() + 4 {
            canvas.put_str(2, self.h as isize - 2, info, theme.c(1).scale(1.2));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disk_and_stars() {
        let mut s = BlackHoleScene::new(8);
        s.resize(100, 30);
        assert!(s.disk.len() > 100);
        assert!(s.stars.len() > 50);
        assert!(s.radius > 2.0);
    }
}
