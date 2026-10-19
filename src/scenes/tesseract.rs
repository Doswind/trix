//! tesseract — a rotating four-dimensional hypercube.
//!
//! Sixteen vertices and thirty-two edges rotate in the XW / YZ planes, project
//! 4D -> 3D -> 2D, and draw as a depth-shaded ASCII wireframe.  Pure ASCII.

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::hash2;
use crate::scene::{Frame, Scene};

pub struct TesseractScene {
    w: usize,
    h: usize,
    verts: Vec<[f32; 4]>,
    edges: Vec<(usize, usize)>,
}

impl TesseractScene {
    pub fn new(_seed: u64) -> Self {
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
        TesseractScene {
            w: 0,
            h: 0,
            verts,
            edges,
        }
    }
}

fn rotate(p: [f32; 4], a: f32, b: f32, c: f32) -> [f32; 4] {
    // XW rotation.
    let (sa, ca) = a.sin_cos();
    let x = p[0] * ca - p[3] * sa;
    let w = p[0] * sa + p[3] * ca;
    // YZ rotation.
    let (sb, cb) = b.sin_cos();
    let y = p[1] * cb - p[2] * sb;
    let z = p[1] * sb + p[2] * cb;
    // A slow XY rotation breaks the symmetry of the 4D tumble.
    let (sc, cc) = c.sin_cos();
    let x2 = x * cc - y * sc;
    let y2 = x * sc + y * cc;
    [x2, y2, z, w]
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

impl Scene for TesseractScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, _f: &Frame) {}

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!(
            "4D {:.2} rad | {} EDGES",
            f.t as f32 * 0.7,
            self.edges.len()
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
        let theme = f.theme;
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;
        let scale = (self.w as f32 * 0.18).min(self.h as f32 * 0.55);
        let a = t * 0.45;
        let b = t * 0.60;
        let c = t * 0.30;

        // A few background stars give the wireframe a place to live.
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                let hsh = hash2(x as i64, y as i64, 0x5445_5353);
                if hsh % 173 == 0 {
                    canvas.set(x, y, '.', theme.c(2).scale(0.18));
                }
            }
        }

        let mut pts = Vec::with_capacity(16);
        for p in &self.verts {
            let r = rotate(*p, a, b, c);
            let k4 = 2.2 / (2.6 - r[3]).max(0.7);
            let q = [r[0] * k4, r[1] * k4, r[2] * k4];
            let k3 = 2.0 / (3.2 - q[2]).max(0.7);
            let x = cx + q[0] * k3 * scale;
            // Terminal cells are about twice as tall as wide.
            let y = cy + q[1] * k3 * scale * 0.5;
            pts.push((x, y, r[3], q[2]));
        }

        let mut order: Vec<(f32, usize)> = self
            .edges
            .iter()
            .enumerate()
            .map(|(i, (a, b))| ((pts[*a].2 + pts[*b].2) * 0.5, i))
            .collect();
        order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        for (depth, ei) in order {
            let (i, j) = self.edges[ei];
            let (x0, y0, _, _) = pts[i];
            let (x1, y1, _, _) = pts[j];
            let dx = x1 - x0;
            let dy = y1 - y0;
            let ch = if dx.abs() > dy.abs() * 1.8 {
                '-'
            } else if dy.abs() > dx.abs() * 1.8 {
                '|'
            } else if dx * dy > 0.0 {
                '\\'
            } else {
                '/'
            };
            let front = ((depth + 1.0) * 0.5).clamp(0.0, 1.0);
            let col = theme
                .c(2)
                .lerp(theme.hot(), front)
                .scale(0.30 + 0.75 * front);
            draw_edge(
                canvas,
                x0.round() as isize,
                y0.round() as isize,
                x1.round() as isize,
                y1.round() as isize,
                max_y,
                ch,
                col,
            );
        }

        for (x, y, w, _) in &pts {
            let front = ((w + 1.0) * 0.5).clamp(0.0, 1.0);
            let col = theme.c(3).lerp(Rgb::WHITE, front).scale(0.5 + 0.5 * front);
            let xx = x.round() as isize;
            let yy = y.round() as isize;
            if yy >= 1 && yy <= max_y {
                canvas.set(xx, yy, '@', col);
                canvas.add_light(xx, yy, col, 0.2, '.');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tesseract_has_edges_and_vertices() {
        let mut scene = TesseractScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let frame = Frame {
            t: 3.0,
            dt: 1.0 / 30.0,
            w: 80,
            h: 24,
            fps: 30,
            theme: crate::theme::find("ice").unwrap(),
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('@'), "no vertices");
        assert!(
            text.contains('-') || text.contains('|') || text.contains('/'),
            "no edges"
        );
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
