//! 太阳系 — the Sun and eight planets laid out on a single orbital line.
//!
//! A starfield, an asteroid belt and a passing comet fill the rest of the
//! screen; the planets themselves are ASCII spheres with distinct palettes,
//! labels and (for Saturn) rings.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

struct Planet {
    name: &'static str,
    abbr: &'static str,
    x: f32,
    r: f32,
    color: Rgb,
    accent: Rgb,
    kind: u8,
}

struct Asteroid {
    x: f32,
    y: f32,
    ch: u8,
    bright: f32,
}

pub struct SolarScene {
    rng: Rng,
    w: usize,
    h: usize,
    planets: Vec<Planet>,
    asteroids: Vec<Asteroid>,
    stars: Vec<(f32, f32, f32, f32)>,
    sun_x: f32,
    sun_y: f32,
    sun_r: f32,
    baseline: f32,
    label_y: f32,
}

const NAMES: [(&str, &str, f32, u32, u32, u8); 8] = [
    ("MERCURY", "MER", 1.0, 0xa8a29e, 0xe7e5e4, 0),
    ("VENUS", "VEN", 1.5, 0xf5c542, 0xfff1b8, 0),
    ("EARTH", "EAR", 1.7, 0x38bdf8, 0x22c55e, 1),
    ("MARS", "MAR", 1.3, 0xef4444, 0xfca5a5, 0),
    ("JUPITER", "JUP", 3.4, 0xf59e0b, 0xfff1c1, 2),
    ("SATURN", "SAT", 3.0, 0xfde68a, 0xd6b25e, 3),
    ("URANUS", "URA", 2.2, 0x67e8f9, 0xcffafe, 0),
    ("NEPTUNE", "NEP", 2.1, 0x3b82f6, 0x93c5fd, 0),
];

impl SolarScene {
    pub fn new(seed: u64) -> Self {
        SolarScene {
            rng: Rng::new(seed ^ 0x50a4_5e11),
            w: 0,
            h: 0,
            planets: Vec::new(),
            asteroids: Vec::new(),
            stars: Vec::new(),
            sun_x: 0.0,
            sun_y: 0.0,
            sun_r: 0.0,
            baseline: 0.0,
            label_y: 0.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.planets.clear();
        self.asteroids.clear();
        self.stars.clear();
        let unit = ((self.w as f32 / 120.0).min(self.h as f32 / 34.0)).clamp(0.50, 1.25);
        self.sun_r = (h as f32 * 0.105).clamp(2.2, 4.6);
        // The light source is fixed in the upper-left; the planets stay on
        // the lower orbital line, so the fan is clipped before it crosses it.
        self.sun_x = (w as f32 * 0.055).max(self.sun_r + 2.0);
        let max_y = (h as f32 * 0.30).max(3.0);
        let min_y = (self.sun_r + 1.0).min(max_y);
        self.sun_y = (h as f32 * 0.17).clamp(min_y, max_y);
        self.baseline = (h as f32 * 0.62).clamp(2.0, (h as f32 - 2.0).max(2.0));
        let start = self.sun_x + self.sun_r + 4.5;
        let end = w as f32 - 7.0;
        let span = (end - start).max(8.0);
        for (i, (name, abbr, r, col, accent, kind)) in NAMES.iter().enumerate() {
            let x = start + span * i as f32 / 7.0;
            self.planets.push(Planet {
                name,
                abbr,
                x,
                r: r * unit,
                color: Rgb::hex(*col),
                accent: Rgb::hex(*accent),
                kind: *kind,
            });
        }
        self.label_y = (self.baseline + 5.0 * unit + 1.0)
            .min(h as f32 - 3.0)
            .max(1.0);
        // asteroid belt between Mars and Jupiter
        let mars = self.planets[3].x;
        let jup = self.planets[4].x;
        let n = ((jup - mars).abs() * 7.0) as usize;
        for _ in 0..n {
            self.asteroids.push(Asteroid {
                x: self.rng.range(mars + 1.0, jup - 1.0),
                y: self.baseline + self.rng.range(-2.2, 2.2),
                ch: [b'.', b'\'', b',', b'`'][self.rng.below(4)],
                bright: self.rng.range(0.2, 0.8),
            });
        }
        let stars = (self.w * self.h / 28).clamp(120, 700);
        for _ in 0..stars {
            self.stars.push((
                self.rng.range(0.0, self.w as f32),
                self.rng.range(0.0, self.h as f32),
                self.rng.range(0.0, TAU),
                self.rng.range(0.15, 0.7),
            ));
        }
    }

    fn draw_sphere(&self, canvas: &mut Canvas, p: &Planet, t: f32) {
        let cx = p.x.round() as isize;
        let cy = self.baseline.round() as isize;
        let r = p.r.max(0.7);
        let ry = (r * 0.78).max(0.55);
        let ri = r.ceil() as isize;
        let ryi = ry.ceil() as isize;
        // Saturn rings behind/around the disc.
        if p.kind == 3 {
            let rr = r * 1.9;
            for dx in -rr.round() as isize..=rr.round() as isize {
                let d = dx as f32 / rr;
                let yy = cy + (d * 2.2).sin().round() as isize;
                let col = p
                    .accent
                    .scale(0.55 + 0.25 * (dx as f32 * 0.2 + t).sin().abs());
                canvas.set(cx + dx, yy - 1, '=', col);
                if dx.abs() as f32 > r * 0.5 {
                    canvas.set(cx + dx, yy, '-', col.scale(0.6));
                }
            }
        }
        for dy in -ryi..=ryi {
            for dx in -ri..=ri {
                let nx = dx as f32 / r;
                let ny = dy as f32 / ry;
                let d2 = nx * nx + ny * ny;
                if d2 > 1.0 {
                    continue;
                }
                let val = (1.0 - d2).sqrt();
                let light = (0.58 + 0.42 * (-(nx * 0.65 + ny * 0.55))).clamp(0.1, 1.0);
                let mut col = p.color.scale(0.55 + 0.75 * light);
                let mut ch = if val > 0.75 {
                    '@'
                } else if val > 0.5 {
                    '%'
                } else if val > 0.28 {
                    '*'
                } else if val > 0.12 {
                    '+'
                } else {
                    '.'
                };
                if p.kind == 2 {
                    // gas-giant bands
                    let band = (ny * 8.0 + (nx * 2.0 + t * 0.2).sin() * 0.8).sin();
                    col = col.lerp(
                        if band > 0.0 {
                            p.accent
                        } else {
                            p.color.scale(0.55)
                        },
                        0.35,
                    );
                    if band.abs() > 0.82 && val > 0.3 {
                        ch = '=';
                    }
                } else if p.kind == 1 {
                    // Earth continents
                    let h = hash2(dx as i64 + 31, dy as i64 + 17, 0xea47);
                    if h % 9 < 2 && val > 0.25 {
                        col = p.accent.scale(0.75 + 0.25 * light);
                        ch = '#';
                    }
                }
                if p.kind == 0 && val < 0.28 {
                    ch = '.';
                }
                canvas.set(cx + dx, cy + dy, ch, col);
            }
        }
        if p.kind == 3 {
            // front half of the rings
            let rr = r * 1.55;
            for dx in -rr.round() as isize..=rr.round() as isize {
                let d = dx as f32 / rr;
                if d.abs() > 0.5 {
                    continue;
                }
                let yy = cy + 1 + (d * 2.0).sin().round() as isize;
                canvas.set(cx + dx, yy, '=', p.accent.scale(0.8));
            }
        }
        // label
        let label = if self.w >= 104 { p.name } else { p.abbr };
        let lx = cx - label.len() as isize / 2;
        let col = theme_label(p.color);
        if lx >= 0 && lx + label.len() as isize <= self.w as isize {
            canvas.put_str(lx, self.label_y.round() as isize, label, col);
        }
    }

    fn draw_sun(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        let cx = self.sun_x.round() as isize;
        let cy = self.sun_y.round() as isize;
        let r = self.sun_r;
        let ry = (r * 0.78).max(0.6);
        let ri = r.ceil() as isize;
        let ryi = ry.ceil() as isize;
        for dy in -ryi..=ryi {
            for dx in -ri..=ri {
                let nx = dx as f32 / r;
                let ny = dy as f32 / ry;
                let d2 = nx * nx + ny * ny;
                if d2 > 1.0 {
                    continue;
                }
                let val = (1.0 - d2).sqrt();
                let col = Rgb::hex(0xfbbf24)
                    .lerp(Rgb::hex(0xfffbeb), val * 0.8)
                    .scale(0.9 + 0.1 * (t * 3.0 + dx as f32).sin().abs());
                let ch = if val > 0.72 {
                    '@'
                } else if val > 0.4 {
                    '%'
                } else {
                    '*'
                };
                canvas.set(cx + dx, cy + dy, ch, col);
                if val > 0.25 {
                    canvas.add_light(cx + dx, cy + dy, Rgb::hex(0xfbbf24), 0.12, '.');
                }
            }
        }
        // rotating rays
        for k in 0..12 {
            let a = k as f32 / 12.0 * TAU + t * 0.25;
            let len = r + 2.5 + 1.6 * (t * 1.6 + k as f32).sin().abs();
            for s in (r.round() as isize + 1)..=(len.round() as isize) {
                let x = cx + (a.cos() * s as f32).round() as isize;
                let y = cy + (a.sin() * s as f32 * 0.48).round() as isize;
                if y < self.baseline as isize {
                    canvas.set(
                        x,
                        y,
                        if s % 2 == 0 { '-' } else { '.' },
                        theme.c(3).scale(0.55),
                    );
                }
            }
        }
        canvas.put_str(
            cx - 2,
            (cy as f32 + ry + 2.0) as isize,
            "SOL",
            theme.c(3).scale(0.9),
        );
    }
    fn draw_sunburst(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        let ox = self.sun_x;
        let oy = self.sun_y;
        let max_len = (self.w as f32).max(self.h as f32) * 1.20;
        let rays = 30usize;
        let fan = std::f32::consts::FRAC_PI_2 + 0.18;
        // A dim permanent fan from the upper-left sun, directed right/down.
        for k in 0..rays {
            let a = -0.04 + (k as f32 + 0.5) / rays as f32 * fan;
            let x = ox + a.cos() * max_len;
            let y = oy + a.sin() * max_len * 0.58;
            canvas.line(
                ox as isize,
                oy as isize,
                x as isize,
                (y as isize).min(self.baseline as isize - 1),
                if k % 3 == 0 { '-' } else { '.' },
                Rgb::hex(0xfbbf24).scale(0.16),
            );
        }

        // A bright wavefront travels outward from the sun, lighting each ray
        // from the inside out.  This is the same "scanning bar" idea as the
        // world-map data sweep: the wave front is the bright layer.
        let phase = (t * 0.19).rem_euclid(1.0);
        let front = phase * max_len;
        let band = (max_len * 0.10).max(2.5);
        for k in 0..rays {
            let a = -0.04 + (k as f32 + 0.5) / rays as f32 * fan;
            let ca = a.cos();
            let sa = a.sin();
            let mut s = 2.0f32;
            while s < max_len {
                let x = (ox + ca * s).round() as isize;
                let y = (oy + sa * s * 0.58).round() as isize;
                if x >= 0 && y >= 0 && x < self.w as isize && y < self.baseline as isize {
                    let behind = front - s;
                    if behind >= 0.0 {
                        let trail = (1.0 - behind / (front + 1.0)).clamp(0.0, 1.0);
                        let heat = if behind < band {
                            1.0
                        } else {
                            (0.12 + trail * 0.42).clamp(0.0, 0.65)
                        };
                        let col = Rgb::hex(0xfbbf24)
                            .lerp(Rgb::hex(0xfff7d6), heat * 0.85)
                            .scale(heat);
                        let ch = if behind < band {
                            if s as isize % 4 == 0 {
                                '*'
                            } else {
                                '-'
                            }
                        } else if s as isize % 7 == 0 {
                            '*'
                        } else {
                            '.'
                        };
                        canvas.set(x, y, ch, col);
                    }
                }
                s += 1.0;
            }
        }

        // The wave front itself: a few concentric bright arcs so the
        // propagation reads as distinct layers, not a single random streak.
        let arc_n = 72;
        for (lag, strength) in [(0.0, 1.0f32), (0.10, 0.55), (0.20, 0.28)] {
            let radius = (front - max_len * lag).max(0.0);
            for k in 0..arc_n {
                let a = -0.04 + (k as f32 + 0.5) / arc_n as f32 * fan;
                let x = (ox + a.cos() * radius).round() as isize;
                let y = (oy + a.sin() * radius * 0.58).round() as isize;
                if x >= 0 && y >= 0 && x < self.w as isize && y < self.baseline as isize {
                    let col = Rgb::hex(0xfff7d6).scale((0.45 + 0.45 * (1.0 - phase)) * strength);
                    canvas.set(x, y, if k % 5 == 0 { '*' } else { '.' }, col);
                    if lag == 0.0 && k % 3 == 0 {
                        canvas.add_light(x, y, Rgb::hex(0xfbbf24), 0.30, '.');
                    }
                }
            }
        }

        // Hot core glow at the sun.
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (2, 0), (0, 2), (-1, 0), (0, -1)] {
            canvas.add_light(
                ox as isize + dx,
                oy as isize + dy,
                Rgb::hex(0xffd166),
                0.55,
                '.',
            );
        }
        let _ = theme;
    }
}

fn theme_label(c: Rgb) -> Rgb {
    c.lerp(Rgb::hex(0xffffff), 0.25).scale(0.85)
}

impl Scene for SolarScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        // stars
        for (x, y, phase, bright) in &self.stars {
            let tw = 0.45 + 0.55 * (t * 1.7 + phase).sin().abs();
            canvas.set(
                *x as isize,
                *y as isize,
                if *bright > 0.55 { '+' } else { '.' },
                theme.c(2).scale(bright * tw * 0.42),
            );
        }
        // holographic star-chart grid
        let y = self.baseline.round() as isize;
        let grid = theme.c(1).scale(0.12);
        for gx in (6..self.w as isize - 2).step_by(12) {
            for gy in (2..y - 1).step_by(2) {
                canvas.set(gx, gy, '.', grid);
            }
            canvas.set(gx, 1, '+', theme.c(2).scale(0.35));
        }
        for gy in (3..y - 1).step_by(4) {
            for gx in (2..self.w as isize - 2).step_by(3) {
                canvas.set(gx, gy, '.', grid);
            }
        }
        self.draw_sunburst(canvas, theme, t);
        // orbital line
        for x in (self.sun_x + self.sun_r + 2.0) as isize..(self.w as isize - 2) {
            canvas.set(x, y, '-', theme.c(1).scale(0.55));
        }
        for p in &self.planets {
            let x = p.x.round() as isize;
            canvas.set(x, y, '+', theme.c(2).scale(0.7));
            if self.w > 100 {
                for d in 1..=(p.r.round() as isize).max(1) {
                    canvas.set(x, y - d, '\'', theme.c(1).scale(0.25));
                }
            }
        }
        // asteroid belt
        for a in &self.asteroids {
            canvas.set(
                a.x as isize,
                a.y as isize,
                a.ch as char,
                Rgb::hex(0x9ca3af).scale(a.bright * 0.55),
            );
        }
        // comet
        let cx = ((t * 7.0) % (self.w as f32 + 24.0)) - 12.0;
        let cy = self.h as f32 * 0.20 + (t * 0.7).sin() * 3.0;
        for i in 0..12 {
            let x = cx - i as f32 * 0.8;
            let yy = cy - i as f32 * 0.25;
            canvas.set(
                x as isize,
                yy as isize,
                if i < 2 { '*' } else { '.' },
                Rgb::hex(0x67e8f9).scale(1.0 - i as f32 / 13.0),
            );
        }
        // planets / sun
        self.draw_sun(canvas, theme, t);
        for p in &self.planets {
            self.draw_sphere(canvas, p, t);
        }
        // a few moons / ring sparks
        let earth = &self.planets[2];
        for (i, (dx, dy)) in [(-3.0f32, -2.0f32), (3.0, -1.0), (2.0, 2.0)]
            .iter()
            .enumerate()
        {
            let mx = earth.x + dx;
            let my = self.baseline + dy + (t * 1.3 + i as f32).sin();
            canvas.set(
                mx as isize,
                my as isize,
                if i == 0 { 'o' } else { '.' },
                Rgb::hex(0xbfdbfe),
            );
        }
        let jup = &self.planets[4];
        for i in 0..4 {
            let a = t * 0.8 + i as f32 * std::f32::consts::TAU / 4.0;
            let mx = jup.x + a.cos() * (jup.r * 2.1);
            let my = self.baseline + a.sin() * (jup.r * 0.9);
            canvas.set(mx as isize, my as isize, 'o', Rgb::hex(0xfde68a).scale(0.7));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nine_planets_on_a_line() {
        let mut s = SolarScene::new(5);
        s.resize(120, 36);
        assert_eq!(s.planets.len(), 8);
        for p in &s.planets {
            assert!((p.x - p.x.round()).abs() < 1.0);
        }
    }
}
