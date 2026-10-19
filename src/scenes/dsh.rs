//! dsh — a giant DEEPSEEK HARNESS wordmark with afterimage and scan beams.
//!
//! The wordmark is hand-built from a compact 3x5 / 5x7 pixel font, so the
//! scene needs no external font files.  It decodes in from random glyphs,
//! drags a slightly swaying afterimage downward, and is crossed by horizontal
//! and vertical light beams.  A short VHS-style slice glitch fires every few
//! seconds.  ASCII is the default; `--unicode` swaps the pixel fill for solid
//! blocks and the drips for box-drawing glyphs.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, hash2, Rng};
use crate::scene::{Frame, Scene};

const TEXT: &str = "DEEPSEEK HARNESS";
const NOISE: &[u8] = b"01#*+=-<>/\\|";

// ---------------------------------------------------------------------------
// Hand-drawn font.  Only the glyphs used by TEXT are needed.
// ---------------------------------------------------------------------------

const F5_D: [&str; 7] = [
    "####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####.",
];
const F5_E: [&str; 7] = [
    "#####", "#....", "#....", "####.", "#....", "#....", "#####",
];
const F5_P: [&str; 7] = [
    "####.", "#...#", "#...#", "####.", "#....", "#....", "#....",
];
const F5_S: [&str; 7] = [
    ".####", "#....", "#....", ".###.", "....#", "....#", "####.",
];
const F5_K: [&str; 7] = [
    "#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#",
];
const F5_H: [&str; 7] = [
    "#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
];
const F5_A: [&str; 7] = [
    ".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
];
const F5_R: [&str; 7] = [
    "####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#",
];
const F5_N: [&str; 7] = [
    "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#", "#...#",
];

const F4_D: [&str; 5] = ["###.", "#..#", "#..#", "#..#", "###."];
const F4_E: [&str; 5] = ["####", "#...", "###.", "#...", "####"];
const F4_P: [&str; 5] = ["###.", "#..#", "###.", "#...", "#..."];
const F4_S: [&str; 5] = [".###", "#...", ".###", "...#", "###."];
const F4_K: [&str; 5] = ["#..#", "#.#.", "##..", "#.#.", "#..#"];
const F4_H: [&str; 5] = ["#..#", "#..#", "####", "#..#", "#..#"];
const F4_A: [&str; 5] = [".##.", "#..#", "####", "#..#", "#..#"];
const F4_R: [&str; 5] = ["###.", "#..#", "###.", "#.#.", "#..#"];
const F4_N: [&str; 5] = ["#..#", "##.#", "#.##", "#..#", "#..#"];

const F3_D: [&str; 5] = ["##.", "#.#", "#.#", "#.#", "##."];
const F3_E: [&str; 5] = ["###", "#..", "##.", "#..", "###"];
const F3_P: [&str; 5] = ["###", "#.#", "###", "#..", "#.."];
const F3_S: [&str; 5] = ["###", "#..", "###", "..#", "###"];
const F3_K: [&str; 5] = ["#.#", "#.#", "##.", "#.#", "#.#"];
const F3_H: [&str; 5] = ["#.#", "#.#", "###", "#.#", "#.#"];
const F3_A: [&str; 5] = [".#.", "#.#", "###", "#.#", "#.#"];
const F3_R: [&str; 5] = ["###", "#.#", "##.", "#.#", "#.#"];
const F3_N: [&str; 5] = ["#.#", "###", "#.#", "#.#", "#.#"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Face {
    Wide,
    Mid,
    Compact,
}

impl Face {
    fn rows(self) -> usize {
        if self == Face::Wide {
            7
        } else {
            5
        }
    }

    fn glyph_w(self) -> usize {
        match self {
            Face::Wide => 5,
            Face::Mid => 4,
            Face::Compact => 3,
        }
    }

    fn space_w(self) -> usize {
        match self {
            Face::Wide => 3,
            Face::Mid | Face::Compact => 2,
        }
    }
}

fn glyph(ch: char, face: Face) -> Option<&'static [&'static str]> {
    match face {
        Face::Wide => match ch {
            'D' => Some(&F5_D[..]),
            'E' => Some(&F5_E[..]),
            'P' => Some(&F5_P[..]),
            'S' => Some(&F5_S[..]),
            'K' => Some(&F5_K[..]),
            'H' => Some(&F5_H[..]),
            'A' => Some(&F5_A[..]),
            'R' => Some(&F5_R[..]),
            'N' => Some(&F5_N[..]),
            _ => None,
        },
        Face::Mid => match ch {
            'D' => Some(&F4_D[..]),
            'E' => Some(&F4_E[..]),
            'P' => Some(&F4_P[..]),
            'S' => Some(&F4_S[..]),
            'K' => Some(&F4_K[..]),
            'H' => Some(&F4_H[..]),
            'A' => Some(&F4_A[..]),
            'R' => Some(&F4_R[..]),
            'N' => Some(&F4_N[..]),
            _ => None,
        },
        Face::Compact => match ch {
            'D' => Some(&F3_D[..]),
            'E' => Some(&F3_E[..]),
            'P' => Some(&F3_P[..]),
            'S' => Some(&F3_S[..]),
            'K' => Some(&F3_K[..]),
            'H' => Some(&F3_H[..]),
            'A' => Some(&F3_A[..]),
            'R' => Some(&F3_R[..]),
            'N' => Some(&F3_N[..]),
            _ => None,
        },
    }
}

fn text_width(face: Face, gap: usize) -> usize {
    let chars: Vec<char> = TEXT.chars().collect();
    let mut width = 0;
    for (i, ch) in chars.iter().enumerate() {
        width += if *ch == ' ' {
            face.space_w()
        } else {
            face.glyph_w()
        };
        if i + 1 < chars.len() {
            width += gap;
        }
    }
    width
}

// ---------------------------------------------------------------------------
// Layout and state.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Layout {
    face: Face,
    gap: usize,
    sy: usize,
    left: isize,
    top: isize,
    text_w: usize,
    rows: usize,
}

impl Layout {
    fn box_h(self) -> usize {
        self.rows * self.sy
    }
}

#[derive(Clone, Copy)]
struct MaskCell {
    x: isize,
    y: isize,
    col: u8,
    row_frac: f32,
}

struct Drip {
    x: f32,
    y: f32,
    vy: f32,
    sway: f32,
    phase: f32,
    life: f32,
    max_life: f32,
}

pub struct DshScene {
    rng: Rng,
    w: usize,
    h: usize,
    layout: Option<Layout>,
    mask: Vec<MaskCell>,
    grid: Vec<bool>,
    drips: Vec<Drip>,
    h_phase: f32,
    v_phase: f32,
    g_phase: f32,
    spawn_clock: f32,
}

impl DshScene {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x4453_4800);
        let h_phase = rng.f32();
        let v_phase = rng.f32();
        let g_phase = rng.range(0.0, 11.0);
        DshScene {
            rng,
            w: 0,
            h: 0,
            layout: None,
            mask: Vec::new(),
            grid: Vec::new(),
            drips: Vec::new(),
            h_phase,
            v_phase,
            g_phase,
            spawn_clock: 0.0,
        }
    }

    fn choose_layout(w: usize, h: usize) -> Option<Layout> {
        if w < 8 || h < 8 {
            return None;
        }
        let mut best: Option<(f32, Layout)> = None;
        let wide_gap = if w >= 190 {
            3
        } else if w >= 155 {
            2
        } else {
            1
        };
        for (face, gap) in [
            (Face::Wide, wide_gap),
            (Face::Mid, 2),
            (Face::Mid, 1),
            (Face::Compact, 3),
            (Face::Compact, 2),
            (Face::Compact, 1),
            (Face::Compact, 0),
        ] {
            if face == Face::Wide && w < 140 {
                continue;
            }
            let tw = text_width(face, gap);
            if tw + 4 > w {
                continue;
            }
            let rows = face.rows();
            let sy = if rows * 2 <= (h as f32 * 0.36) as usize {
                2
            } else {
                1
            };
            let box_h = rows * sy;
            let max_top = h as isize - 3 - box_h as isize;
            if max_top < 2 {
                continue;
            }
            let top = ((h as f32 * 0.14).round() as isize).clamp(2, max_top);
            let layout = Layout {
                face,
                gap,
                sy,
                left: ((w - tw) / 2) as isize,
                top,
                text_w: tw,
                rows,
            };
            // Aim for roughly 60% of the width and 30% of the height, with a
            // small preference for the more detailed faces.
            let mut score = (tw as f32 / w as f32 - 0.60).abs() * 2.0
                + (box_h as f32 / h as f32 - 0.30).abs() * 2.0;
            match face {
                Face::Wide => score -= 0.03,
                Face::Mid => score -= 0.015,
                Face::Compact => {}
            }
            if best.as_ref().map_or(true, |(s, _)| score < *s) {
                best = Some((score, layout));
            }
        }
        best.map(|(_, layout)| layout)
    }

    fn rebuild(&mut self) {
        self.mask.clear();
        self.grid.clear();
        self.drips.clear();
        self.spawn_clock = 0.0;
        let Some(layout) = Self::choose_layout(self.w, self.h) else {
            self.layout = None;
            return;
        };
        self.layout = Some(layout);
        self.grid = vec![false; self.w * self.h];
        let chars: Vec<char> = TEXT.chars().collect();
        let mut cx = layout.left;
        for (ci, ch) in chars.iter().enumerate() {
            if *ch == ' ' {
                cx += layout.face.space_w() as isize;
            } else if let Some(art) = glyph(*ch, layout.face) {
                for (r, line) in art.iter().enumerate() {
                    for (c, bit) in line.chars().enumerate() {
                        if bit != '#' {
                            continue;
                        }
                        for yy in 0..layout.sy {
                            let y = layout.top + (r * layout.sy + yy) as isize;
                            let row = r * layout.sy + yy;
                            let cell = MaskCell {
                                x: cx + c as isize,
                                y,
                                col: ci as u8,
                                row_frac: row as f32 / layout.box_h().max(1) as f32,
                            };
                            self.mask.push(cell);
                            if cell.x >= 0
                                && cell.y >= 0
                                && (cell.x as usize) < self.w
                                && (cell.y as usize) < self.h
                            {
                                self.grid[cell.y as usize * self.w + cell.x as usize] = true;
                            }
                        }
                    }
                }
                cx += layout.face.glyph_w() as isize;
            }
            if ci + 1 < chars.len() {
                cx += layout.gap as isize;
            }
        }
    }

    fn draw_compact(&self, canvas: &mut Canvas, f: &Frame) {
        let t = f.t as f32;
        let theme = f.theme;
        let max_y = self.h as isize - 3;
        let lines = ["DEEPSEEK", "HARNESS"];
        for (i, line) in lines.iter().enumerate() {
            let y = 2 + i as isize * 2;
            if y > max_y {
                break;
            }
            let x = ((self.w.saturating_sub(line.chars().count())) / 2) as isize;
            for k in 1..=2 {
                let yy = y + k;
                if yy > max_y {
                    break;
                }
                let col = theme.c(2).scale(0.28 / k as f32);
                canvas.put_str(x, yy, line, col);
            }
            canvas.put_str_gradient(x, y, line, &[theme.c(3), theme.hot(), theme.c(2)]);
        }
        if max_y >= 2 {
            let span = (max_y - 1).max(1) as f32;
            let hy = 2 + ((t * 2.4) % span) as isize;
            for x in 0..self.w as isize {
                canvas.set_if_blank(x, hy, '-', theme.c(3).scale(0.22));
            }
            let vx = ((t * 3.1) as isize).rem_euclid(self.w.max(1) as isize);
            for y in 2..=max_y {
                canvas.set_if_blank(vx, y, '|', theme.c(2).scale(0.18));
            }
        }
    }

    fn draw_trail(&self, canvas: &mut Canvas, f: &Frame, layout: Layout, max_y: isize) {
        let trail = f.tune.trail.clamp(0.0, 3.0);
        if trail <= 0.01 {
            return;
        }
        let t = f.t as f32;
        let glyph = if f.charset == Charset::Unicode {
            '█'
        } else {
            '#'
        };
        let len = (layout.box_h() as f32 * 0.85).round().max(1.0) as isize;
        for k in 1..=len {
            let kf = k as f32 / len as f32;
            let fade = (-kf * 3.2).exp() * 0.30 * trail;
            if fade < 0.02 {
                continue;
            }
            let sway = (t * 1.45 + k as f32 * 0.42).sin() * (0.2 + k as f32 * 0.07);
            for c in &self.mask {
                let y = c.y + k;
                if y > max_y || y < 1 {
                    continue;
                }
                let wave = (c.x as f32 * 0.035 + t * 0.8).sin() * 0.30;
                let x = c.x + (sway + wave).round() as isize;
                if hash2(x as i64, y as i64, 0x7a11) % 8 == 0 {
                    continue;
                }
                if kf > 0.55 && hash2(x as i64, y as i64, 0x7a12) % 3 == 0 {
                    continue;
                }
                let trail_glyph = if kf < 0.45 {
                    glyph
                } else if kf < 0.78 {
                    if f.charset == Charset::Unicode {
                        '▓'
                    } else {
                        '+'
                    }
                } else if f.charset == Charset::Unicode {
                    '░'
                } else {
                    ':'
                };
                let col = f.theme.c(2).lerp(f.theme.hot(), (1.0 - c.row_frac) * 0.35);
                canvas.set(x, y, trail_glyph, col.scale(fade * 1.9));
            }
        }
    }

    fn draw_wordmark(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let t = f.t as f32;
        let glyph = if f.charset == Charset::Unicode {
            '█'
        } else {
            '#'
        };
        let chars = TEXT.chars().count().max(1) as f32;
        for c in &self.mask {
            if c.y > max_y || c.y < 1 {
                continue;
            }
            let delay = (c.col as f32 / chars) * 0.75
                + c.row_frac * 0.30
                + hash01(c.x as i64, c.y as i64, 0x1234) * 0.20;
            let reveal = 0.15 + delay;
            let (ch, col) = if t < reveal {
                let frame = (t * 18.0) as i64;
                let idx =
                    hash2(c.x as i64, c.y as i64, 0x900d_u64 ^ frame as u64) as usize % NOISE.len();
                (
                    NOISE[idx] as char,
                    f.theme
                        .c(2)
                        .scale(0.40 + 0.18 * (t * 12.0 + c.x as f32).sin().abs()),
                )
            } else {
                let shimmer = 0.86 + 0.14 * (t * 2.1 - c.x as f32 * 0.12 + c.y as f32 * 0.25).sin();
                let breath = 0.92 + 0.08 * (t * 1.4 + c.col as f32 * 0.2).sin();
                (
                    glyph,
                    f.theme
                        .c(3)
                        .lerp(f.theme.hot(), (1.0 - c.row_frac) * 0.65)
                        .scale(shimmer * breath),
                )
            };
            canvas.set(c.x, c.y, ch, col);
            if (c.x + c.y) % 4 == 0 {
                canvas.add_light(c.x + 1, c.y, col, 0.10, '.');
            }
        }
    }

    fn draw_beams(&self, canvas: &mut Canvas, f: &Frame, layout: Layout, max_y: isize) {
        let t = f.t as f32;
        let w = self.w as isize;
        let left = layout.left - 3;
        let right = layout.left + layout.text_w as isize + 3;
        let top = (layout.top - 2).max(1);
        let bottom = (layout.top + layout.box_h() as isize + 4).min(max_y);
        if bottom < top {
            return;
        }
        let glyph = if f.charset == Charset::Unicode {
            '█'
        } else {
            '#'
        };

        // Horizontal sweep: top to bottom, with a tail above the head.
        let phase = (t / 3.2 + self.h_phase).rem_euclid(1.0);
        let hy = top as f32 + phase * (bottom - top).max(1) as f32;
        let hcol = f.theme.c(3);
        for i in 0..5 {
            let y = hy.round() as isize - i;
            if y < 1 || y > max_y {
                continue;
            }
            let alpha = if i == 0 {
                1.10
            } else {
                0.48 * (1.0 - i as f32 / 5.0)
            };
            for x in left..=right {
                if x < 0 || x >= w {
                    continue;
                }
                let hit = self.grid[y as usize * self.w + x as usize];
                if hit {
                    canvas.add_light(x, y, hcol, alpha, glyph);
                } else if alpha > 0.2 {
                    let (mark, amount) = if i == 0 {
                        ('-', 0.55)
                    } else {
                        ('.', alpha * 0.30)
                    };
                    canvas.add_light(x, y, hcol, amount, mark);
                }
            }
        }
        let gy = hy.round() as isize + 1;
        if gy >= 1 && gy <= max_y {
            for x in left..=right {
                if x >= 0 && x < w {
                    canvas.add_light(x, gy, hcol, 0.12, '.');
                }
            }
        }

        // Vertical sweep: left to right, tail behind the head.
        let phase = (t / 5.1 + self.v_phase).rem_euclid(1.0);
        let vx = left as f32 + phase * (right - left).max(1) as f32;
        let vcol = f.theme.hot();
        for i in 0..4 {
            let x = vx.round() as isize - i;
            if x < 0 || x >= w {
                continue;
            }
            let alpha = if i == 0 {
                1.05
            } else {
                0.42 * (1.0 - i as f32 / 4.0)
            };
            for y in top..=bottom {
                let hit = self.grid[y as usize * self.w + x as usize];
                if hit {
                    canvas.add_light(x, y, vcol, alpha, glyph);
                } else if alpha > 0.18 {
                    let (mark, amount) = if i == 0 {
                        ('|', 0.50)
                    } else {
                        (':', alpha * 0.28)
                    };
                    canvas.add_light(x, y, vcol, amount, mark);
                }
            }
        }

        // Bright cross-point where the two sweeps meet.
        let ix = vx.round() as isize;
        let iy = hy.round() as isize;
        if ix >= left && ix <= right && iy >= top && iy <= bottom {
            canvas.add_light(ix, iy, Rgb::WHITE, 1.4, '+');
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                canvas.add_light(ix + dx, iy + dy, f.theme.c(2), 0.25, '.');
            }
        }
    }

    fn draw_glitch(&self, canvas: &mut Canvas, f: &Frame, layout: Layout, max_y: isize) {
        let t = f.t as f32;
        if (t + self.g_phase).rem_euclid(11.0) >= 0.14 {
            return;
        }
        let box_h = layout.box_h().max(1) as u64;
        let row = hash2((t * 20.0) as i64, 0, 0x91) as u64 % box_h;
        let slice = layout.top + row as isize;
        let dx = if hash2((t * 30.0) as i64, 1, 0x92) & 1 == 0 {
            3
        } else {
            -3
        };
        let red = Rgb::hex(0xff3b5c);
        let cyan = Rgb::hex(0x22d3ee);
        let glyph = if f.charset == Charset::Unicode {
            '█'
        } else {
            '#'
        };
        for c in &self.mask {
            if c.y != slice && c.y != slice + 1 {
                continue;
            }
            if c.y < 1 || c.y > max_y {
                continue;
            }
            canvas.blend(c.x + dx, c.y, glyph, red, 0.55);
            canvas.blend(c.x - dx, c.y, glyph, cyan, 0.45);
        }
        for x in layout.left..layout.left + layout.text_w as isize {
            if x < 0 || x >= self.w as isize {
                continue;
            }
            if hash2(x as i64, slice as i64, 0x93) % 3 == 0 {
                canvas.blend(
                    x,
                    slice,
                    if f.charset == Charset::Unicode {
                        '▒'
                    } else {
                        '='
                    },
                    f.theme.hot(),
                    0.30,
                );
            }
        }
    }
}

impl Scene for DshScene {
    fn resize(&mut self, w: usize, h: usize) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        self.rebuild();
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let trail = f.tune.trail.clamp(0.0, 3.0);
        if trail > 0.01 {
            self.spawn_clock += dt;
            let interval = 0.16 / trail.max(0.1);
            while self.spawn_clock >= interval {
                self.spawn_clock -= interval;
                if self.drips.len() >= 70 {
                    break;
                }
                let Some(layout) = self.layout else {
                    break;
                };
                if self.mask.is_empty() {
                    break;
                }
                let cell = self.mask[self.rng.below(self.mask.len())];
                let y = cell.y as f32 + layout.box_h() as f32 * 0.9;
                if y >= self.h as f32 - 2.0 {
                    continue;
                }
                let life = self.rng.range(1.4, 3.0);
                let drip = Drip {
                    x: cell.x as f32 + self.rng.range(-0.4, 0.4),
                    y,
                    vy: self.rng.range(8.0, 24.0),
                    sway: self.rng.range(0.4, 1.8),
                    phase: self.rng.range(0.0, TAU),
                    life,
                    max_life: life,
                };
                self.drips.push(drip);
            }
        }
        let t = f.t as f32;
        let bottom = self.h as f32 - 2.0;
        for d in &mut self.drips {
            d.y += d.vy * dt;
            d.x += (t * 2.0 + d.phase).sin() * d.sway * dt;
            d.life -= dt;
        }
        self.drips.retain(|d| d.life > 0.0 && d.y < bottom);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let max_y = self.h as isize - 3;
        if max_y < 1 {
            return;
        }
        let t = f.t as f32;
        let theme = f.theme;

        // Sparse twinkling starfield, kept out of the wordmark box so the
        // letter gaps stay readable.
        let text_box = self
            .layout
            .map(|l| (l.left, l.top, l.text_w as isize, l.box_h() as isize));
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                if let Some((bx, by, bw, bh)) = text_box {
                    if x >= bx && x < bx + bw && y >= by && y < by + bh {
                        continue;
                    }
                }
                let hsh = hash2(x as i64, y as i64, 0x51ee);
                if hsh % 113 != 0 {
                    continue;
                }
                let tw = 0.5 + 0.5 * (t * 1.3 + (hsh % 100) as f32 * 0.1).sin().abs();
                let ch = if hsh & 7 == 0 { '+' } else { '.' };
                canvas.set(x, y, ch, theme.c(2).scale(0.16 * tw));
            }
        }

        let Some(layout) = self.layout else {
            self.draw_compact(canvas, f);
            return;
        };

        self.draw_trail(canvas, f, layout, max_y);
        self.draw_wordmark(canvas, f, max_y);

        for d in &self.drips {
            let y = d.y.round() as isize;
            if y < 1 || y > max_y {
                continue;
            }
            let fade = (d.life / d.max_life.max(0.01)).clamp(0.0, 1.0);
            let ch = if f.charset == Charset::Unicode {
                '│'
            } else {
                '|'
            };
            canvas.set(
                d.x.round() as isize,
                y,
                ch,
                theme.c(2).scale(0.25 + 0.7 * fade),
            );
        }

        self.draw_beams(canvas, f, layout, max_y);
        self.draw_glitch(canvas, f, layout, max_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_fits_common_terminal_sizes() {
        for &(w, h) in &[(160, 48), (120, 36), (100, 30), (80, 24), (64, 24)] {
            let layout = DshScene::choose_layout(w, h).expect("layout");
            assert!(layout.left >= 0, "{w}x{h}");
            assert!(
                layout.left + layout.text_w as isize <= w as isize,
                "{w}x{h}"
            );
            assert!(
                layout.top + layout.box_h() as isize <= h as isize - 3,
                "{w}x{h}"
            );
        }
        assert!(DshScene::choose_layout(40, 12).is_none());
    }

    #[test]
    fn wordmark_mask_stays_inside_the_safe_area() {
        let mut scene = DshScene::new(7);
        scene.resize(120, 36);
        let layout = scene.layout.unwrap();
        assert!(!scene.mask.is_empty());
        for cell in &scene.mask {
            assert!(cell.x >= 0 && cell.x < 120);
            assert!(cell.y >= 1 && cell.y <= 33);
        }
        assert!(layout.text_w > 60);
    }

    #[test]
    fn default_ascii_output_is_ascii_only() {
        let mut scene = DshScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        for step in 0..20 {
            let frame = Frame {
                t: step as f64 / 30.0,
                dt: 1.0 / 30.0,
                w: 80,
                h: 24,
                fps: 30,
                theme: crate::theme::find("ice").unwrap(),
                charset: Charset::Ascii,
                seed: 7,
                speed: 1.0,
                tune: crate::scene::Tune::default(),
            };
            scene.update(&frame);
            scene.draw(&mut canvas, &frame);
        }
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }
}
