//! 心率跳动 — a glowing ECG monitor.
//!
//! A green oscilloscope grid, a scrolling P-QRS-T waveform, a sweep cursor,
//! and live-looking vitals fill the screen.  Unicode mode uses box drawing /
//! block glyphs; `--ascii` falls back to a pure ASCII monitor.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const GREEN: Rgb = Rgb::hex(0x22c55e);
const GREEN_BRIGHT: Rgb = Rgb::hex(0x86efac);
const GREEN_DIM: Rgb = Rgb::hex(0x14532d);
const GREEN_MID: Rgb = Rgb::hex(0x16a34a);

pub struct EcgScene {
    rng: Rng,
    seed: u64,
    w: usize,
    h: usize,
    dust: Vec<(f32, f32, f32, f32)>,
    pattern: Vec<f32>,
}

impl EcgScene {
    pub fn new(seed: u64) -> Self {
        EcgScene {
            rng: Rng::new(seed ^ 0x0ec9_04a1),
            seed,
            w: 0,
            h: 0,
            dust: Vec::new(),
            pattern: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.dust.clear();
        let n = (w * h / 32).clamp(40, 220);
        for _ in 0..n {
            self.dust.push((
                self.rng.range(0.0, self.w as f32),
                self.rng.range(0.0, self.h as f32),
                self.rng.range(0.0, TAU),
                self.rng.range(0.05, 0.22),
            ));
        }
        // A scrolling waveform with irregular beat spacing, heights and flat
        // segments.  The buffer wraps seamlessly, so the trace never restarts.
        self.pattern.clear();
        let target = (self.w * 5).max(640);
        while self.pattern.len() < target {
            let gap = self.rng.below(10) + 4;
            for _ in 0..gap {
                self.pattern.push(0.0);
            }
            let amp = 0.38 + 0.85 * self.rng.f32();
            let width = 0.72 + 0.80 * self.rng.f32();
            let flat = 0.04 + 0.20 * self.rng.f32();
            let points = 10 + (width * 7.0) as usize;
            for j in 0..points {
                let local = j as f32 / points as f32;
                self.pattern.push(beat_variable(local, amp, width, flat));
            }
            let tail = self.rng.below(10);
            for _ in 0..tail {
                self.pattern.push(0.0);
            }
        }
    }
}

fn gauss(p: f32, mu: f32, sig: f32) -> f32 {
    let d = (p - mu) / sig;
    (-0.5 * d * d).exp()
}

#[cfg(test)]
fn beat(p: f32) -> f32 {
    let p = p.rem_euclid(1.0);
    // P, Q, R, S and T lobes of a stylized lead-II complex.
    -0.12 * gauss(p, 0.18, 0.035) + 0.10 * gauss(p, 0.25, 0.020) + 1.05 * gauss(p, 0.30, 0.012)
        - 0.32 * gauss(p, 0.34, 0.014)
        + 0.24 * gauss(p, 0.52, 0.060)
        + 0.025 * (p * TAU * 7.0).sin()
}

fn beat_variable(p: f32, amp: f32, width: f32, flat: f32) -> f32 {
    let p = p.rem_euclid(1.0);
    let v = -0.12 * gauss(p, 0.18, 0.035 * width)
        + 0.10 * gauss(p, 0.25, 0.020 * width)
        + amp * gauss(p, 0.30, 0.012 * width)
        - 0.32 * amp * gauss(p, 0.34, 0.014 * width)
        + 0.24 * amp * gauss(p, 0.52, 0.060 * width)
        + 0.025 * (p * TAU * 7.0).sin();
    // Randomly lengthen/shorten the flat line between T and the next P wave.
    if p > 0.58 && p < (0.58 + flat).min(0.92) {
        0.0
    } else {
        v
    }
}

impl Scene for EcgScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let bpm = 56 + ((f.t as f32 * 0.15).sin() * 2.0) as i32;
        Some(format!("HR {:3} BPM", bpm))
    }

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let unicode = f.charset == Charset::Unicode;
        let w = self.w as isize;
        let h = self.h as isize;
        let base = (self.h as f32 * 0.54) as isize;
        let amp = (self.h as f32 * 0.23).max(3.0);

        // faint phosphor dust
        for (x, y, phase, bright) in &self.dust {
            let tw = 0.45 + 0.55 * (t * 1.2 + phase).sin().abs();
            canvas.set(
                *x as isize,
                *y as isize,
                if unicode { '·' } else { '.' },
                GREEN_DIM.scale(bright * tw * 2.2),
            );
        }

        // oscilloscope grid
        let grid_col = GREEN_DIM.scale(1.35);
        for x in (2..w - 1).step_by(4) {
            let major = x % 20 == 2;
            for y in (2..h - 2).step_by(if major { 2 } else { 3 }) {
                let ch = match (unicode, major) {
                    (true, true) => '┼',
                    (true, false) => '·',
                    (false, true) => '+',
                    (false, false) => '.',
                };
                canvas.set(
                    x,
                    y,
                    ch,
                    if major {
                        grid_col
                    } else {
                        grid_col.scale(0.55)
                    },
                );
            }
        }
        for y in (4..h - 2).step_by(6) {
            canvas.hline(
                2,
                w - 2,
                y,
                if unicode { '─' } else { '-' },
                grid_col.scale(0.65),
            );
        }

        // main ECG trace: x maps to phase, so the complex scrolls right-to-left
        let mut prev: Option<(isize, isize)> = None;
        let pat = &self.pattern;
        let scroll = if pat.is_empty() {
            0
        } else {
            (t * 28.0) as usize % pat.len()
        };
        for x in 0..w {
            let v = if pat.is_empty() {
                0.0
            } else {
                pat[(x as usize + scroll) % pat.len()]
            };
            let y = (base as f32 - v * amp).round() as isize;
            if let Some((px, py)) = prev {
                canvas.line(px, py, x, y, if unicode { '█' } else { '*' }, GREEN_BRIGHT);
            }
            prev = Some((x, y));
            canvas.set(
                x,
                y,
                if unicode { '█' } else { '#' },
                GREEN_BRIGHT.lerp(Rgb::hex(0xffffff), 0.25),
            );
            if x % 2 == 0 {
                canvas.add_light(x, y, GREEN, 0.35, '.');
            }
        }

        // a soft secondary trace (respiration-ish) at the lower third
        let mut prev2: Option<(isize, isize)> = None;
        for x in 0..w {
            let phase = (x as f32 * 0.020 - t * 0.55).rem_euclid(1.0);
            let y = (base as f32 + amp * 1.20 + (phase * TAU).sin() * 2.0) as isize;
            if let Some((px, py)) = prev2 {
                canvas.line(
                    px,
                    py,
                    x,
                    y,
                    if unicode { '·' } else { '.' },
                    GREEN_MID.scale(0.55),
                );
            }
            prev2 = Some((x, y));
        }

        // sweep cursor
        let sx = ((t * 27.0) as isize).rem_euclid(w.max(1));
        for y in 2..h - 2 {
            canvas.set(sx, y, if unicode { '║' } else { '|' }, GREEN.scale(0.55));
        }
        canvas.add_light(sx, base, GREEN_BRIGHT, 0.8, '*');
        for y in (base - 2)..=(base + 2) {
            canvas.add_light(sx, y, GREEN, 0.30, '.');
        }

        // Monitor labels stay diegetic and above the reserved HUD rows.
        let spo2 = "SpO2 98%";
        if w > 40 {
            canvas.put_str(2, h - 4, spo2, GREEN_MID);
        }
        let bp = "SYS 120 / DIA 80";
        let bx = (w - bp.len() as isize - 2).max(2);
        if w > 40 {
            canvas.put_str(bx, h - 4, bp, GREEN_MID);
        }
        let lead = "25 mm/s  10 mm/mV  //  SINUS RHYTHM";
        if w > (lead.len() as isize + 4) {
            canvas.put_str(2, h - 3, lead, GREEN_DIM.scale(1.8));
        }

        // pulsing heart mark
        let pulse = 0.6 + 0.4 * (t * 6.0).sin().abs();
        let heart = if unicode { "♥" } else { "<3" };
        canvas.put_str(3, 1, heart, GREEN_BRIGHT.scale(pulse));
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (2, 0)] {
            canvas.add_light(4 + dx, 1 + dy, GREEN, 0.22 * pulse, '.');
        }
        let _ = theme;
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beat_has_qrs_spike() {
        assert!(beat(0.30) > 0.8);
        assert!(beat(0.34) < 0.0);
        assert!(beat(0.70).abs() < 0.2);
    }
}
