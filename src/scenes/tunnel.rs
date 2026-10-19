//! tunnel — a first-person warp tunnel built from projected rings.
//!
//! Rings are spaced in depth and expand toward the viewer; ten radial spokes
//! rotate slowly and a handful of stars streak past.  Pure ASCII: `+`, `*`,
//! `#` and `@` carry the whole perspective.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

struct Star {
    angle: f32,
    radius: f32,
    z: f32,
    speed: f32,
}

pub struct TunnelScene {
    w: usize,
    h: usize,
    rng: Rng,
    stars: Vec<Star>,
}

impl TunnelScene {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x7475_6e6e_656c);
        let mut stars = Vec::with_capacity(48);
        for _ in 0..48 {
            stars.push(Star {
                angle: rng.range(0.0, TAU),
                radius: rng.range(0.08, 0.62),
                z: rng.range(0.08, 1.0),
                speed: rng.range(0.6, 1.8),
            });
        }
        TunnelScene {
            w: 0,
            h: 0,
            rng,
            stars,
        }
    }

    fn speed(t: f32) -> f32 {
        0.7 + 1.8 * (1.0 - (-t * 0.45).exp())
    }
}

impl Scene for TunnelScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let speed = Self::speed(f.t as f32);
        for s in &mut self.stars {
            s.z -= s.speed * speed * dt * 0.55;
            if s.z < 0.06 {
                s.z = self.rng.range(0.85, 1.0);
                s.angle = self.rng.range(0.0, TAU);
                s.radius = self.rng.range(0.08, 0.62);
            }
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!("WARP {:.2}c | RINGS 16", Self::speed(f.t as f32)))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let t = f.t as f32;
        let max_y = self.h as isize - 3;
        if max_y < 1 {
            return;
        }
        let theme = f.theme;
        let cx = self.w as f32 * 0.5 + (t * 0.5).sin() * self.w as f32 * 0.035;
        let cy = self.h as f32 * 0.5 + (t * 0.75).cos() * self.h as f32 * 0.035;
        let scale = self.w as f32 * 0.58;
        let speed = Self::speed(t);

        // Rotating spokes behind the rings.
        for i in 0..10 {
            let a = i as f32 / 10.0 * TAU + t * 0.22;
            let steps = 26;
            for k in 1..=steps {
                let r = k as f32 / steps as f32;
                let x = cx + a.cos() * r * scale;
                let y = cy + a.sin() * r * scale * 0.5;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                canvas.set(
                    x.round() as isize,
                    y.round() as isize,
                    if k % 3 == 0 { '-' } else { '.' },
                    theme.c(2).scale(0.10 + 0.16 * (1.0 - r)),
                );
            }
        }

        // Sixteen rings, spaced in depth.  Near rings are brighter and thicker.
        for i in 0..16 {
            let z = (i as f32 / 16.0 + t * 0.20 * speed).rem_euclid(1.0);
            let r = (self.w as f32 * 0.04 + z * z * scale).min(scale * 1.4);
            let near = z;
            let ch = if near > 0.72 {
                '#'
            } else if near > 0.38 {
                '*'
            } else {
                '+'
            };
            let col = theme.c(2).lerp(theme.hot(), near).scale(0.25 + 0.85 * near);
            let steps = (r * 3.0) as usize + 18;
            for k in 0..steps {
                let a = k as f32 / steps as f32 * TAU;
                let x = cx + a.cos() * r;
                let y = cy + a.sin() * r * 0.5;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                let xx = x.round() as isize;
                let yy = y.round() as isize;
                canvas.set(xx, yy, ch, col);
                if near > 0.75 {
                    if yy > 1 {
                        canvas.set(xx, yy - 1, ch, col.scale(0.35));
                    }
                    if yy < max_y {
                        canvas.set(xx, yy + 1, ch, col.scale(0.35));
                    }
                }
            }
        }

        // Bright vanishing point.
        let cxi = cx.round() as isize;
        let cyi = cy.round() as isize;
        canvas.set(cxi, cyi, '@', theme.hot());
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let x = cxi + dx;
            let y = cyi + dy;
            if y >= 1 && y <= max_y {
                canvas.add_light(x, y, theme.c(3), 0.35, '*');
            }
        }

        // Projected stars: a short inward trail plus a bright head.
        for s in &self.stars {
            let rr = s.radius / s.z.max(0.06) * self.w as f32 * 0.5;
            for k in 0..3 {
                let r = rr - k as f32 * 1.8;
                if r <= 0.0 {
                    continue;
                }
                let x = cx + s.angle.cos() * r;
                let y = cy + s.angle.sin() * r * 0.5;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                let ch = if k == 0 { '*' } else { '-' };
                let col = theme
                    .hot()
                    .lerp(theme.c(3), 0.25 + k as f32 * 0.15)
                    .scale(0.85 - k as f32 * 0.2);
                canvas.add_light(x.round() as isize, y.round() as isize, col, 0.55, ch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_the_screen_with_glyphs() {
        let mut scene = TunnelScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 3.0,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("neon").unwrap(),
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        assert!(canvas.ink() > 80 * 24 / 6, "ink {}", canvas.ink());
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
