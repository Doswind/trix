//! dsh — a giant DEEPSEEK HARNESS wordmark, revealed, dropped and lit.
//!
//! The wordmark is built from a hand-drawn pixel font and laid out as two
//! centred lines so the letters can be much wider than a single line allows.
//! The lines are sized down a notch to leave room for a hand-traced blue whale
//! mark that fades in to the right of the K after the drop.
//! The intro reveals the letters one by one, then drops the lockup in stuttered
//! steps; when it lands it makes one quick two-row hop back up before settling,
//! leaving a faint ghost stack under HARNESS.  After that the wordmark stays on
//! screen forever while a narrow diagonal band of light runs from its
//! upper-left corner to its lower-right corner: only the letters under the
//! band brighten to white, with soft edges, and they fall back to ice blue
//! behind it before the next pass.  A small star twinkles at the whale's tail
//! tip and repeats the double-blink flare from the interstellar vanishing
//! star.  ASCII is the default; `--unicode` swaps the fill for solid blocks.

#![allow(clippy::needless_range_loop)]

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::hash2;
use crate::scene::{Frame, Scene};

/// One diagonal light pass from the lockup's upper-left corner to its
/// lower-right corner.  The pulse starts and ends just outside the lockup so
/// it enters and leaves the frame instead of splitting at the wrap point.
const SWEEP_PERIOD: f32 = 3.8;
/// Width of the bright pulse as a fraction of the lockup diagonal.  At 0.10
/// the hot core is about one letter wide and the soft half-max span reaches
/// two to three letters.
const SWEEP_SIGMA: f32 = 0.10;
/// How far outside the lockup diagonal the pulse starts and ends.
const SWEEP_MARGIN: f32 = 0.15;

/// A small star twinkles at the whale's tail tip.  On terminals too narrow for
/// the whale it falls back to the K's top-right corner and starts earlier.
const K_STAR_START: f32 = 2.4;
const K_STAR_PERIOD: f32 = 4.6;

const T_DROP: f32 = 3.0;
const DROP_STEP: f32 = 0.14;
/// A quick two-row rebound once the lockup has landed.
const BOUNCE_TIME: f32 = 0.45;
const BOUNCE_ROWS: f32 = 2.0;

const RED: Rgb = Rgb::hex(0xff3b5c);
const CYAN: Rgb = Rgb::hex(0x38bdf8);
/// The whale mark keeps the brand blue from the reference logo.
const LOGO_BLUE: Rgb = Rgb::hex(0x2f66fe);
const LOGO_WHITE: Rgb = Rgb::hex(0xf9fafe);
const LOGO_EDGE: Rgb = Rgb::hex(0x7aa2ff);
/// The mark fades in after the lockup has landed and bounced.
const LOGO_START: f32 = 4.2;
const LOGO_FADE: f32 = 0.6;

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

// Two-tone whale mark traced from the DeepSeek logo.  `#` is the blue body,
// `@` the white belly/eye patch and `+` a soft edge.  The 22x11 and 20x10
// versions are used on wide terminals, the 13x6 and 11x5 ones on smaller
// screens.
const WHALE_LARGE: [&str; 10] = [
    "            ##   ## ",
    "           #### ####",
    "      +#############",
    "   +################",
    "  ##################",
    "####@@@@@@@##@@#####",
    "###@@@@@@@@@@#@#####",
    "###@@@@@@@@@@#######",
    " +####@@@@@######## ",
    "   #####  #####     ",
];
const WHALE_SMALL: [&str; 6] = [
    "        ## ##",
    "  ####  #####",
    "#############",
    "##@@@@@##@###",
    " #@@@@@@@####",
    "  ####  ###  ",
];
const WHALE_TINY: [&str; 5] = [
    "      ## ##",
    " ####  ####",
    "###########",
    "#@@@@##@###",
    " ##@@##    ",
];

fn logo_art(w: usize) -> Option<&'static [&'static str]> {
    if w >= 140 {
        Some(&WHALE_LARGE)
    } else if w >= 120 {
        Some(&WHALE_SMALL)
    } else if w >= 100 {
        Some(&WHALE_TINY)
    } else {
        None
    }
}

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
struct LogoLayout {
    left: isize,
    top: isize,
    w: isize,
    h: isize,
    art: &'static [&'static str],
}

impl LogoLayout {
    fn right(self) -> isize {
        self.left + self.w
    }

    fn bottom(self) -> isize {
        self.top + self.h
    }

    /// Topmost lit cell of the whale; the right fluke tip is where the little
    /// star twinkles.
    fn tail_tip(self) -> (isize, isize) {
        let mut top_row = isize::MAX;
        let mut col = 0;
        for (r, line) in self.art.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let r = r as isize;
                if r < top_row || (r == top_row && c as isize > col) {
                    top_row = r;
                    col = c as isize;
                }
            }
        }
        (self.left + col + 1, self.top + top_row - 1)
    }
}

#[derive(Clone, Copy)]
struct Layout {
    line1: LineLayout,
    line2: LineLayout,
    drop_rows: isize,
    ghost_rows: isize,
    logo: Option<LogoLayout>,
}

#[derive(Clone, Copy)]
struct MaskCell {
    x: isize,
    y: isize,
    letter: u8,
    row_frac: f32,
}

pub struct DshScene {
    w: usize,
    h: usize,
    layout: Option<Layout>,
    mask: Vec<MaskCell>,
    grid: Vec<bool>,
    /// Undropped canvas position of the fallback twinkle at the K's top-right
    /// corner (used only when no whale mark fits).
    k_corner: Option<(isize, isize)>,
}

impl DshScene {
    pub fn new(_seed: u64) -> Self {
        DshScene {
            w: 0,
            h: 0,
            layout: None,
            mask: Vec::new(),
            grid: Vec::new(),
            k_corner: None,
        }
    }

    fn choose_layout(w: usize, h: usize) -> Option<Layout> {
        if w < 60 || h < 16 {
            return None;
        }
        let art = logo_art(w);
        let (logo_w, logo_h) = art
            .map(|a| (a[0].chars().count() as isize, a.len() as isize))
            .unwrap_or((0, 0));
        // Letter gaps are part of the design, not a leftover: the 3x face
        // always keeps two blank columns between letters, and the 2x faces
        // try 3 then 2 before falling back to 1.  A zero gap is never used.
        let candidates = [
            (Face::Eight, 3usize, 2usize),
            (Face::Eight, 2, 3),
            (Face::Eight, 2, 2),
            (Face::Eight, 2, 1),
            (Face::Six, 2, 3),
            (Face::Six, 2, 2),
            (Face::Six, 2, 1),
            (Face::Wide, 2, 3),
            (Face::Wide, 2, 2),
            (Face::Wide, 2, 1),
            (Face::Mid, 2, 3),
            (Face::Mid, 2, 2),
            (Face::Mid, 2, 1),
            (Face::Compact, 2, 3),
            (Face::Compact, 2, 2),
            (Face::Compact, 2, 1),
        ];
        let pick = |with_logo: bool| {
            for (face, sx, gap_x) in candidates {
                if face == Face::Eight && h < 34 {
                    continue;
                }
                let w1 = text_width(face, sx, gap_x, "DEEPSEEK");
                let w2 = text_width(face, sx, gap_x, "HARNESS");
                if w1 + 4 <= w && w2 + 4 <= w {
                    if with_logo {
                        // The mark now touches the K, so only its right edge
                        // has to fit on screen; no symmetric margin.
                        let left1 = ((w - w1) / 2).max(2) as isize;
                        if left1 + w1 as isize + logo_w > w as isize - 1 {
                            continue;
                        }
                    }
                    return Some((face, sx, gap_x, w1, w2));
                }
            }
            None
        };
        let picked_with_logo = if art.is_some() { pick(true) } else { None };
        let use_logo = picked_with_logo.is_some();
        let (face, sx, gap_x, w1, w2) = picked_with_logo.or_else(|| pick(false))?;
        let rows = face.rows();
        let line_gap = if rows >= 9 { 2 } else { 1 };
        let block = rows * 2 + line_gap;
        let max_y = h as isize - 3;
        let mut drop = ((h as f32 * 0.10).round() as isize).clamp(2, 5);
        // The ghost stack is roughly one glyph-height tall: noticeably longer
        // than the old three-to-six row trail, but still clipped to the safe
        // area on short terminals.
        let mut ghost = ((rows as f32) * 0.9).round() as isize;
        ghost = ghost.clamp(4, 10);
        while block as isize + drop + ghost > max_y && (ghost > 3 || drop > 2) {
            if ghost > 3 {
                ghost -= 1;
            } else {
                drop -= 1;
            }
        }
        if block as isize + drop + ghost > max_y {
            return None;
        }
        let left1 = ((w - w1) / 2).max(2) as isize;
        // Both lines sit on the same centre line; HARNESS is only shorter, so
        // it starts further right without any extra stagger.
        let left2 = ((w - w2) / 2).max(2) as isize;
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
        let logo = if use_logo {
            art.map(|art| {
                // The whale's mouth touches the K: the first logo column is
                // the cell directly after the K's last column.
                let left = (left1 + w1 as isize).min(w as isize - logo_w - 1);
                LogoLayout {
                    left,
                    // Anchor to the lockup's resting position, not its
                    // pre-drop row, so the mouth lines up with the K.
                    top: (top + drop - logo_h + 4).max(1),
                    w: logo_w,
                    h: logo_h,
                    art,
                }
            })
        } else {
            None
        };
        Some(Layout {
            line1,
            line2,
            drop_rows: drop,
            ghost_rows: ghost,
            logo,
        })
    }

    fn rebuild(&mut self) {
        self.mask.clear();
        self.grid.clear();
        self.k_corner = None;
        let Some(layout) = Self::choose_layout(self.w, self.h) else {
            self.layout = None;
            return;
        };
        self.layout = Some(layout);
        self.grid = vec![false; self.w * self.h];
        self.build_line(layout.line1, "DEEPSEEK", 0);
        self.build_line(layout.line2, "HARNESS", 8);
        // Anchor the corner twinkle just outside the top-right pixel of the K
        // (letter 7 is the last letter of DEEPSEEK).
        let mut top = isize::MAX;
        let mut right = isize::MIN;
        for cell in self.mask.iter().filter(|c| c.letter == 7) {
            if cell.y < top || (cell.y == top && cell.x > right) {
                top = cell.y;
                right = cell.x;
            }
        }
        if top != isize::MAX {
            self.k_corner = Some((right + 1, top - 1));
        }
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
        let clean_logo = self
            .layout
            .and_then(|layout| layout.logo)
            .map(|logo| (logo.left - 1, logo.right(), logo.top - 1, logo.bottom()));
        for y in 1..=max_y {
            for x in 0..self.w as isize {
                if let Some((x0, x1, y0, y1)) = clean {
                    if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                        continue;
                    }
                }
                if let Some((x0, x1, y0, y1)) = clean_logo {
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
        let offset = landed_offset(t, layout.drop_rows);
        let onset = smoothstep((t - 2.2) / 0.8);
        if onset <= 0.0 {
            return;
        }
        let glyph = if f.charset == Charset::Unicode {
            ['█', '▓', '░']
        } else {
            ['#', '+', ':']
        };
        // The afterimage belongs to HARNESS only, and its first row starts at
        // HARNESS's feet: no ghost over the body of either line.
        let feet = layout.line2.top + offset + layout.line2.box_h() as isize;
        // Draw the deepest copies first so the nearest one wins where several
        // offsets land in the same cell.
        for k in (1..=layout.ghost_rows).rev() {
            let kf = k as f32 / (layout.ghost_rows + 1) as f32;
            // Kept clearly dimmer than the wordmark itself: a soft blue
            // afterimage, not a second copy of the text.
            let a = (1.0 - kf).powf(1.05) * 0.27 * trail * onset;
            if a < 0.012 {
                continue;
            }
            for c in &self.mask {
                if c.letter < 8 {
                    continue;
                }
                let y = c.y + offset + k;
                if y < feet || y > max_y {
                    continue;
                }
                let alpha = reveal_alpha(c, t - k as f32 * 0.035) * a;
                if alpha < 0.01 {
                    continue;
                }
                let ch = if k == 1 {
                    glyph[0]
                } else if k <= 3 {
                    glyph[1]
                } else {
                    glyph[2]
                };
                let col = f.theme.c(2).lerp(RED, 0.08 + 0.18 * kf);
                canvas.set(c.x, y, ch, col.scale(alpha * 1.9));
                if k == 1 && alpha > 0.12 {
                    canvas.add_light(c.x, y, CYAN, alpha * 0.10, '.');
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
            // Keep the glyphs close to the theme's near-white so the hollow
            // ASCII letters stay legible on large terminals; only the lower
            // rows pick up a noticeable ice-blue tint.
            let base = f.theme.hot().lerp(f.theme.c(2), 0.05 + c.row_frac * 0.22);
            // The diagonal band lifts each glyph from ice blue to white as it
            // passes; nothing is drawn between the letters, so this stays a
            // pure font-brightness gradient rather than a second beam.
            let glow = sweep_glow(sweep_u(layout, offset, c.x, y), t);
            let dim = f.theme.c(2).lerp(base, 0.52);
            let col = dim.lerp(Rgb::WHITE, glow);
            let level = 0.68 + 0.32 * glow;
            // Chromatic fringe: cyan to the left, red to the right.  Only
            // paint into an empty cell: writing over a glyph's own edge used
            // to black out most of the letters once the wider gap enabled
            // this path on large terminals.  A three-column gap leaves one
            // untouched column between the two fringes.
            let gap_x = if c.letter < 8 {
                layout.line1.gap_x
            } else {
                layout.line2.gap_x
            };
            if alpha > 0.35 && gap_x >= 3 {
                for (nx, color, amount) in [(c.x + 1, RED, 0.30), (c.x - 1, CYAN, 0.26)] {
                    if !self.cell_at(nx, y, offset) {
                        canvas.set(nx, y, ch, color.scale(alpha * amount));
                    }
                }
            }
            let shimmer = 0.95 + 0.05 * (t * 2.1 - c.x as f32 * 0.12 + c.y as f32 * 0.2).sin();
            canvas.set(c.x, y, ch, col.scale(alpha * shimmer * level));
            if alpha > 0.8 && (c.x + c.y) % 7 == 0 {
                canvas.add_light(c.x, y, f.theme.c(2), 0.07, ch);
            }
        }
    }

    /// The blue whale mark to the right of the K, fading in after the drop.
    fn draw_logo(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let Some(layout) = self.layout else {
            return;
        };
        let Some(logo) = layout.logo else {
            return;
        };
        let alpha = smoothstep((f.t as f32 - LOGO_START) / LOGO_FADE);
        if alpha <= 0.01 {
            return;
        }
        let unicode = f.charset == Charset::Unicode;
        for (r, line) in logo.art.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                let (color, glyph) = match ch {
                    '#' => (LOGO_BLUE, if unicode { '█' } else { '#' }),
                    '@' => (LOGO_WHITE, if unicode { '█' } else { '#' }),
                    '+' => (LOGO_EDGE, if unicode { '▒' } else { '+' }),
                    _ => continue,
                };
                let x = logo.left + c as isize;
                let y = logo.top + r as isize;
                if x < 0 || x >= self.w as isize || y < 1 || y > max_y {
                    continue;
                }
                canvas.set(x, y, glyph, color.scale(alpha));
            }
        }
    }

    /// A tiny star at the whale's tail tip (or the K's top-right corner when no
    /// logo fits).  It keeps a faint twinkle and, once per period, replays the
    /// double-blink flare from the interstellar star's vanishing moment: hot
    /// core, cross arms, diagonal rays and a short expanding ring.
    fn draw_k_star(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let Some(layout) = self.layout else {
            return;
        };
        let t = f.t as f32;
        let (x, y, start) = if let Some(logo) = layout.logo {
            let (tx, ty) = logo.tail_tip();
            (tx, ty, LOGO_START + LOGO_FADE)
        } else {
            let Some((corner_x, corner_y)) = self.k_corner else {
                return;
            };
            let offset = drop_offset(t, layout.drop_rows);
            (corner_x, corner_y + offset, K_STAR_START)
        };
        if t < start {
            return;
        }
        let x = x.clamp(1, self.w as isize - 2);
        let y = y.clamp(1, max_y);

        // A quiet, always-present glint.
        let base = 0.10 + 0.10 * (0.5 + 0.5 * (t * 2.2).sin());
        canvas.add_light(x, y, Rgb::WHITE, base, if base > 0.15 { '+' } else { '.' });

        let age = (t - start).rem_euclid(K_STAR_PERIOD);
        let p1 = gauss(age, 0.20, 0.09);
        let p2 = gauss(age, 0.62, 0.12) * 1.25;
        let p = p1.max(p2).clamp(0.0, 1.0);
        if p < 0.03 {
            return;
        }

        // Same beat as the vanishing star: first a three-armed flash, then a
        // brighter four-armed flash.
        let core = if f.charset == Charset::Unicode {
            '✷'
        } else {
            '*'
        };
        canvas.set(x, y, core, Rgb::WHITE);
        canvas.add_light(x, y, Rgb::WHITE, 1.0 + 1.4 * p, '.');
        if p > 0.40 {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                canvas.add_light(x + dx, y + dy, Rgb::WHITE, 0.55 * p, '+');
            }
        }
        let second = p2 > p1;
        let arms = if second { 4 } else { 3 };
        for i in 1..=arms {
            let a = (1.0 - i as f32 / (arms as f32 + 1.0)) * p;
            let hch = if i == 1 { '-' } else { '.' };
            canvas.add_light(x + i, y, Rgb::WHITE, a, hch);
            canvas.add_light(x - i, y, Rgb::WHITE, a, hch);
            let vch = if i == 1 { '|' } else { '.' };
            canvas.add_light(x, y + i, Rgb::WHITE, a, vch);
            canvas.add_light(x, y - i, Rgb::WHITE, a, vch);
        }
        if second && p2 > 0.55 {
            for i in 1..=2 {
                let a = p * (0.65 - i as f32 * 0.16);
                canvas.add_light(x + i, y + i, Rgb::WHITE, a, '\\');
                canvas.add_light(x - i, y - i, Rgb::WHITE, a, '\\');
                canvas.add_light(x + i, y - i, Rgb::WHITE, a, '/');
                canvas.add_light(x - i, y + i, Rgb::WHITE, a, '/');
            }
        }
        if p > 0.30 {
            let rr = 2.0 + age * 2.0;
            for k in 0..8 {
                let ang = k as f32 / 8.0 * std::f32::consts::TAU + 0.2;
                let px = (x as f32 + ang.cos() * rr).round() as isize;
                let py = (y as f32 + ang.sin() * rr * 0.6).round() as isize;
                canvas.add_light(px, py, CYAN, 0.14 * p, '.');
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
        self.draw_logo(canvas, f, max_y);
        self.draw_k_star(canvas, f, max_y);
    }
}

/// Steady drop position, without the landing rebound.  The afterimage uses
/// this so it stays put while the wordmark bounces back up.
fn landed_offset(t: f32, rows: isize) -> isize {
    if t < T_DROP {
        0
    } else if t >= T_DROP + DROP_STEP * rows as f32 {
        rows
    } else {
        (((t - T_DROP) / DROP_STEP).floor() as isize).clamp(0, rows)
    }
}

fn drop_offset(t: f32, rows: isize) -> isize {
    let offset = landed_offset(t, rows);
    if offset < rows {
        return offset;
    }
    let landed = T_DROP + DROP_STEP * rows as f32;
    let u = (t - landed) / BOUNCE_TIME;
    if u < 1.0 {
        // One quick hop up, then settle back on the bottom step.
        let bounce = ((u * std::f32::consts::PI).sin() * BOUNCE_ROWS).round() as isize;
        offset - bounce
    } else {
        offset
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

fn gauss(x: f32, at: f32, width: f32) -> f32 {
    (-((x - at) / width).powi(2)).exp()
}

/// Normalised diagonal coordinate: 0 at the lockup's upper-left corner and 1
/// at its lower-right.  Y is doubled so the bright wavefront reads as a 45°
/// line on a terminal whose cells are about twice as tall as they are wide.
fn sweep_u(layout: Layout, offset: isize, x: isize, y: isize) -> f32 {
    let left = layout.line1.left.min(layout.line2.left) as f32;
    let top = (layout.line1.top + offset) as f32;
    let right = (layout.line1.left + layout.line1.width as isize)
        .max(layout.line2.left + layout.line2.width as isize) as f32;
    let bottom = (layout.line2.top + offset + layout.line2.box_h() as isize) as f32;
    let span = ((right - left) + 2.0 * (bottom - top)).max(1.0);
    ((x as f32 - left) + 2.0 * (y as f32 - top)) / span
}

/// Brightness of the narrow diagonal light pulse at normalised position `u`.
///
/// The pulse crosses the whole diagonal once per period, starting and ending
/// just outside the lockup so the wrap happens while it is invisible.  A
/// gaussian profile keeps the leading and trailing edges soft.
fn sweep_glow(u: f32, t: f32) -> f32 {
    let cycle = (t / SWEEP_PERIOD).rem_euclid(1.0);
    let phase = -SWEEP_MARGIN + (1.0 + 2.0 * SWEEP_MARGIN) * cycle;
    let d = u - phase;
    (-(d / SWEEP_SIGMA).powi(2)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_frame(t: f32) -> Frame<'static> {
        Frame {
            t: t as f64,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: crate::theme::find("ice").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    fn render(scene: &mut DshScene, t: f32) -> Canvas {
        let frame = test_frame(t);
        let mut canvas = Canvas::new(frame.w, frame.h);
        scene.draw(&mut canvas, &frame);
        canvas
    }

    fn render_sized(scene: &mut DshScene, t: f32, w: usize, h: usize) -> Canvas {
        let mut frame = test_frame(t);
        frame.w = w;
        frame.h = h;
        let mut canvas = Canvas::new(w, h);
        scene.draw(&mut canvas, &frame);
        canvas
    }

    #[test]
    fn layout_fits_common_terminal_sizes() {
        for &(w, h) in &[
            (248, 56),
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
            let c1 = layout.line1.left * 2 + layout.line1.width as isize;
            let c2 = layout.line2.left * 2 + layout.line2.width as isize;
            assert!(
                (c1 - w as isize).abs() <= 3 && (c2 - w as isize).abs() <= 3,
                "both lines should share the screen centre at {w}x{h}: {c1} / {c2} vs {w}"
            );
            let fill = layout.line1.width as f32 / w as f32;
            assert!(fill >= 0.60, "{w}x{h}: only {:.0}% wide", fill * 100.0);
            assert!(layout.line1.box_h() >= 5);
            if layout.line1.sx == 3 {
                assert!(
                    layout.line1.gap_x >= 2,
                    "{w}x{h}: 3x letters only have a {} column gap",
                    layout.line1.gap_x
                );
            } else {
                assert!(layout.line1.gap_x >= 1, "{w}x{h}: letters touch");
            }
            match layout.logo {
                Some(logo) => {
                    assert!(w >= 100, "{w}x{h}: logo on a small terminal");
                    assert_eq!(
                        logo.left,
                        layout.line1.left + layout.line1.width as isize,
                        "{w}x{h}: logo should touch the K"
                    );
                    assert!(logo.right() < w as isize, "{w}x{h}: logo off screen");
                    assert!(logo.top >= 1 && logo.bottom() <= h as isize - 2);
                    for row in logo.art {
                        assert_eq!(row.chars().count() as isize, logo.w);
                        assert!(row.is_ascii());
                    }
                }
                None => assert!(w < 100, "{w}x{h}: missing logo"),
            }
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
        assert!(layout.line1.width >= 80);
        assert!(layout.line2.left > layout.line1.left);
        assert_eq!(drop_offset(0.0, 4), 0);
        assert_eq!(drop_offset(T_DROP + DROP_STEP * 1.1, 4), 1);
        assert_eq!(drop_offset(T_DROP + DROP_STEP * 3.1, 4), 3);
        assert!(reveal_alpha(&scene.mask[0], 0.2) > 0.0);
        assert_eq!(reveal_alpha(&scene.mask[0], 0.0), 0.0);
    }

    #[test]
    fn wide_layout_keeps_every_glyph_out_of_the_fringe_shadow() {
        let mut scene = DshScene::new(7);
        scene.resize(160, 48);
        let layout = scene.layout.unwrap();
        assert!(
            layout.line1.gap_x >= 2,
            "the regression only fires on the wide-gap layouts"
        );
        // 10 s: the drop has settled and the logo is fully in, so every mask
        // cell should show the wordmark's own colour rather than a fringe.
        let canvas = render_sized(&mut scene, 10.0, 160, 48);
        let offset = drop_offset(10.0, layout.drop_rows);
        let mut checked = 0;
        for c in &scene.mask {
            let y = c.y + offset;
            if !(1..=45).contains(&y) {
                continue;
            }
            let cell = canvas.cells[y as usize * 160 + c.x as usize];
            if cell.ch == ' ' {
                continue;
            }
            assert!(
                cell.fg.luminance() > 0.40,
                "glyph pixel ({}, {y}) was painted by the fringe: {:?}",
                c.x,
                cell.fg
            );
            checked += 1;
        }
        assert!(checked > 500, "only {checked} glyph cells checked");
    }

    #[test]
    fn lockup_hops_up_once_after_landing() {
        let rows = 4;
        let landed = T_DROP + DROP_STEP * rows as f32;
        assert_eq!(landed_offset(landed, rows), rows);
        assert_eq!(drop_offset(landed, rows), rows);
        assert_eq!(
            drop_offset(landed - DROP_STEP * 0.5, rows),
            rows - 1,
            "the lockup should still be falling just before it lands"
        );

        // The hop peaks two rows above the final resting row...
        let peak = landed + BOUNCE_TIME * 0.5;
        assert_eq!(drop_offset(peak, rows), rows - 2);
        // ...but the afterimage itself stays on the bottom row.
        assert_eq!(landed_offset(peak, rows), rows);

        // And it settles back on the bottom step, once, quickly.
        assert_eq!(drop_offset(landed + BOUNCE_TIME + 0.01, rows), rows);
        assert_eq!(drop_offset(landed + BOUNCE_TIME * 2.0, rows), rows);
    }

    #[test]
    fn logo_tail_twinkles_with_the_vanishing_star_flare() {
        let mut scene = DshScene::new(7);
        scene.resize(120, 36);
        let layout = scene.layout.unwrap();
        let logo = layout.logo.expect("logo");
        let (tx, ty) = logo.tail_tip();
        assert!(tx > logo.left && tx <= logo.right());
        assert!(ty < logo.top + logo.h);
        let x = tx.clamp(1, 118) as usize;
        let y = ty.clamp(1, 33) as usize;

        let flash_t = LOGO_START + LOGO_FADE + 0.62;
        let flash = render(&mut scene, flash_t);
        let cell = flash.cells[y * 120 + x];
        assert_eq!(cell.ch, '*', "the tail should flare at the second blink");
        assert!(cell.fg.luminance() > 0.8);

        let quiet_t = LOGO_START + LOGO_FADE + 1.4;
        let quiet = render(&mut scene, quiet_t);
        let cell = quiet.cells[y * 120 + x];
        assert_ne!(cell.ch, '*');
        assert!(
            cell.fg.luminance() < 0.75,
            "the quiet glint should stay faint: {:?}",
            cell.fg
        );
    }

    #[test]
    fn logo_fades_in_after_the_landing_bounce() {
        let mut scene = DshScene::new(7);
        scene.resize(120, 36);
        let layout = scene.layout.unwrap();
        let logo = layout.logo.expect("logo");
        // A solid body cell that the star does not cover.
        let body = logo
            .art
            .iter()
            .enumerate()
            .find_map(|(r, row)| {
                row.chars().enumerate().find_map(|(c, ch)| {
                    (ch == '#' && r > 2).then_some((logo.left + c as isize, logo.top + r as isize))
                })
            })
            .expect("body cell");
        let idx = body.1 as usize * 120 + body.0 as usize;
        let before = render(&mut scene, LOGO_START - 0.1).cells[idx];
        let after = render(&mut scene, LOGO_START + LOGO_FADE + 0.1).cells[idx];
        assert_eq!(
            before.ch, ' ',
            "the logo must not be drawn before it fades in"
        );
        assert_ne!(after.ch, ' ');
        assert!(after.fg.b > after.fg.r, "the logo body should be blue");
    }

    #[test]
    fn small_terminals_fall_back_to_the_k_corner_star() {
        let mut scene = DshScene::new(7);
        scene.resize(80, 24);
        let layout = scene.layout.unwrap();
        assert!(layout.logo.is_none());
        let (corner_x, corner_y) = scene.k_corner.expect("K corner");
        let top = scene
            .mask
            .iter()
            .filter(|c| c.letter == 7)
            .map(|c| c.y)
            .min()
            .unwrap();
        let right = scene
            .mask
            .iter()
            .filter(|c| c.letter == 7 && c.y == top)
            .map(|c| c.x)
            .max()
            .unwrap();
        assert_eq!((corner_x, corner_y), (right + 1, top - 1));
    }

    #[test]
    fn narrow_light_pulse_moves_from_upper_left_to_lower_right() {
        // At half a period the pulse is centred on the diagonal; a quarter
        // period later it has moved towards the lower-right corner.
        let peak = |t: f32| {
            (0..=1000)
                .map(|i| i as f32 / 1000.0)
                .max_by(|a, b| {
                    sweep_glow(*a, t)
                        .partial_cmp(&sweep_glow(*b, t))
                        .expect("finite glow")
                })
                .expect("sample")
        };
        let quarter = peak(0.25 * SWEEP_PERIOD);
        let middle = peak(0.5 * SWEEP_PERIOD);
        let three_quarters = peak(0.75 * SWEEP_PERIOD);
        assert!(
            quarter < middle && middle < three_quarters,
            "the pulse should walk down-right: {quarter} < {middle} < {three_quarters}"
        );
        assert!((middle - 0.5).abs() < 0.01);

        // It is a small patch of light, not a broad wash: 0.2 of the diagonal
        // away from the crest is already dark.
        let mid = 0.5 * SWEEP_PERIOD;
        assert!(sweep_glow(0.5, mid) > 0.99);
        assert!(sweep_glow(0.3, mid) < 0.05);
        assert!(sweep_glow(0.7, mid) < 0.05);

        // It starts and ends outside the lockup, so the wrap is a clean gap
        // rather than two half-pulses at opposite corners.
        assert!(sweep_glow(0.5, 0.0) < 0.05);
        assert!(sweep_glow(0.5, 0.999 * SWEEP_PERIOD) < 0.05);

        // One period later the same pulse returns.
        for i in 0..=10 {
            let u = i as f32 / 10.0;
            let a = sweep_glow(u, 7.3);
            let b = sweep_glow(u, 7.3 + SWEEP_PERIOD);
            assert!((a - b).abs() < 1e-4, "pulse does not wrap at u={u}");
        }
    }

    #[test]
    fn diagonal_sweep_is_a_font_gradient_not_a_beam() {
        let mut scene = DshScene::new(7);
        scene.resize(160, 48);
        let layout = scene.layout.unwrap();
        let t = 10.0;
        let offset = drop_offset(t, layout.drop_rows);
        let canvas = render_sized(&mut scene, t, 160, 48);
        let mut min = 1.0f32;
        let mut max = 0.0f32;
        let mut lit = 0;
        let mut bright = 0;
        for c in &scene.mask {
            let y = c.y + offset;
            if !(1..=45).contains(&y) {
                continue;
            }
            let cell = canvas.cells[y as usize * 160 + c.x as usize];
            if cell.ch == ' ' {
                continue;
            }
            let lum = cell.fg.luminance();
            min = min.min(lum);
            max = max.max(lum);
            if lum > 0.90 {
                bright += 1;
            }
            lit += 1;
        }
        assert!(lit > 500);
        assert!(
            max > 0.92,
            "the crest should reach near-white, got {max:.2}"
        );
        assert!(
            min < 0.72,
            "the trailing side should be visibly dimmer, got {min:.2}"
        );
        assert!(
            max - min > 0.20,
            "the diagonal gradient is too flat: {min:.2}..{max:.2}"
        );
        assert!(
            bright * 4 < lit,
            "the pulse covers too much of the wordmark: {bright}/{lit} bright cells"
        );

        // The bright cells should be the ones furthest along the diagonal at
        // this phase, and they should move further down-right a moment later.
        let mean_projection = |scene: &mut DshScene, t: f32| {
            let offset = drop_offset(t, scene.layout.unwrap().drop_rows);
            let canvas = render_sized(scene, t, 160, 48);
            let mut sum = 0.0;
            let mut n = 0.0;
            for c in &scene.mask {
                let y = c.y + offset;
                if !(1..=45).contains(&y) {
                    continue;
                }
                let cell = canvas.cells[y as usize * 160 + c.x as usize];
                if cell.fg.luminance() > 0.82 {
                    sum += (c.x + 2 * c.y) as f32;
                    n += 1.0;
                }
            }
            if n == 0.0 {
                0.0
            } else {
                sum / n
            }
        };
        let earlier = mean_projection(&mut scene, 10.0);
        let later = mean_projection(&mut scene, 10.0 + SWEEP_PERIOD * 0.16);
        assert!(
            later > earlier,
            "the bright band should move down-right: {earlier} -> {later}"
        );
    }

    #[test]
    fn ghost_starts_at_harness_feet_and_stays_faint() {
        let mut scene = DshScene::new(7);
        scene.resize(120, 36);
        let layout = scene.layout.unwrap();
        let t = 10.0;
        let frame = test_frame(t);
        let offset = drop_offset(t, layout.drop_rows);
        let mut canvas = Canvas::new(120, 36);
        scene.draw_ghosts(&mut canvas, &frame, 33);

        // The gap between DEEPSEEK and HARNESS stays clean: no DEEPSEEK ghost.
        let gap_top = layout.line1.top + offset + layout.line1.box_h() as isize;
        let gap_bottom = layout.line2.top + offset;
        for y in gap_top..gap_bottom {
            for x in 0..120 {
                let ch = canvas.cells[y as usize * 120 + x].ch;
                assert!(
                    ch == ' ',
                    "unexpected ghost {ch:?} in the inter-line gap at ({x}, {y})"
                );
            }
        }

        // The first ghost row starts at HARNESS's feet and stays dim.
        let base = scene
            .mask
            .iter()
            .find(|c| c.letter >= 8 && c.row_frac > 0.85)
            .copied()
            .unwrap();
        let feet = layout.line2.top + offset + layout.line2.box_h() as isize;
        let cell = canvas.cells[feet as usize * 120 + base.x as usize];
        assert_eq!(
            cell.ch, '#',
            "the first ghost row should start under the feet"
        );
        assert!(
            cell.fg.luminance() < 0.35,
            "the ghost is too close to the wordmark colour: {:?}",
            cell.fg
        );
        // Nothing is painted over the body of HARNESS.
        let body_y = layout.line2.top + offset + 2;
        for x in 0..120 {
            let ch = canvas.cells[body_y as usize * 120 + x].ch;
            assert!(
                ch == ' ',
                "ghost painted over the HARNESS body at ({x}, {body_y})"
            );
        }
        let ghost_bottom =
            layout.line2.top + layout.line2.box_h() as isize + layout.drop_rows + layout.ghost_rows;
        assert!(
            ghost_bottom <= 34,
            "ghost stack runs into the key bar: {ghost_bottom}"
        );
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
