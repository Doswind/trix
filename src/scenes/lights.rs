//! lights — aurora curtains over a mountain lake.
//!
//! Three noisy vertical curtains drift across a starry sky; a moon, snow-capped
//! ridges and a rippling lake reflection give the scene depth.  ASCII only:
//! `|`, `:`, `.`, `*` are all the resolution an aurora needs.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::value_noise;
use crate::scene::{Frame, Scene};

pub struct LightsScene {
    w: usize,
    h: usize,
    ridge: Vec<f32>,
    ridge2: Vec<f32>,
    stars: Vec<(f32, f32, f32)>,
    seed: u64,
}

impl LightsScene {
    pub fn new(seed: u64) -> Self {
        LightsScene {
            w: 0,
            h: 0,
            ridge: Vec::new(),
            ridge2: Vec::new(),
            stars: Vec::new(),
            seed,
        }
    }
}

impl Scene for LightsScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let w = self.w;
        let h = self.h;
        let horizon = h as f32 * 0.66;
        self.ridge.clear();
        self.ridge2.clear();
        self.ridge.resize(w, horizon);
        self.ridge2.resize(w, horizon);
        for x in 0..w {
            let fx = x as f32;
            let n = value_noise(fx * 0.07, 0.0, 0x7269_6467_6531);
            let n2 = value_noise(fx * 0.17 + 13.0, 0.0, 0x7269_6467_6532);
            self.ridge[x] = horizon - 2.0 - n * h as f32 * 0.30 - n2 * 2.0;
            self.ridge2[x] = horizon - n2 * h as f32 * 0.16;
        }
        // Rebuild a proportional star field (the old one was sized for a
        // different terminal).
        let mut rng = crate::rng::Rng::new(self.seed ^ 0x5354_4152_5345);
        self.stars.clear();
        for _ in 0..30 {
            self.stars.push((
                rng.range(0.0, w as f32),
                rng.range(1.0, horizon - 1.0),
                rng.range(0.0, TAU),
            ));
        }
    }

    fn update(&mut self, _f: &Frame) {}

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!(
            "AURORA {:02}% | RIDGE {}",
            ((f.t as f32 * 0.05).sin().abs() * 100.0) as u32,
            self.ridge.len()
        ))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let t = f.t as f32;
        let w = self.w as f32;
        let h = self.h as f32;
        let max_y = self.h as isize - 3;
        if max_y < 2 {
            return;
        }
        let horizon = (h * 0.66) as isize;
        let theme = f.theme;

        // Stars.
        for (sx, sy, phase) in &self.stars {
            let tw = 0.45 + 0.55 * (t * 1.2 + phase).sin().abs();
            let y = *sy as isize;
            if y >= 1 && y < horizon {
                canvas.set(*sx as isize, y, '.', theme.hot().scale(0.10 + 0.18 * tw));
            }
        }

        // Moon and its halo.
        let mx = (w * 0.78) as isize;
        let my = (h * 0.18) as isize;
        if mx > 2 && mx < self.w as isize - 2 && my > 1 && my < horizon - 1 {
            canvas.set(mx, my, 'O', theme.hot());
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                canvas.add_light(mx + dx, my + dy, theme.hot(), 0.18, '.');
            }
        }

        // Aurora ribbons: three wavy, layered bands with short hanging
        // curtains.  Ribbons read much better than full-height bars.
        let colors = [Rgb::hex(0x34d399), Rgb::hex(0x22d3ee), Rgb::hex(0xa78bfa)];
        for (layer, color) in colors.iter().enumerate() {
            let color = color.lerp(theme.c(3), 0.15);
            let amp = h * 0.075;
            for x in 0..self.w as isize {
                let n = value_noise(
                    x as f32 * 0.085,
                    t * 0.22 + layer as f32 * 2.1,
                    0x4155_524f_5241 + layer as u64,
                );
                let base = h * 0.20
                    + layer as f32 * 2.2
                    + (x as f32 * 0.045 + t * 0.30 + layer as f32 * 1.7).sin() * amp
                    + n * 3.0;
                for dy in -1..=1 {
                    let y = base.round() as isize + dy;
                    if y < 1 || y >= horizon {
                        continue;
                    }
                    let inten = 1.0 - dy.abs() as f32 / 2.0;
                    canvas.set(
                        x,
                        y,
                        if dy == 0 { '*' } else { '~' },
                        color.scale(0.40 + 0.70 * inten),
                    );
                }
                if x % 4 == 0 {
                    for k in 1..=3 {
                        let y = base.round() as isize + k;
                        if y >= 1 && y < horizon {
                            canvas.add_light(x, y, color, 0.22, if k == 1 { ':' } else { '.' });
                        }
                    }
                }
            }
        }

        // Two mountain ridges: the far one is dim, the near one is solid.
        for x in 0..self.w as isize {
            let y2 = self.ridge2[x as usize].round() as isize;
            for y in y2.max(1)..horizon {
                canvas.set_if_blank(x, y, ':', theme.c(2).scale(0.24));
            }
        }
        for x in 0..self.w as isize {
            let ry = self.ridge[x as usize].round() as isize;
            let top = ry.max(1);
            for y in top..horizon {
                let ch = if y == top {
                    '^'
                } else if y > top + 1 {
                    ':'
                } else {
                    '#'
                };
                canvas.set(x, y, ch, theme.c(2).scale(0.40));
            }
            if ry < horizon - (h * 0.22) as isize {
                canvas.set(x, (ry - 1).max(1), '*', theme.hot().scale(0.75));
            }
        }

        // Lake surface: dark ripples, a moon path and a soft aurora reflection.
        for y in horizon..=max_y {
            let depth = (y - horizon) as f32 / (max_y - horizon).max(1) as f32;
            for x in 0..self.w as isize {
                let wave = (x as f32 * 0.13 + y as f32 * 0.7 + t * 1.2).sin();
                if (x + y) % 7 == 0 {
                    canvas.set(
                        x,
                        y,
                        if wave > 0.4 { '~' } else { '-' },
                        theme.c(2).scale(0.13 + 0.10 * depth),
                    );
                }
            }
            // A widening moon path down the lake.
            let path = (1.0 + (1.0 - depth) * 4.0) as isize;
            let rx = mx + ((y as f32 * 0.5 + t).sin() * 1.5) as isize;
            for dx in -path..=path {
                let x = rx + dx;
                if x > 0 && x < self.w as isize && (x + y) % 2 == 0 {
                    canvas.set(x, y, '~', theme.hot().scale(0.30 - 0.18 * depth));
                }
            }
        }

        // Mirror the brightest aurora band into the lake, compressed and dim.
        for x in 0..self.w as isize {
            let n = value_noise(x as f32 * 0.085, t * 0.20, 0x4155_524f_5241);
            let col = Rgb::hex(0x34d399)
                .lerp(theme.c(3), 0.25)
                .scale(0.30 + 0.35 * n);
            for y in horizon..=(horizon + 3).min(max_y) {
                if (x + y) % 4 == 0 {
                    canvas.add_light(x, y, col, 0.25, '.');
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aurora_has_sky_mountains_and_lake() {
        let mut scene = LightsScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 5.0,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("aurora").unwrap(),
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('^'), "no ridge");
        assert!(text.contains('~'), "no lake");
        assert!(text.contains('|') || text.contains('*'), "no aurora");
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
