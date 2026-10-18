//! 龙卷风 — tornadoes sweeping across a storm plain.
//!
//! One or several funnels of different sizes and heights travel left to right.
//! `--tornado-count` and `--tornado-speed` tune the number and travel speed.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const SKY: Rgb = Rgb::hex(0x0b1220);
const CLOUD: Rgb = Rgb::hex(0x334155);
const CLOUD_DARK: Rgb = Rgb::hex(0x1e293b);
const FUNNEL: Rgb = Rgb::hex(0x94a3b8);
const FUNNEL_DARK: Rgb = Rgb::hex(0x475569);
const DUST: Rgb = Rgb::hex(0xa16207);
const LIGHT: Rgb = Rgb::hex(0xe2e8f0);

struct Tornado {
    x: f32,
    base_y: f32,
    size: f32,
    height: f32,
    speed: f32,
    phase: f32,
}

pub struct TornadoScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    funnels: Vec<Tornado>,
}

impl TornadoScene {
    pub fn new(seed: u64) -> Self {
        TornadoScene {
            seed,
            rng: Rng::new(seed ^ 0x544f_524e_4144),
            w: 0,
            h: 0,
            funnels: Vec::new(),
        }
    }

    fn make(&mut self, initial: bool) -> Tornado {
        let h = self.h as f32;
        Tornado {
            x: if initial {
                self.rng.range(0.0, self.w as f32)
            } else {
                -20.0
            },
            base_y: h * self.rng.range(0.82, 0.94),
            size: self.rng.range(2.5, 7.5),
            height: h * self.rng.range(0.38, 0.78),
            speed: self.rng.range(3.5, 10.0),
            phase: self.rng.range(0.0, TAU),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.funnels.clear();
        for _ in 0..3 {
            let t = self.make(true);
            self.funnels.push(t);
        }
    }
}

impl Scene for TornadoScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let target = f.tune.tornado_count.clamp(1, 8) as usize;
        let speed = f.tune.tornado_speed.clamp(0.05, 4.0);
        while self.funnels.len() < target {
            let t = self.make(false);
            self.funnels.push(t);
        }
        if self.funnels.len() > target {
            self.funnels.truncate(target);
        }
        for (i, t) in self.funnels.iter_mut().enumerate() {
            t.x += t.speed * speed * dt * (0.8 + i as f32 * 0.08);
            t.phase += dt * (1.5 + t.size * 0.2);
            if t.x > self.w as f32 + 18.0 {
                let h = self.h as f32;
                t.x = -18.0 - i as f32 * 4.0;
                t.base_y = h * self.rng.range(0.82, 0.94);
                t.size = self.rng.range(2.5, 7.5);
                t.height = h * self.rng.range(0.38, 0.78);
                t.speed = self.rng.range(3.5, 10.0);
                t.phase = self.rng.range(0.0, TAU);
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        let h = self.h as isize;
        // storm sky
        for y in 0..h {
            let v = y as f32 / h.max(1) as f32;
            let c = SKY.lerp(CLOUD_DARK, v * 0.5);
            for x in 0..self.w as isize {
                if (x + y * 4) % 8 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.6));
                }
            }
        }
        // heavy cloud ceiling
        for k in 0..6 {
            let y = 1 + k * 2;
            for x in 0..self.w as isize {
                let puff = ((x as f32 * 0.13 + k as f32 * 1.7 + t * 0.2).sin()) > 0.2;
                if puff {
                    canvas.set(
                        x,
                        y,
                        if unicode { '▒' } else { ':' },
                        CLOUD.scale(0.35 + k as f32 * 0.03),
                    );
                }
            }
        }
        // ground
        for y in (self.h as f32 * 0.90) as isize..h {
            for x in 0..self.w as isize {
                canvas.set(x, y, if unicode { '█' } else { '#' }, DUST.scale(0.28));
            }
        }
        for funnel in &self.funnels {
            let cx = funnel.x;
            let base = funnel.base_y;
            let top = base - funnel.height;
            let hgt = funnel.height.max(1.0);
            // fill the cone with rotating debris
            for y in top as isize..base as isize {
                let tt = ((base - y as f32) / hgt).clamp(0.0, 1.0);
                // Top is 1.5–2x wider than before; bottom tapers to a point.
                let radius = 0.15 + funnel.size * (tt * 1.0 + tt * tt * 2.8);
                let curve =
                    (tt * 7.0 + t * 1.5 + funnel.phase).sin() * funnel.size * (0.50 + tt * 1.20);
                for k in 0..28 {
                    let a = funnel.phase + tt * 5.0 + k as f32 / 28.0 * TAU;
                    let x = cx + curve + a.cos() * radius;
                    let yy = y as f32 + a.sin() * radius * 0.25;
                    let ch = if unicode {
                        match k % 3 {
                            0 => '░',
                            1 => '▒',
                            _ => '·',
                        }
                    } else {
                        match k % 3 {
                            0 => ':',
                            1 => '.',
                            _ => '~',
                        }
                    };
                    canvas.set(x as isize, yy as isize, ch, FUNNEL.scale(0.66 + tt * 0.72));
                }
            }
            // bright spiral edges
            for k in 0..100 {
                let tt = k as f32 / 100.0;
                let y = base - tt * hgt;
                // Top is 1.5–2x wider than before; bottom tapers to a point.
                let radius = 0.15 + funnel.size * (tt * 1.0 + tt * tt * 2.8);
                let curve =
                    (tt * 7.0 + t * 1.5 + funnel.phase).sin() * funnel.size * (0.50 + tt * 1.20);
                let a = funnel.phase + tt * 9.0 + (t * 0.9).sin();
                let x = cx + curve + a.cos() * radius;
                canvas.set(
                    x as isize,
                    y as isize,
                    if unicode { '✦' } else { '*' },
                    FUNNEL.scale(1.0),
                );
                canvas.set(
                    (cx + curve - a.cos() * radius) as isize,
                    y as isize,
                    if unicode { '·' } else { '.' },
                    FUNNEL_DARK.scale(0.85),
                );
            }
            // dust fan at the base and cloud cap
            // Heavy debris that is lifted high into and around the funnel.
            for k in 0..110 {
                let tt = ((k * 37 + (funnel.phase * 100.0) as usize) % 100) as f32 / 100.0;
                let y = base - tt * hgt * 1.06;
                let radius = 0.2 + funnel.size * (tt * 1.0 + tt * tt * 2.8);
                let curve =
                    (tt * 7.0 + t * 1.5 + funnel.phase).sin() * funnel.size * (0.50 + tt * 1.20);
                let a = funnel.phase * 1.7 + tt * 11.0 + k as f32 * 0.9 + t * 0.5;
                let spread = radius * (1.15 + (k % 5) as f32 * 0.12);
                let x = cx + curve + a.cos() * spread;
                let yy = y + a.sin() * spread * 0.30;
                let ch = if unicode {
                    match k % 6 {
                        0 => '░',
                        1 => '▒',
                        2 => '✦',
                        3 => '·',
                        4 => '▴',
                        _ => ':',
                    }
                } else if k % 3 == 0 {
                    '*'
                } else {
                    ':'
                };
                let col = if tt > 0.55 {
                    FUNNEL.scale(0.35 + 0.35 * (1.0 - tt))
                } else {
                    DUST.scale(0.45)
                };
                canvas.set(x as isize, yy as isize, ch, col);
            }
            for k in 0..52 {
                let a = k as f32 / 52.0 * TAU + funnel.phase + (t * 1.2).sin() * 0.15;
                let reach = funnel.size * (1.2 + (k % 6) as f32 * 0.30);
                let x = cx + a.cos() * reach;
                let y = base + (a.sin() * 2.6).abs() + (k % 3) as f32 * 0.45;
                let ch = if unicode { '░' } else { ':' };
                canvas.set(
                    x as isize,
                    y as isize,
                    ch,
                    DUST.scale(0.35 + (k % 4) as f32 * 0.08),
                );
            }
            for k in 0..18 {
                let x = cx + (k as f32 - 9.0) * funnel.size * 0.55;
                let y = base + 1.0 + (k % 3) as f32;
                canvas.set(
                    x as isize,
                    y as isize,
                    if unicode { '▴' } else { '^' },
                    DUST.scale(0.55),
                );
            }
            let cap_y = (top as isize).max(1);
            let cap_half = (funnel.size * 2.8) as isize;
            for dx in -cap_half..=cap_half {
                canvas.set(
                    cx as isize + dx,
                    cap_y,
                    if unicode { '≈' } else { '~' },
                    CLOUD.scale(0.5),
                );
            }
            canvas.add_light(cx as isize, base as isize, DUST, 0.25, '.');
        }
        // lightning flicker in the clouds
        if (t * 7.0).sin() > 0.96 {
            for x in (0..self.w as isize).step_by(3) {
                canvas.set_if_blank(
                    x,
                    2 + (x as usize % 2) as isize,
                    if unicode { '╱' } else { '/' },
                    LIGHT.scale(0.35),
                );
            }
        }
        let w = self.w as isize;
        let _ = h;
        let title = if unicode {
            "╡ TORNADO // STORM PLAIN ╞"
        } else {
            " TORNADO // STORM PLAIN "
        };
        canvas.put_str(2, 0, title, FUNNEL.scale(0.85));
        let status = format!(
            "{} FUNNELS  {:.1}x",
            self.funnels.len(),
            f.tune.tornado_speed
        );
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            LIGHT.scale(0.65),
        );
        canvas.put_str(
            2,
            self.h as isize - 1,
            "--tornado-count / --tornado-speed",
            DUST.scale(0.55),
        );
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn funnels_exist() {
        let mut s = TornadoScene::new(7);
        s.resize(120, 36);
        assert_eq!(s.funnels.len(), 3);
    }
}
