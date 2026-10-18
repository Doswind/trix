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

/// Visual tilt of the orbital plane: the ellipse's major axis runs "/".
const ORBIT_TILT: f32 = 0.5;
/// Flattening of the projected disk (minor / major axis).
const ORBIT_MINOR: f32 = 0.52;
/// Push the ring just past the screen edges so it fills the whole frame.
const RING_REACH: f32 = 1.1;
/// Global speed-up for everything orbiting the hole.
const SPIN_BOOST: f32 = 1.7;
/// Terminal cells are roughly twice as tall as they are wide.
const CELL_ASPECT: f32 = 0.5;

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

struct Meteor {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
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
    meteors: Vec<Meteor>,
    next_meteor: f32,
    cx: f32,
    cy: f32,
    radius: f32,
    /// Outermost visible orbit (in cells); sized so the tilted ring reaches
    /// the top and bottom screen edges.
    outer: f32,
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
            meteors: Vec::new(),
            next_meteor: 1.0,
            cx: 0.0,
            cy: 0.0,
            radius: 4.0,
            outer: 8.0,
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
        let (st, ct) = ORBIT_TILT.sin_cos();
        let vertical = CELL_ASPECT * (st * st + ORBIT_MINOR * ORBIT_MINOR * ct * ct).sqrt();
        // Push the far edge of the tilted ring just past the top/bottom edges.
        self.outer = (self.cy * RING_REACH / vertical.max(0.05))
            .clamp(self.radius * 3.5, self.radius * 12.0);
        self.disk.clear();
        self.dust.clear();
        self.stars.clear();
        self.infall.clear();
        self.meteors.clear();
        self.next_meteor = 1.0;
        let n = (self.w * self.h * 4 / 30).clamp(230, 690);
        for _ in 0..n {
            let r = self.sample_radius(2.3);
            let d = self.make_disk(r, false);
            self.disk.push(d);
        }
        let dust_n = (self.w * self.h / 13).clamp(150, 500);
        for i in 0..dust_n {
            let r = match i % 20 {
                // Faint boundary of the disk.
                0 | 1 => self.outer * self.rng.range(0.92, 1.0),
                // A light sprinkle in the band just inside the boundary.
                2..=5 => self.outer * self.rng.range(0.70, 0.94),
                // Fill the gap between the dense core and the outer band.
                6..=9 => self.outer * self.rng.range(0.40, 0.72),
                // Inner dust keeps the usual density bias.
                _ => self.sample_radius(1.25),
            };
            let d = self.make_disk(r, true);
            self.dust.push(d);
        }
        let star_n = (self.w * self.h / 36).clamp(60, 260);
        for _ in 0..star_n {
            self.stars.push(Star {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                base: self.rng.range(0.15, 0.9),
                phase: self.rng.range(0.0, TAU),
            });
        }
        for _ in 0..12 {
            let p = self.make_infall();
            self.infall.push(p);
        }
    }

    /// Map polar coordinates on the tilted orbital plane to terminal cells.
    ///
    /// The disk is an ellipse in aspect-corrected (visual) space and its major
    /// axis is rotated so that it reads as "/" on screen.
    #[inline]
    fn orbit(&self, r: f32, a: f32) -> (f32, f32) {
        let (sa, ca) = a.sin_cos();
        let u = ca * r;
        let v = sa * r * ORBIT_MINOR;
        let (st, ct) = ORBIT_TILT.sin_cos();
        let x = u * ct + v * st;
        let y = -u * st + v * ct;
        (self.cx + x, self.cy + y * CELL_ASPECT)
    }

    #[inline]
    fn inner_orbit(&self) -> f32 {
        self.radius * 1.22
    }

    /// Radius sample biased towards the hole: `bias > 1` crowds the inner
    /// orbits, leaving the outskirts sparse.
    fn sample_radius(&mut self, bias: f32) -> f32 {
        let inner = self.inner_orbit();
        let span = (self.outer - inner).max(1.0);
        inner + span * self.rng.range(0.0, 1.0).powf(bias)
    }

    fn make_disk(&mut self, r: f32, dust: bool) -> Disk {
        // The shear itself is applied in `update` from the current radius, so
        // stars spin up as they fall in; this is just a per-star jitter.
        Disk {
            r,
            a: self.rng.range(0.0, TAU),
            speed: self.rng.range(0.55, 1.35),
            size: self.rng.below(if dust { 12 } else { 4 }) as u8,
            hue: self.rng.below(6) as u8,
        }
    }

    fn make_infall(&mut self) -> Infall {
        let far = (self.outer / self.radius).max(4.5);
        Infall {
            r: self.radius * self.rng.range(4.0, far),
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

    fn spawn_meteor(&mut self) {
        let edge = self.rng.below(4);
        let (x, y) = match edge {
            0 => (self.rng.range(0.0, self.w as f32), -2.0),
            1 => (self.w as f32 + 2.0, self.rng.range(0.0, self.h as f32)),
            2 => (self.rng.range(0.0, self.w as f32), self.h as f32 + 2.0),
            _ => (-2.0, self.rng.range(0.0, self.h as f32)),
        };
        let target_x = self.rng.range(0.1, 0.9) * self.w as f32;
        let target_y = self.rng.range(0.1, 0.8) * self.h as f32;
        let dx = target_x - x;
        let dy = target_y - y;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let speed = self.rng.range(18.0, 42.0);
        let color = match self.rng.below(3) {
            0 => Rgb::hex(0xbfd7ff),
            1 => Rgb::hex(0xfff3c4),
            _ => Rgb::hex(0x9be7ff),
        };
        self.meteors.push(Meteor {
            x,
            y,
            vx: dx / len * speed,
            vy: dy / len * speed,
            life: self.rng.range(0.8, 1.8),
            max_life: 1.8,
            color,
        });
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
            // Every ring now uses only the simulated angle.  The old extra
            // `t * ...` term bypassed `spin` for the disk (and, earlier, dust).
            let a = p.a;
            let r = p.r;
            let (x, y) = self.orbit(r, a);
            let col = self.disk_color(r, p.hue);
            let ch = if dust {
                match p.size {
                    0..=8 => '.',
                    9 | 10 => ':',
                    _ => '+',
                }
            } else {
                match p.size {
                    0 => '.',
                    1 => '+',
                    2 => '*',
                    _ => '#',
                }
            };
            let radial = ((r - self.radius) / (self.outer - self.radius).max(1.0)).clamp(0.0, 1.0);
            let bright = if dust { 0.28 } else { 0.72 } * (1.0 - 0.45 * radial);
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

    fn draw_ring(&self, canvas: &mut Canvas, theme: &Theme, t: f32, spin: f32) {
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
            let a = k as f32 / 4.0 * TAU + t * 0.05 * spin * 0.18 * SPIN_BOOST;
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
            let a = k as f32 / 140.0 * TAU + t * 0.15 * spin * 0.18;
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
            let rr = r * (2.18 + band as f32 * 0.82);
            let n = 90 + band * 20;
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU
                    + t * (0.35 - band as f32 * 0.07) * spin * 0.7 * SPIN_BOOST;
                let (px, py) = self.orbit(rr, a);
                let x = px.round() as isize;
                let y = py.round() as isize;
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
        // Low-density dots mark the outer edge.  No line glyphs here: they
        // would read as a second, artificial ring around the disk.
        let n = ((self.outer * 4.0) as usize).clamp(140, 360);
        for k in 0..n {
            if k % 2 == 0 {
                continue;
            }
            let a = k as f32 / n as f32 * TAU + t * 0.12 * spin * SPIN_BOOST;
            let rr = self.outer * (0.94 + 0.05 * ((k as f32 * 0.7).sin()));
            let (px, py) = self.orbit(rr, a);
            let twinkle = 0.20 + 0.13 * (t * 1.1 + k as f32 * 0.9).sin().abs();
            let glyph = if k % 11 == 0 {
                '+'
            } else if k % 3 == 0 {
                ':'
            } else {
                '.'
            };
            canvas.set(
                px.round() as isize,
                py.round() as isize,
                glyph,
                Rgb::hex(0xbfd7ff).scale(twinkle),
            );
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
        let spin = f.tune.spin.clamp(0.05, 4.0);
        if f.t as f32 > self.next_meteor {
            if self.meteors.len() < f.tune.meteors as usize {
                self.spawn_meteor();
            }
            self.next_meteor = f.t as f32 + self.rng.range(0.7, 2.2);
        }
        for m in self.meteors.iter_mut() {
            m.x += m.vx * dt;
            m.y += m.vy * dt;
            m.life -= dt;
        }
        self.meteors.retain(|m| {
            m.life > 0.0
                && m.x > -12.0
                && m.x < self.w as f32 + 12.0
                && m.y > -12.0
                && m.y < self.h as f32 + 12.0
        });
        let mut respawn_disk = Vec::new();
        for (i, p) in self.disk.iter_mut().enumerate() {
            p.r -= dt * (0.25 + 0.9 * self.radius / p.r.max(0.3));
            // Mild Keplerian shear: the whole disk turns together while the
            // inner radii stay noticeably quicker than the outskirts.
            let shear = (self.radius / p.r.max(self.radius)).powf(0.8);
            p.a += dt * p.speed * 1.75 * SPIN_BOOST * shear * 0.75 * spin;
            if p.r <= self.radius * 0.95 {
                self.consumed += 1;
                self.flash = 1.0;
                respawn_disk.push(i);
            }
        }
        for i in respawn_disk {
            let nr = self.sample_radius(2.0);
            let p = self.make_disk(nr, false);
            self.disk[i] = p;
        }
        for p in self.dust.iter_mut() {
            // Outer dust turns with the same direction and a gentler gradient,
            // so the nebula reads as one rotating ellipse instead of a still
            // halo around a fast inner swirl.
            let shear = (self.radius / p.r.max(self.radius)).powf(0.55);
            p.a += dt * p.speed * 0.6 * SPIN_BOOST * shear * 0.62 * spin;
        }
        let mut respawn_infall = Vec::new();
        for (i, p) in self.infall.iter_mut().enumerate() {
            p.life -= dt;
            p.r -= dt * (1.0 + 2.0 * self.radius / p.r.max(0.3));
            // Falling stars whip around faster as they approach the horizon.
            let shear = (self.radius * 2.0 / p.r.max(self.radius)).powf(1.5);
            p.a += dt * p.speed * spin * 0.25 * SPIN_BOOST * shear;
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
                Rgb::hex(0xbfd7ff).scale(s.base * tw * 0.4),
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
        self.draw_ring(canvas, theme, t, f.tune.spin.clamp(0.05, 4.0));

        // Infalling stars, with a bright trail pointing at the horizon.
        for p in &self.infall {
            let (x, y) = self.orbit(p.r, p.a);
            for i in 0..5 {
                let rr = p.r + i as f32 * 0.45;
                let (xx, yy) = self.orbit(rr, p.a);
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

        // random-direction meteors across the sky
        for m in &self.meteors {
            let fade = (m.life / m.max_life).clamp(0.0, 1.0);
            let head_x = m.x.round() as isize;
            let head_y = m.y.round() as isize;
            canvas.set(head_x, head_y, '*', m.color.lerp(Rgb::hex(0xffffff), 0.6));
            canvas.add_light(head_x, head_y, m.color, 0.7, '.');
            for i in 1..=10 {
                let x = (m.x - m.vx * (i as f32 * 0.035)).round() as isize;
                let y = (m.y - m.vy * (i as f32 * 0.035)).round() as isize;
                canvas.set(
                    x,
                    y,
                    if i < 3 { '=' } else { '.' },
                    m.color.scale(fade * (1.0 - i as f32 / 12.0)),
                );
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

    #[test]
    fn orbit_is_tilted_slash_and_reaches_edges() {
        let mut s = BlackHoleScene::new(8);
        s.resize(120, 36);
        // The near end of the major axis sits to the right of and above centre.
        let (x, y) = s.orbit(s.outer, 0.0);
        assert!(x > s.cx);
        assert!(y < s.cy);
        // The far edge is sized so the ring just touches the top/bottom edges.
        let (st, ct) = ORBIT_TILT.sin_cos();
        let vertical = CELL_ASPECT * (st * st + ORBIT_MINOR * ORBIT_MINOR * ct * ct).sqrt();
        assert!((s.cy - s.outer * vertical).abs() < s.cy * 0.2);
    }

    #[test]
    fn star_density_favours_the_inner_orbits() {
        let mut s = BlackHoleScene::new(8);
        s.resize(120, 36);
        let inner = s.inner_orbit();
        let midpoint = inner + (s.outer - inner) * 0.5;
        let inside = s.disk.iter().filter(|p| p.r < midpoint).count();
        assert!(
            inside * 2 > s.disk.len(),
            "expected more than half the disk inside the midpoint"
        );
    }
}
