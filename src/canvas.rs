//! Double-buffered character canvas with ANSI diff rendering.
//!
//! A frame is built into `cells`; `present` only emits cells that differ
//! from the previous frame.  This keeps terminal traffic (and CPU spent in
//! the terminal emulator) low for sparse effects, while busy effects still
//! write one contiguous buffer per frame.

use std::io::{self, Write};

use crate::color::{write_fg, ColorMode, Rgb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: Rgb::BLACK,
    };

    #[inline]
    pub fn new(ch: char, fg: Rgb) -> Self {
        Cell { ch, fg }
    }

    #[inline]
    fn visible_eq(&self, other: &Cell) -> bool {
        self.ch == other.ch && (self.ch == ' ' || self.fg == other.fg)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::EMPTY
    }
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    prev: Vec<Cell>,
    last_fg: Option<Rgb>,
    out: Vec<u8>,
    full_ansi: Vec<u8>,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        Canvas {
            w,
            h,
            cells: vec![Cell::EMPTY; w * h],
            prev: vec![Cell::EMPTY; w * h],
            last_fg: None,
            out: Vec::with_capacity(w * h * 8 + 512),
            full_ansi: Vec::with_capacity(w * h * 8 + 512),
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        self.cells.clear();
        self.cells.resize(w * h, Cell::EMPTY);
        self.prev.clear();
        self.prev.resize(w * h, Cell::EMPTY);
        self.last_fg = None;
        self.out.clear();
        self.out.reserve(w * h * 8 + 512);
        self.full_ansi.clear();
    }

    /// Forget what is on screen so the next `present` repaints all cells.
    pub fn invalidate(&mut self) {
        self.prev.fill(Cell::EMPTY);
        self.last_fg = None;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cells.fill(Cell::EMPTY);
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn get(&self, x: isize, y: isize) -> Option<Cell> {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            None
        } else {
            Some(self.cells[y as usize * self.w + x as usize])
        }
    }

    #[inline]
    pub fn set(&mut self, x: isize, y: isize, ch: char, fg: Rgb) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        self.cells[y as usize * self.w + x as usize] = Cell { ch, fg };
    }

    /// Set only if the target is still blank, useful for backgrounds.
    pub fn set_if_blank(&mut self, x: isize, y: isize, ch: char, fg: Rgb) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let idx = y as usize * self.w + x as usize;
        if self.cells[idx].ch == ' ' {
            self.cells[idx] = Cell { ch, fg };
        }
    }

    /// Alpha-blend a glyph into the cell (used for glows and HUD glass).
    pub fn blend(&mut self, x: isize, y: isize, ch: char, fg: Rgb, alpha: f32) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let a = alpha.clamp(0.0, 1.0);
        let idx = y as usize * self.w + x as usize;
        let old = self.cells[idx];
        let new_fg = if old.ch == ' ' {
            fg.scale(a)
        } else {
            old.fg.lerp(fg, a)
        };
        // Fading into empty space: keep the glyph but with a dim color.
        self.cells[idx] = Cell { ch, fg: new_fg };
    }

    /// Add light to a cell.  `ch` is used only when the cell was blank.
    pub fn add_light(&mut self, x: isize, y: isize, fg: Rgb, amount: f32, ch: char) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let idx = y as usize * self.w + x as usize;
        let old = self.cells[idx];
        let col = old.fg.add(fg, amount.clamp(0.0, 2.0));
        let glyph = if old.ch == ' ' { ch } else { old.ch };
        self.cells[idx] = Cell { ch: glyph, fg: col };
    }

    pub fn put_str(&mut self, x: isize, y: isize, text: &str, fg: Rgb) {
        for (cx, ch) in (x..).zip(text.chars()) {
            self.set(cx, y, ch, fg);
        }
    }

    /// Draw text with a horizontal brightness ramp (used for HUD titles).
    pub fn put_str_gradient(&mut self, x: isize, y: isize, text: &str, colors: &[Rgb]) {
        if colors.is_empty() {
            return;
        }
        let n = text.chars().count().max(1);
        for (i, ch) in text.chars().enumerate() {
            let f = i as f32 / (n - 1).max(1) as f32;
            let pos = f * (colors.len() - 1) as f32;
            let i0 = pos.floor() as usize;
            let i1 = (i0 + 1).min(colors.len() - 1);
            let col = colors[i0].lerp(colors[i1], pos - i0 as f32);
            self.set(x + i as isize, y, ch, col);
        }
    }

    pub fn hline(&mut self, x0: isize, x1: isize, y: isize, ch: char, fg: Rgb) {
        if x0 > x1 {
            return;
        }
        for x in x0..=x1 {
            self.set(x, y, ch, fg);
        }
    }

    pub fn vline(&mut self, x: isize, y0: isize, y1: isize, ch: char, fg: Rgb) {
        if y0 > y1 {
            return;
        }
        for y in y0..=y1 {
            self.set(x, y, ch, fg);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn line(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, ch: char, fg: Rgb) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.set(x, y, ch, fg);
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

    pub fn rect(&mut self, x: isize, y: isize, w: isize, h: isize, fg: Rgb) {
        if w <= 0 || h <= 0 {
            return;
        }
        let x1 = x + w - 1;
        let y1 = y + h - 1;
        self.hline(x, x1, y, '-', fg);
        self.hline(x, x1, y1, '-', fg);
        self.vline(x, y, y1, '|', fg);
        self.vline(x1, y, y1, '|', fg);
        self.set(x, y, '+', fg);
        self.set(x1, y, '+', fg);
        self.set(x, y1, '+', fg);
        self.set(x1, y1, '+', fg);
    }

    /// Fill a rectangle with spaces (erases whatever is beneath).
    pub fn clear_rect(&mut self, x: isize, y: isize, w: isize, h: isize) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, ' ', Rgb::BLACK);
            }
        }
    }

    /// Diff-compose this frame into the internal buffer and write it.
    pub fn present<W: Write>(&mut self, dst: &mut W, mode: ColorMode) -> io::Result<usize> {
        let mut out = std::mem::take(&mut self.out);
        out.clear();
        let mut last_fg = self.last_fg;

        for y in 0..self.h {
            let row = y * self.w;
            let mut x = 0;
            while x < self.w {
                let idx = row + x;
                if self.cells[idx].visible_eq(&self.prev[idx]) {
                    x += 1;
                    continue;
                }
                // Start of a changed run: position the cursor once.
                let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
                while x < self.w {
                    let idx = row + x;
                    let cell = self.cells[idx];
                    if cell.visible_eq(&self.prev[idx]) {
                        break;
                    }
                    if cell.ch != ' ' && mode.is_color() && last_fg != Some(cell.fg) {
                        write_fg(&mut out, cell.fg, mode);
                        last_fg = Some(cell.fg);
                    }
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
                    self.prev[idx] = cell;
                    x += 1;
                }
            }
        }

        self.last_fg = last_fg;
        let res = dst.write_all(&out).and_then(|_| dst.flush());
        self.out = out;
        res?;
        Ok(self.out.len())
    }

    /// Full, line-oriented ANSI rendering (for snapshots / docs).
    pub fn render_ansi(&mut self, mode: ColorMode) -> Vec<u8> {
        let mut out = std::mem::take(&mut self.full_ansi);
        out.clear();
        let _ = write!(out, "\x1b[0m");
        let mut last_fg: Option<Rgb> = None;
        for y in 0..self.h {
            let row = y * self.w;
            // Trim trailing blanks but keep a couple of cells for readability.
            let mut end = self.w;
            while end > 0 && self.cells[row + end - 1].ch == ' ' {
                end -= 1;
            }
            for x in 0..end {
                let cell = self.cells[row + x];
                if cell.ch != ' ' && mode.is_color() && last_fg != Some(cell.fg) {
                    write_fg(&mut out, cell.fg, mode);
                    last_fg = Some(cell.fg);
                }
                let mut buf = [0u8; 4];
                out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
            }
            if y + 1 < self.h {
                out.extend_from_slice(b"\x1b[0m\n");
                last_fg = None;
            }
        }
        out.extend_from_slice(b"\x1b[0m\n");
        self.full_ansi = out;
        self.full_ansi.clone()
    }

    pub fn render_plain(&self) -> String {
        let mut s = String::with_capacity(self.w * (self.h + 1));
        for y in 0..self.h {
            for x in 0..self.w {
                s.push(self.cells[y * self.w + x].ch);
            }
            s.push('\n');
        }
        s
    }

    /// Number of non-blank cells, used by tests to ensure scenes actually draw.
    pub fn ink(&self) -> usize {
        self.cells.iter().filter(|c| c.ch != ' ').count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_does_not_panic() {
        let mut c = Canvas::new(20, 10);
        c.set(-5, -5, 'x', Rgb::WHITE);
        c.set(100, 100, 'x', Rgb::WHITE);
        c.line(-10, -10, 40, 40, '*', Rgb::WHITE);
        c.put_str(2, 4, "hello", Rgb::WHITE);
        c.rect(-2, -2, 10, 10, Rgb::WHITE);
        c.clear_rect(-5, -5, 5, 5);
        assert_eq!(c.get(3, 4).unwrap().ch, 'e');
    }

    #[test]
    fn diff_only_writes_changes() {
        let mut c = Canvas::new(10, 3);
        c.set(1, 1, 'a', Rgb::WHITE);
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::True).unwrap();
        let first = sink.len();
        assert!(first > 0);
        sink.clear();
        c.present(&mut sink, ColorMode::True).unwrap();
        assert_eq!(sink.len(), 0);
        c.set(2, 1, 'b', Rgb::WHITE);
        c.present(&mut sink, ColorMode::True).unwrap();
        assert!(sink.windows(4).any(|w| w == b"\x1b[2;"));
    }

    #[test]
    fn blank_color_is_irrelevant_for_diff() {
        let mut c = Canvas::new(4, 1);
        c.set(0, 0, ' ', Rgb::new(255, 0, 0));
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::True).unwrap();
        assert_eq!(sink.len(), 0);
    }

    #[test]
    fn ansi_render_contains_text() {
        let mut c = Canvas::new(12, 2);
        c.put_str(0, 0, "hi", Rgb::new(0, 255, 0));
        let out = c.render_ansi(ColorMode::True);
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("hi"));
        assert!(s.contains("38;2;0;255;0"));
    }
}
