//! 巨龙盘旋 — a procedural dragon coiling around a plasma core.
//!
//! The dragon is a 3D-ish elliptical coil projected to the character grid,
//! splatted into a small intensity buffer so the tube reads as a solid
//! serpent.  Head, horns, whiskers, legs, dorsal spines and a fire breath
//! are drawn on top.  The core is a shaded dot-matrix sphere with rings.

use std::f32::consts::{PI, TAU};

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash2, value_noise, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const BODY_RAMP: &[u8] = b" .:-=+*#%@";
const SPAN: f32 = 1.75 * PI;
const SAMPLES: usize = 190;

struct Star {
    x: usize,
    y: usize,
    phase: f32,
    bright: f32,
    big: bool,
}

struct Petal {
    angle: f32,
    radius: f32,
    speed: f32,
    phase: f32,
}

struct Breath {
    /// Column units.
    x: f32,
    /// Visual units (two per terminal row).
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
}

#[derive(Clone, Copy)]
struct Geom {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    unit: f32,
    orb_r: f32,
    head_angle: f32,
}

#[derive(Clone, Copy)]
struct RawPt {
    x: f32,
    y: f32,
    z: f32,
    u: f32,
}

#[derive(Clone, Copy)]
struct PathPt {
    x: f32,
    y: f32,
    z: f32,
    u: f32,
    r: f32,
    nx: f32,
    ny: f32,
}

#[derive(Clone, Copy)]
struct HeadFrame {
    x: f32,
    y: f32,
    z: f32,
    cos: f32,
    sin: f32,
    hs: f32,
    /// -1 mirrors the head so it can face back toward the core.
    flip: f32,
}

pub struct DragonScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    stars: Vec<Star>,
    petals: Vec<Petal>,
    body: Vec<f32>,
    body_col: Vec<Rgb>,
    depth: Vec<f32>,
    glow: Vec<f32>,
    breath: Vec<Breath>,
    breath_t: f32,
    next_breath: f32,
}

impl DragonScene {
    pub fn new(seed: u64) -> Self {
        DragonScene {
            seed,
            rng: Rng::new(seed ^ 0xd4a6_0f1e),
            w: 0,
            h: 0,
            stars: Vec::new(),
            petals: Vec::new(),
            body: Vec::new(),
            body_col: Vec::new(),
            depth: Vec::new(),
            glow: Vec::new(),
            breath: Vec::new(),
            breath_t: 0.0,
            next_breath: 3.5,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let cells = self.w * self.h;
        self.body = vec![0.0; cells];
        self.body_col = vec![Rgb::BLACK; cells];
        self.depth = vec![-2.0; cells];
        self.glow = vec![0.0; cells];
        self.breath.clear();

        self.stars.clear();
        let count = (cells / 100).max(16);
        for _ in 0..count {
            self.stars.push(Star {
                x: self.rng.below(self.w),
                y: self.rng.below(self.h),
                phase: self.rng.range(0.0, TAU),
                bright: self.rng.range(0.12, 0.9),
                big: self.rng.chance(0.1),
            });
        }
        self.petals.clear();
        for _ in 0..22 {
            self.petals.push(Petal {
                angle: self.rng.range(0.0, TAU),
                radius: self.rng.range(1.35, 2.5),
                speed: self.rng.range(-0.5, 0.5),
                phase: self.rng.range(0.0, TAU),
            });
        }
    }

    fn geom(&self, t: f32) -> Geom {
        let cx = self.w as f32 * 0.5 - 0.5;
        let cy = self.h as f32 * 0.5 - 0.5;
        let unit = (self.w as f32 / 100.0).clamp(0.62, 1.7);
        let mut rx = (self.w as f32 * 0.30)
            .min((self.h as f32 - 3.0) * 1.9)
            .max(5.0);
        let mut ry = (rx * 0.32).max(1.8);
        if ry > self.h as f32 * 0.5 - 1.8 {
            ry = (self.h as f32 * 0.5 - 1.8).max(1.2);
            rx = rx.min(ry * 4.2);
        }
        rx = rx.max(4.0);
        let orb_r = (rx * 0.23).min(ry * 1.75).max(2.8);
        Geom {
            cx,
            cy,
            rx,
            ry,
            unit,
            orb_r,
            head_angle: 0.55 + 0.06 * (t * 0.3).sin(),
        }
    }

    fn raw_point(&self, g: &Geom, t: f32, u: f32) -> RawPt {
        let theta = g.head_angle - u * SPAN;
        let radial = 1.0 - 0.62 * u + 0.04 * (u * TAU * 2.0 + t * 1.3).sin();
        let x = g.cx + theta.cos() * g.rx * radial;
        let y = g.cy + theta.sin() * g.ry + (u * TAU * 2.5 + t * 2.1).sin() * (2.2 - 1.4 * u);
        let z = theta.sin() * 0.9 + 0.14 * (u * TAU * 3.0 - t * 1.7).sin();
        RawPt { x, y, z, u }
    }

    fn build_path(&self, g: &Geom, t: f32) -> Vec<PathPt> {
        let mut raw = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let u = i as f32 / (SAMPLES - 1) as f32;
            raw.push(self.raw_point(g, t, u));
        }
        let mut pts = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let i0 = i.saturating_sub(1);
            let i1 = (i + 1).min(SAMPLES - 1);
            let a = raw[i0];
            let b = raw[i1];
            // visual-space tangent: x is columns, y is 2x rows
            let mut tx = b.x - a.x;
            let mut ty = (b.y - a.y) * 2.0;
            let len = (tx * tx + ty * ty).sqrt().max(1e-4);
            tx /= len;
            ty /= len;
            let mut nx = -ty;
            let mut ny = tx;
            let p = raw[i];
            let ox = p.x - g.cx;
            let oy = (p.y - g.cy) * 2.0;
            if nx * ox + ny * oy < 0.0 {
                nx = -nx;
                ny = -ny;
            }
            let r = g.unit * (0.6 + 2.0 * (1.0 - p.u).powf(1.7));
            pts.push(PathPt {
                x: p.x,
                y: p.y,
                z: p.z,
                u: p.u,
                r,
                nx,
                ny,
            });
        }
        pts
    }

    fn head_frame(&self, g: &Geom, t: f32) -> HeadFrame {
        let p0 = self.raw_point(g, t, 0.0);
        // The head stays oriented toward the viewer's right, with a small
        // breathing tilt, so its silhouette remains readable while the body
        // coils around the core.
        let angle = 0.42 + 0.07 * (t * 1.15).sin();
        let fx = angle.cos();
        let fy = angle.sin();
        let bob = (t * 1.1).sin() * 0.35;
        let hs = g.unit * (1.7 + 0.03 * (t * 1.4).sin());
        HeadFrame {
            // The dragon's head turns back toward the pearl at the centre.
            x: p0.x - fx * 0.9 * g.unit,
            y: p0.y - fy * 0.45 * g.unit / 2.0 + bob,
            z: p0.z,
            cos: fx,
            sin: fy,
            hs,
            flip: -1.0,
        }
    }

    fn orb_occludes(&self, g: &Geom, x: f32, y: f32, z: f32) -> bool {
        let dx = x - g.cx;
        let dy = (y - g.cy) * 2.0;
        let d2 = dx * dx + dy * dy;
        if d2 >= g.orb_r * g.orb_r {
            return false;
        }
        let oz = (g.orb_r * g.orb_r - d2).sqrt() / g.orb_r;
        z < oz * 0.92
    }

    fn splat_body(&mut self, g: &Geom, pt: &PathPt, color: Rgb, theme: &Theme) {
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
                let dx = px as f32 - pt.x;
                let dyv = (py as f32 - pt.y) * 2.0;
                let d = (dx * dx + dyv * dyv).sqrt();
                if d <= r {
                    if self.orb_occludes(g, px as f32, py as f32, pt.z) {
                        continue;
                    }
                    let t = 1.0 - d / r;
                    let mut val = t.sqrt();
                    let idx = py as usize * self.w + px as usize;
                    // Dorsal ridge highlight on the outer flank.
                    let side = (dx * pt.nx + dyv * pt.ny) / r.max(0.001);
                    let mut col = color;
                    if side > 0.35 {
                        let k = ((side - 0.35) / 0.65).clamp(0.0, 1.0);
                        col = col.lerp(theme.hot(), 0.35 * k);
                        val *= 1.0 + 0.1 * k;
                    }
                    if pt.z + 0.02 >= self.depth[idx] && val > self.body[idx] {
                        self.body[idx] = val;
                        self.body_col[idx] = col;
                        self.depth[idx] = pt.z;
                    }
                }
                // Soft halo.
                let gr = r * 1.9;
                if d <= gr {
                    let gv = ((1.0 - d / gr).powf(2.2) * 0.32).min(1.0);
                    let idx = py as usize * self.w + px as usize;
                    if gv > self.glow[idx] {
                        self.glow[idx] = gv;
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn splat_ellipse(
        &mut self,
        g: &Geom,
        hf: &HeadFrame,
        ex: f32,
        ey: f32,
        rx: f32,
        ry: f32,
        color: Rgb,
        z: f32,
    ) {
        let hs = hf.hs;
        let bound = (rx.max(ry) * hs).ceil() as isize + 1;
        let ox = hf.x;
        let oy = hf.y * 2.0;
        let px0 = (ox - bound as f32) as isize;
        let px1 = (ox + bound as f32) as isize;
        let py0 = ((oy - bound as f32) / 2.0) as isize;
        let py1 = ((oy + bound as f32) / 2.0) as isize;
        for py in py0..=py1 {
            if py < 0 || py >= self.h as isize {
                continue;
            }
            for px in px0..=px1 {
                if px < 0 || px >= self.w as isize {
                    continue;
                }
                let dx = px as f32 - hf.x;
                let dyv = (py as f32 - hf.y) * 2.0;
                // inverse rotation, then into head-local units
                let lx = (dx * hf.cos + dyv * hf.sin) / hs * hf.flip;
                let ly = (-dx * hf.sin + dyv * hf.cos) / hs;
                let nx = (lx - ex) / rx;
                let ny = (ly - ey) / ry;
                let d2 = nx * nx + ny * ny;
                if d2 < 1.0 {
                    if self.orb_occludes(g, px as f32, py as f32, z) {
                        continue;
                    }
                    let val = (1.0 - d2).sqrt();
                    let idx = py as usize * self.w + px as usize;
                    if z + 0.01 >= self.depth[idx]
                        && (val > self.body[idx] * 0.55 || z > self.depth[idx] + 0.05)
                    {
                        self.body[idx] = val.max(0.25);
                        self.body_col[idx] = color;
                        self.depth[idx] = z;
                    }
                }
            }
        }
    }

    fn splat_head(&mut self, g: &Geom, hf: &HeadFrame, theme: &Theme) {
        let z = (hf.z + 0.25).min(1.0);
        let depth = 0.55 + 0.45 * ((z + 1.0) * 0.5).clamp(0.0, 1.0);
        let gold = theme.c(3).lerp(theme.hot(), 0.35).scale(depth);
        let pale = theme.c(3).lerp(theme.hot(), 0.8).scale(depth);
        let jade = theme.c(2).scale(depth * 0.85);
        // neck, skull, cheek, snout, brow and an open lower jaw
        self.splat_ellipse(g, hf, -2.6, 0.4, 2.5, 1.9, jade, z - 0.03);
        self.splat_ellipse(g, hf, 0.1, -0.1, 3.0, 1.95, gold, z);
        self.splat_ellipse(g, hf, 0.7, 0.7, 2.3, 1.0, gold, z - 0.01);
        self.splat_ellipse(g, hf, 2.9, 0.0, 2.4, 1.0, gold, z + 0.01);
        self.splat_ellipse(g, hf, 4.3, -0.25, 1.5, 0.75, pale, z + 0.02);
        self.splat_ellipse(g, hf, 0.4, -1.35, 1.8, 0.55, pale, z + 0.01);
        self.splat_ellipse(g, hf, 2.9, 1.7, 2.2, 0.5, jade, z + 0.015);

        // A dark mouth slit separates the jaws and reads even at a glance.
        let mut s = 0.0;
        while s <= 1.0 {
            let lx = 1.7 + s * 3.3;
            let ly = 1.02 + s * 0.1;
            let (x, y) = head_map(hf, lx, ly);
            if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
                let idx = y as usize * self.w + x as usize;
                self.body[idx] = 0.0;
                self.glow[idx] = 0.0;
            }
            s += 0.06;
        }
    }
}

fn head_map(hf: &HeadFrame, lx: f32, ly: f32) -> (isize, isize) {
    let lx = lx * hf.flip;
    let wx = hf.x + (lx * hf.cos - ly * hf.sin) * hf.hs;
    let wy = hf.y * 2.0 + (lx * hf.sin + ly * hf.cos) * hf.hs;
    (wx.round() as isize, (wy / 2.0).round() as isize)
}

impl Scene for DragonScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let t = f.t as f32;
        let g = self.geom(t);

        for p in self.petals.iter_mut() {
            p.angle += p.speed * dt;
        }

        if t > self.next_breath {
            self.breath_t = 1.8;
            self.next_breath = t + self.rng.range(7.0, 12.0);
        }
        if self.breath_t > 0.0 {
            self.breath_t -= dt;
            let hf = self.head_frame(&g, t);
            let mouth = head_map(&hf, 4.6, 0.15);
            let fx = hf.cos * hf.flip;
            let fy = hf.sin * hf.flip;
            for _ in 0..7 {
                if self.breath.len() >= 420 {
                    break;
                }
                let spread = self.rng.range(-0.65, 0.65);
                let speed = self.rng.range(16.0, 38.0);
                let life = self.rng.range(0.6, 1.35);
                let dx = fx * (1.0 - spread.abs() * 0.3) - fy * spread;
                let dy = fy * (1.0 - spread.abs() * 0.3) + fx * spread;
                self.breath.push(Breath {
                    x: mouth.0 as f32,
                    y: mouth.1 as f32 * 2.0,
                    vx: dx * speed,
                    vy: dy * speed,
                    life,
                    max_life: life,
                });
            }
        }
        for b in self.breath.iter_mut() {
            b.life -= dt;
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.vx *= 0.985;
            b.vy = b.vy * 0.985 - 2.0 * dt;
        }
        self.breath
            .retain(|b| b.life > 0.0 && b.x > -4.0 && b.x < self.w as f32 + 4.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let g = self.geom(t);

        // Clear the per-pixel layers.
        self.body.fill(0.0);
        self.depth.fill(-2.0);
        self.glow.fill(0.0);

        // Deep space.
        for s in &self.stars {
            let tw = 0.35 + 0.65 * (t * 1.4 + s.phase).sin().abs();
            let col = theme.c(2).scale(s.bright * tw * 0.30);
            canvas.set_if_blank(
                s.x as isize,
                s.y as isize,
                if s.big { '+' } else { '.' },
                col,
            );
        }

        // Orbital dust.
        for p in &self.petals {
            let rad = g.orb_r * p.radius;
            let x = g.cx + p.angle.cos() * rad;
            let y = g.cy + p.angle.sin() * rad * 0.45;
            let tw = 0.4 + 0.6 * (t * 2.0 + p.phase).sin().abs();
            canvas.blend(
                x.round() as isize,
                y.round() as isize,
                '.',
                theme.c(2).scale(0.5 * tw),
                0.5,
            );
        }

        // Plasma core.
        self.draw_core(canvas, &g, theme, t);

        // Build and splat the body.
        let path = self.build_path(&g, t);
        for pt in &path {
            let z01 = ((pt.z + 1.0) * 0.5).clamp(0.0, 1.0);
            let depth = 0.4 + 0.6 * z01;
            let base = if pt.u < 0.3 {
                let k = pt.u / 0.3;
                theme.c(2).lerp(theme.c(1), k * 0.55)
            } else {
                let k = ((pt.u - 0.3) / 0.7).clamp(0.0, 1.0);
                theme.c(2).lerp(theme.c(0), k * 0.9)
            };
            let shimmer = 0.78 + 0.22 * (pt.u * 95.0 - t * 7.0).sin().abs();
            let tail = if pt.u > 0.84 {
                1.0 - (pt.u - 0.84) / 0.16 * 0.65
            } else {
                1.0
            };
            let mut color = base.scale(depth * shimmer * tail);
            // Scale bands catch the light along the serpent's back.
            let band = (pt.u * 120.0 - t * 2.0).sin() * (pt.u * 120.0 - t * 2.0).cos();
            if band > 0.55 {
                color = color.lerp(theme.c(4), 0.16);
            }
            self.splat_body(&g, pt, color, theme);
        }

        // Head volume (splatted so the core can occlude it).
        let hf = self.head_frame(&g, t);
        self.splat_head(&g, &hf, theme);

        // Halo.
        for y in 0..self.h {
            for x in 0..self.w {
                let idx = y * self.w + x;
                let gv = self.glow[idx];
                if gv > 0.03 && self.body[idx] < 0.55 {
                    canvas.add_light(
                        x as isize,
                        y as isize,
                        theme.c(2).scale(0.55),
                        gv * 0.6,
                        '.',
                    );
                }
            }
        }

        // Body surface.
        for y in 0..self.h {
            for x in 0..self.w {
                let idx = y * self.w + x;
                let v = self.body[idx];
                if v < 0.02 {
                    continue;
                }
                let ramp = ((v.powf(0.82) * (BODY_RAMP.len() - 1) as f32).round() as usize)
                    .min(BODY_RAMP.len() - 1);
                if ramp == 0 {
                    continue;
                }
                canvas.set(
                    x as isize,
                    y as isize,
                    BODY_RAMP[ramp] as char,
                    self.body_col[idx],
                );
            }
        }

        // Spines along the outer flank.
        for (i, pt) in path.iter().enumerate() {
            if i % 11 != 0 || pt.u < 0.04 || pt.u > 0.86 {
                continue;
            }
            if self.orb_occludes(&g, pt.x, pt.y, pt.z) || pt.z < -0.2 {
                continue;
            }
            let len = (1.7 + (1.0 - pt.u) * 2.6) * g.unit;
            let base_col = theme.c(3).scale(0.45 + 0.45 * ((pt.z + 1.0) * 0.5));
            let bx = pt.x.round() as isize;
            let by = pt.y.round() as isize;
            for s in 1..=len.round() as isize {
                let f = s as f32 / len.max(1.0);
                let x = bx + (pt.nx * s as f32).round() as isize;
                let y = by + (pt.ny * s as f32 / 2.0).round() as isize;
                canvas.set(
                    x,
                    y,
                    if s == 1 { '*' } else { '.' },
                    base_col.scale(1.0 - f * 0.5),
                );
            }
            let tip_x = bx + (pt.nx * len).round() as isize;
            let tip_y = by + (pt.ny * len / 2.0).round() as isize;
            canvas.set(tip_x, tip_y, '^', theme.c(3).scale(0.8));
        }

        // Tail fin.
        if path.len() >= 2 {
            let last = path[path.len() - 1];
            let prev = path[path.len() - 2];
            let dx = last.x - prev.x;
            let dyv = (last.y - prev.y) * 2.0;
            let len = (dx * dx + dyv * dyv).sqrt().max(1e-4);
            let tx = dx / len;
            let ty = dyv / len;
            if !self.orb_occludes(&g, last.x, last.y, last.z) {
                let depth = 0.4 + 0.6 * ((last.z + 1.0) * 0.5).clamp(0.0, 1.0);
                for k in -3i32..=3 {
                    let spread = k as f32 * 0.16;
                    let fl = (1.8 + (3 - k.unsigned_abs()) as f32 * 0.6) * g.unit;
                    let ex = last.x - (tx - spread * ty) * fl;
                    let ey = last.y - (ty + spread * tx) * fl / 2.0;
                    canvas.line(
                        last.x.round() as isize,
                        last.y.round() as isize,
                        ex.round() as isize,
                        ey.round() as isize,
                        if k == 0 { '*' } else { '.' },
                        theme.c(3).lerp(theme.c(2), 0.4).scale(depth),
                    );
                }
            }
        }

        // Legs with claws.
        self.draw_leg(canvas, &g, &path, 0.17, theme, t);
        self.draw_leg(canvas, &g, &path, 0.43, theme, t);

        // Head details.
        self.draw_head_details(canvas, &g, &hf, theme, t);

        // A filled cone of fire from the jaws, aimed at the pearl.
        if self.breath_t > 0.0 {
            let progress = (1.0 - self.breath_t / 1.8).clamp(0.0, 1.0);
            let reach = 42.0 * (progress * PI).sin().max(0.3);
            let mouth = head_map(&hf, 4.7, 0.15);
            let fx = hf.cos * hf.flip;
            let fy = hf.sin * hf.flip;
            let ox = mouth.0 as f32;
            let oy = mouth.1 as f32 * 2.0;
            let rx = reach as isize + 2;
            let ry = (reach * 0.62) as isize + 2;
            let cx = mouth.0;
            let cy = mouth.1;
            for py in (cy - ry)..=(cy + ry) {
                if py < 0 || py >= self.h as isize {
                    continue;
                }
                for px in (cx - rx)..=(cx + rx) {
                    if px < 0 || px >= self.w as isize {
                        continue;
                    }
                    let dx = px as f32 + 0.5 - ox;
                    let dyv = (py as f32 + 0.5) * 2.0 - oy;
                    let along = dx * fx + dyv * fy;
                    if along < 1.0 || along >= reach {
                        continue;
                    }
                    let perp = -dx * fy + dyv * fx;
                    let wobble = (value_noise(along * 0.22, t * 2.6, self.seed ^ 0x33) - 0.5) * 0.9;
                    let width = (0.35 + along * 0.145) * (1.0 + wobble);
                    let q = (perp / width).abs();
                    if q >= 1.0 {
                        continue;
                    }
                    if (hash2(px as i64, py as i64, self.seed ^ 0x91) & 15) < 2 && q > 0.35 {
                        continue;
                    }
                    let t01 = along / reach;
                    let flick = 0.82
                        + ((hash2(px as i64, py as i64 ^ (t * 30.0) as i64, self.seed ^ 0x77)
                            & 0xff) as f32
                            / 255.0)
                            * 0.35;
                    let hot = Rgb::hex(0xfff3c4)
                        .lerp(Rgb::hex(0xffb52e), (t01 * 2.0).min(1.0))
                        .lerp(Rgb::hex(0xff4a00), (t01 * 1.35).min(1.0))
                        .lerp(Rgb::hex(0x9c1b00), t01.powf(2.0));
                    let col = hot.scale((1.0 - q * 0.65) * (1.0 - t01.powf(2.0) * 0.55) * flick);
                    let ch = if q < 0.3 && t01 < 0.55 {
                        '@'
                    } else if q < 0.62 {
                        '*'
                    } else if t01 < 0.8 {
                        ':'
                    } else {
                        '.'
                    };
                    canvas.set(px, py, ch, col);
                }
            }
        }

        // Fire breath.
        if self.breath_t > 0.0 {
            let (mx, my) = head_map(&hf, 4.8, 0.2);
            canvas.add_light(mx, my, theme.c(3), 1.2, '*');
            canvas.add_light(mx, my - 1, theme.hot(), 0.8, '.');
            canvas.add_light(mx, my + 1, theme.c(3), 0.8, '.');
        }
        for b in &self.breath {
            let life = (b.life / b.max_life).clamp(0.0, 1.0);
            let x = b.x.round() as isize;
            let y = (b.y / 2.0).round() as isize;
            let ch = if life > 0.7 {
                '@'
            } else if life > 0.45 {
                '*'
            } else if life > 0.22 {
                ':'
            } else {
                '.'
            };
            let col = theme
                .c(3)
                .lerp(theme.hot(), life)
                .lerp(theme.c(2), (1.0 - life) * 0.55)
                .scale(0.55 + life * 0.9);
            canvas.set(x, y, ch, col);
            canvas.add_light(x, y, theme.c(3), 0.4 * life, '.');
            canvas.add_light(x + 1, y, theme.c(3), 0.22 * life, '.');
        }

        // Minimal HUD.
        if self.w >= 60 {
            let title = ":: ORBITAL DRAGON ::";
            canvas.put_str_gradient(2, 0, title, &[theme.c(2), theme.c(3), theme.hot()]);
            let right = "CORE STABLE // COIL 1.75 TURN";
            let rx = self.w as isize - right.len() as isize - 2;
            if rx > 22 {
                canvas.put_str(rx, 0, right, theme.c(2).scale(0.7));
            }
        }
    }
}

impl DragonScene {
    fn draw_core(&mut self, canvas: &mut Canvas, g: &Geom, theme: &Theme, t: f32) {
        let r = g.orb_r;
        let cx = g.cx;
        let cy = g.cy;
        let y_lo = (cy - r / 2.0 - 1.0).floor() as isize;
        let y_hi = (cy + r / 2.0 + 1.0).ceil() as isize;
        let x_lo = (cx - r - 1.0).floor() as isize;
        let x_hi = (cx + r + 1.0).ceil() as isize;
        let light = [-0.35f32, -0.55, 0.76];
        for y in y_lo..=y_hi {
            if y < 0 || y >= self.h as isize {
                continue;
            }
            for x in x_lo..=x_hi {
                if x < 0 || x >= self.w as isize {
                    continue;
                }
                let dx = x as f32 - cx;
                let dyv = (y as f32 - cy) * 2.0;
                let d2 = dx * dx + dyv * dyv;
                let d = d2.sqrt();
                if d2 <= r * r {
                    let nz = (r * r - d2).sqrt() / r;
                    let nx = dx / r;
                    let ny = dyv / r;
                    let ndl = (nx * light[0] + ny * light[1] + nz * light[2]).max(0.0);
                    let angle = dyv.atan2(dx);
                    let swirl = value_noise(
                        angle * 1.8 + t * 0.45,
                        y as f32 * 0.13 - t * 0.35,
                        self.seed,
                    );
                    let rim = (1.0 - nz).powf(3.0);
                    let bright = (0.18 + ndl * 0.72 + swirl * 0.22 + rim * 0.5).clamp(0.0, 1.3);
                    let idx = ((bright.min(1.0) * (BODY_RAMP.len() - 1) as f32).round() as usize)
                        .min(BODY_RAMP.len() - 1);
                    if idx == 0 {
                        continue;
                    }
                    let mut col = theme
                        .c(1)
                        .lerp(theme.c(2), ndl)
                        .lerp(theme.c(3), (swirl * 0.55).clamp(0.0, 1.0));
                    if rim > 0.45 {
                        col = col.lerp(theme.c(4), (rim - 0.45) * 0.8);
                    }
                    canvas.set(x, y, BODY_RAMP[idx] as char, col);
                } else if d2 < (r * 1.65) * (r * 1.65) {
                    let falloff = 1.0 - (d - r) / (r * 0.65);
                    if falloff > 0.0 {
                        canvas.add_light(x, y, theme.c(2).scale(0.5), falloff * 0.5, '.');
                    }
                }
            }
        }

        // Tilted orbital rings.
        for k in 0..3 {
            let rr = r * (1.35 + k as f32 * 0.26);
            let tilt = 0.26 + k as f32 * 0.12;
            let phase = t * (0.18 + k as f32 * 0.07) + k as f32 * 1.7;
            let steps = (rr * 10.0) as usize;
            for i in 0..steps {
                let a = i as f32 / steps as f32 * TAU + phase;
                let x = cx + a.cos() * rr;
                let y = cy + a.sin() * rr * tilt;
                let d = ((x - cx).powi(2) + ((y - cy) * 2.0).powi(2)).sqrt();
                if d < r * 1.02 {
                    continue;
                }
                if i % 2 == 0 {
                    canvas.blend(
                        x.round() as isize,
                        y.round() as isize,
                        '.',
                        theme.c(2).scale(0.55),
                        0.7,
                    );
                }
            }
        }
    }

    fn draw_leg(
        &mut self,
        canvas: &mut Canvas,
        g: &Geom,
        path: &[PathPt],
        u: f32,
        theme: &Theme,
        t: f32,
    ) {
        let idx = ((u * (path.len() - 1) as f32) as usize).min(path.len() - 1);
        let pt = path[idx];
        if self.orb_occludes(g, pt.x, pt.y, pt.z) {
            return;
        }
        let depth = 0.45 + 0.55 * ((pt.z + 1.0) * 0.5);
        let col = theme.c(2).scale(depth * 0.9);
        let claw_col = theme.c(3).scale(depth);
        let swing = (t * 1.6 + u * 9.0).sin() * 0.6;
        let bx = pt.x;
        let by = pt.y;
        // Outward normal is in visual units; convert to cell rows.
        let nx = pt.nx;
        let ny = pt.ny / 2.0;
        let tx = -ny;
        let ty = nx;
        let knee_x = bx + nx * 2.7 * g.unit + tx * (1.2 + swing) * g.unit;
        let knee_y = by + ny * 2.7 * g.unit + ty * (1.2 + swing) * g.unit;
        let foot_x = knee_x + nx * 2.5 * g.unit - tx * 1.0 * g.unit;
        let foot_y = knee_y + ny * 2.5 * g.unit - ty * 1.0 * g.unit;
        let char_for = |dx: f32, dy: f32| -> char {
            if dx.abs() > dy.abs() * 1.4 {
                '-'
            } else if (dx < 0.0) == (dy < 0.0) {
                '\\'
            } else {
                '/'
            }
        };
        canvas.line(
            bx.round() as isize,
            by.round() as isize,
            knee_x.round() as isize,
            knee_y.round() as isize,
            char_for(knee_x - bx, knee_y - by),
            col,
        );
        canvas.line(
            knee_x.round() as isize,
            knee_y.round() as isize,
            foot_x.round() as isize,
            foot_y.round() as isize,
            char_for(foot_x - knee_x, foot_y - knee_y),
            col,
        );
        // Three claws.
        for k in -1..=1 {
            let spread = k as f32 * 0.55;
            let cx = foot_x + nx * 1.1 * g.unit + tx * spread * g.unit;
            let cy = foot_y + ny * 1.1 * g.unit + ty * spread * g.unit;
            canvas.line(
                foot_x.round() as isize,
                foot_y.round() as isize,
                cx.round() as isize,
                cy.round() as isize,
                if k == 0 { '^' } else { '.' },
                claw_col,
            );
        }
    }

    fn draw_head_details(
        &mut self,
        canvas: &mut Canvas,
        g: &Geom,
        hf: &HeadFrame,
        theme: &Theme,
        t: f32,
    ) {
        let z = hf.z + 0.25;
        let depth = 0.5 + 0.5 * ((z + 1.0) * 0.5).clamp(0.0, 1.0);
        let gold = theme.c(3).scale(depth);
        let pale = theme.hot().scale(depth);
        let jade = theme.c(2).scale(depth);

        // Eye socket: a dark patch makes the bright eye read at a distance.
        for (lx, ly) in [(1.3, -0.75), (1.85, -0.75), (1.3, -0.25), (1.85, -0.25)] {
            let (x, y) = head_map(hf, lx, ly);
            if !self.orb_occludes(g, x as f32, y as f32, z) {
                canvas.set(x, y, ' ', Rgb::BLACK);
            }
        }
        let (ex, ey) = head_map(hf, 1.6, -0.55);
        if !self.orb_occludes(g, ex as f32, ey as f32, z) {
            canvas.set(ex, ey, '@', pale);
            canvas.add_light(ex, ey, theme.c(3), 1.0, '*');
            let (bx, by) = head_map(hf, 1.2, -1.35);
            canvas.set(bx, by, '-', gold);
        }

        // Mouth line, fangs and a small beard.
        for i in 0..=4 {
            let s = i as f32 / 4.0;
            let lx = 1.4 + s * 3.4;
            let ly = 0.85 + s * 0.28;
            let (x, y) = head_map(hf, lx, ly);
            if !self.orb_occludes(g, x as f32, y as f32, z) {
                canvas.set(
                    x,
                    y,
                    if i % 2 == 0 { '_' } else { ' ' },
                    theme.c(1).scale(0.6),
                );
            }
        }
        for i in 0..3 {
            let (x, y) = head_map(hf, 2.0 + i as f32 * 0.95, 1.25);
            if !self.orb_occludes(g, x as f32, y as f32, z) {
                canvas.set(x, y, 'v', pale.scale(0.85));
            }
        }
        for i in 0..4 {
            let (x0, y0) = head_map(hf, 1.4 + i as f32 * 0.65, 1.45);
            let (x1, y1) = head_map(
                hf,
                1.2 + i as f32 * 0.7,
                2.5 + (t * 2.0 + i as f32).sin() * 0.2,
            );
            canvas.line(x0, y0, x1, y1, '.', jade.scale(0.7));
        }

        // Horns (two quadratic sweeps).
        for (base, ctrl, end, cc) in [
            ((-0.9, -1.3), (-2.8, -3.8), (-5.2, -4.2), gold),
            ((-1.7, -0.9), (-3.8, -2.8), (-5.8, -2.2), pale),
        ] {
            for i in 0..=18 {
                let s = i as f32 / 18.0;
                let lx = quad(base.0, ctrl.0, end.0, s);
                let ly = quad(base.1, ctrl.1, end.1, s);
                let (x, y) = head_map(hf, lx, ly);
                let (x2, y2) = head_map(hf, lx, ly + 0.42);
                let ch = if s > 0.85 {
                    '.'
                } else if s > 0.45 {
                    '/'
                } else {
                    '='
                };
                if !self.orb_occludes(g, x as f32, y as f32, z) {
                    canvas.set(x, y, ch, cc.scale(1.05 - s * 0.35));
                    canvas.set(x2, y2, '.', cc.scale(0.55));
                }
            }
        }

        // Mane spikes behind the head.
        for k in 0..5 {
            let a = -0.9 - k as f32 * 0.22;
            let len = 2.2 + k as f32 * 0.35;
            let (x0, y0) = head_map(hf, -1.2, -0.6 + k as f32 * 0.45);
            let (x1, y1) = head_map(
                hf,
                -1.2 + a.cos() * len,
                -0.6 + k as f32 * 0.45 - len * 0.55,
            );
            canvas.line(
                x0,
                y0,
                x1,
                y1,
                if k % 2 == 0 { '/' } else { '.' },
                jade.scale(0.85),
            );
        }

        // Whiskers: long, waving, two per side.
        for side in [-1.0f32, 1.0] {
            let bx = 4.0;
            let by = 0.2 * side;
            let wob = (t * 2.1 + side).sin() * 1.1;
            let ctrl = (5.8, (1.4 + wob) * side);
            let end = (8.6, (2.8 + wob * 1.2) * side);
            for i in 0..=22 {
                let s = i as f32 / 22.0;
                let lx = quad(bx, ctrl.0, end.0, s);
                let ly = quad(by, ctrl.1, end.1, s) + (s * 7.0 + t * 2.4).sin() * 0.45 * s;
                let (x, y) = head_map(hf, lx, ly);
                if !self.orb_occludes(g, x as f32, y as f32, z) {
                    canvas.set(
                        x,
                        y,
                        if i % 2 == 0 { '-' } else { '.' },
                        theme.c(3).scale((1.0 - s * 0.5) * depth * 1.1),
                    );
                }
            }
        }

        // Nostril and jaw highlight.
        let (nx, ny) = head_map(hf, 4.05, -0.15);
        if !self.orb_occludes(g, nx as f32, ny as f32, z) {
            canvas.set(nx, ny, '.', gold.scale(0.7));
        }
    }
}

fn quad(a: f32, b: f32, c: f32, t: f32) -> f32 {
    let inv = 1.0 - t;
    inv * inv * a + 2.0 * inv * t * b + t * t * c
}
