//! 烟花 — layered fireworks bursting over a neon skyline.
//!
//! Rockets climb from the bottom, bloom into two concentric rings of sparks,
//! then crackle into a delayed second stage.  Stars, smoke and a city
//! silhouette keep the whole terminal alive between bursts.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};

const PALETTE: [Rgb; 8] = [
    Rgb::hex(0xffd166),
    Rgb::hex(0xff4d6d),
    Rgb::hex(0x22d3ee),
    Rgb::hex(0x4ade80),
    Rgb::hex(0xd946ef),
    Rgb::hex(0xf97316),
    Rgb::hex(0xffffff),
    Rgb::hex(0x93c5fd),
];

struct Rocket {
    x: f32,
    y: f32,
    vy: f32,
    color: Rgb,
    fuse: f32,
    trail: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    color: Rgb,
    ch: char,
    gravity: f32,
    delay: f32,
}

struct Burst {
    x: f32,
    y: f32,
    color: Rgb,
    age: f32,
    max_age: f32,
}

struct Building {
    x: f32,
    w: f32,
    h: f32,
    windows: Vec<(f32, f32, bool)>,
}

struct BgStar {
    x: f32,
    y: f32,
    phase: f32,
    bright: f32,
}

pub struct FireworksScene {
    rng: Rng,
    w: usize,
    h: usize,
    rockets: Vec<Rocket>,
    particles: Vec<Particle>,
    stars: Vec<BgStar>,
    buildings: Vec<Building>,
    rings: Vec<Burst>,
    next_launch: f32,
    next_flash: f32,
    burst_count: u32,
}

impl FireworksScene {
    pub fn new(seed: u64) -> Self {
        FireworksScene {
            rng: Rng::new(seed ^ 0xf17e_2a11),
            w: 0,
            h: 0,
            rockets: Vec::new(),
            particles: Vec::new(),
            stars: Vec::new(),
            buildings: Vec::new(),
            rings: Vec::new(),
            next_launch: 0.8,
            next_flash: 0.0,
            burst_count: 0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.rockets.clear();
        self.particles.clear();
        self.stars.clear();
        self.buildings.clear();
        self.rings.clear();
        let n = (self.w * self.h / 55).clamp(50, 180);
        for _ in 0..n {
            self.stars.push(BgStar {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32 * 0.75),
                phase: self.rng.range(0.0, TAU),
                bright: self.rng.range(0.15, 0.9),
            });
        }
        // Skyline silhouette along the bottom.
        let ground = self.h as f32 - 2.0;
        let mut x = 0.0;
        while x < self.w as f32 {
            let bw = self.rng.range(3.0, 8.0);
            let bh = self.rng.range(3.0, self.h as f32 * 0.22);
            let mut windows = Vec::new();
            let cols = (bw / 2.0) as usize;
            let rows = (bh / 2.0) as usize;
            for wy in 0..rows {
                for wx in 0..cols {
                    if self.rng.chance(0.35) {
                        windows.push((
                            x + 1.0 + wx as f32 * 2.0,
                            ground - bh + 1.0 + wy as f32 * 2.0,
                            self.rng.chance(0.5),
                        ));
                    }
                }
            }
            self.buildings.push(Building {
                x,
                w: bw,
                h: bh,
                windows,
            });
            x += bw + self.rng.range(0.0, 1.5);
        }
    }

    fn launch(&mut self, max_rockets: usize) {
        if self.rockets.len() >= max_rockets {
            return;
        }
        let x = self.rng.range(self.w as f32 * 0.08, self.w as f32 * 0.92);
        let color = PALETTE[self.rng.below(PALETTE.len())];
        let target_y = self.rng.range(self.h as f32 * 0.10, self.h as f32 * 0.52);
        let dy = (self.h as f32 - 3.0 - target_y).max(4.0);
        let vy = -self.rng.range(16.0, 30.0);
        let fuse = (dy / -vy).clamp(0.55, 2.4);
        self.rockets.push(Rocket {
            x,
            y: self.h as f32 - 3.0,
            vy,
            color,
            fuse,
            trail: 0.0,
        });
    }

    fn burst(&mut self, x: f32, y: f32, color: Rgb, density: f32) {
        self.burst_count += 1;
        self.next_flash = 0.16;
        self.rings.push(Burst {
            x,
            y,
            color,
            age: 0.0,
            max_age: 0.9,
        });
        let scale = density.sqrt().clamp(0.25, 2.4);
        let outer = (74.0 * scale) as usize;
        let inner = (52.0 * scale) as usize;
        let core = ((self.rng.below(10) + 28) as f32 * scale) as usize;
        let n = outer + inner + core;
        let second = PALETTE[self.rng.below(PALETTE.len())];
        for i in 0..n {
            let (a, speed, life, col, ch) = if i < outer {
                (
                    i as f32 / outer as f32 * TAU,
                    self.rng.range(9.0, 14.0),
                    self.rng.range(1.8, 2.8),
                    color,
                    '*',
                )
            } else if i < outer + inner {
                let j = (i - outer) as f32;
                (
                    j / inner as f32 * TAU,
                    self.rng.range(4.5, 7.5),
                    self.rng.range(1.2, 2.2),
                    second,
                    '+',
                )
            } else {
                (
                    self.rng.range(0.0, TAU),
                    self.rng.range(1.0, 4.0),
                    self.rng.range(1.0, 2.0),
                    if i % 2 == 0 { color } else { second },
                    '.',
                )
            };
            let col = if i % 4 == 0 {
                PALETTE[(i / 4 + self.burst_count as usize) % PALETTE.len()]
            } else {
                col
            };
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * speed,
                vy: a.sin() * speed * 0.75 - 1.0,
                life,
                max_life: life,
                color: col,
                ch,
                gravity: self.rng.range(3.0, 6.0),
                delay: 0.0,
            });
        }
        // delayed outer ring / crackle
        for _ in 0..18 {
            let a = self.rng.range(0.0, TAU);
            let speed = self.rng.range(2.0, 9.0);
            let life = self.rng.range(0.7, 1.8);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * speed,
                vy: a.sin() * speed * 0.65 - 2.0,
                life: life + 0.5,
                max_life: life + 0.5,
                color: PALETTE[self.rng.below(PALETTE.len())],
                ch: '.',
                gravity: 4.0,
                delay: self.rng.range(0.35, 0.9),
            });
        }
    }
}

impl Scene for FireworksScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let density = f.tune.density.clamp(0.05, 4.0);
        if f.t as f32 > self.next_launch {
            let max_rockets = (4.0 * density).round().clamp(1.0, 12.0) as usize;
            self.launch(max_rockets);
            self.next_launch = f.t as f32 + self.rng.range(0.55, 1.15) / density.sqrt();
        }
        self.next_flash = (self.next_flash - dt).max(0.0);
        for b in self.rings.iter_mut() {
            b.age += dt;
        }
        self.rings.retain(|b| b.age < b.max_age);
        let mut explosions = Vec::new();
        for r in self.rockets.iter_mut() {
            r.y += r.vy * dt;
            r.vy += 5.0 * dt;
            r.trail -= dt;
            r.fuse -= dt;
            if r.fuse <= 0.0 || r.vy >= -2.0 {
                explosions.push((r.x, r.y, r.color));
            }
        }
        for (x, y, c) in explosions {
            self.burst(x, y, c, density);
        }
        self.rockets.retain(|r| r.fuse > 0.0 && r.vy < -2.0);
        for p in self.particles.iter_mut() {
            if p.delay > 0.0 {
                p.delay -= dt;
                continue;
            }
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += p.gravity * dt;
            p.vx *= 0.992;
            p.vy *= 0.995;
            p.life -= dt;
        }
        self.particles.retain(|p| p.delay > 0.0 || p.life > 0.0);
        if self.particles.len() > 2400 {
            let drop = self.particles.len() - 2400;
            self.particles.drain(0..drop);
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        for s in &self.stars {
            let tw = 0.4 + 0.6 * (t * 1.5 + s.phase).sin().abs();
            canvas.set(
                s.x as isize,
                s.y as isize,
                if s.bright > 0.6 { '+' } else { '.' },
                Rgb::hex(0xdbeafe).scale(s.bright * tw * 0.5),
            );
        }
        // city silhouette
        let ground = self.h as f32 - 2.0;
        for b in &self.buildings {
            for y in (ground - b.h).round() as isize..ground as isize {
                for x in b.x.round() as isize..(b.x + b.w).round() as isize {
                    let edge = x == b.x.round() as isize || x == (b.x + b.w).round() as isize - 1;
                    canvas.set(
                        x,
                        y,
                        if edge { '|' } else { '#' },
                        Rgb::hex(0x0f172a).lerp(theme.c(2).scale(0.3), 0.25),
                    );
                }
            }
            for (wx, wy, warm) in &b.windows {
                canvas.set(
                    *wx as isize,
                    *wy as isize,
                    if *warm { '+' } else { '.' },
                    if *warm {
                        Rgb::hex(0xfbbf24).scale(0.55)
                    } else {
                        Rgb::hex(0x67e8f9).scale(0.45)
                    },
                );
            }
        }
        // rockets and their trails
        for r in &self.rockets {
            let x = r.x.round() as isize;
            let y = r.y.round() as isize;
            canvas.set(x, y, '|', r.color.lerp(Rgb::hex(0xffffff), 0.7));
            for i in 1..7 {
                canvas.set(
                    x,
                    y + i,
                    if i < 2 { '*' } else { '.' },
                    r.color.scale(1.0 - i as f32 / 8.0),
                );
            }
            canvas.add_light(x, y, r.color, 0.7, '.');
        }
        // sparks
        for p in &self.particles {
            if p.delay > 0.0 {
                continue;
            }
            let fade = (p.life / p.max_life).clamp(0.0, 1.0);
            let col = p.color.scale(0.35 + 0.8 * fade);
            if p.life > p.max_life * 0.35 {
                let tx = (p.x - p.vx * 0.12).round() as isize;
                let ty = (p.y - p.vy * 0.12).round() as isize;
                canvas.line(tx, ty, p.x as isize, p.y as isize, '.', col.scale(0.45));
            }
            canvas.set(p.x as isize, p.y as isize, p.ch, col);
            if p.life > p.max_life * 0.7 {
                canvas.add_light(p.x as isize, p.y as isize, p.color, 0.25, '.');
            }
        }
        if self.next_flash > 0.0 {
            for y in 0..self.h {
                for x in (0..self.w).step_by(11) {
                    canvas.add_light(
                        x as isize,
                        y as isize,
                        Rgb::hex(0xffffff),
                        self.next_flash * 0.015,
                        '.',
                    );
                }
            }
        }
        // expanding bloom rings
        for b in &self.rings {
            let f = (b.age / b.max_age).clamp(0.0, 1.0);
            let r = 2.0 + f * 15.0;
            let n = 64usize;
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU;
                let x = b.x + a.cos() * r;
                let y = b.y + a.sin() * r * 0.55;
                canvas.set(
                    x as isize,
                    y as isize,
                    if k % 3 == 0 { '*' } else { '+' },
                    b.color
                        .lerp(Rgb::hex(0xffffff), 0.35)
                        .scale((1.0 - f) * 1.0),
                );
            }
            if b.age < 0.10 {
                for dy in -2..=2 {
                    for dx in -3..=3 {
                        canvas.add_light(
                            b.x as isize + dx,
                            b.y as isize + dy,
                            b.color,
                            0.4 * (1.0 - b.age / 0.1),
                            '.',
                        );
                    }
                }
            }
        }
        // HUD
        let title = ":: PYRO // GLOBAL CELEBRATION ::";
        if self.w >= title.len() + 4 {
            canvas.put_str_gradient(2, 0, title, &[theme.c(2), theme.c(3), theme.c(4)]);
        }
        let right = format!("BURSTS {}", self.burst_count);
        if self.w >= right.len() + 10 {
            canvas.put_str(
                self.w as isize - right.len() as isize - 2,
                0,
                &right,
                theme.c(2).scale(0.8),
            );
        }
        let _ = hash2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launches_and_bursts() {
        let mut s = FireworksScene::new(6);
        s.resize(100, 30);
        s.launch(4);
        assert!(!s.rockets.is_empty());
        s.burst(50.0, 15.0, PALETTE[0], 1.0);
        assert!(s.particles.len() > 40);
    }
}
