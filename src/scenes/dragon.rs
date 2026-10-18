//! 巨龙盘旋 — a Chinese celestial dragon flying across the whole terminal.
//!
//! The dragon follows a large Lissajous flight path; the body is sampled at
//! past times so the tail trails the head exactly.  It has a large side-view
//! head with antler horns, whiskers, mane, fangs and a beard, four clawed
//! legs, a dorsal ridge, a tail fin, and a small flaming pearl ahead of it.
//!
//! The whole body carries a slow serpentine wave that travels from the head
//! down to the tail, so the flight path reads as a coiling dragon instead of a
//! smooth circle.  The wave stays gentle and well inside the loop: it fades in
//! behind the neck and grows towards the tail, leaving the head steady.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const BODY_RAMP: &[u8] = b" .:-=+*#%@";
const SAMPLES: usize = 250;

/// Whole-body serpentine wave: roughly 2.6 crests travel from head to tail at
/// a slow, steady pace.  The amplitude fades in behind the neck and grows
/// towards the tail, so the loop keeps its coiling shape instead of becoming a
/// zig-zag.
const WAVE_CYCLES: f32 = 2.6;
const WAVE_SPEED: f32 = 2.3;
const WAVE_AMP: f32 = 4.8;
const WAVE_RAMP: f32 = 0.18;

const GOLD: Rgb = Rgb::hex(0xf5c542);
const PALE_GOLD: Rgb = Rgb::hex(0xfff3c4);
const DEEP_GOLD: Rgb = Rgb::hex(0xb45309);
const JADE: Rgb = Rgb::hex(0x34d399);
const EMERALD: Rgb = Rgb::hex(0x10b981);
const TEAL: Rgb = Rgb::hex(0x0f766e);
const RED: Rgb = Rgb::hex(0xef4444);
const WHITE: Rgb = Rgb::hex(0xffffff);

struct Star {
    x: usize,
    y: usize,
    phase: f32,
    bright: f32,
    big: bool,
}

struct Cloud {
    x: f32,
    y: f32,
    width: f32,
    phase: f32,
}

#[derive(Clone, Copy)]
struct Pt {
    x: f32,
    y: f32,
    u: f32,
    r: f32,
    nx: f32,
    ny: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct HeadFrame {
    x: f32,
    y: f32,
    cos: f32,
    sin: f32,
    hs: f32,
    flip: f32,
}

pub struct DragonScene {
    rng: Rng,
    w: usize,
    h: usize,
    stars: Vec<Star>,
    clouds: Vec<Cloud>,
    body: Vec<f32>,
    body_col: Vec<Rgb>,
    depth: Vec<f32>,
}

impl DragonScene {
    pub fn new(seed: u64) -> Self {
        DragonScene {
            rng: Rng::new(seed ^ 0xd4a6_0f1e),
            w: 0,
            h: 0,
            stars: Vec::new(),
            clouds: Vec::new(),
            body: Vec::new(),
            body_col: Vec::new(),
            depth: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let cells = self.w * self.h;
        self.body = vec![0.0; cells];
        self.body_col = vec![Rgb::BLACK; cells];
        self.depth = vec![-1.0; cells];

        self.stars.clear();
        let count = (cells / 80).max(20);
        for _ in 0..count {
            self.stars.push(Star {
                x: self.rng.below(self.w),
                y: self.rng.below(self.h),
                phase: self.rng.range(0.0, TAU),
                bright: self.rng.range(0.12, 0.9),
                big: self.rng.chance(0.1),
            });
        }
        self.clouds.clear();
        // The first six are the old faint "plain" clouds; the extra ones
        // build the denser cloud sea used by the default backdrop.
        for _ in 0..6 {
            self.clouds.push(Cloud {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(self.h as f32 * 0.55, self.h as f32),
                width: self.rng.range(4.0, 12.0),
                phase: self.rng.range(0.0, TAU),
            });
        }
        let extra = (self.w / 8).clamp(10, 30);
        for _ in 0..extra {
            self.clouds.push(Cloud {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(self.h as f32 * 0.50, self.h as f32 * 0.94),
                width: self.rng.range(12.0, 30.0),
                phase: self.rng.range(0.0, TAU),
            });
        }
    }

    fn unit(&self) -> f32 {
        (self.w as f32 / 110.0)
            .min(self.h as f32 / 32.0)
            .clamp(0.42, 1.65)
    }

    fn head_pos(&self, t: f32) -> (f32, f32) {
        let w = self.w as f32;
        let h = self.h as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let ax = w * 0.33;
        let ay = h * 0.31;
        let a = t * 0.52;
        let b = t * 0.71 + 1.3;
        let am = 0.82 + 0.18 * (t * 0.13 + 0.4).sin();
        let x = cx + ax * am * ((a).sin() * 0.86 + 0.14 * (a * 1.7 + 0.5).sin());
        let y = cy + ay * am * ((b).sin() * 0.84 + 0.16 * (b * 1.31 + 0.8).sin());
        (x, y)
    }

    fn frame_at(&self, t: f32) -> (f32, f32, f32, f32) {
        let (x, y) = self.head_pos(t);
        let (xa, ya) = self.head_pos(t - 0.035);
        let (xb, yb) = self.head_pos(t + 0.035);
        let mut tx = xb - xa;
        let mut ty = (yb - ya) * 2.0;
        let len = (tx * tx + ty * ty).sqrt().max(1e-4);
        tx /= len;
        ty /= len;
        (x, y, tx, ty)
    }

    /// Lateral offset of the serpentine wave at body coordinate `u`
    /// (0 = head, 1 = tail).  The phase runs head → tail: crests move towards
    /// the tail as time advances.
    fn body_wave(&self, u: f32, t: f32) -> f32 {
        let env = WAVE_AMP * smoothstep(0.0, WAVE_RAMP, u) * (0.30 + 0.70 * u) * self.unit();
        env * (u * TAU * WAVE_CYCLES - t * WAVE_SPEED).sin()
    }

    fn build_body(&self, t: f32) -> Vec<Pt> {
        let unit = self.unit();
        // Keep a roughly constant body length in space: sample the flight
        // path backwards and resample it by arc length.  This avoids the
        // "accordion" effect of a pure time lag at Lissajous turning points.
        let body_len = (self.w as f32 * 1.20).clamp(90.0, 230.0);
        let step = 0.022;
        let mut path: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(1000);
        let (hx, hy) = self.head_pos(t);
        path.push((hx, hy, 0.0, t));
        let (mut prev_x, mut prev_y) = (hx, hy);
        let mut cum = 0.0;
        let mut tb = t;
        while cum < body_len + 4.0 && path.len() < 1000 {
            tb -= step;
            let (x, y) = self.head_pos(tb);
            let d = ((x - prev_x).powi(2) + ((y - prev_y) * 2.0).powi(2)).sqrt();
            cum += d;
            path.push((x, y, cum, tb));
            prev_x = x;
            prev_y = y;
        }
        let total = path.last().map(|p| p.2).unwrap_or(0.0);
        let maxd = body_len.min(total);
        let mut pts = Vec::with_capacity(SAMPLES);
        let mut j = 0usize;
        for i in 0..SAMPLES {
            let u = i as f32 / (SAMPLES - 1) as f32;
            let target = u * maxd;
            while j + 2 < path.len() && path[j + 1].2 < target {
                j += 1;
            }
            let a = path[j];
            let b = path[(j + 1).min(path.len() - 1)];
            let seg = (b.2 - a.2).max(1e-4);
            let f = ((target - a.2) / seg).clamp(0.0, 1.0);
            let x = a.0 + (b.0 - a.0) * f;
            let y = a.1 + (b.1 - a.1) * f;
            let tt = a.3 + (b.3 - a.3) * f;
            let (_, _, fx, fy) = self.frame_at(tt);
            // Shading normal points away from the screen centre.
            let mut nx = -fy;
            let mut ny = fx;
            let ox = x - self.w as f32 * 0.5;
            let oy = (y - self.h as f32 * 0.5) * 2.0;
            if nx * ox + ny * oy < 0.0 {
                nx = -nx;
                ny = -ny;
            }
            let r = unit * (1.0 + 2.6 * (1.0 - u).powf(1.32));
            pts.push(Pt {
                x,
                y,
                u,
                r,
                nx,
                ny,
                z: 1.0 - 0.55 * u,
            });
        }
        // Apply the serpentine wave as a second pass, using a tangent smoothed
        // over several neighbouring samples.  The raw path has a near-cusp
        // where its direction turns very fast; using the instantaneous normal
        // there would make the body jump between frames.
        let base: Vec<(f32, f32)> = pts.iter().map(|p| (p.x, p.y)).collect();
        let n = pts.len();
        let span = 6usize;
        for i in 0..n {
            let a = base[i.saturating_sub(span)];
            let b = base[(i + span).min(n - 1)];
            let mut tx = b.0 - a.0;
            let mut ty = (b.1 - a.1) * 2.0;
            let len = (tx * tx + ty * ty).sqrt().max(1e-4);
            tx /= len;
            ty /= len;
            let wave = self.body_wave(pts[i].u, t);
            pts[i].x += -ty * wave;
            pts[i].y += tx * wave * 0.5;
        }
        pts
    }

    fn splat_body(&mut self, pt: &Pt, color: Rgb) {
        let r = pt.r;
        let x0 = (pt.x - r).floor() as isize;
        let x1 = (pt.x + r).ceil() as isize;
        let y0 = (pt.y - r * 0.5).floor() as isize;
        let y1 = (pt.y + r * 0.5).ceil() as isize;
        for py in y0..=y1 {
            if py < 0 || py >= self.h as isize {
                continue;
            }
            for px in x0..=x1 {
                if px < 0 || px >= self.w as isize {
                    continue;
                }
                let dx = px as f32 + 0.5 - pt.x;
                let dyv = (py as f32 + 0.5 - pt.y) * 2.0;
                let d2 = (dx * dx + dyv * dyv) / (r * r);
                if d2 >= 1.0 {
                    continue;
                }
                let val = (1.0 - d2).sqrt();
                let idx = py as usize * self.w + px as usize;
                if pt.z + 0.01 < self.depth[idx] {
                    continue;
                }
                let side = (dx * pt.nx + dyv * pt.ny) / r.max(0.001);
                let mut col = color;
                if side > 0.18 {
                    col = col.lerp(GOLD, (side - 0.18) * 0.65);
                } else if side < -0.25 {
                    col = col.lerp(PALE_GOLD, (-side - 0.25) * 0.35);
                }
                if val > self.body[idx] || pt.z > self.depth[idx] + 0.04 {
                    self.body[idx] = val;
                    self.body_col[idx] = col;
                    self.depth[idx] = pt.z;
                }
            }
        }
    }

    fn draw_mountains(&self, canvas: &mut Canvas, theme: &Theme) {
        let h = self.h as f32;
        for layer in 0..2 {
            let base = h * (0.60 + layer as f32 * 0.080);
            let amp = h * (0.065 + layer as f32 * 0.030);
            let ridge = theme.c(1).scale(0.18 + layer as f32 * 0.060);
            let hatch = theme.c(2).scale(0.07 + layer as f32 * 0.025);
            let mut prev: Option<isize> = None;
            for x in 0..self.w {
                let xf = x as f32;
                let yf = base
                    - amp
                        * (0.55
                            + 0.45 * (xf * 0.040 + layer as f32 * 2.1).sin()
                            + 0.25 * (xf * 0.095 + layer as f32 * 1.3).sin());
                let y = yf.round().clamp(0.0, h - 1.0) as isize;
                let ch = match prev {
                    Some(p) if y > p => '\\',
                    Some(p) if y < p => '/',
                    _ => '_',
                };
                canvas.set(x as isize, y, ch, ridge);
                for dy in 1..=3 {
                    if (x as isize + dy) % 3 == 0 {
                        canvas.set_if_blank(
                            x as isize,
                            y + dy,
                            if dy == 1 { ':' } else { '.' },
                            hatch,
                        );
                    }
                }
                prev = Some(y);
            }
        }
    }

    fn draw_cloud_sea(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        let h = self.h as f32;
        let mist = theme.c(2).scale(0.085);
        // Long horizontal strata make the bottom read as a "cloud sea".
        for (k, frac) in [0.72f32, 0.80, 0.88, 0.95].iter().enumerate() {
            for x in (0..self.w).step_by(2) {
                let xf = x as f32;
                let yy = h * frac + ((xf * 0.07 + t * 0.10 + k as f32 * 1.7).sin() * 1.4);
                let ch = if (x / 2 + k) % 4 == 0 { '~' } else { '-' };
                canvas.set_if_blank(x as isize, yy.round() as isize, ch, mist);
            }
        }
        let base = theme.c(2).scale(0.23);
        for cloud in self.clouds.iter().skip(6) {
            let drift = (t * 0.11 + cloud.phase).sin() * 4.0;
            let x = cloud.x + drift;
            let y = cloud.y;
            let col = base.scale(0.82 + 0.30 * (cloud.phase.sin() * 0.5 + 0.5));
            let width = cloud.width as isize;
            for i in 0..width {
                let px = (x as isize + i - width / 2).clamp(0, self.w as isize - 1);
                let yy = y as isize + ((i as f32 * 0.55 + cloud.phase).sin() * 1.5) as isize;
                let ch = match i.rem_euclid(7) {
                    0 => '=',
                    1 | 2 => '-',
                    3 | 4 => '~',
                    _ => ':',
                };
                canvas.set_if_blank(px, yy, ch, col);
                if i % 2 == 0 {
                    canvas.set_if_blank(px, yy + 1, ':', col.scale(0.75));
                }
                if i % 3 == 0 {
                    canvas.set_if_blank(px, yy + 2, '~', col.scale(0.5));
                }
            }
        }
    }

    fn draw_clouds(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        for cloud in &self.clouds {
            let drift = (t * 0.18 + cloud.phase).sin() * 3.0;
            let x = cloud.x + drift;
            let y = cloud.y;
            let col = theme.c(2).scale(0.12);
            for i in 0..cloud.width as isize {
                let px = (x as isize + i - cloud.width as isize / 2).max(0);
                let yy = y as isize + ((i as f32 * 0.9 + cloud.phase).sin() * 1.2) as isize;
                if px < self.w as isize && yy >= 0 && yy < self.h as isize {
                    canvas.set_if_blank(px, yy, if i % 3 == 0 { '-' } else { '~' }, col);
                }
            }
        }
    }

    fn head_frame(&self, t: f32) -> HeadFrame {
        let (px, py, fx, fy) = self.frame_at(t);
        let unit = self.unit();
        let hs = unit * (2.08 + 0.02 * (t * 1.3).sin());
        let flip = if fx < 0.0 { -1.0 } else { 1.0 };
        let tilt = (fy / (fx.abs() + 3.2)).atan() * 0.55;
        let tilt = tilt.clamp(-0.30, 0.30);
        HeadFrame {
            x: px + fx * 2.6 * unit,
            y: py + fy * 2.6 * unit * 0.5,
            cos: tilt.cos(),
            sin: tilt.sin(),
            hs,
            flip,
        }
    }

    fn head_map(&self, hf: &HeadFrame, lx: f32, ly: f32) -> (isize, isize) {
        let lx = lx * hf.flip;
        let wx = hf.x + (lx * hf.cos - ly * hf.sin) * hf.hs;
        let wy = hf.y * 2.0 + (lx * hf.sin + ly * hf.cos) * hf.hs;
        (wx.round() as isize, (wy / 2.0).round() as isize)
    }

    #[allow(clippy::too_many_arguments)]
    fn head_fill(
        &self,
        canvas: &mut Canvas,
        hf: &HeadFrame,
        ex: f32,
        ey: f32,
        rx: f32,
        ry: f32,
        top: Rgb,
        bottom: Rgb,
    ) {
        let reach = (rx.max(ry) * hf.hs).ceil() as isize + 2;
        let cx = hf.x;
        let cy = hf.y;
        for py in (cy as isize - reach)..=(cy as isize + reach) {
            if py < 0 || py >= self.h as isize {
                continue;
            }
            for px in (cx as isize - reach)..=(cx as isize + reach) {
                if px < 0 || px >= self.w as isize {
                    continue;
                }
                let dx = px as f32 + 0.5 - hf.x;
                let dyv = (py as f32 + 0.5 - hf.y) * 2.0;
                let lx = (dx * hf.cos + dyv * hf.sin) / hf.hs * hf.flip;
                let ly = (-dx * hf.sin + dyv * hf.cos) / hf.hs;
                let nx = (lx - ex) / rx;
                let ny = (ly - ey) / ry;
                let d2 = nx * nx + ny * ny;
                if d2 >= 1.0 {
                    continue;
                }
                let val = (1.0 - d2).sqrt();
                let idx = ((val.powf(0.55) * (BODY_RAMP.len() - 1) as f32).round() as usize)
                    .min(BODY_RAMP.len() - 1);
                if idx == 0 {
                    continue;
                }
                let vertical = ((ly + ry) / (2.0 * ry)).clamp(0.0, 1.0);
                let col = top.lerp(bottom, vertical).scale(0.72 + 0.35 * val);
                let (ch, col) = if val < 0.26 {
                    ('+', PALE_GOLD)
                } else if val < 0.48 {
                    ('*', col)
                } else {
                    (BODY_RAMP[idx] as char, col)
                };
                canvas.set(px, py, ch, col);
            }
        }
    }

    fn head_line(
        &self,
        canvas: &mut Canvas,
        hf: &HeadFrame,
        pts: &[(f32, f32)],
        color: Rgb,
        char_: char,
        thick: f32,
    ) {
        for w in pts.windows(2) {
            let (ax, ay) = w[0];
            let (bx, by) = w[1];
            let steps = (((bx - ax).abs() + (by - ay).abs()) * hf.hs * 2.0) as usize + 1;
            for i in 0..=steps {
                let s = i as f32 / steps as f32;
                let lx = ax + (bx - ax) * s;
                let ly = ay + (by - ay) * s;
                let (x, y) = self.head_map(hf, lx, ly);
                canvas.set(x, y, char_, color);
                if thick > 0.1 {
                    let (x2, y2) = self.head_map(hf, lx, ly + thick);
                    canvas.set(x2, y2, '.', color.scale(0.75));
                }
            }
        }
    }

    fn draw_head(&self, canvas: &mut Canvas, t: f32) {
        let hf = self.head_frame(t);
        // Neck, skull, cheek, muzzle, brow, lower jaw.
        self.head_fill(canvas, &hf, -3.1, 0.5, 2.0, 1.7, EMERALD, TEAL);
        self.head_fill(canvas, &hf, -0.9, -0.1, 3.0, 2.2, PALE_GOLD, GOLD);
        self.head_fill(canvas, &hf, 0.4, 0.9, 2.4, 1.25, GOLD, DEEP_GOLD);
        self.head_fill(canvas, &hf, 2.6, 0.1, 2.8, 1.15, PALE_GOLD, GOLD);
        self.head_fill(canvas, &hf, 4.9, -0.1, 0.95, 0.8, PALE_GOLD, GOLD);
        self.head_fill(canvas, &hf, 0.4, -1.4, 2.2, 0.7, PALE_GOLD, GOLD);
        self.head_fill(canvas, &hf, 2.6, 1.55, 2.6, 0.58, JADE, TEAL);

        // Mouth slit and fangs.
        let mut lx = 1.25;
        while lx <= 5.7 {
            let (x, y) = self.head_map(&hf, lx, 1.05);
            if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
                canvas.set(x, y, ' ', Rgb::BLACK);
            }
            lx += 0.12;
        }
        for (i, tx) in [1.9f32, 2.8, 3.7, 4.6].iter().enumerate() {
            let (x, y) = self.head_map(&hf, *tx, 1.02);
            let _ = i;
            canvas.set(x, y, if i % 2 == 0 { 'v' } else { '\'' }, WHITE);
        }

        // Eye socket and eye.
        for (lx, ly) in [(0.55, -1.0), (1.25, -1.0), (0.55, -0.35), (1.25, -0.35)] {
            let (x, y) = self.head_map(&hf, lx, ly);
            canvas.set(x, y, ' ', Rgb::BLACK);
        }
        let (ex, ey) = self.head_map(&hf, 0.95, -0.68);
        canvas.set(ex, ey, '@', WHITE);
        let (px, py) = self.head_map(&hf, 1.35, -0.68);
        canvas.set(px, py, 'o', RED);
        canvas.add_light(ex, ey, GOLD, 1.1, '.');
        let (bx, by) = self.head_map(&hf, 0.5, -1.3);
        canvas.set(bx, by, '-', PALE_GOLD);

        // Nostrils.
        for ly in [-0.2f32, 0.4] {
            let (x, y) = self.head_map(&hf, 5.25, ly);
            canvas.set(x, y, 'o', DEEP_GOLD);
        }

        // Antler horns: explicit thick segments so the silhouette reads.
        let horns: [[(f32, f32); 3]; 2] = [
            [(-0.5, -1.4), (-2.0, -3.7), (-3.8, -5.9)],
            [(-1.5, -0.9), (-3.4, -2.8), (-5.4, -4.0)],
        ];
        for (hi, seg) in horns.iter().enumerate() {
            for w in seg.windows(2) {
                let (x1, y1) = self.head_map(&hf, w[0].0, w[0].1);
                let (x2, y2) = self.head_map(&hf, w[1].0, w[1].1);
                canvas.line(x1, y1, x2, y2, '*', if hi == 0 { WHITE } else { PALE_GOLD });
                canvas.line(x1, y1 + 1, x2, y2 + 1, '.', GOLD.scale(0.7));
            }
            let tip = seg[2];
            let (tx, ty) = self.head_map(&hf, tip.0, tip.1);
            canvas.set(tx, ty, '@', WHITE);
            // tines
            let (bx, by) = self.head_map(&hf, seg[1].0, seg[1].1);
            let (ux, uy) = self.head_map(&hf, seg[1].0 + 1.3, seg[1].1 - 2.6);
            canvas.line(bx, by, ux, uy, '*', WHITE);
            let (vx, vy) = self.head_map(&hf, seg[1].0 - 1.6, seg[1].1 - 1.4);
            canvas.line(bx, by, vx, vy, '.', PALE_GOLD);
        }

        // Fiery mane behind the head.
        for k in 0..7 {
            let kf = k as f32;
            let wob = (t * 2.2 + kf * 1.3).sin() * 0.7;
            let (x0, y0) = self.head_map(&hf, -2.35, -0.75 + kf * 0.34);
            let (x1, y1) = self.head_map(
                &hf,
                -5.2 - (kf * 0.5).sin().abs() * 1.4,
                -1.6 + kf * 0.55 + wob,
            );
            let col = if k % 3 == 0 {
                RED
            } else if k % 3 == 1 {
                GOLD
            } else {
                JADE
            };
            canvas.line(x0, y0, x1, y1, if k % 2 == 0 { '*' } else { '~' }, col);
            canvas.line(x0, y0 + 1, x1, y1 + 1, '.', col.scale(0.6));
        }

        // Two long whiskers.
        for sign in [-1.0f32, 1.0] {
            let wob = (t * 2.4 + sign).sin() * 0.7;
            let mut pts = Vec::new();
            for i in 0..=22 {
                let s = i as f32 / 22.0;
                let lx = 4.8 + s * 4.8;
                let ly =
                    sign * (0.2 + s * (1.7 + wob * 0.7)) + (s * 6.5 + t * 2.7).sin() * 0.75 * s;
                pts.push((lx, ly));
            }
            self.head_line(
                canvas,
                &hf,
                &pts,
                PALE_GOLD,
                if sign > 0.0 { '~' } else { '-' },
                0.0,
            );
        }

        // Beard.
        for k in 0..4 {
            let kf = k as f32;
            let mut pts = Vec::new();
            for i in 0..=8 {
                let s = i as f32 / 8.0;
                pts.push((
                    2.0 + kf * 0.7 + s * 0.8,
                    1.7 + s * (1.8 + (t * 2.0 + kf).sin() * 0.3),
                ));
            }
            self.head_line(
                canvas,
                &hf,
                &pts,
                if k % 2 == 0 { RED } else { GOLD },
                '.',
                0.0,
            );
        }

        // A warm halo around the head.
        let cx = hf.x.round() as isize;
        let cy = hf.y.round() as isize;
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-2, 0), (2, 0)] {
            canvas.add_light(cx + dx, cy + dy, GOLD, 0.16, '.');
        }
    }

    fn draw_legs(&self, canvas: &mut Canvas, body: &[Pt], t: f32) {
        for (i, u) in [0.16f32, 0.32, 0.53, 0.69].iter().enumerate() {
            let idx = ((u * (body.len() - 1) as f32) as usize).min(body.len() - 1);
            let pt = body[idx];
            if pt.u > 0.9 {
                continue;
            }
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let side = if pt.ny * sign > 0.0 { 1.0 } else { -1.0 };
            let mut dx = pt.nx * side;
            let mut dy = pt.ny * side + 0.9;
            let l = (dx * dx + dy * dy).sqrt().max(1e-4);
            dx /= l;
            dy /= l;
            let unit = self.unit();
            let swing = (t * 2.7 + i as f32 * 1.7).sin() * 0.45;
            let bx = pt.x + pt.nx * sign * pt.r * 0.45;
            let by = pt.y + pt.ny * sign * pt.r * 0.22;
            let kx = bx + dx * 2.1 * unit - pt.ny * 0.8 * unit + swing * 0.6;
            let ky = by + dy * 2.1 * unit + pt.nx * 0.35 * unit;
            let fx = kx + dx * 2.4 * unit + pt.ny * 0.6 * unit;
            let fy = ky + dy * 2.4 * unit;
            let col = JADE.lerp(GOLD, 0.35);
            let col2 = GOLD;
            canvas.line(
                bx.round() as isize,
                by.round() as isize,
                kx.round() as isize,
                ky.round() as isize,
                '\\',
                col,
            );
            canvas.line(
                kx.round() as isize,
                ky.round() as isize,
                fx.round() as isize,
                fy.round() as isize,
                '/',
                col,
            );
            canvas.line(
                (bx + 1.0).round() as isize,
                by.round() as isize,
                (kx + 1.0).round() as isize,
                ky.round() as isize,
                '.',
                col.scale(0.6),
            );
            for k in -1i32..=1 {
                let spread = k as f32 * 0.7;
                let ex = fx + dx * 1.1 * unit + pt.ny * spread;
                let ey = fy + dy * 1.1 * unit - pt.nx * spread * 0.5;
                canvas.line(
                    fx.round() as isize,
                    fy.round() as isize,
                    ex.round() as isize,
                    ey.round() as isize,
                    if k == 0 { '^' } else { '.' },
                    col2,
                );
            }
        }
    }

    fn draw_tail(&self, canvas: &mut Canvas, body: &[Pt]) {
        let last = body[body.len() - 1];
        let prev = body[body.len() - 2];
        let dx = last.x - prev.x;
        let dyv = (last.y - prev.y) * 2.0;
        let len = (dx * dx + dyv * dyv).sqrt().max(1e-4);
        let tx = dx / len;
        let ty = dyv / len;
        let unit = self.unit();
        for k in -3i32..=3 {
            let spread = k as f32 * 0.28;
            let fl = (2.8 + (3 - k.unsigned_abs()) as f32 * 1.15) * unit;
            let ex = last.x - (tx - spread * ty) * fl;
            let ey = last.y - (ty + spread * tx) * fl / 2.0;
            let col = if k % 2 == 0 { GOLD } else { RED };
            canvas.line(
                last.x.round() as isize,
                last.y.round() as isize,
                ex.round() as isize,
                ey.round() as isize,
                if k == 0 { '*' } else { '.' },
                col.scale(0.85),
            );
        }
    }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl Scene for DragonScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {
        // Flight is analytic: t drives the head and the trailing body.
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        self.body.fill(0.0);
        self.depth.fill(-1.0);

        for s in &self.stars {
            let tw = 0.35 + 0.65 * (t * 1.4 + s.phase).sin().abs();
            let col = theme.c(2).scale(s.bright * tw * 0.32);
            canvas.set_if_blank(
                s.x as isize,
                s.y as isize,
                if s.big { '+' } else { '.' },
                col,
            );
        }
        if f.tune.plain {
            self.draw_clouds(canvas, theme, t);
        } else {
            self.draw_mountains(canvas, theme);
            self.draw_cloud_sea(canvas, theme, t);
        }

        // Compute and splat the trailing body, tail first.
        let body = self.build_body(t);
        for pt in body.iter().rev() {
            let base = if pt.u < 0.10 {
                EMERALD.lerp(JADE, 0.4 + (0.10 - pt.u) * 3.0)
            } else if pt.u < 0.38 {
                let k = (pt.u - 0.10) / 0.28;
                GOLD.lerp(JADE, k)
            } else if pt.u < 0.78 {
                let k = (pt.u - 0.38) / 0.40;
                JADE.lerp(EMERALD, k)
            } else {
                let k = (pt.u - 0.78) / 0.22;
                EMERALD.lerp(TEAL, k)
            };
            let band = 0.78 + 0.22 * (pt.u * 135.0 - t * 3.2).sin().abs();
            let depth = 0.62 + 0.38 * (1.0 - pt.u);
            let col = base.scale(band * depth);
            self.splat_body(pt, col);
        }

        // Dense scaled body.
        for y in 0..self.h {
            for x in 0..self.w {
                let idx = y * self.w + x;
                let v = self.body[idx];
                if v < 0.02 {
                    continue;
                }
                let ri = ((v.powf(0.8) * (BODY_RAMP.len() - 1) as f32).round() as usize)
                    .min(BODY_RAMP.len() - 1);
                if ri == 0 {
                    continue;
                }
                canvas.set(
                    x as isize,
                    y as isize,
                    BODY_RAMP[ri] as char,
                    self.body_col[idx],
                );
            }
        }

        // Dorsal spines.
        for (i, pt) in body.iter().enumerate() {
            if i % 9 != 0 || pt.u < 0.05 || pt.u > 0.9 {
                continue;
            }
            let len = (1.2 + (1.0 - pt.u) * 2.6) * self.unit();
            let bx = pt.x.round() as isize;
            let by = pt.y.round() as isize;
            for s in 1..=len.round() as isize {
                let x = bx + (pt.nx * s as f32).round() as isize;
                let y = by + (pt.ny * s as f32 / 2.0).round() as isize;
                let col = GOLD.scale(0.55 + 0.35 * (1.0 - pt.u));
                canvas.set(x, y, if s == 1 { '*' } else { '.' }, col);
            }
            let tx = bx + (pt.nx * len).round() as isize;
            let ty = by + (pt.ny * len / 2.0).round() as isize;
            canvas.set(tx, ty, '^', RED.scale(0.9));
        }

        self.draw_legs(canvas, &body, t);
        self.draw_tail(canvas, &body);
        self.draw_head(canvas, t);

        // A thin golden rim along the outer body edge.
        for (i, pt) in body.iter().enumerate() {
            if i % 2 != 0 {
                continue;
            }
            let x = pt.x + pt.nx * pt.r * 0.88;
            let y = pt.y + pt.ny * pt.r * 0.44;
            let col = GOLD.scale(0.25 + 0.25 * (1.0 - pt.u));
            canvas.add_light(x.round() as isize, y.round() as isize, col, 0.35, '.');
        }

        // Small HUD.
        if self.w >= 62 {
            let title = ":: SHENLONG // CELESTIAL DRAGON ::";
            canvas.put_str_gradient(2, 0, title, &[JADE, GOLD, PALE_GOLD]);
            let right = "CLOUD SEA // FLIGHT NOMINAL";
            let rx = self.w as isize - right.len() as isize - 2;
            if rx > 32 {
                canvas.put_str(rx, 0, right, theme.c(2).scale(0.75));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    fn dragon() -> DragonScene {
        let mut scene = DragonScene::new(7);
        scene.resize(80, 24);
        scene
    }

    fn frame(plain: bool) -> crate::scene::Frame<'static> {
        let tune = crate::scene::Tune {
            plain,
            ..Default::default()
        };
        crate::scene::Frame {
            t: 3.0,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune,
        }
    }

    #[test]
    fn backdrop_adds_ink() {
        let mut scene = DragonScene::new(7);
        scene.resize(120, 36);
        let mut plain = Canvas::new(120, 36);
        scene.draw(&mut plain, &frame(true));
        let mut rich = Canvas::new(120, 36);
        scene.draw(&mut rich, &frame(false));
        assert!(
            rich.ink() > plain.ink() + 80,
            "plain={} rich={}",
            plain.ink(),
            rich.ink()
        );
    }

    #[test]
    fn wave_leaves_the_head_steady() {
        let scene = dragon();
        for i in 0..900 {
            let t = i as f32 / 30.0;
            assert_eq!(scene.body_wave(0.0, t), 0.0);
            let (hx, hy) = scene.head_pos(t);
            let body = scene.build_body(t);
            assert!((body[0].x - hx).abs() < 1e-3, "x at t={t}");
            assert!((body[0].y - hy).abs() < 1e-3, "y at t={t}");
        }
    }

    #[test]
    fn wave_grows_towards_the_tail() {
        let scene = dragon();
        let mut neck = 0.0;
        let mut mid = 0.0;
        let mut tail = 0.0;
        for i in 0..900 {
            let t = i as f32 / 30.0;
            neck += scene.body_wave(0.08, t).abs();
            mid += scene.body_wave(0.50, t).abs();
            tail += scene.body_wave(0.92, t).abs();
        }
        assert!(
            tail > mid && mid > neck,
            "neck={neck} mid={mid} tail={tail}"
        );
        assert!(tail / 900.0 > scene.unit() * 1.2, "tail wave too flat");
    }

    #[test]
    fn coiled_body_stays_on_canvas() {
        for (w, h) in [(80usize, 24usize), (120, 36), (160, 48)] {
            let mut scene = DragonScene::new(7);
            scene.resize(w, h);
            for i in 0..1500 {
                let t = i as f32 / 30.0;
                for pt in scene.build_body(t) {
                    assert!(pt.x.is_finite() && pt.y.is_finite() && pt.r.is_finite());
                    assert!((0.0..=w as f32).contains(&pt.x), "x={} w={w} t={t}", pt.x);
                    assert!((0.0..=h as f32).contains(&pt.y), "y={} h={h} t={t}", pt.y);
                }
            }
        }
    }

    #[test]
    fn body_motion_is_smooth() {
        let scene = dragon();
        let mut prev: Option<Vec<Pt>> = None;
        let mut max_step = 0.0f32;
        let mut max_at = (0.0f32, 0usize);
        for i in 0..3600 {
            let t = i as f32 / 60.0;
            let body = scene.build_body(t);
            if let Some(old) = &prev {
                for (k, (a, b)) in old.iter().zip(body.iter()).enumerate() {
                    let d = ((a.x - b.x).powi(2) + ((a.y - b.y) * 2.0).powi(2)).sqrt();
                    if d > max_step {
                        max_step = d;
                        max_at = (t, k);
                    }
                }
            }
            prev = Some(body);
        }
        assert!(
            max_step < 1.8,
            "max body step per frame: {max_step} at t={} u={}",
            max_at.0,
            max_at.1 as f32 / 249.0
        );
    }
}
