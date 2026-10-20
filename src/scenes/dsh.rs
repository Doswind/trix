//! dsh — a giant DEEPSEEK HARNESS wordmark, revealed, dropped and scanned.
//!
//! The wordmark is built from a hand-drawn pixel font and laid out as two
//! staggered lines so the letters can be much wider than a single line allows.
//! The intro reveals the letters one by one, then drops the lockup in stuttered
//! steps and leaves a ghost stack behind.  After that the wordmark stays on
//! screen forever and a cross of light loops over it: a horizontal beam floats
//! down the lockup, holds at the centre while a vertical beam joins it to form
//! a cross, then the vertical beam sweeps across while the horizontal one fades.
//! ASCII is the default; `--unicode` swaps the fill for solid blocks.

#![allow(clippy::needless_range_loop)]

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::hash2;
use crate::scene::{Frame, Scene};

/// One full horizontal + vertical beam pass.  The intro (reveal and drop) runs
/// once at scene start; the beam loop continues for as long as the scene lives.
const BEAM_START: f32 = 4.4;
const BEAM_LOOP: f32 = 7.0;
const H_DOWN: f32 = 1.0;
const CROSS_HOLD: f32 = 1.7;
const H_END: f32 = 2.9;
const V_START: f32 = 3.4;
const V_END: f32 = 6.0;

const T_DROP: f32 = 3.0;
const DROP_STEP: f32 = 0.14;

const RED: Rgb = Rgb::hex(0xff3b5c);
const CYAN: Rgb = Rgb::hex(0x38bdf8);

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

// 6x9 display face: 2-row bars, 1-column stems.  Drawn with sx=2 so the
// stems become two columns wide, matching the chunky reference lockup.
const F6_D: [&str; 9] = [
    "######", "#....#", "#....#", "#....#", "#....#", "#....#", "#....#", "#....#", "######",
];
const F6_E: [&str; 9] = [
    "######", "#.....", "#.....", "#.....", "#####.", "#.....", "#.....", "#.....", "######",
];
const F6_P: [&str; 9] = [
    "######", "#....#", "#....#", "#....#", "######", "#.....", "#.....", "#.....", "#.....",
];
const F6_S: [&str; 9] = [
    ".#####", "#.....", "#.....", ".#####", ".....#", ".....#", ".....#", "#....#", ".#####",
];
const F6_K: [&str; 9] = [
    "#....#", "#....#", "#...#.", "#..#..", "###...", "###...", "#..#..", "#...#.", "#....#",
];
const F6_H: [&str; 9] = [
    "#....#", "#....#", "#....#", "#....#", "######", "#....#", "#....#", "#....#", "#....#",
];
const F6_A: [&str; 9] = [
    "..##..", ".#..#.", "#....#", "#....#", "######", "#....#", "#....#", "#....#", "#....#",
];
const F6_R: [&str; 9] = [
    "######", "#....#", "#....#", "#....#", "######", "#..#..", "#...#.", "#....#", "#....#",
];
const F6_N: [&str; 9] = [
    "#....#", "##...#", "##...#", "#.#..#", "#.#..#", "#..#.#", "#...##", "#....#", "#....#",
];

// 8x11 display face for wide terminals.
const F8_D: [&str; 11] = [
    "########", "##....##", "##....##", "##....##", "##....##", "##....##", "##....##", "##....##",
    "##....##", "##....##", "########",
];
const F8_E: [&str; 11] = [
    "########", "##......", "##......", "##......", "######..", "##......", "##......", "##......",
    "##......", "##......", "########",
];
const F8_P: [&str; 11] = [
    "########", "##....##", "##....##", "##....##", "########", "##......", "##......", "##......",
    "##......", "##......", "##......",
];
const F8_S: [&str; 11] = [
    ".######.", "##......", "##......", "##......", ".######.", "......##", "##....##", "##....##",
    "##....##", "##....##", ".######.",
];
const F8_K: [&str; 11] = [
    "##....##", "##...##.", "##..##..", "##.##...", "####....", "####....", "##.##...", "##..##..",
    "##...##.", "##....##", "##....##",
];
const F8_H: [&str; 11] = [
    "##....##", "##....##", "##....##", "##....##", "##....##", "########", "##....##", "##....##",
    "##....##", "##....##", "##....##",
];
const F8_A: [&str; 11] = [
    "...##...", "..####..", ".##..##.", "##....##", "##....##", "########", "##....##", "##....##",
    "##....##", "##....##", "##....##",
];
const F8_R: [&str; 11] = [
    "########", "##....##", "##....##", "##....##", "########", "##..##..", "##...##.", "##....##",
    "##....##", "##....##", "##....##",
];
const F8_N: [&str; 11] = [
    "##....##", "###...##", "###...##", "##.#..##", "##.#..##", "##..#.##", "##...###", "##....##",
    "##....##", "##....##", "##....##",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Face {
    Wide,
    Mid,
    Compact,
    Six,
    Eight,
}

impl Face {
    fn rows(self) -> usize {
        match self {
            Face::Wide => 7,
            Face::Mid | Face::Compact => 5,
            Face::Six => 9,
            Face::Eight => 11,
        }
    }

    fn glyph_w(self) -> usize {
        match self {
            Face::Wide => 5,
            Face::Mid => 4,
            Face::Compact => 3,
            Face::Six => 6,
            Face::Eight => 8,
        }
    }

    fn space_w(self) -> usize {
        2
    }
}

fn glyph(ch: char, face: Face) -> Option<&'static [&'static str]> {
    let table: &'static [&'static str] = match face {
        Face::Wide => match ch {
            'D' => &F5_D,
            'E' => &F5_E,
            'P' => &F5_P,
            'S' => &F5_S,
            'K' => &F5_K,
            'H' => &F5_H,
            'A' => &F5_A,
            'R' => &F5_R,
            'N' => &F5_N,
            _ => return None,
        },
        Face::Mid => match ch {
            'D' => &F4_D,
            'E' => &F4_E,
            'P' => &F4_P,
            'S' => &F4_S,
            'K' => &F4_K,
            'H' => &F4_H,
            'A' => &F4_A,
            'R' => &F4_R,
            'N' => &F4_N,
            _ => return None,
        },
        Face::Compact => match ch {
            'D' => &F3_D,
            'E' => &F3_E,
            'P' => &F3_P,
            'S' => &F3_S,
            'K' => &F3_K,
            'H' => &F3_H,
            'A' => &F3_A,
            'R' => &F3_R,
            'N' => &F3_N,
            _ => return None,
        },
        Face::Six => match ch {
            'D' => &F6_D,
            'E' => &F6_E,
            'P' => &F6_P,
            'S' => &F6_S,
            'K' => &F6_K,
            'H' => &F6_H,
            'A' => &F6_A,
            'R' => &F6_R,
            'N' => &F6_N,
            _ => return None,
        },
        Face::Eight => match ch {
            'D' => &F8_D,
            'E' => &F8_E,
            'P' => &F8_P,
            'S' => &F8_S,
            'K' => &F8_K,
            'H' => &F8_H,
            'A' => &F8_A,
            'R' => &F8_R,
            'N' => &F8_N,
            _ => return None,
        },
    };
    Some(table)
}

fn text_width(face: Face, sx: usize, gap: usize, text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut width = 0;
    for (i, ch) in chars.iter().enumerate() {
        width += if *ch == ' ' {
            face.space_w() * sx
        } else {
            face.glyph_w() * sx
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
struct LineLayout {
    face: Face,
    sx: usize,
    sy: usize,
    left: isize,
    top: isize,
    width: usize,
    rows: usize,
    gap_x: usize,
}

impl LineLayout {
    fn box_h(self) -> usize {
        self.rows * self.sy
    }
}

#[derive(Clone, Copy)]
struct Layout {
    line1: LineLayout,
    line2: LineLayout,
    drop_rows: isize,
    ghost_rows: isize,
}

#[derive(Clone, Copy)]
struct MaskCell {
    x: isize,
    y: isize,
    letter: u8,
    row_frac: f32,
}

/// Position and opacity of the two beams for one instant of the loop.
#[derive(Clone, Copy, Default)]
struct BeamState {
    /// 0 = above the lockup, 1 = below it.
    horizontal_t: f32,
    horizontal_alpha: f32,
    /// 0 = left of the lockup, 1 = right of it.
    vertical_t: f32,
    vertical_alpha: f32,
}

pub struct DshScene {
    w: usize,
    h: usize,
    layout: Option<Layout>,
    mask: Vec<MaskCell>,
    grid: Vec<bool>,
}

impl DshScene {
    pub fn new(_seed: u64) -> Self {
        DshScene {
            w: 0,
            h: 0,
            layout: None,
            mask: Vec::new(),
            grid: Vec::new(),
        }
    }

    fn choose_layout(w: usize, h: usize) -> Option<Layout> {
        if w < 60 || h < 16 {
            return None;
        }
        let candidates = [
            (Face::Eight, 3usize, 1usize),
            (Face::Eight, 3, 0),
            (Face::Eight, 2, 1),
            (Face::Six, 2, 1),
            (Face::Wide, 2, 1),
            (Face::Mid, 2, 1),
            (Face::Compact, 2, 1),
        ];
        let mut picked = None;
        for (face, sx, gap_x) in candidates {
            if sx == 3 && gap_x == 1 && w < 204 {
                continue;
            }
            if sx == 3 && gap_x == 0 && w < 192 {
                continue;
            }
            if face == Face::Eight && sx == 2 && h < 34 {
                continue;
            }
            let w1 = text_width(face, sx, gap_x, "DEEPSEEK");
            let w2 = text_width(face, sx, gap_x, "HARNESS");
            if w1 + 4 <= w && w2 + 4 <= w {
                picked = Some((face, sx, gap_x, w1, w2));
                break;
            }
        }
        let (face, sx, gap_x, w1, w2) = picked?;
        let rows = face.rows();
        let line_gap = if rows >= 9 { 2 } else { 1 };
        let block = rows * 2 + line_gap;
        let max_y = h as isize - 3;
        let mut drop = ((h as f32 * 0.10).round() as isize).clamp(2, 5);
        let mut ghost = (drop + 1).clamp(3, 6);
        while block as isize + drop + ghost > max_y && (ghost > 2 || drop > 2) {
            if ghost > 2 {
                ghost -= 1;
            } else {
                drop -= 1;
            }
        }
        if block as isize + drop + ghost > max_y {
            return None;
        }
        let left1 = ((w - w1) / 2).max(2) as isize;
        let max2 = w as isize - w2 as isize - 2;
        let extra = ((w as f32 * 0.03).round() as isize).max(2);
        let centered2 = ((w - w2) / 2) as isize;
        let left2 = (centered2 + extra).min(max2).max(left1 + 3);
        let spare = max_y - (block as isize + drop + ghost);
        let bias = if spare > 4 { 2 } else { 0 };
        let top = ((spare / 2) + bias).max(1);
        let line1 = LineLayout {
            face,
            sx,
            sy: 1,
            left: left1,
            top,
            width: w1,
            rows,
            gap_x,
        };
        let line2 = LineLayout {
            face,
            sx,
            sy: 1,
            left: left2,
            top: top + rows as isize + line_gap as isize,
            width: w2,
            rows,
            gap_x,
        };
        Some(Layout {
            line1,
            line2,
            drop_rows: drop,
            ghost_rows: ghost,
        })
    }

    fn rebuild(&mut self) {
        self.mask.clear();
        self.grid.clear();
        let Some(layout) = Self::choose_layout(self.w, self.h) else {
            self.layout = None;
            return;
        };
        self.layout = Some(layout);
        self.grid = vec![false; self.w * self.h];
        self.build_line(layout.line1, "DEEPSEEK", 0);
        self.build_line(layout.line2, "HARNESS", 8);
    }

    fn build_line(&mut self, line: LineLayout, text: &str, base_letter: u8) {
        let mut x = line.left;
        for (i, ch) in text.chars().enumerate() {
            let Some(art) = glyph(ch, line.face) else {
                x += (line.face.glyph_w() * line.sx + line.gap_x) as isize;
                continue;
            };
            let letter = base_letter + i as u8;
            for (r, row) in art.iter().enumerate() {
                for (c, bit) in row.chars().enumerate() {
                    if bit != '#' {
                        continue;
                    }
                    for dy in 0..line.sy {
                        for dx in 0..line.sx {
                            let px = x + (c * line.sx + dx) as isize;
                            let py = line.top + (r * line.sy + dy) as isize;
                            if px < 0 || py < 0 || px >= self.w as isize || py >= self.h as isize {
                                continue;
                            }
                            let row_frac = (r * line.sy + dy) as f32 / line.box_h().max(1) as f32;
                            self.mask.push(MaskCell {
                                x: px,
                                y: py,
                                letter,
                                row_frac,
                            });
                            self.grid[py as usize * self.w + px as usize] = true;
                        }
                    }
                }
            }
            x += (line.face.glyph_w() * line.sx + line.gap_x) as isize;
        }
    }

    fn beam_state(&self, t: f32) -> BeamState {
        if t < BEAM_START {
            return BeamState::default();
        }
        let q = (t - BEAM_START).rem_euclid(BEAM_LOOP);
        let mut beam = BeamState::default();
        if q < H_END {
            // Horizontal beam: top -> centre, hold while the vertical beam
            // joins it, then centre -> bottom.
            if q < H_DOWN {
                beam.horizontal_t = 0.5 * smoothstep(q / H_DOWN);
            } else if q < CROSS_HOLD {
                beam.horizontal_t = 0.5;
            } else {
                beam.horizontal_t = 0.5 + 0.5 * smoothstep((q - CROSS_HOLD) / (H_END - CROSS_HOLD));
            }
            let fade_in = smoothstep(q / 0.18);
            let fade_out = 1.0 - smoothstep((q - (H_END - 0.25)) / 0.25);
            beam.horizontal_alpha = fade_in * fade_out;
            // The vertical beam fades in just before the horizontal beam
            // reaches the centre and fades out again as it resumes.
            let join = smoothstep((q - (H_DOWN - 0.15)) / 0.30);
            let leave = 1.0 - smoothstep((q - (CROSS_HOLD - 0.30)) / 0.30);
            beam.vertical_t = 0.5;
            beam.vertical_alpha = join * leave;
        } else if (V_START..V_END).contains(&q) {
            // Vertical beam sweeps left -> right across the whole lockup.
            beam.vertical_t = smoothstep((q - V_START) / (V_END - V_START));
            let fade_in = smoothstep((q - V_START) / 0.18);
            let fade_out = 1.0 - smoothstep((q - (V_END - 0.25)) / 0.25);
            beam.vertical_alpha = fade_in * fade_out;
        }
        beam
    }

    fn cell_at(&self, x: isize, y: isize, offset: isize) -> bool {
        let gy = y - offset;
        if x < 0 || gy < 0 || x >= self.w as isize || gy >= self.h as isize {
            return false;
        }
        self.grid[gy as usize * self.w + x as usize]
    }

    fn draw_compact(&self, canvas: &mut Canvas, f: &Frame) {
        let max_y = self.h as isize - 3;
        if max_y < 2 {
            return;
        }
        let theme = f.theme;
        for (i, line) in ["DEEPSEEK", "HARNESS"].iter().enumerate() {
            let y = 2 + i as isize * 2;
            if y > max_y {
                break;
            }
            let x = ((self.w.saturating_sub(line.chars().count())) / 2) as isize;
            canvas.put_str_gradient(x, y, line, &[theme.c(3), theme.hot(), theme.c(2)]);
        }
    }

    fn draw_background(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let theme = f.theme;
        let clean = self.layout.map(|layout| {
            (
                layout.line1.left - 1,
                layout.line1.left + layout.line1.width as isize + 1,
                layout.line1.top - 1,
                layout.line2.top
                    + layout.line2.box_h() as isize
                    + layout.drop_rows
                    + layout.ghost_rows
                    + 1,
            )
        });
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                if let Some((x0, x1, y0, y1)) = clean {
                    if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                        continue;
                    }
                }
                let hsh = hash2(x as i64, y as i64, 0x51ee);
                if hsh % 197 == 0 {
                    let tw = 0.5 + 0.5 * (f.t as f32 * 1.2 + (hsh % 100) as f32 * 0.1).sin().abs();
                    canvas.set(
                        x,
                        y,
                        if hsh & 7 == 0 { '+' } else { '.' },
                        theme.c(2).scale(0.10 + 0.12 * tw),
                    );
                } else if x % 4 == 0 && y % 2 == 0 && hsh % 9 == 0 {
                    canvas.set(x, y, '.', theme.c(2).scale(0.045));
                }
            }
        }
    }

    fn draw_ghosts(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let Some(layout) = self.layout else {
            return;
        };
        let trail = f.tune.trail.clamp(0.0, 3.0);
        if trail <= 0.01 {
            return;
        }
        let t = f.t as f32;
        let offset = drop_offset(t, layout.drop_rows);
        let onset = smoothstep((t - 2.2) / 0.8);
        if onset <= 0.0 {
            return;
        }
        let glyph = if f.charset == Charset::Unicode {
            ['▓', '░']
        } else {
            ['+', ':']
        };
        for k in 1..=layout.ghost_rows {
            let kf = k as f32 / (layout.ghost_rows + 1) as f32;
            let a = (1.0 - kf).powf(1.4) * 0.22 * trail * onset;
            if a < 0.015 {
                continue;
            }
            for c in &self.mask {
                let y = c.y + offset + k;
                // The two lines are only a couple of rows apart.  DEEPSEEK may
                // only cast a short ghost into that gap; HARNESS owns the full
                // afterimage below the lockup.
                if c.letter < 8 && (k > 2 || y >= layout.line2.top) {
                    continue;
                }
                let alpha = reveal_alpha(c, t - k as f32 * 0.035) * a;
                if alpha < 0.01 {
                    continue;
                }
                if y < 1 || y > max_y {
                    continue;
                }
                let ch = if k == 1 { glyph[0] } else { glyph[1] };
                let col = f.theme.c(2).lerp(RED, 0.18 + 0.20 * kf);
                canvas.set(c.x, y, ch, col.scale(alpha * 2.1));
                if k == 1 && alpha > 0.12 {
                    canvas.add_light(c.x, y, CYAN, alpha * 0.16, '.');
                }
            }
        }
    }

    fn draw_wordmark(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let Some(layout) = self.layout else {
            return;
        };
        let t = f.t as f32;
        let offset = drop_offset(t, layout.drop_rows);
        let unicode = f.charset == Charset::Unicode;
        for c in &self.mask {
            let alpha = reveal_alpha(c, t);
            if alpha < 0.012 {
                continue;
            }
            let y = c.y + offset;
            if y < 1 || y > max_y {
                continue;
            }
            let ch = if alpha < 0.30 {
                if unicode {
                    '░'
                } else {
                    ':'
                }
            } else if alpha < 0.68 {
                if unicode {
                    '▓'
                } else {
                    '+'
                }
            } else if unicode {
                '█'
            } else {
                '#'
            };
            let base = f.theme.hot().lerp(f.theme.c(2), 0.18 + c.row_frac * 0.38);
            // Chromatic fringe: cyan to the left, red to the right.  With a
            // one-cell inter-letter gap it would merge the glyphs, so it is
            // only used when the layout leaves room.
            if alpha > 0.35 && layout.line1.gap_x >= 2 {
                canvas.set(c.x + 1, y, ch, RED.scale(alpha * 0.16));
                canvas.set(c.x - 1, y, ch, CYAN.scale(alpha * 0.14));
            }
            let shimmer = 0.88 + 0.12 * (t * 2.1 - c.x as f32 * 0.12 + c.y as f32 * 0.2).sin();
            canvas.set(c.x, y, ch, base.scale(alpha * shimmer));
            if alpha > 0.8 && (c.x + c.y) % 7 == 0 {
                canvas.add_light(c.x, y, f.theme.c(2), 0.07, ch);
            }
        }
    }

    fn draw_beams(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let Some(layout) = self.layout else {
            return;
        };
        let t = f.t as f32;
        let beam = self.beam_state(t);
        if beam.horizontal_alpha <= 0.01 && beam.vertical_alpha <= 0.01 {
            return;
        }
        let offset = drop_offset(t, layout.drop_rows);
        let top = (layout.line1.top + offset) as f32;
        let center = (layout.line1.top + offset + layout.line1.box_h() as isize / 2) as f32;
        let bottom = (layout.line2.top + layout.line2.box_h() as isize + offset) as f32;
        let left = layout.line1.left.min(layout.line2.left) as f32;
        let right = (layout.line1.left + layout.line1.width as isize)
            .max(layout.line2.left + layout.line2.width as isize) as f32;
        let frac = beam.horizontal_t.clamp(0.0, 1.0);
        let h_y = if frac < 0.5 {
            top - 2.0 + (center - (top - 2.0)) * (frac / 0.5)
        } else {
            center + (bottom + 2.0 - center) * ((frac - 0.5) / 0.5)
        };
        let v_x = left - 2.0 + (right - left + 4.0) * beam.vertical_t;
        let unicode = f.charset == Charset::Unicode;
        let hbar = if unicode { '─' } else { '=' };
        let vbar = if unicode { '│' } else { '|' };
        if beam.horizontal_alpha > 0.01 {
            self.draw_horizontal_beam(
                canvas,
                h_y,
                beam.horizontal_alpha,
                left,
                right,
                offset,
                max_y,
                hbar,
            );
        }
        if beam.vertical_alpha > 0.01 {
            self.draw_vertical_beam(
                canvas,
                v_x,
                beam.vertical_alpha,
                top,
                bottom,
                offset,
                max_y,
                vbar,
            );
        }
        // The crossing point of the two beams is the brightest cell.
        if beam.horizontal_alpha > 0.2 && beam.vertical_alpha > 0.2 {
            let x = v_x.round() as isize;
            let y = h_y.round() as isize;
            if y >= 1 && y <= max_y {
                canvas.add_light(x, y, Rgb::WHITE, 2.2, '*');
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    canvas.add_light(x + dx, y + dy, Rgb::WHITE, 0.5, '+');
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_horizontal_beam(
        &self,
        canvas: &mut Canvas,
        y: f32,
        alpha: f32,
        left: f32,
        right: f32,
        offset: isize,
        max_y: isize,
        bar: char,
    ) {
        let y = y.round() as isize;
        if y < 1 || y > max_y {
            return;
        }
        let x0 = (left as isize - 2).max(0);
        let x1 = (right as isize + 2).min(self.w as isize - 1);
        for x in x0..=x1.max(x0) {
            let hit = self.cell_at(x, y, offset);
            let a = alpha * if hit { 1.0 } else { 0.82 };
            canvas.add_light(x, y, Rgb::WHITE, 1.30 * a, bar);
            if hit {
                canvas.add_light(x, y, Rgb::WHITE, 0.8 * a, bar);
            }
            if y > 1 {
                canvas.add_light(x, y - 1, CYAN, 0.40 * a, '.');
            }
            if y > 2 {
                canvas.add_light(x, y - 2, CYAN, 0.12 * a, '.');
            }
            if y < max_y {
                canvas.add_light(x, y + 1, RED, 0.34 * a, '.');
            }
            if y + 2 <= max_y {
                canvas.add_light(x, y + 2, RED, 0.10 * a, '.');
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_vertical_beam(
        &self,
        canvas: &mut Canvas,
        x: f32,
        alpha: f32,
        top: f32,
        bottom: f32,
        offset: isize,
        max_y: isize,
        bar: char,
    ) {
        let x = x.round() as isize;
        if x < 0 || x >= self.w as isize {
            return;
        }
        let y0 = ((top as isize - 2).max(1)).min(max_y);
        let y1 = ((bottom as isize + 2).min(max_y)).max(y0);
        for y in y0..=y1 {
            let hit = self.cell_at(x, y, offset);
            let a = alpha * if hit { 1.0 } else { 0.82 };
            canvas.add_light(x, y, Rgb::WHITE, 1.15 * a, bar);
            if hit {
                canvas.add_light(x, y, Rgb::WHITE, 0.7 * a, bar);
            }
            if x > 0 {
                canvas.add_light(x - 1, y, CYAN, 0.40 * a, '.');
            }
            if x > 1 {
                canvas.add_light(x - 2, y, CYAN, 0.12 * a, '.');
            }
            if x + 1 < self.w as isize {
                canvas.add_light(x + 1, y, RED, 0.34 * a, '.');
            }
            if x + 2 < self.w as isize {
                canvas.add_light(x + 2, y, RED, 0.10 * a, '.');
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

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let max_y = self.h as isize - 3;
        if max_y < 1 {
            return;
        }
        self.draw_background(canvas, f, max_y);
        if self.layout.is_none() {
            self.draw_compact(canvas, f);
            return;
        }
        self.draw_ghosts(canvas, f, max_y);
        self.draw_wordmark(canvas, f, max_y);
        self.draw_beams(canvas, f, max_y);
    }
}

fn drop_offset(t: f32, rows: isize) -> isize {
    if t < T_DROP {
        0
    } else {
        (((t - T_DROP) / DROP_STEP).floor() as isize).clamp(0, rows)
    }
}

fn reveal_alpha(c: &MaskCell, t: f32) -> f32 {
    let delay = c.letter as f32 * 0.13 + c.row_frac * 0.10;
    smoothstep((t - delay) / 0.38)
}

fn smoothstep(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_fits_common_terminal_sizes() {
        for &(w, h) in &[
            (200, 50),
            (160, 48),
            (120, 36),
            (100, 30),
            (80, 24),
            (60, 20),
        ] {
            let layout = DshScene::choose_layout(w, h).expect("layout");
            for line in [layout.line1, layout.line2] {
                assert!(line.left >= 0, "{w}x{h}");
                assert!(line.left + line.width as isize <= w as isize, "{w}x{h}");
                assert!(
                    line.top + line.box_h() as isize <= h as isize - 3,
                    "{w}x{h}"
                );
            }
            assert!(
                layout.line2.left > layout.line1.left + 2,
                "second line is not staggered at {w}x{h}"
            );
            let fill = layout.line1.width as f32 / w as f32;
            assert!(fill >= 0.72, "{w}x{h}: only {:.0}% wide", fill * 100.0);
            assert!(layout.line1.box_h() >= 5);
        }
        assert!(DshScene::choose_layout(40, 12).is_none());
        assert!(DshScene::choose_layout(59, 16).is_none());
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
        assert!(layout.line1.width >= 90);
        assert!(layout.line2.left > layout.line1.left);
        assert_eq!(drop_offset(0.0, 4), 0);
        assert_eq!(drop_offset(T_DROP + DROP_STEP * 1.1, 4), 1);
        assert_eq!(drop_offset(T_DROP + DROP_STEP * 3.1, 4), 3);
        assert!(reveal_alpha(&scene.mask[0], 0.2) > 0.0);
        assert_eq!(reveal_alpha(&scene.mask[0], 0.0), 0.0);
    }

    #[test]
    fn default_ascii_output_is_ascii_only() {
        let mut scene = DshScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        for step in 0..300 {
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
