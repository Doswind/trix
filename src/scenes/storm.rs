//! 电闪雷鸣 — a thunderstorm with full-screen lightning flashes.
//!
//! Dark clouds, slanted rain, a city silhouette and jagged bolts.  On a strike
//! the whole canvas flashes white for a couple of frames, then decays.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};

const SKY: Rgb = Rgb::hex(0x020617);
const CLOUD: Rgb = Rgb::hex(0x1e293b);
const RAIN: Rgb = Rgb::hex(0x7dd3fc);
const BOLT: Rgb = Rgb::hex(0xf8fafc);
const BOLT_BLUE: Rgb = Rgb::hex(0x93c5fd);
const GROUND: Rgb = Rgb::hex(0x0f172a);
const WINDOW: Rgb = Rgb::hex(0xfacc15);

struct Drop {
    x: f32,
    y: f32,
    speed: f32,
    len: usize,
}

pub struct StormScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    drops: Vec<Drop>,
    flash: f32,
    next_bolt: f32,
    strikes: u32,
    bolt: Vec<(isize, isize)>,
}

impl StormScene {
    pub fn new(seed: u64) -> Self {
        StormScene {
            seed,
            rng: Rng::new(seed ^ 0x0053_544f_524d),
            w: 0,
            h: 0,
            drops: Vec::new(),
            flash: 0.0,
            next_bolt: 0.5,
            strikes: 0,
            bolt: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.drops.clear();
        for _ in 0..(w * h / 7).clamp(120, 520) {
            self.drops.push(Drop {
                x: self.rng.range(0.0, self.w as f32 + self.h as f32 * 0.5),
                y: self.rng.range(0.0, self.h as f32),
                speed: self.rng.range(22.0, 60.0),
                len: self.rng.below(4) + 2,
            });
        }
        self.bolt.clear();
    }

    fn strike(&mut self) {
        self.flash = 1.0;
        self.strikes = self.strikes.saturating_add(1);
        let x0 = self.rng.range(self.w as f32 * 0.15, self.w as f32 * 0.85) as isize;
        let y0 = (self.h as f32 * 0.12) as isize;
        let y1 = (self.h as f32 * 0.78) as isize;
        self.bolt.clear();
        let mut x = x0;
        for y in y0..y1 {
            if self.rng.chance(0.34) {
                x += self.rng.range(-4.0, 4.0) as isize;
            }
            self.bolt.push((x, y));
            if self.rng.chance(0.08) {
                let bx = x + self.rng.range(-6.0, 6.0) as isize;
                let by = y + self.rng.range(2.0, 8.0) as isize;
                self.bolt.push((bx, by));
            }
        }
        self.next_bolt = self.rng.range(0.9, 3.4);
    }
}

impl Scene for StormScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        for d in &mut self.drops {
            d.x -= d.speed * 0.30 * dt;
            d.y += d.speed * dt;
            if d.y > self.h as f32 || d.x < -4.0 {
                d.y = -4.0;
                d.x = self.rng.range(0.0, self.w as f32 + self.h as f32 * 0.5);
            }
        }
        self.flash = (self.flash - dt * 5.5).max(0.0);
        self.next_bolt -= dt;
        if self.next_bolt <= 0.0 {
            self.strike();
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let w = self.w as isize;
        let h = self.h as isize;
        let ground = (self.h as f32 * 0.82) as isize;
        for y in 0..h {
            let v = y as f32 / ground.max(1) as f32;
            let c = SKY.lerp(CLOUD, v * 0.55);
            for x in 0..w {
                if (x + y * 5) % 7 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.7));
                }
            }
        }
        // heavy clouds
        for k in 0..7 {
            let y = 1 + k * 2;
            for x in 0..w {
                let lump = (x as f32 * 0.12 + k as f32 * 2.0 + f.t as f32 * 0.15).sin() > 0.0;
                if lump {
                    canvas.set(
                        x,
                        y,
                        if unicode { '▒' } else { ':' },
                        CLOUD.scale(0.5 + k as f32 * 0.04),
                    );
                }
            }
        }
        // city silhouette + lit windows
        for x in 0..w {
            let bh = 4 + (hash01(x as i64 / 2, 1, self.seed) * 10.0) as isize;
            for y in (ground - bh)..ground {
                canvas.set(x, y, if unicode { '█' } else { '#' }, GROUND);
                if (x + y) % 5 == 0 && hash01(x as i64, y as i64, self.seed + 4) > 0.55 {
                    canvas.set(x, y, if unicode { '▦' } else { 'o' }, WINDOW.scale(0.55));
                }
            }
        }
        // rain
        for d in &self.drops {
            let x = d.x.round() as isize;
            let y = d.y.round() as isize;
            for k in 0..d.len as isize {
                canvas.set(
                    x + k / 2,
                    y + k,
                    if unicode { '╱' } else { '/' },
                    RAIN.scale(0.35 - k as f32 * 0.05),
                );
            }
        }
        // lightning bolt
        for &(x, y) in &self.bolt {
            canvas.set(x, y, if unicode { '█' } else { '#' }, BOLT);
            canvas.add_light(x, y, BOLT_BLUE, 0.55, '*');
            canvas.add_light(x - 1, y, BOLT_BLUE, 0.20, '.');
            canvas.add_light(x + 1, y, BOLT_BLUE, 0.20, '.');
        }
        // full-screen white flash (drawn after the scene; bolt stays readable)
        if self.flash > 0.0 {
            let a = self.flash.powf(0.7);
            for y in 0..h {
                for x in 0..w {
                    canvas.set(
                        x,
                        y,
                        if unicode { '█' } else { ' ' },
                        Rgb::hex(0xffffff).scale(a),
                    );
                }
            }
            for &(x, y) in &self.bolt {
                canvas.set(x, y, if unicode { '█' } else { '*' }, BOLT);
            }
        }
        // wet ground / reflections
        for y in ground..h {
            for x in 0..w {
                let refl = (x as f32 * 0.1 + y as f32 * 0.7 + f.t as f32 * 2.0).sin();
                let ch = if refl > 0.7 {
                    if unicode {
                        '≈'
                    } else {
                        '~'
                    }
                } else if unicode {
                    '░'
                } else {
                    '.'
                };
                canvas.set(x, y, ch, RAIN.scale(0.12));
            }
        }
        let title = if unicode {
            "╡ THUNDERSTORM // STRIKE ╞"
        } else {
            " THUNDERSTORM // STRIKE "
        };
        canvas.put_str(2, 0, title, BOLT_BLUE.scale(0.85));
        let status = format!("STRIKES {:02}", self.strikes);
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            WINDOW.scale(0.7),
        );
        canvas.put_str(
            2,
            h - 1,
            "RAIN // CLOUDS // LIGHTNING",
            BOLT_BLUE.scale(0.4),
        );
        let _ = f.theme;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rain_exists() {
        let mut s = StormScene::new(7);
        s.resize(120, 36);
        assert!(s.drops.len() > 100);
        s.strike();
        assert!(s.flash > 0.0);
        assert!(!s.bolt.is_empty());
    }
}
