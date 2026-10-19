//! plasma — a drifting plasma field made from soft metaballs.
//!
//! Six moving blobs add together into a smooth scalar field; a little sine
//! texture and a cycling palette keep it from looking like a lava lamp.  ASCII
//! is the default; `--unicode` switches to a compact block ramp.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const RAMP: &[u8] = b" .:-=+*#%@";
const RAMP_U: [char; 5] = [' ', '░', '▒', '▓', '█'];

struct Blob {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    r: f32,
    phase: f32,
}

pub struct PlasmaScene {
    w: usize,
    h: usize,
    blobs: Vec<Blob>,
}

impl PlasmaScene {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x504c_4153_4d41);
        let mut blobs = Vec::with_capacity(6);
        for _ in 0..9 {
            blobs.push(Blob {
                x: rng.range(0.12, 0.88),
                y: rng.range(0.18, 0.82),
                vx: rng.range(-0.07, 0.07),
                vy: rng.range(-0.09, 0.09),
                r: rng.range(0.055, 0.11),
                phase: rng.range(0.0, TAU),
            });
        }
        PlasmaScene { w: 0, h: 0, blobs }
    }
}

impl Scene for PlasmaScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        for b in &mut self.blobs {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            if b.x < 0.08 || b.x > 0.92 {
                b.vx = -b.vx;
                b.x = b.x.clamp(0.08, 0.92);
            }
            if b.y < 0.12 || b.y > 0.88 {
                b.vy = -b.vy;
                b.y = b.y.clamp(0.12, 0.88);
            }
            b.phase += dt * 0.7;
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!(
            "PLASMA {:.2} | BLOBS {}",
            (f.t as f32 * 0.12).fract(),
            self.blobs.len()
        ))
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
        let unicode = f.charset == Charset::Unicode;
        let theme = f.theme;
        let w = self.w as f32;
        let h = self.h as f32;
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                let fx = x as f32;
                let fy = y as f32;
                let mut field = 0.0;
                for b in &self.blobs {
                    let dx = fx - b.x * w;
                    // Cell height is roughly twice the width, so squash dy.
                    let dy = (fy - b.y * h) * 2.0;
                    let r = b.r * w;
                    field += (-(dx * dx + dy * dy) / (2.0 * r * r)).exp();
                }
                let texture = (fx * 0.33 + t * 1.8).sin() * 0.04
                    + (fy * 0.55 - t * 1.4).sin() * 0.04
                    + (fx * 0.11 - fy * 0.19 + t * 0.9).sin() * 0.04;
                let b = (1.0 - (-field * 0.95).exp()) + texture;
                let b = (b * 1.35 - 0.18).clamp(0.0, 1.0);
                let ch = if unicode {
                    RAMP_U[((b * 4.0).round() as usize).min(4)]
                } else {
                    RAMP[((b * 9.0).round() as usize).min(RAMP.len() - 1)] as char
                };
                if ch == ' ' {
                    continue;
                }
                let scan = if y % 2 == 0 { 0.86 } else { 1.0 };
                let col = theme
                    .sample(b * 0.75 + fx * 0.0028 + fy * 0.006 + t * 0.03)
                    .scale((0.30 + 0.95 * b) * scan);
                canvas.set(x, y, ch, col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plasma_is_dense_and_ascii() {
        let mut scene = PlasmaScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 2.0,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("plasma").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        assert!(canvas.ink() > 80 * 24 / 3, "ink {}", canvas.ink());
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
