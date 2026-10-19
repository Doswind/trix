//! volcano — a night eruption with lava fountains, ash and lightning.
//!
//! A jagged mountain silhouette fills the lower half; the crater throws lava
//! particles on ballistic arcs, two lava streams crawl down the slopes, ash
//! drifts across the sky and every few seconds a bolt lights the plume.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash2, value_noise, Rng};
use crate::scene::{Frame, Scene};

const LAVA: [Rgb; 5] = [
    Rgb::hex(0xfff6d8),
    Rgb::hex(0xffd166),
    Rgb::hex(0xff8a3d),
    Rgb::hex(0xd63b0a),
    Rgb::hex(0x5a1204),
];

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
}

pub struct VolcanoScene {
    w: usize,
    h: usize,
    rng: Rng,
    ridge: Vec<f32>,
    lava: Vec<Particle>,
    ash: Vec<Particle>,
    spawn: f32,
    ash_spawn: f32,
    flash: f32,
    last_flash: f32,
}

impl VolcanoScene {
    pub fn new(seed: u64) -> Self {
        VolcanoScene {
            w: 0,
            h: 0,
            rng: Rng::new(seed ^ 0x564f_4c43_414e),
            ridge: Vec::new(),
            lava: Vec::new(),
            ash: Vec::new(),
            spawn: 0.0,
            ash_spawn: 0.0,
            flash: 0.0,
            last_flash: 0.0,
        }
    }

    fn rebuild(&mut self) {
        let w = self.w;
        let h = self.h;
        if w == 0 || h == 0 {
            return;
        }
        self.ridge.clear();
        self.ridge.resize(w, h as f32 - 4.0);
        let cx = w as f32 * 0.5;
        let peak = h as f32 * 0.30;
        let base = h as f32 - 3.0;
        for (x, r) in self.ridge.iter_mut().enumerate() {
            let d = ((x as f32 - cx) / (w as f32 * 0.5)).abs();
            let n = value_noise(x as f32 * 0.09, 0.0, 0x564f_4c43) * 2.0;
            *r = peak + d.powf(0.85) * (base - peak) + n * 1.5;
        }
        // A small crater notch at the summit.
        let c = cx.round() as isize;
        for dx in -2..=2 {
            let x = c + dx;
            if x >= 0 && (x as usize) < w {
                self.ridge[x as usize] += 1.5 - dx.abs() as f32 * 0.4;
            }
        }
        self.lava.clear();
        self.ash.clear();
    }

    fn spawn_lava(&mut self, burst: f32) {
        let cx = self.w as f32 * 0.5;
        let peak = self.ridge.get(cx as usize).copied().unwrap_or(4.0);
        let life = self.rng.range(0.9, 2.0);
        self.lava.push(Particle {
            x: cx + self.rng.range(-1.2, 1.2),
            y: peak - 1.0,
            vx: self.rng.range(-9.0, 9.0) * burst,
            vy: self.rng.range(-20.0, -9.0) * burst,
            life,
            max_life: life,
        });
    }

    fn spawn_ash(&mut self) {
        let life = self.rng.range(3.0, 7.0);
        self.ash.push(Particle {
            x: self.rng.range(0.0, self.w as f32),
            y: self.rng.range(1.0, self.h as f32 * 0.25),
            vx: self.rng.range(0.3, 1.6),
            vy: self.rng.range(0.2, 0.8),
            life,
            max_life: life,
        });
    }
}

impl Scene for VolcanoScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.rebuild();
    }

    fn update(&mut self, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let t = f.t as f32;
        let cycle = t % 9.0;
        let burst = if cycle < 3.8 {
            0.55 + 0.45 * (cycle * 0.8).sin().abs()
        } else {
            0.18
        };
        self.spawn += dt * (12.0 + 85.0 * burst);
        while self.spawn >= 1.0 {
            self.spawn -= 1.0;
            if self.lava.len() < 220 {
                self.spawn_lava(burst);
            }
        }
        self.ash_spawn += dt * 10.0;
        while self.ash_spawn >= 1.0 {
            self.ash_spawn -= 1.0;
            if self.ash.len() < 90 {
                self.spawn_ash();
            }
        }
        if t - self.last_flash > 6.2 {
            self.last_flash = t;
            self.flash = 1.0;
        }
        self.flash = (self.flash - dt * 3.0).max(0.0);

        let base = self.h as f32 - 3.0;
        for p in &mut self.lava {
            p.vy += 22.0 * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
            if p.y > base {
                p.y = base;
                p.vy *= -0.2;
            }
        }
        self.lava.retain(|p| p.life > 0.0 && p.y < base + 1.0);
        for p in &mut self.ash {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.ash
            .retain(|p| p.life > 0.0 && p.y < self.h as f32 - 2.0);
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let t = f.t as f32;
        Some(format!(
            "VEI 4 | ASH {:02}KM | {}",
            (6.0 + (t * 0.2).sin() * 2.0) as u32,
            if self.flash > 0.1 { "STRIKE" } else { "ERUPT" }
        ))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let t = f.t as f32;
        let max_y = self.h as isize - 3;
        if max_y < 2 {
            return;
        }
        let unicode = f.charset == Charset::Unicode;
        let theme = f.theme;
        let w = self.w as f32;
        let h = self.h as f32;
        let cx = (w * 0.5) as isize;

        // Night sky.
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                let hsh = hash2(x as i64, y as i64, 0x564f_4c43);
                if hsh % 149 == 0 {
                    let tw = 0.4 + 0.6 * (t + (hsh % 100) as f32 * 0.1).sin().abs();
                    canvas.set(x, y, '.', theme.hot().scale(0.18 * tw));
                }
            }
        }

        // Ash plume, narrow at the crater and widening with height.
        let plume_h = h * 0.32;
        for y in 1..=max_y {
            let yf = y as f32;
            if yf > plume_h {
                continue;
            }
            let widen = 1.0 - yf / plume_h;
            let half = w * (0.06 + 0.16 * widen);
            let cxw = w * 0.5;
            for x in ((cxw - half) as isize)..=((cxw + half) as isize) {
                if x < 0 || x >= self.w as isize {
                    continue;
                }
                let n = value_noise(x as f32 * 0.11 + t * 0.25, yf * 0.18 - t * 0.1, 0x4153_4821);
                if n < 0.50 {
                    continue;
                }
                let ch = if n > 0.78 { '%' } else { ':' };
                canvas.set(x, y, ch, theme.c(2).scale(0.20 + 0.22 * n));
            }
        }

        // Mountain silhouette, two glyphs deep for a thin rim.
        for x in 0..self.w as isize {
            let ry = self.ridge[x as usize].round() as isize;
            let top = ry.max(1);
            for y in top..=max_y {
                let crest = y <= top + 2;
                let far_base = y > top + (max_y - top) * 2 / 3;
                let ch = if unicode {
                    if crest || !far_base {
                        '▓'
                    } else {
                        '░'
                    }
                } else if crest || !far_base {
                    '#'
                } else {
                    ':'
                };
                let rim = (x - cx).abs() < 4 && y <= top + 2;
                let col = if rim {
                    LAVA[2].scale(0.55)
                } else {
                    theme.shadow().lerp(theme.c(1), 0.18)
                };
                canvas.set(x, y, ch, col);
            }
            if !unicode && ry > 2 && ry < max_y {
                canvas.set(x, ry - 1, '^', theme.c(1).scale(0.55));
            }
        }

        // Crater glow.
        let peak = self.ridge.get(cx as usize).copied().unwrap_or(4.0) as isize;
        for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
            canvas.add_light(cx + dx, peak + dy, LAVA[1], 0.55, '*');
        }
        for k in 1..=7 {
            let y = peak - k;
            if y > 0 {
                canvas.set(cx, y, '*', LAVA[0].scale(1.0 - k as f32 * 0.10));
            }
        }

        // Two lava streams down the slopes.
        let base = max_y;
        for side in [-1.0f32, 1.0] {
            let mut x = cx as f32;
            for dy in 0..(base - peak).max(0) {
                let y = peak + dy;
                x += side * 0.55 + (dy as f32 * 0.35 + t * 2.0 + side).sin() * 0.6;
                let xx = x.round() as isize;
                if xx <= 1 || xx >= self.w as isize - 1 || y < 1 || y > max_y {
                    continue;
                }
                let k = (dy as f32 / 12.0).min(1.0);
                let col = LAVA[1].lerp(LAVA[4], k).scale(0.7);
                canvas.set(xx, y, if dy % 3 == 0 { '=' } else { '~' }, col);
                if dy % 5 == 0 {
                    canvas.add_light(xx, y, LAVA[2], 0.25, '.');
                }
            }
        }

        // Lava fountain.
        for p in &self.lava {
            if p.y < 1.0 || p.y > max_y as f32 {
                continue;
            }
            let fade = (p.life / p.max_life.max(0.01)).clamp(0.0, 1.0);
            let idx = ((1.0 - fade) * 4.0).round() as usize;
            let col = LAVA[idx.min(4)].scale(0.55 + 0.45 * fade);
            let ch = if fade > 0.6 {
                '*'
            } else if fade > 0.3 {
                '+'
            } else {
                '.'
            };
            let x = p.x.round() as isize;
            let y = p.y.round() as isize;
            canvas.set(x, y, ch, col);
            if fade > 0.7 {
                canvas.add_light(x, y, LAVA[1], 0.35, '.');
            }
        }

        // Ash particles.
        for p in &self.ash {
            if p.y < 1.0 || p.y > max_y as f32 {
                continue;
            }
            let fade = (p.life / p.max_life.max(0.01)).clamp(0.0, 1.0);
            canvas.set(
                p.x.round() as isize,
                p.y.round() as isize,
                if fade > 0.5 { ':' } else { '.' },
                theme.c(1).scale(0.18 + 0.25 * fade),
            );
        }

        // Lightning in the ash plume.
        if self.flash > 0.35 {
            let bx = (w * 0.25 + (t * 3.0).sin().abs() * w * 0.5) as isize;
            let mut y = 2isize;
            let mut x = bx;
            while y < peak && y <= max_y {
                canvas.set(x, y, '#', Rgb::hex(0xf8fafc));
                canvas.add_light(x, y, Rgb::hex(0xbfdbfe), 0.5, '*');
                y += 2;
                x += if (y / 2) % 2 == 0 { -1 } else { 1 };
                if x <= 0 || x >= self.w as isize - 1 {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eruption_draws_lava_and_mountain() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let mut frame = Frame {
            t: 2.0,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("ember").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        for _ in 0..8 {
            scene.update(&frame);
            frame.t += 1.0 / 30.0;
        }
        scene.draw(&mut canvas, &frame);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('#'), "no mountain");
        assert!(text.contains('*') || text.contains('~'), "no lava");
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
