//! helix — a rotating DNA double helix.
//!
//! Two sine strands wrap around a vertical axis; base-pair rungs join them and
//! carry A/T/C/G letters.  Depth comes from the cosine of each strand phase, so
//! the front strand is bright and the back strand is dim.  Pure ASCII: the
//! letters and backbone glyphs are the effect.

use std::f32::consts::PI;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::scene::{Frame, Scene};

const BASES: [u8; 4] = *b"ATCG";
const BASE_COLORS: [Rgb; 4] = [
    Rgb::hex(0x4ade80), // A
    Rgb::hex(0xfb7185), // T
    Rgb::hex(0x60a5fa), // C
    Rgb::hex(0xfacc15), // G
];

pub struct HelixScene {
    w: usize,
    h: usize,
}

impl HelixScene {
    pub fn new(_seed: u64) -> Self {
        HelixScene { w: 0, h: 0 }
    }
}

fn partner(base: u8) -> u8 {
    match base {
        b'A' => b'T',
        b'T' => b'A',
        b'C' => b'G',
        _ => b'C',
    }
}

impl Scene for HelixScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, _f: &Frame) {}

    fn hud_status(&self, _f: &Frame) -> Option<String> {
        let pairs = (self.h.saturating_sub(5)) / 2;
        Some(format!("BASE PAIRS {pairs}"))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let t = f.t as f32;
        let max_y = self.h as isize - 3;
        if max_y < 2 || self.w < 8 {
            return;
        }
        let theme = f.theme;
        let cx = self.w as f32 * 0.5;
        let amp = (self.w as f32 * 0.23).clamp(3.0, 28.0);
        let top = 2isize;
        let bottom = max_y;

        // Faint star/lab dust keeps the background from being dead black.
        for y in 1..=bottom {
            for x in 0..self.w as isize {
                let hsh = crate::rng::hash2(x as i64, y as i64, 0x4845_4c49);
                if hsh % 137 != 0 {
                    continue;
                }
                canvas.set(x, y, '.', theme.c(2).scale(0.16));
            }
        }

        let mut row = 0usize;
        let mut y = top;
        while y <= bottom {
            let phase = t * 1.35 + y as f32 * 0.24;
            let x1 = cx + amp * phase.sin();
            let x2 = cx + amp * (phase + PI).sin();
            let z1 = phase.cos();
            let z2 = -z1;
            let front_is_one = z1 >= z2;

            let base = BASES[row % BASES.len()];
            let mate = partner(base);
            let dim = ((z1.abs() + 1.0) * 0.5).clamp(0.35, 1.0);

            // Rung behind the letters.
            let lo = x1.min(x2).round() as isize;
            let hi = x1.max(x2).round() as isize;
            for x in lo..=hi {
                if x <= 1 || x >= self.w as isize - 1 {
                    continue;
                }
                let ch = if (x - lo) % 3 == 0 { '=' } else { '-' };
                canvas.set(x, y, ch, theme.c(2).scale(0.25 + 0.25 * dim));
            }

            // Base letters around the middle of the rung.
            let mid = ((x1 + x2) * 0.5).round() as isize;
            if mid > 2 && mid < self.w as isize - 3 {
                let bi = (row) % BASES.len();
                let ci = (row + 2) % BASES.len();
                canvas.set(
                    mid - 1,
                    y,
                    base as char,
                    BASE_COLORS[bi].scale(0.45 + 0.55 * dim),
                );
                canvas.set(
                    mid + 1,
                    y,
                    mate as char,
                    BASE_COLORS[ci].scale(0.45 + 0.55 * dim),
                );
            }

            // Back strand first, then the front strand on top.
            let (back_x, back_z, front_x, front_z) = if front_is_one {
                (x2, z2, x1, z1)
            } else {
                (x1, z1, x2, z2)
            };
            let back_ch = if back_z > -0.2 { '+' } else { '.' };
            canvas.set(
                back_x.round() as isize,
                y,
                back_ch,
                theme.c(2).scale(0.35 + 0.25 * (back_z + 1.0)),
            );

            let front_ch = if front_z > 0.55 {
                '@'
            } else if front_z > 0.0 {
                '#'
            } else {
                '+'
            };
            let front_col = theme
                .c(3)
                .lerp(theme.hot(), (front_z + 1.0) * 0.5)
                .scale(0.75 + 0.25 * front_z);
            let fx = front_x.round() as isize;
            canvas.set(fx, y, front_ch, front_col);
            canvas.add_light(fx, y, front_col, 0.25, '.');

            row += 1;
            y += 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helix_has_strands_and_pairs() {
        let mut scene = HelixScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 4.0,
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
        assert!(text.contains('@') || text.contains('#'), "no strand");
        assert!(text.contains('A') || text.contains('T'), "no base letters");
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
