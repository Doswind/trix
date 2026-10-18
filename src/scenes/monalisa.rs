//! 蒙娜丽莎 — the Mona Lisa as a cut-out terminal portrait.
//!
//! The figure was baked offline from the reference pixel art
//! (`docs/proposals/monalisa-v2/ref.webp`) into `src/data/monalisa.bin`:
//! 148×134 premultiplied RGBA cells.  At runtime each cell becomes
//!
//! * `bg` — the reference colour of that cell (skin, hair, dress, hands);
//! * `fg` + glyph — an ASCII ramp character for the local tone.
//!
//! There is no landscape, frame, title or floating dust: only the figure,
//! centered on the terminal's default background.  The colour portrait is
//! static (no blinking).  `--ghost` switches to the flat pure-ASCII portrait,
//! which keeps its old blink.
//!
//! `--unicode` upgrades the same data to half blocks (`▀` / `▄`), which gives
//! two vertical colours per cell.  The default charset is ASCII.

use std::f32::consts::PI;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const SRC_W: usize = 148;
const SRC_H: usize = 134;
const DATA: &[u8] = include_bytes!("../data/monalisa.bin");

// The flat "ghost" portrait the user liked: a fixed pure-ASCII grid, no
// background cells.  It lives behind `--ghost`.
const GHOST_W: usize = 77;
const GHOST_H: usize = 56;
const GHOST_ART: &str = include_str!("../data/monalisa_ascii.txt");
const GHOST_COLORS: &str = include_str!("../data/monalisa_color.txt");
const GHOST_EYES: [usize; 2] = [29, 41];
const GHOST_EYE_ROW: usize = 11;
const PAL_GHOST_INK: u8 = 0xA;

// The reference figure is a little too slender for the original painting;
// widen the body horizontally without changing its height.
const WIDTH_SCALE: f32 = 1.12;

const RAMP: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

#[derive(Clone, Copy)]
struct ArtCell {
    ch: char,
    fg: Rgb,
    bg: Rgb,
}

#[derive(Clone, Copy)]
struct UniCell {
    ch: char,
    fg: Rgb,
    bg: Rgb,
}

#[derive(Clone, Copy)]
struct GlyphCell {
    ch: char,
    color: u8,
}

#[derive(Clone, Copy)]
struct Layout {
    cols: usize,
    rows: usize,
    left: isize,
    top: isize,
    sx0: f32,
    sy0: f32,
    sx1: f32,
    sy1: f32,
}

pub struct MonaLisaScene {
    src: Vec<[u8; 4]>,
    w: usize,
    h: usize,
    layout: Layout,
    ascii: Vec<Option<ArtCell>>,
    unicode: Vec<Option<UniCell>>,
    ghost: Vec<GlyphCell>,
    ghost_left: isize,
    ghost_top: isize,
    ghost_mode: bool,
    gallery_mode: bool,
    rng: Rng,
    next_blink: f64,
    blink_start: f64,
    blink_len: f64,
}

impl MonaLisaScene {
    pub fn new(seed: u64) -> Self {
        let src = parse_bake();
        MonaLisaScene {
            src,
            w: 0,
            h: 0,
            layout: Layout {
                cols: 0,
                rows: 0,
                left: 0,
                top: 0,
                sx0: 0.0,
                sy0: 0.0,
                sx1: SRC_W as f32,
                sy1: SRC_H as f32,
            },
            ascii: Vec::new(),
            unicode: Vec::new(),
            ghost: parse_ghost(),
            ghost_left: 0,
            ghost_top: 0,
            ghost_mode: false,
            gallery_mode: false,
            rng: Rng::new(seed ^ 0x4d4f_4e41_4c49_5341),
            next_blink: 3.2,
            blink_start: -10.0,
            blink_len: 0.26,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);

        // Tall terminals show the full figure; short ones show a head-and-
        // shoulders crop so the face stays readable.
        let full = self.h >= 34;
        let (sx0, sy0, sx1, sy1) = if full {
            (0.0, 0.0, SRC_W as f32, SRC_H as f32)
        } else {
            (18.0, 0.0, 130.0, 84.0)
        };
        let aspect = WIDTH_SCALE * (sx1 - sx0) / (2.0 * (sy1 - sy0));

        let mut rows = self.h;
        let mut cols = (rows as f32 * 2.0 * aspect).round() as usize;
        if cols > self.w {
            cols = self.w;
            rows = (cols as f32 / (2.0 * aspect)).round() as usize;
        }
        rows = rows.clamp(1, self.h);
        cols = cols.clamp(1, self.w);

        let layout = Layout {
            cols,
            rows,
            left: ((self.w - cols) / 2) as isize,
            top: ((self.h - rows) / 2) as isize,
            sx0,
            sy0,
            sx1,
            sy1,
        };
        self.layout = layout;
        self.ghost_left = (self.w as isize - GHOST_W as isize) / 2;
        self.ghost_top = if self.h >= GHOST_H {
            (self.h as isize - GHOST_H as isize) / 2
        } else {
            0
        };
        self.build_cache();
        self.next_blink = self.rng.range(3.0, 6.5) as f64;
        self.blink_start = -10.0;
    }

    fn build_cache(&mut self) {
        let l = self.layout;
        let mut ascii = Vec::with_capacity(l.cols * l.rows);
        let mut unicode = Vec::with_capacity(l.cols * l.rows);
        for ty in 0..l.rows {
            for tx in 0..l.cols {
                let x0 = l.sx0 + (l.sx1 - l.sx0) * tx as f32 / l.cols as f32;
                let x1 = l.sx0 + (l.sx1 - l.sx0) * (tx + 1) as f32 / l.cols as f32;
                let y0 = l.sy0 + (l.sy1 - l.sy0) * ty as f32 / l.rows as f32;
                let y1 = l.sy0 + (l.sy1 - l.sy0) * (ty + 1) as f32 / l.rows as f32;

                ascii.push(self.cell_art(x0, x1, y0, y1));
                let ym = (y0 + y1) * 0.5;
                unicode.push(self.cell_unicode(x0, x1, y0, ym, y1));
            }
        }
        self.ascii = ascii;
        self.unicode = unicode;
    }

    fn sample(&self, x0: f32, x1: f32, y0: f32, y1: f32) -> Option<(Rgb, f32)> {
        let ix0 = x0.floor().clamp(0.0, SRC_W as f32 - 1.0) as usize;
        let ix1 = x1.ceil().clamp(1.0, SRC_W as f32) as usize;
        let iy0 = y0.floor().clamp(0.0, SRC_H as f32 - 1.0) as usize;
        let iy1 = y1.ceil().clamp(1.0, SRC_H as f32) as usize;
        let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for y in iy0..iy1.max(iy0 + 1) {
            for x in ix0..ix1.max(ix0 + 1) {
                let p = self.src[y.min(SRC_H - 1) * SRC_W + x.min(SRC_W - 1)];
                r += p[0] as u32;
                g += p[1] as u32;
                b += p[2] as u32;
                a += p[3] as u32;
                n += 1;
            }
        }
        if n == 0 {
            return None;
        }
        let alpha = a as f32 / (n as f32 * 255.0);
        if alpha < 0.42 {
            return None;
        }
        Some((Rgb::new((r / n) as u8, (g / n) as u8, (b / n) as u8), alpha))
    }

    fn cell_art(&self, x0: f32, x1: f32, y0: f32, y1: f32) -> Option<ArtCell> {
        let (c, _) = self.sample(x0, x1, y0, y1)?;
        let lum = c.luminance();
        let idx = (lum * (RAMP.len() - 1) as f32 + 0.5).clamp(0.0, 9.0) as usize;
        let fg = brighten(c, 0.12);
        Some(ArtCell {
            ch: RAMP[idx],
            fg,
            bg: c,
        })
    }

    fn cell_unicode(&self, x0: f32, x1: f32, y0: f32, ym: f32, y1: f32) -> Option<UniCell> {
        let top = self.sample(x0, x1, y0, ym);
        let bot = self.sample(x0, x1, ym, y1);
        match (top, bot) {
            (Some((t, _)), Some((b, _))) => Some(UniCell {
                ch: '▀',
                fg: brighten(t, 0.06),
                bg: b,
            }),
            (Some((t, _)), None) => Some(UniCell {
                ch: '▀',
                fg: brighten(t, 0.06),
                bg: t,
            }),
            (None, Some((b, _))) => Some(UniCell {
                ch: '▄',
                fg: brighten(b, 0.06),
                bg: b,
            }),
            (None, None) => None,
        }
    }

    fn ghost_closed(&self, x: usize, y: usize) -> Option<(char, u8)> {
        for &ex in &GHOST_EYES {
            if x + 2 < ex || x > ex + 2 {
                continue;
            }
            match y {
                row if row + 1 == GHOST_EYE_ROW => return Some(('#', 2)),
                row if row == GHOST_EYE_ROW => {
                    return Some(match x as isize - ex as isize {
                        -2 => ('\\', PAL_GHOST_INK),
                        -1..=1 => ('_', PAL_GHOST_INK),
                        2 => ('/', PAL_GHOST_INK),
                        _ => return None,
                    })
                }
                row if row == GHOST_EYE_ROW + 1 => return Some(('%', 3)),
                _ => {}
            }
        }
        None
    }

    fn draw_ghost(&self, canvas: &mut Canvas, t: f32) {
        let blink = self.blink(t as f64) > 0.45;
        for y in 0..GHOST_H {
            for x in 0..GHOST_W {
                let cell = self.ghost[y * GHOST_W + x];
                if cell.ch == ' ' {
                    continue;
                }
                let (ch, color) = if blink {
                    self.ghost_closed(x, y).unwrap_or((cell.ch, cell.color))
                } else {
                    (cell.ch, cell.color)
                };
                canvas.paint(
                    self.ghost_left + x as isize,
                    self.ghost_top + y as isize,
                    ch,
                    ghost_palette(color),
                    None,
                );
            }
        }
    }

    fn blink(&self, t: f64) -> f32 {
        if t < self.blink_start {
            return 0.0;
        }
        let p = (t - self.blink_start) / self.blink_len.max(0.01);
        if !(0.0..=1.0).contains(&p) {
            return 0.0;
        }
        (p as f32 * PI).sin().clamp(0.0, 1.0).powf(0.55)
    }
    fn draw_gallery(&self, canvas: &mut Canvas) {
        let l = &self.layout;
        if l.cols == 0 || l.rows == 0 {
            return;
        }
        let cx = l.left as f32 + l.cols as f32 * 0.50;
        let cy = l.top as f32 + l.rows as f32 * 0.34;
        let rx = l.cols as f32 * 0.58;
        let ry = l.rows as f32 * 0.42;
        let halo = Rgb::hex(0xffe6b0);
        let x0 = (cx - rx).floor() as isize;
        let x1 = (cx + rx).ceil() as isize;
        let y0 = (cy - ry).floor() as isize;
        let y1 = (cy + ry).ceil() as isize;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x as f32 - cx) / rx;
                let dy = (y as f32 - cy) / ry;
                let d = dx * dx + dy * dy;
                if d <= 1.0 && (x + y) % 4 == 0 {
                    let a = (1.0 - d) * 0.16;
                    canvas.set_if_blank(x, y, if d < 0.55 { ':' } else { '.' }, halo.scale(a));
                }
            }
        }
        // A very faint warm glow under the hands, so the figure sits in the room.
        let fy = l.top as f32 + l.rows as f32 * 0.84;
        let frx = l.cols as f32 * 0.38;
        let fry = l.rows as f32 * 0.07;
        let floor = Rgb::hex(0x6b4a2a);
        for y in (fy - fry).floor() as isize..=(fy + fry).ceil() as isize {
            for x in (cx - frx).floor() as isize..=(cx + frx).ceil() as isize {
                let dx = (x as f32 - cx) / frx;
                let dy = (y as f32 - fy) / fry;
                let d = dx * dx + dy * dy;
                if d <= 1.0 && (x + y) % 3 == 0 {
                    canvas.set_if_blank(x, y, '.', floor.scale((1.0 - d) * 0.20));
                }
            }
        }
    }
}

impl Scene for MonaLisaScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        if f.tune.ghost && f.t >= self.next_blink {
            self.blink_start = f.t;
            self.blink_len = self.rng.range(0.20, 0.34) as f64;
            self.next_blink = f.t + self.rng.range(3.4, 8.2) as f64;
            if self.rng.chance(0.22) {
                self.next_blink = f.t + self.blink_len + 0.28;
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if f.tune.ghost != self.ghost_mode {
            canvas.clear();
            self.ghost_mode = f.tune.ghost;
        }
        if f.tune.ghost {
            self.draw_ghost(canvas, f.t as f32);
            return;
        }
        if f.tune.gallery != self.gallery_mode {
            canvas.clear();
            self.gallery_mode = f.tune.gallery;
        }
        if f.tune.gallery {
            self.draw_gallery(canvas);
        }

        let l = &self.layout;
        if f.charset == Charset::Unicode {
            for ty in 0..l.rows {
                for tx in 0..l.cols {
                    if let Some(c) = self.unicode[ty * l.cols + tx] {
                        canvas.paint(
                            l.left + tx as isize,
                            l.top + ty as isize,
                            c.ch,
                            c.fg,
                            Some(c.bg),
                        );
                    }
                }
            }
        } else {
            for ty in 0..l.rows {
                for tx in 0..l.cols {
                    if let Some(c) = self.ascii[ty * l.cols + tx] {
                        canvas.paint(
                            l.left + tx as isize,
                            l.top + ty as isize,
                            c.ch,
                            c.fg,
                            Some(c.bg),
                        );
                    }
                }
            }
        }
    }
}

fn parse_bake() -> Vec<[u8; 4]> {
    assert_eq!(&DATA[0..4], b"MONA", "monalisa.bin: bad magic");
    let w = u16::from_le_bytes([DATA[8], DATA[9]]) as usize;
    let h = u16::from_le_bytes([DATA[10], DATA[11]]) as usize;
    assert_eq!((w, h), (SRC_W, SRC_H), "monalisa.bin: unexpected size");
    let pixels = &DATA[12..];
    assert_eq!(pixels.len(), w * h * 4, "monalisa.bin: truncated");
    pixels
        .chunks_exact(4)
        .map(|p| [p[0], p[1], p[2], p[3]])
        .collect()
}

fn parse_ghost() -> Vec<GlyphCell> {
    let mut out = Vec::with_capacity(GHOST_W * GHOST_H);
    let mut lines = GHOST_ART.lines();
    let mut colors = GHOST_COLORS.lines();
    for _ in 0..GHOST_H {
        let line = lines.next().unwrap_or("");
        let color_line = colors.next().unwrap_or("");
        let mut chars = line.chars();
        let mut color_chars = color_line.chars();
        for _ in 0..GHOST_W {
            let ch = chars.next().unwrap_or(' ');
            let color = color_chars.next().and_then(|c| c.to_digit(16)).unwrap_or(0) as u8;
            out.push(GlyphCell { ch, color });
        }
    }
    out
}

fn ghost_palette(idx: u8) -> Rgb {
    match idx {
        1 => Rgb::hex(0xf7ddb4), // skin highlight
        2 => Rgb::hex(0xe8c496), // skin
        3 => Rgb::hex(0xb88a60), // skin shadow
        4 => Rgb::hex(0x7a5232), // hair highlight
        5 => Rgb::hex(0x5a3c26), // hair
        6 => Rgb::hex(0x382418), // hair dark
        7 => Rgb::hex(0x564632), // dress fold
        8 => Rgb::hex(0x362c22), // dress
        9 => Rgb::hex(0x1c1610), // dress dark
        PAL_GHOST_INK => Rgb::hex(0x24160e),
        0xB => Rgb::hex(0xfff6e6), // catchlight
        _ => Rgb::WHITE,
    }
}

fn brighten(c: Rgb, amount: f32) -> Rgb {
    c.lerp(Rgb::WHITE, amount.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::THEMES;

    fn frame(t: f64, charset: Charset, w: usize, h: usize) -> Frame<'static> {
        Frame {
            t,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme: &THEMES[0],
            charset,
            seed: 7,
            speed: 1.0,
            tune: Default::default(),
        }
    }

    #[test]
    fn bake_parses() {
        let s = MonaLisaScene::new(7);
        assert_eq!(s.src.len(), SRC_W * SRC_H);
    }

    #[test]
    fn default_is_ascii_with_no_outside_background() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut c = Canvas::new(120, 60);
        let f = frame(1.0, Charset::Ascii, 120, 60);
        s.draw(&mut c, &f);
        assert!(c.ink() > 500);
        for cell in &c.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
        for y in 0..60 {
            let cell = c.get(0, y).unwrap();
            assert_eq!(cell.ch, ' ');
            assert!(cell.bg.is_none(), "background leaked at x=0");
        }
    }

    #[test]
    fn default_mode_paints_colour_cells() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut c = Canvas::new(120, 60);
        let f = frame(1.0, Charset::Ascii, 120, 60);
        s.draw(&mut c, &f);
        assert!(c.cells.iter().any(|cell| cell.bg.is_some()));
    }

    #[test]
    fn ghost_mode_is_flat_ascii() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut c = Canvas::new(120, 60);
        let mut f = frame(1.0, Charset::Ascii, 120, 60);
        f.tune.ghost = true;
        s.draw(&mut c, &f);
        assert!(c.ink() > 500);
        for cell in &c.cells {
            assert!(cell.ch.is_ascii(), "ghost leaked {:?}", cell.ch);
            assert!(cell.bg.is_none(), "ghost must not paint a background");
        }
    }

    #[test]
    fn switching_to_ghost_clears_coloured_cells() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut c = Canvas::new(120, 60);
        let mut f = frame(1.0, Charset::Ascii, 120, 60);
        s.draw(&mut c, &f);
        assert!(c.cells.iter().any(|cell| cell.bg.is_some()));
        f.tune.ghost = true;
        s.draw(&mut c, &f);
        assert!(c.cells.iter().all(|cell| cell.bg.is_none()));
    }

    #[test]
    fn ghost_mode_blinks() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut f = frame(1.0, Charset::Ascii, 120, 60);
        f.tune.ghost = true;
        let mut a = Canvas::new(120, 60);
        s.draw(&mut a, &f);
        s.blink_start = 0.5;
        s.blink_len = 1.0;
        let mut b = Canvas::new(120, 60);
        s.draw(&mut b, &f);
        let changed = a
            .cells
            .iter()
            .zip(b.cells.iter())
            .filter(|(x, y)| x.ch != y.ch)
            .count();
        assert!((4..=80).contains(&changed), "changed {changed}");
    }

    #[test]
    fn gallery_adds_a_soft_glow() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut f = frame(1.0, Charset::Ascii, 120, 60);
        let mut plain = Canvas::new(120, 60);
        s.draw(&mut plain, &f);
        f.tune.gallery = true;
        let mut gallery = Canvas::new(120, 60);
        s.draw(&mut gallery, &f);
        assert!(
            gallery.ink() > plain.ink(),
            "plain={} gallery={}",
            plain.ink(),
            gallery.ink()
        );
        assert!(gallery.cells.iter().all(|cell| cell.ch.is_ascii()));
    }

    #[test]
    fn colour_portrait_is_static() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let f = frame(1.0, Charset::Ascii, 120, 60);
        let mut a = Canvas::new(120, 60);
        s.draw(&mut a, &f);
        s.blink_start = 0.5;
        s.blink_len = 1.0;
        let mut b = Canvas::new(120, 60);
        s.draw(&mut b, &f);
        assert!(a.cells.iter().zip(b.cells.iter()).all(|(x, y)| x == y));
    }

    #[test]
    fn unicode_uses_half_blocks() {
        let mut s = MonaLisaScene::new(7);
        s.resize(120, 60);
        let mut c = Canvas::new(120, 60);
        let f = frame(1.0, Charset::Unicode, 120, 60);
        s.draw(&mut c, &f);
        assert!(c.cells.iter().any(|cell| cell.ch == '▀' || cell.ch == '▄'));
    }

    #[test]
    fn tiny_and_short_terminals_do_not_panic() {
        for &(w, h) in &[(20, 8), (64, 24), (212, 60)] {
            let mut s = MonaLisaScene::new(7);
            s.resize(w, h);
            let mut c = Canvas::new(w, h);
            let f = frame(1.0, Charset::Ascii, w, h);
            s.draw(&mut c, &f);
            assert!(c.ink() > 20, "{w}x{h} ink={}", c.ink());
        }
    }
}
