//! tesseract — a neon four-dimensional hypercube in deep space.
//!
//! Sixteen vertices and thirty-two edges rotate in 4D, project to 3D and then
//! 2D, and draw as a depth-sorted, colour-coded wireframe.  The outer cube is
//! cyan, the inner cube violet and the eight connectors gold; a ghost hypercube,
//! a perspective starfield, coloured dust, three orbit rings and travelling
//! edge pulses supply the space around it.  ASCII is the default; `--unicode`
//! swaps in double/single box-drawing edges and solid vertices.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const OUTER: Rgb = Rgb::hex(0x22d3ee);
const INNER: Rgb = Rgb::hex(0xc084fc);
const LINK: Rgb = Rgb::hex(0xfbbf24);
const STAR: Rgb = Rgb::hex(0xe0f2fe);
const DUST: [Rgb; 4] = [
    Rgb::hex(0x1e3a8a),
    Rgb::hex(0x6d28d9),
    Rgb::hex(0xbe185d),
    Rgb::hex(0x0e7490),
];

#[derive(Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    z: f32,
    speed: f32,
}

#[derive(Clone, Copy)]
struct Dust {
    x: f32,
    y: f32,
    phase: f32,
    color: Rgb,
}

#[derive(Clone, Copy)]
struct Pulse {
    edge: usize,
    s: f32,
    speed: f32,
    color: Rgb,
}

pub struct TesseractScene {
    w: usize,
    h: usize,
    verts: Vec<[f32; 4]>,
    edges: Vec<(usize, usize)>,
    stars: Vec<Star>,
    dust: Vec<Dust>,
    pulses: Vec<Pulse>,
    rng: Rng,
}

impl TesseractScene {
    pub fn new(seed: u64) -> Self {
        let mut verts = Vec::with_capacity(16);
        for i in 0..16 {
            let mut p = [0.0f32; 4];
            for (k, v) in p.iter_mut().enumerate() {
                *v = if (i >> k) & 1 == 0 { -1.0 } else { 1.0 };
            }
            verts.push(p);
        }
        let mut edges = Vec::with_capacity(32);
        for i in 0..16 {
            for k in 0..4 {
                let j = i ^ (1 << k);
                if i < j {
                    edges.push((i, j));
                }
            }
        }
        let mut rng = Rng::new(seed ^ 0x5445_5353_4552);
        let mut stars = Vec::with_capacity(70);
        for _ in 0..70 {
            stars.push(Star {
                x: rng.range(-1.0, 1.0),
                y: rng.range(-1.0, 1.0),
                z: rng.range(0.12, 1.0),
                speed: rng.range(0.15, 0.65),
            });
        }
        let mut dust = Vec::with_capacity(35);
        for _ in 0..35 {
            dust.push(Dust {
                x: rng.range(0.0, 1.0),
                y: rng.range(0.0, 1.0),
                phase: rng.range(0.0, TAU),
                color: DUST[rng.below(DUST.len())],
            });
        }
        let mut pulses = Vec::with_capacity(8);
        for _ in 0..8 {
            let edge = rng.below(edges.len());
            let (a, b) = edges[edge];
            pulses.push(Pulse {
                edge,
                s: rng.f32(),
                speed: rng.range(0.06, 0.16),
                color: edge_color(verts[a], verts[b]),
            });
        }
        TesseractScene {
            w: 0,
            h: 0,
            verts,
            edges,
            stars,
            dust,
            pulses,
            rng,
        }
    }
}

fn edge_color(a: [f32; 4], b: [f32; 4]) -> Rgb {
    let wa = a[3];
    let wb = b[3];
    if (wa < 0.0) != (wb < 0.0) {
        LINK
    } else if wa < 0.0 {
        OUTER
    } else {
        INNER
    }
}

fn rotate(p: [f32; 4], a: f32, b: f32, c: f32) -> [f32; 4] {
    let (sa, ca) = a.sin_cos();
    let x = p[0] * ca - p[3] * sa;
    let w = p[0] * sa + p[3] * ca;
    let (sb, cb) = b.sin_cos();
    let y = p[1] * cb - p[2] * sb;
    let z = p[1] * sb + p[2] * cb;
    let (sc, cc) = c.sin_cos();
    let x2 = x * cc - y * sc;
    let y2 = x * sc + y * cc;
    [x2, y2, z, w]
}

/// Project a 4D point all the way to screen cells.  Returns `(x, y, w4, z3)`.
fn project(
    p: [f32; 4],
    a: f32,
    b: f32,
    c: f32,
    cx: f32,
    cy: f32,
    scale: f32,
) -> (f32, f32, f32, f32) {
    let r = rotate(p, a, b, c);
    let k4 = 2.2 / (2.6 - r[3]).max(0.7);
    let q = [r[0] * k4, r[1] * k4, r[2] * k4];
    let k3 = 2.0 / (3.2 - q[2]).max(0.7);
    let x = cx + q[0] * k3 * scale;
    // Terminal cells are about twice as tall as wide.
    let y = cy + q[1] * k3 * scale * 0.5;
    (x, y, r[3], q[2])
}

#[allow(clippy::too_many_arguments)]
fn draw_edge(
    canvas: &mut Canvas,
    x0: isize,
    y0: isize,
    x1: isize,
    y1: isize,
    max_y: isize,
    ch: char,
    col: Rgb,
    glow: bool,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        if y >= 1 && y <= max_y {
            canvas.set(x, y, ch, col);
            if glow && (x + y) % 3 == 0 {
                canvas.add_light(x + 1, y, col, 0.07, '.');
            }
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

impl TesseractScene {
    fn paint_space(&self, canvas: &mut Canvas, max_y: isize) {
        let top = Rgb::hex(0x050818);
        let mid = Rgb::hex(0x0d1433);
        let bottom = Rgb::hex(0x02030a);
        for y in 1..=max_y {
            let f = (y - 1) as f32 / (max_y - 1).max(1) as f32;
            let bg = if f < 0.55 {
                top.lerp(mid, f / 0.55)
            } else {
                mid.lerp(bottom, (f - 0.55) / 0.45)
            };
            canvas.fill(0, y, self.w as isize, 1, ' ', Rgb::BLACK, Some(bg));
        }
        // A soft violet nebula behind the cube.
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;
        let rx = self.w as f32 * 0.42;
        let ry = self.h as f32 * 0.42;
        let tint = Rgb::hex(0x3b0764);
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                let nx = (x as f32 - cx) / rx;
                let ny = (y as f32 - cy) / ry;
                let d = (nx * nx + ny * ny).sqrt();
                if d >= 1.0 {
                    continue;
                }
                if let Some(mut bg) = canvas.get(x, y).and_then(|c| c.bg) {
                    let k = (1.0 - d) * 0.40;
                    bg = bg.lerp(tint, k * 0.45);
                    canvas.set_bg(x, y, Some(bg));
                }
            }
        }
    }
}

impl Scene for TesseractScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        for s in &mut self.stars {
            s.z -= s.speed * dt * 0.8;
            if s.z < 0.08 {
                s.z = self.rng.range(0.85, 1.0);
                s.x = self.rng.range(-1.0, 1.0);
                s.y = self.rng.range(-1.0, 1.0);
            }
        }
        for d in &mut self.dust {
            d.phase += dt * 0.35;
            d.x = (d.x + dt * 0.004 * (d.phase * 0.7).sin()).rem_euclid(1.0);
            d.y = (d.y + dt * 0.003 * (d.phase * 0.9).cos()).rem_euclid(1.0);
        }
        for i in 0..self.pulses.len() {
            let mut p = self.pulses[i];
            p.s += p.speed * dt;
            if p.s > 1.0 {
                p.s = 0.0;
                p.edge = self.rng.below(self.edges.len());
                p.speed = self.rng.range(0.06, 0.16);
                let (a, b) = self.edges[p.edge];
                p.color = edge_color(self.verts[a], self.verts[b]);
            }
            self.pulses[i] = p;
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!(
            "4D {:.2} | EDGES {} | W {:.2}",
            (f.t as f32 * 0.42).rem_euclid(TAU),
            self.edges.len(),
            (f.t as f32 * 0.31).sin()
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
        let cx = w * 0.5 + (t * 0.23).sin() * w * 0.025;
        let cy = h * 0.5 + (t * 0.31).cos() * h * 0.035;
        let scale = (w * 0.19).min(h * 0.62) * (1.0 + 0.03 * (t * 0.8).sin());

        self.paint_space(canvas, max_y);

        // Perspective starfield with short inward trails.
        for s in &self.stars {
            let sx = cx + s.x / s.z * w * 0.5;
            let sy = cy + s.y / s.z * h * 0.5;
            if sy < 1.0 || sy > max_y as f32 {
                continue;
            }
            let col = STAR.lerp(OUTER, 0.28).scale(0.16 + (1.0 - s.z) * 0.55);
            for k in 0..2 {
                let f = 1.0 - k as f32 * 0.025;
                let x = cx + (sx - cx) * f;
                let y = cy + (sy - cy) * f;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                canvas.add_light(
                    x.round() as isize,
                    y.round() as isize,
                    col,
                    0.22 / (k as f32 + 1.0),
                    if k == 0 { '*' } else { '-' },
                );
            }
        }

        // Coloured nebula dust.
        for d in &self.dust {
            let x = d.x * w;
            let y = d.y * h;
            if y < 1.0 || y > max_y as f32 {
                continue;
            }
            let tw = 0.5 + 0.5 * (t * 0.4 + d.phase).sin();
            canvas.add_light(
                x.round() as isize,
                y.round() as isize,
                d.color,
                0.035 + 0.055 * tw,
                '.',
            );
        }

        // Three orbit rings around the cube.
        let rings = [
            (0.26, 0.18, 0.0, 0.18, OUTER),
            (0.34, 0.24, 2.1, -0.12, INNER),
            (0.42, 0.30, 4.2, 0.08, LINK),
        ];
        for (rx, ry, phase, speed, color) in rings {
            let rot = t * speed + phase;
            let (cr, sr) = rot.sin_cos();
            let n = 64;
            for i in 0..n {
                let a = i as f32 / n as f32 * TAU;
                let ex = a.cos() * rx * w;
                let ey = a.sin() * ry * h * 0.5;
                let x = cx + ex * cr - ey * sr;
                let y = cy + ex * sr + ey * cr;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                canvas.add_light(
                    x.round() as isize,
                    y.round() as isize,
                    color,
                    0.10,
                    if i % 12 == 0 { '+' } else { '.' },
                );
            }
            let a = t * 0.45 + phase;
            let ex = a.cos() * rx * w;
            let ey = a.sin() * ry * h * 0.5;
            let x = cx + ex * cr - ey * sr;
            let y = cy + ex * sr + ey * cr;
            if y >= 1.0 && y <= max_y as f32 {
                canvas.add_light(
                    x.round() as isize,
                    y.round() as isize,
                    color,
                    0.55,
                    if unicode { '✦' } else { '*' },
                );
            }
        }

        // Ghost hypercube: the same object rotated 90 degrees, dimmed.
        let ga = 0.65 + 0.28 * (t * 0.35).sin() + 0.65;
        let gb = 0.35 + 0.22 * (t * 0.27 + 1.0).sin() + 0.50;
        let gc = 0.18 * (t * 0.22).sin() + 0.30;
        let ghost: Vec<(f32, f32, f32, f32)> = self
            .verts
            .iter()
            .map(|p| project(*p, ga, gb, gc, cx, cy, scale * 1.07))
            .collect();
        for (ei, (i, j)) in self.edges.iter().enumerate() {
            if ei % 4 != 0 {
                continue;
            }
            let (x0, y0, _, _) = ghost[*i];
            let (x1, y1, _, _) = ghost[*j];
            let col = edge_color(self.verts[*i], self.verts[*j]).scale(0.09);
            draw_edge(
                canvas,
                x0.round() as isize,
                y0.round() as isize,
                x1.round() as isize,
                y1.round() as isize,
                max_y,
                if unicode { '┄' } else { ':' },
                col,
                false,
            );
        }

        // Main hypercube.
        let a = 0.65 + 0.28 * (t * 0.35).sin();
        let b = 0.35 + 0.22 * (t * 0.27 + 1.0).sin();
        let c = 0.18 * (t * 0.22).sin();
        let pts: Vec<(f32, f32, f32, f32)> = self
            .verts
            .iter()
            .map(|p| project(*p, a, b, c, cx, cy, scale))
            .collect();
        let mut order: Vec<(f32, usize)> = self
            .edges
            .iter()
            .enumerate()
            .map(|(i, (u, v))| ((pts[*u].3 + pts[*v].3) * 0.5, i))
            .collect();
        order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, ei) in order {
            let (i, j) = self.edges[ei];
            let (x0, y0, w0, _) = pts[i];
            let (x1, y1, w1, _) = pts[j];
            let connector = (w0 < 0.0) != (w1 < 0.0);
            let base = edge_color(self.verts[i], self.verts[j]);
            let front = (((pts[i].3 + pts[j].3) * 0.5 + 1.2) / 2.4).clamp(0.25, 1.0);
            let scale = if connector {
                0.30 + 0.20 * front
            } else {
                0.55 + 0.50 * front
            };
            let col = base
                .lerp(theme.c(3), 0.12)
                .hue_rotate(t * 0.006)
                .scale(scale);
            let dx = x1 - x0;
            let dy = y1 - y0;
            let slope = if dx.abs() > dy.abs() * 2.0 {
                0
            } else if dy.abs() > dx.abs() * 2.0 {
                1
            } else if dx * dy > 0.0 {
                2
            } else {
                3
            };
            let ch = if connector {
                if unicode {
                    '╋'
                } else {
                    '+'
                }
            } else if unicode {
                match (slope, w0 < 0.0) {
                    (0, true) => '═',
                    (1, true) => '║',
                    (2, _) => '╲',
                    (3, _) => '╱',
                    (0, false) => '─',
                    _ => '│',
                }
            } else {
                match slope {
                    0 => '=',
                    1 => '|',
                    2 => '\\',
                    _ => '/',
                }
            };
            draw_edge(
                canvas,
                x0.round() as isize,
                y0.round() as isize,
                x1.round() as isize,
                y1.round() as isize,
                max_y,
                ch,
                col,
                true,
            );
        }

        // Vertices: cyan for the outer cube, violet for the inner one.
        for (x, y, w4, z3) in &pts {
            if *y < 1.0 || *y > max_y as f32 {
                continue;
            }
            let base = if *w4 < 0.0 { OUTER } else { INNER };
            let front = ((*z3 + 1.2) / 2.4).clamp(0.25, 1.0);
            let col = base
                .lerp(Rgb::WHITE, 0.35 + 0.45 * front)
                .scale(0.65 + 0.35 * front);
            let xx = x.round() as isize;
            let yy = y.round() as isize;
            canvas.set(xx, yy, if unicode { '●' } else { '@' }, col);
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let ny = yy + dy;
                if ny >= 1 && ny <= max_y {
                    canvas.add_light(xx + dx, ny, col, 0.12, '.');
                }
            }
        }

        // Travelling pulses along the edges.
        for p in &self.pulses {
            let (i, j) = self.edges[p.edge];
            let (x0, y0, _, _) = pts[i];
            let (x1, y1, _, _) = pts[j];
            for k in 0..2 {
                let s = (p.s - k as f32 * 0.030).max(0.0);
                let x = x0 + (x1 - x0) * s;
                let y = y0 + (y1 - y0) * s;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                let col = if k == 0 {
                    Rgb::WHITE
                } else {
                    p.color.scale(0.65 - k as f32 * 0.18)
                };
                canvas.add_light(
                    x.round() as isize,
                    y.round() as isize,
                    col,
                    if k == 0 { 0.85 } else { 0.28 },
                    if k == 0 {
                        if unicode {
                            '✦'
                        } else {
                            '*'
                        }
                    } else if unicode {
                        '·'
                    } else {
                        '.'
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn frame(t: f64, charset: Charset) -> Frame<'static> {
        Frame {
            t,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("neon").unwrap(),
            charset,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    #[test]
    fn tesseract_has_edges_colors_and_vertices() {
        let mut scene = TesseractScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let f = frame(3.0, Charset::Ascii);
        scene.update(&f);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('@'), "no vertices");
        assert!(
            text.contains('=') || text.contains('|') || text.contains('/'),
            "no edges"
        );
        let colors: HashSet<_> = canvas
            .cells
            .iter()
            .filter(|c| c.ch != ' ')
            .map(|c| c.fg)
            .collect();
        assert!(colors.len() >= 3, "too few colors: {}", colors.len());
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }

    #[test]
    fn unicode_mode_uses_box_drawing_glyphs() {
        let mut scene = TesseractScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let f = frame(3.0, Charset::Unicode);
        scene.update(&f);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(!text.is_ascii(), "unicode mode stayed ASCII");
    }
}
