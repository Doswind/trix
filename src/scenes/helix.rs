//! helix — a three-dimensional DNA double helix in a holographic chamber.
//!
//! The two strands are real 3D helices, rotated by a moving camera and
//! perspective-projected.  Base pairs are 3D rungs with A/T/C/G letters, all
//! primitives are depth-sorted so the front strand occludes the back one, and
//! the top of the molecule periodically unzips into a replication fork.  A
//! scan beam sweeps down the helix, coloured dust and a wireframe containment
//! cylinder provide depth, and the whole scene sits on a deep teal gradient.
//! ASCII is the default; `--unicode` uses box-drawing backbones and solid
//! phosphate nodes.

use std::f32::consts::{PI, TAU};

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const BASES: [u8; 4] = *b"ATCG";
const BASE_COLORS: [Rgb; 4] = [
    Rgb::hex(0x4ade80), // A
    Rgb::hex(0xfb7185), // T
    Rgb::hex(0x60a5fa), // C
    Rgb::hex(0xfacc15), // G
];
const TURNS: f32 = 2.35;
const SAMPLES: usize = 96;
const PAIR_STEP: usize = 8;
const WHITE: Rgb = Rgb::hex(0xffffff);

#[derive(Clone, Copy)]
struct Dust {
    x: f32,
    y: f32,
    phase: f32,
    color: Rgb,
}

enum Mark {
    Line {
        x0: isize,
        y0: isize,
        x1: isize,
        y1: isize,
        ch: char,
        col: Rgb,
        glow: bool,
    },
    Glyph {
        x: isize,
        y: isize,
        ch: char,
        col: Rgb,
        glow: bool,
    },
}

pub struct HelixScene {
    w: usize,
    h: usize,
    dust: Vec<Dust>,
}

impl HelixScene {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x4845_4c49_5833);
        let colors = [
            Rgb::hex(0x2dd4bf),
            Rgb::hex(0xfbbf24),
            Rgb::hex(0x67e8f9),
            Rgb::hex(0x4ade80),
        ];
        let mut dust = Vec::with_capacity(55);
        for _ in 0..55 {
            dust.push(Dust {
                x: rng.range(0.0, 1.0),
                y: rng.range(0.0, 1.0),
                phase: rng.range(0.0, TAU),
                color: colors[rng.below(colors.len())],
            });
        }
        HelixScene { w: 0, h: 0, dust }
    }

    fn paint_space(&self, canvas: &mut Canvas, max_y: isize) {
        let top = Rgb::hex(0x021014);
        let mid = Rgb::hex(0x06313a);
        let bottom = Rgb::hex(0x01060a);
        for y in 1..=max_y {
            let f = (y - 1) as f32 / (max_y - 1).max(1) as f32;
            let bg = if f < 0.55 {
                top.lerp(mid, f / 0.55)
            } else {
                mid.lerp(bottom, (f - 0.55) / 0.45)
            };
            canvas.fill(0, y, self.w as isize, 1, ' ', Rgb::BLACK, Some(bg));
        }
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;
        let rx = self.w as f32 * 0.45;
        let ry = self.h as f32 * 0.45;
        let tint = Rgb::hex(0x0e7490);
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                let nx = (x as f32 - cx) / rx;
                let ny = (y as f32 - cy) / ry;
                let d = (nx * nx + ny * ny).sqrt();
                if d >= 1.0 {
                    continue;
                }
                if let Some(mut bg) = canvas.get(x, y).and_then(|c| c.bg) {
                    bg = bg.lerp(tint, (1.0 - d) * 0.30);
                    canvas.set_bg(x, y, Some(bg));
                }
            }
        }
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

fn base_index(base: u8) -> usize {
    match base {
        b'A' => 0,
        b'T' => 1,
        b'C' => 2,
        _ => 3,
    }
}

#[allow(clippy::too_many_arguments)]
fn project(
    x: f32,
    y: f32,
    z: f32,
    cam: f32,
    tilt: f32,
    roll: f32,
    cx: f32,
    cy: f32,
    scale_x: f32,
    scale_y: f32,
) -> (f32, f32, f32) {
    let (sc, cc) = cam.sin_cos();
    let xr = x * cc + z * sc;
    let zr = -x * sc + z * cc;
    let (st, ct) = tilt.sin_cos();
    let yr = y * ct - zr * st;
    let z2 = y * st + zr * ct;
    let (sr, cr) = roll.sin_cos();
    let xr2 = xr * cr - yr * sr;
    let yr2 = xr * sr + yr * cr;
    let k = 2.4 / (2.4 + z2 * 0.80).max(0.65);
    (cx + xr2 * scale_x * k, cy + yr2 * scale_y * k, z2)
}

fn line_ch(dx: f32, dy: f32, unicode: bool) -> char {
    if dx.abs() > dy.abs() * 2.0 {
        if unicode {
            '═'
        } else {
            '='
        }
    } else if dy.abs() > dx.abs() * 2.0 {
        if unicode {
            '║'
        } else {
            '|'
        }
    } else if dx * dy > 0.0 {
        if unicode {
            '╲'
        } else {
            '\\'
        }
    } else if unicode {
        '╱'
    } else {
        '/'
    }
}

fn draw_mark(canvas: &mut Canvas, max_y: isize, mark: Mark) {
    match mark {
        Mark::Line {
            x0,
            y0,
            x1,
            y1,
            ch,
            col,
            glow,
        } => {
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
                        canvas.add_light(x + 1, y, col, 0.08, '.');
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
        Mark::Glyph {
            x,
            y,
            ch,
            col,
            glow,
        } => {
            if y < 1 || y > max_y {
                return;
            }
            canvas.set(x, y, ch, col);
            if glow {
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let ny = y + dy;
                    if ny >= 1 && ny <= max_y {
                        canvas.add_light(x + dx, ny, col, 0.12, '.');
                    }
                }
            }
        }
    }
}

impl Scene for HelixScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        for d in &mut self.dust {
            d.phase += dt * 0.35;
            d.x = (d.x + dt * 0.003 * (d.phase * 0.8).sin()).rem_euclid(1.0);
            d.y = (d.y + dt * 0.002 * (d.phase * 0.6).cos()).rem_euclid(1.0);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let scan = ((f.t as f32 * 0.13).rem_euclid(1.0) * 100.0) as i32;
        let pairs = SAMPLES / PAIR_STEP;
        Some(format!("HELIX {TURNS:.1}T | BP {pairs} | SCAN {scan:02}%"))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w < 8 || self.h < 8 {
            return;
        }
        let max_y = self.h as isize - 3;
        if max_y < 2 {
            return;
        }
        let t = f.t as f32;
        let unicode = f.charset == Charset::Unicode;
        let theme = f.theme;
        let w = self.w as f32;
        let h = self.h as f32;
        let cx = w * 0.5;
        let cy = h * 0.5;
        let scale_x = w * 0.25;
        let scale_y = h * 0.40;
        let cam = t * 0.38 + 0.22 * (t * 0.17).sin();
        let tilt = 0.40 + 0.05 * (t * 0.21).sin();
        let roll = 0.20 + 0.05 * (t * 0.11).sin();
        let scan = (t * 0.13).rem_euclid(1.0);
        let scan_y = -1.0 + 2.0 * scan;

        self.paint_space(canvas, max_y);

        // Floating fluorescent dust.
        for d in &self.dust {
            let x = d.x * w;
            let y = d.y * h;
            if y < 1.0 || y > max_y as f32 {
                continue;
            }
            let tw = 0.5 + 0.5 * (t * 0.5 + d.phase).sin();
            canvas.add_light(
                x.round() as isize,
                y.round() as isize,
                d.color,
                0.05 + 0.09 * tw,
                '.',
            );
        }

        // A very dim holographic containment cylinder.
        for k in 0..8 {
            let a = k as f32 * TAU / 8.0 + cam * 0.35;
            for j in 0..14 {
                let gy = -1.0 + 2.0 * j as f32 / 13.0;
                let x = a.cos() * 1.55;
                let z = a.sin() * 1.55;
                let (px, py, _) = project(x, gy, z, cam, tilt, roll, cx, cy, scale_x, scale_y);
                if py >= 1.0 && py <= max_y as f32 {
                    canvas.add_light(
                        px.round() as isize,
                        py.round() as isize,
                        theme.c(2),
                        0.055,
                        if j % 4 == 0 { '+' } else { '.' },
                    );
                }
            }
        }
        for gy in [-0.96, 0.0, 0.96] {
            for i in 0..48 {
                let a = i as f32 * TAU / 48.0;
                let (px, py, _) = project(
                    a.cos() * 1.55,
                    gy,
                    a.sin() * 1.55,
                    cam,
                    tilt,
                    roll,
                    cx,
                    cy,
                    scale_x,
                    scale_y,
                );
                if py >= 1.0 && py <= max_y as f32 {
                    canvas.add_light(
                        px.round() as isize,
                        py.round() as isize,
                        theme.c(3),
                        0.07,
                        if i % 8 == 0 { '+' } else { '.' },
                    );
                }
            }
        }

        // Scan beam row, drawn over the molecule after the marks.
        let scan_row = project(0.0, scan_y, 0.0, cam, tilt, roll, cx, cy, scale_x, scale_y)
            .1
            .round() as isize;

        // Build the 3D helix and its base pairs.
        let fork_open = 0.5 - 0.5 * (t * 0.45).cos();
        let mut p0 = Vec::with_capacity(SAMPLES);
        let mut p1 = Vec::with_capacity(SAMPLES);
        for i in 0..SAMPLES {
            let gy = -1.0 + 2.0 * i as f32 / (SAMPLES - 1) as f32;
            let theta = gy * TURNS * TAU + t * 0.55;
            let fork_t = ((-gy - 0.55) / 0.45).clamp(0.0, 1.0);
            let fork = fork_open * fork_t.powf(1.2);
            let a = project(
                theta.cos() + fork * 0.55,
                gy,
                theta.sin() + fork * 0.18,
                cam,
                tilt,
                roll,
                cx,
                cy,
                scale_x,
                scale_y,
            );
            let b = project(
                (theta + PI).cos() - fork * 0.55,
                gy,
                (theta + PI).sin() - fork * 0.18,
                cam,
                tilt,
                roll,
                cx,
                cy,
                scale_x,
                scale_y,
            );
            p0.push((a.0, a.1, a.2, gy, fork));
            p1.push((b.0, b.1, b.2, gy, fork));
        }

        let mut marks: Vec<(f32, Mark)> = Vec::with_capacity(SAMPLES * 3);
        for i in 0..SAMPLES - 1 {
            for (strand, pts) in [&p0, &p1].iter().enumerate() {
                let (x0, y0, z0, _, _) = pts[i];
                let (x1, y1, z1, _, _) = pts[i + 1];
                let depth = (z0 + z1) * 0.5;
                let d01 = ((depth + 1.3) / 2.6).clamp(0.2, 1.0);
                let base = if strand == 0 {
                    theme.c(2)
                } else {
                    theme.c(3).lerp(theme.hot(), 0.25)
                };
                let ch = line_ch(x1 - x0, y1 - y0, unicode);
                marks.push((
                    depth,
                    Mark::Line {
                        x0: x0.round() as isize,
                        y0: y0.round() as isize,
                        x1: x1.round() as isize,
                        y1: y1.round() as isize,
                        ch,
                        col: base.scale(0.45 + 0.55 * d01),
                        glow: d01 > 0.58,
                    },
                ));
            }
            {
                for pts in [&p0, &p1] {
                    let (x, y, z, _, _) = pts[i];
                    let d01 = ((z + 1.3) / 2.6).clamp(0.2, 1.0);
                    let col = theme
                        .c(2)
                        .lerp(WHITE, 0.2 + 0.4 * d01)
                        .scale(0.70 + 0.30 * d01);
                    marks.push((
                        z + 0.01,
                        Mark::Glyph {
                            x: x.round() as isize,
                            y: y.round() as isize,
                            ch: if unicode {
                                if d01 > 0.65 {
                                    '●'
                                } else {
                                    '○'
                                }
                            } else if d01 > 0.72 {
                                '@'
                            } else if d01 > 0.36 {
                                '#'
                            } else {
                                '+'
                            },
                            col,
                            glow: d01 > 0.65,
                        },
                    ));
                }
            }
        }

        // Base-pair rungs, letters and replication fork.
        for i in (2..SAMPLES - 2).step_by(PAIR_STEP) {
            let (x0, y0, z0, gy, fork) = p0[i];
            let (x1, y1, z1, _, _) = p1[i];
            let depth = (z0 + z1) * 0.5;
            let d01 = ((depth + 1.3) / 2.6).clamp(0.2, 1.0);
            let base = BASES[(i / PAIR_STEP) % BASES.len()];
            let mate = partner(base);
            let base_col = BASE_COLORS[base_index(base)].scale(0.45 + 0.55 * d01);
            let mate_col = BASE_COLORS[base_index(mate)].scale(0.45 + 0.55 * d01);
            let bond = Rgb::hex(0x94a3b8).scale(0.16 + 0.24 * d01);
            let flash = (gy - scan_y).abs() < 0.045;
            let base_col = if flash {
                base_col.lerp(WHITE, 0.70)
            } else {
                base_col
            };
            let mate_col = if flash {
                mate_col.lerp(WHITE, 0.70)
            } else {
                mate_col
            };

            if fork > 0.12 {
                let cut = (fork * 0.38).clamp(0.08, 0.42);
                let ax = x0 + (x1 - x0) * (0.5 - cut);
                let ay = y0 + (y1 - y0) * (0.5 - cut);
                let bx = x0 + (x1 - x0) * (0.5 + cut);
                let by = y0 + (y1 - y0) * (0.5 + cut);
                marks.push((
                    depth,
                    Mark::Line {
                        x0: x0.round() as isize,
                        y0: y0.round() as isize,
                        x1: ax.round() as isize,
                        y1: ay.round() as isize,
                        ch: if unicode { '═' } else { '-' },
                        col: bond,
                        glow: d01 > 0.75,
                    },
                ));
                marks.push((
                    depth,
                    Mark::Line {
                        x0: x1.round() as isize,
                        y0: y1.round() as isize,
                        x1: bx.round() as isize,
                        y1: by.round() as isize,
                        ch: if unicode { '═' } else { '-' },
                        col: bond,
                        glow: d01 > 0.75,
                    },
                ));
            } else {
                marks.push((
                    depth,
                    Mark::Line {
                        x0: x0.round() as isize,
                        y0: y0.round() as isize,
                        x1: x1.round() as isize,
                        y1: y1.round() as isize,
                        ch: if unicode { '═' } else { '-' },
                        col: bond,
                        glow: d01 > 0.75,
                    },
                ));
                marks.push((
                    depth + 0.02,
                    Mark::Glyph {
                        x: ((x0 + x1) * 0.5).round() as isize,
                        y: ((y0 + y1) * 0.5).round() as isize,
                        ch: if unicode { '✦' } else { '*' },
                        col: bond,
                        glow: d01 > 0.75,
                    },
                ));
            }

            let lt = if fork > 0.12 { 0.10 } else { 0.14 };
            let rt = 1.0 - lt;
            let lx = x0 + (x1 - x0) * lt;
            let ly = y0 + (y1 - y0) * lt;
            let rx = x0 + (x1 - x0) * rt;
            let ry = y0 + (y1 - y0) * rt;
            let bias = 0.03;
            marks.push((
                depth + bias,
                Mark::Glyph {
                    x: lx.round() as isize,
                    y: ly.round() as isize,
                    ch: base as char,
                    col: base_col,
                    glow: d01 > 0.6,
                },
            ));
            marks.push((
                depth + bias,
                Mark::Glyph {
                    x: rx.round() as isize,
                    y: ry.round() as isize,
                    ch: mate as char,
                    col: mate_col,
                    glow: d01 > 0.6,
                },
            ));
        }

        marks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        for (_, mark) in marks {
            draw_mark(canvas, max_y, mark);
        }

        // The scan beam sweeps over the finished molecule, lighting a row.
        if scan_row >= 1 && scan_row <= max_y {
            let half = (w * 0.36) as isize;
            for x in (cx as isize - half)..=(cx as isize + half) {
                canvas.add_light(
                    x,
                    scan_row,
                    Rgb::hex(0x67e8f9),
                    0.42,
                    if x % 3 == 0 { '=' } else { '-' },
                );
            }
            canvas.add_light(
                cx as isize,
                (scan_row - 1).max(1),
                Rgb::hex(0x22d3ee),
                0.10,
                '.',
            );
            if scan_row < max_y {
                canvas.add_light(cx as isize, scan_row + 1, WHITE, 0.22, '.');
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
            theme: crate::theme::find("aurora").unwrap(),
            charset,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    #[test]
    fn helix_has_strands_pairs_and_depth_colors() {
        let mut scene = HelixScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let f = frame(8.0, Charset::Ascii);
        scene.update(&f);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(
            text.contains('@') || text.contains('#') || text.contains('+'),
            "no strand"
        );
        assert!(text.contains('A') || text.contains('T'), "no base letters");
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
    fn helix_unicode_uses_box_drawing_glyphs() {
        let mut scene = HelixScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        let f = frame(8.0, Charset::Unicode);
        scene.update(&f);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(!text.is_ascii(), "unicode mode stayed ASCII");
    }
}
