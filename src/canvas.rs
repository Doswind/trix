//! Double-buffered character canvas with ANSI diff rendering.
//!
//! A frame is built into `cells`; `present` only emits cells that differ
//! from what the terminal is currently showing.  This keeps terminal traffic
//! (and CPU spent in the terminal emulator) low for sparse effects, while
//! busy effects still write one contiguous buffer per frame.
//!
//! The encoder is deliberately frugal because it is the only part of `trix`
//! that runs over a network in an SSH session:
//!
//! * it tracks the cursor and the live SGR colours, so a run that starts where
//!   the previous one ended costs no escape sequence at all;
//! * cleared trailing cells use `EL` instead of a row of spaces;
//! * neighbouring colours may differ by a couple of levels and still share one
//!   SGR sequence (`PresentOptions::color_slack`), which collapses the tiny
//!   per-cell deltas of a moving gradient without a visible change;
//! * frames can be wrapped in synchronized-output markers so the emulator
//!   paints them atomically instead of tearing at 30 Hz.

use std::io::{self, Write};

use crate::color::{write_bg, write_fg, ColorMode, Rgb};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Rgb,
    /// Explicit background color.  `None` leaves the terminal default showing,
    /// which is what every pre-existing effect relies on.
    pub bg: Option<Rgb>,
}

impl Cell {
    pub const EMPTY: Cell = Cell {
        ch: ' ',
        fg: Rgb::BLACK,
        bg: None,
    };

    #[inline]
    pub fn new(ch: char, fg: Rgb) -> Self {
        Cell { ch, fg, bg: None }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::EMPTY
    }
}

/// How a frame is encoded for the terminal.
///
/// `color_slack` is a per-channel tolerance: a cell whose colour is within
/// `slack` levels of the colour the emitter already has selected is painted
/// with that colour instead of costing another SGR sequence.  Gradients are
/// built from one-level steps, so a small slack removes most of the colour
/// churn of a moving scene while keeping the difference (at most `slack`/255)
/// below what an eye can pick out.  Zero means "exactly the requested RGB".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentOptions {
    pub color_slack: u8,
    /// Wrap the frame in `DECSET 2026` synchronized-output markers.
    pub sync: bool,
    /// Use `EL` for a cleared row tail instead of writing spaces.
    pub erase_tail: bool,
    /// Avoid redundant cursor addressing.
    pub cursor_moves: bool,
}

impl Default for PresentOptions {
    fn default() -> Self {
        PresentOptions {
            color_slack: 0,
            sync: false,
            erase_tail: true,
            cursor_moves: true,
        }
    }
}

/// Per-channel colour distance test used by the encoder.
#[inline]
fn near(a: Rgb, b: Rgb, slack: u8) -> bool {
    a.r.abs_diff(b.r) <= slack && a.g.abs_diff(b.g) <= slack && a.b.abs_diff(b.b) <= slack
}

#[inline]
fn bg_eq(a: Option<Rgb>, b: Option<Rgb>, slack: u8) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => near(a, b, slack),
        _ => false,
    }
}

#[inline]
fn fg_eq(a: Rgb, b: Rgb, slack: u8) -> bool {
    near(a, b, slack)
}

#[inline]
fn digits(mut n: usize) -> usize {
    let mut d = 1;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// Move the terminal cursor to `(y, x)` with the shortest sequence we can
/// assume every Unix terminal understands.
fn move_cursor(out: &mut Vec<u8>, cursor: &mut Option<(usize, usize)>, y: usize, x: usize) {
    if *cursor == Some((y, x)) {
        return;
    }
    if let Some((cy, cx)) = *cursor {
        if cy == y {
            if cx < x {
                let forward = x - cx;
                // `CUF` is ESC [ digits C: three bytes plus the digits.
                let fwd_len = 3 + digits(forward);
                let abs_len = 4 + digits(y + 1) + digits(x + 1);
                if fwd_len < abs_len {
                    let _ = write!(out, "\x1b[{forward}C");
                    *cursor = Some((y, x));
                    return;
                }
            } else if cx > x {
                // Carriage return plus a short forward move is cheaper than
                // a full `CUP` when the target is near the left edge.
                let len = 1 + if x == 0 { 0 } else { 3 + digits(x) };
                let abs_len = 4 + digits(y + 1) + digits(x + 1);
                if len < abs_len {
                    out.push(b'\r');
                    if x > 0 {
                        let _ = write!(out, "\x1b[{x}C");
                    }
                    *cursor = Some((y, x));
                    return;
                }
            }
        }
    }
    let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
    *cursor = Some((y, x));
}

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    /// What the terminal is currently displaying (not what we asked for: with
    /// colour slack the two can differ slightly).
    screen: Vec<Cell>,
    last_fg: Option<Rgb>,
    last_bg: Option<Rgb>,
    cursor: Option<(usize, usize)>,
    out: Vec<u8>,
    full_ansi: Vec<u8>,
    /// Test-only guard: scenes must not write text into the HUD rows.
    chrome_guard: bool,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        let w = w.max(1);
        let h = h.max(1);
        Canvas {
            w,
            h,
            cells: vec![Cell::EMPTY; w * h],
            screen: vec![Cell::EMPTY; w * h],
            last_fg: None,
            last_bg: None,
            cursor: None,
            out: Vec::with_capacity(w * h * 8 + 512),
            full_ansi: Vec::with_capacity(w * h * 8 + 512),
            chrome_guard: false,
        }
    }

    /// Test-only: fail if a scene writes text into row 0 or row h-1.
    #[cfg(test)]
    pub fn set_chrome_guard(&mut self, on: bool) {
        self.chrome_guard = on;
    }

    fn check_chrome_row(&self, y: isize) {
        if self.chrome_guard {
            assert!(
                y != 0 && y != self.h as isize - 1,
                "scene text written into reserved HUD row {y}"
            );
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
        self.screen.clear();
        self.screen.resize(w * h, Cell::EMPTY);
        self.last_fg = None;
        self.last_bg = None;
        self.cursor = None;
        self.out.clear();
        self.out.reserve(w * h * 8 + 512);
        self.full_ansi.clear();
    }

    /// Forget what is on screen so the next `present` repaints all cells.
    pub fn invalidate(&mut self) {
        self.screen.fill(Cell::EMPTY);
        self.last_fg = None;
        self.last_bg = None;
        self.cursor = None;
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
        let idx = y as usize * self.w + x as usize;
        // Keep whatever background is already painted behind this cell.
        self.cells[idx] = Cell {
            ch,
            fg,
            bg: self.cells[idx].bg,
        };
    }

    /// Write glyph, foreground and background in one go.
    #[inline]
    pub fn paint(&mut self, x: isize, y: isize, ch: char, fg: Rgb, bg: Option<Rgb>) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        self.cells[y as usize * self.w + x as usize] = Cell { ch, fg, bg };
    }

    /// Set only the background colour of an existing cell.
    #[inline]
    pub fn set_bg(&mut self, x: isize, y: isize, bg: Option<Rgb>) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        self.cells[y as usize * self.w + x as usize].bg = bg;
    }

    /// Fill a rectangle with a solid colour (glyph `ch`, colour `fg`).
    #[allow(clippy::too_many_arguments)]
    pub fn fill(
        &mut self,
        x: isize,
        y: isize,
        w: isize,
        h: isize,
        ch: char,
        fg: Rgb,
        bg: Option<Rgb>,
    ) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.paint(xx, yy, ch, fg, bg);
            }
        }
    }

    /// Set only if the target is still blank, useful for backgrounds.
    pub fn set_if_blank(&mut self, x: isize, y: isize, ch: char, fg: Rgb) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let idx = y as usize * self.w + x as usize;
        if self.cells[idx].ch == ' ' {
            self.cells[idx] = Cell {
                ch,
                fg,
                bg: self.cells[idx].bg,
            };
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
        self.cells[idx] = Cell {
            ch,
            fg: new_fg,
            bg: old.bg,
        };
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
        self.cells[idx] = Cell {
            ch: glyph,
            fg: col,
            bg: old.bg,
        };
    }

    pub fn put_str(&mut self, x: isize, y: isize, text: &str, fg: Rgb) {
        self.check_chrome_row(y);
        for (cx, ch) in (x..).zip(text.chars()) {
            self.set(cx, y, ch, fg);
        }
    }

    /// Draw text with a horizontal brightness ramp (used for HUD titles).
    pub fn put_str_gradient(&mut self, x: isize, y: isize, text: &str, colors: &[Rgb]) {
        self.check_chrome_row(y);
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
                self.paint(xx, yy, ' ', Rgb::BLACK, None);
            }
        }
    }

    /// Diff-compose this frame into the internal buffer and write it.
    pub fn present<W: Write>(&mut self, dst: &mut W, mode: ColorMode) -> io::Result<usize> {
        self.present_with(dst, mode, PresentOptions::default())
    }

    /// Diff-compose with an explicit encoder profile.
    pub fn present_with<W: Write>(
        &mut self,
        dst: &mut W,
        mode: ColorMode,
        opts: PresentOptions,
    ) -> io::Result<usize> {
        let mut out = std::mem::take(&mut self.out);
        out.clear();
        // Reserve the sync marker at the front; if the frame turns out to be
        // empty we drop it again instead of waking the terminal up twice.
        let body_start = if opts.sync {
            out.extend_from_slice(b"\x1b[?2026h");
            out.len()
        } else {
            0
        };
        let color = mode.is_color();
        let slack = if color { opts.color_slack } else { 0 };
        let mut last_fg = self.last_fg;
        let mut last_bg = self.last_bg;
        let mut cursor = self.cursor;

        // Leaving a scene that painted a background: make sure the terminal
        // goes back to the baseline black even if this frame changes nothing.
        if color && last_bg.is_some() && !self.cells.iter().any(|c| c.bg.is_some()) {
            out.extend_from_slice(b"\x1b[40m");
            last_bg = None;
        }

        let cell_eq = |a: Cell, b: Cell| -> bool {
            if a.ch != b.ch {
                return false;
            }
            if !color {
                // Mono mode has no background or foreground: only the glyph
                // is visible.
                return true;
            }
            if !bg_eq(a.bg, b.bg, slack) {
                return false;
            }
            a.ch == ' ' || fg_eq(a.fg, b.fg, slack)
        };

        for y in 0..self.h {
            let row = y * self.w;

            // Trailing blank cells can be cleared with one `EL` instead of a
            // row of spaces (and a colour reset per space).
            let mut tail = self.w;
            while tail > 0 {
                let cell = self.cells[row + tail - 1];
                if cell.ch == ' ' && cell.bg.is_none() {
                    tail -= 1;
                } else {
                    break;
                }
            }
            let tail_dirty =
                (tail..self.w).any(|x| !cell_eq(self.cells[row + x], self.screen[row + x]));
            let use_el = opts.erase_tail && tail_dirty && self.w - tail >= 4;
            let end = if use_el { tail } else { self.w };

            let mut x = 0;
            while x < end {
                let idx = row + x;
                if cell_eq(self.cells[idx], self.screen[idx]) {
                    x += 1;
                    continue;
                }
                if opts.cursor_moves {
                    move_cursor(&mut out, &mut cursor, y, x);
                } else {
                    let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
                    cursor = Some((y, x));
                }
                let mut wide = false;
                while x < end {
                    let idx = row + x;
                    let cell = self.cells[idx];
                    if cell_eq(cell, self.screen[idx]) {
                        break;
                    }
                    if !cell.ch.is_ascii() {
                        wide = true;
                    }
                    let display_fg;
                    let display_bg;
                    if color {
                        if cell.ch == ' ' {
                            // A space never shows the foreground colour, so
                            // leaving the current SGR fg alone is free.
                            display_fg = self.screen[idx].fg;
                        } else {
                            match last_fg {
                                Some(fg) if near(cell.fg, fg, slack) => {
                                    display_fg = fg;
                                }
                                _ => {
                                    write_fg(&mut out, cell.fg, mode);
                                    last_fg = Some(cell.fg);
                                    display_fg = cell.fg;
                                }
                            }
                        }
                        display_bg = match (last_bg, cell.bg) {
                            (None, None) => None,
                            (Some(bg), Some(target)) if near(bg, target, slack) => Some(bg),
                            (_, Some(target)) => {
                                write_bg(&mut out, target, mode);
                                last_bg = Some(target);
                                Some(target)
                            }
                            (Some(_), None) => {
                                out.extend_from_slice(b"\x1b[40m");
                                last_bg = None;
                                None
                            }
                        };
                    } else {
                        display_fg = cell.fg;
                        display_bg = cell.bg;
                    }
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
                    self.screen[idx] = Cell {
                        ch: cell.ch,
                        fg: display_fg,
                        bg: display_bg,
                    };
                    cursor = if x + 1 < self.w {
                        Some((y, x + 1))
                    } else {
                        // The terminal may keep the cursor on the last column
                        // when autowrap is off; do not trust it.
                        None
                    };
                    x += 1;
                }
                if wide {
                    // Wide glyphs advance two columns in some terminals, so a
                    // relative move after this run would be unsafe.
                    cursor = None;
                }
            }

            if use_el {
                if opts.cursor_moves {
                    move_cursor(&mut out, &mut cursor, y, tail);
                } else {
                    let _ = write!(out, "\x1b[{};{}H", y + 1, tail + 1);
                }
                if color && last_bg.is_some() {
                    // `EL` paints with the current background; make sure that
                    // is the default before erasing.
                    out.extend_from_slice(b"\x1b[40m");
                    last_bg = None;
                }
                out.extend_from_slice(b"\x1b[K");
                for cell in &mut self.screen[row + tail..row + self.w] {
                    *cell = Cell::EMPTY;
                }
                cursor = Some((y, tail));
            }
        }

        if out.len() == body_start {
            // Nothing changed: skip the write entirely, sync markers included.
            out.clear();
        } else if opts.sync {
            out.extend_from_slice(b"\x1b[?2026l");
        }

        self.last_fg = last_fg;
        self.last_bg = last_bg;
        self.cursor = cursor;
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
        let mut last_bg: Option<Rgb> = None;
        for y in 0..self.h {
            let row = y * self.w;
            // Trim trailing blanks, but never trim a cell that paints a
            // background colour: that colour is part of the picture.
            let mut end = self.w;
            while end > 0 {
                let cell = self.cells[row + end - 1];
                if cell.ch == ' ' && cell.bg.is_none() {
                    end -= 1;
                } else {
                    break;
                }
            }
            for x in 0..end {
                let cell = self.cells[row + x];
                if mode.is_color() {
                    if cell.ch != ' ' && last_fg != Some(cell.fg) {
                        write_fg(&mut out, cell.fg, mode);
                        last_fg = Some(cell.fg);
                    }
                    if last_bg != cell.bg {
                        match cell.bg {
                            Some(bg) => write_bg(&mut out, bg, mode),
                            None => out.extend_from_slice(b"\x1b[40m"),
                        }
                        last_bg = cell.bg;
                    }
                }
                let mut buf = [0u8; 4];
                out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
            }
            if y + 1 < self.h {
                out.extend_from_slice(b"\x1b[0m\n");
                last_fg = None;
                last_bg = None;
            }
        }
        out.extend_from_slice(b"\x1b[0m\n");
        self.full_ansi = out;
        self.full_ansi.clone()
    }

    /// Render one full-width ANSI string per row.  Unlike `render_ansi`, rows
    /// are not trimmed and no separators are added: this is what the help
    /// preview needs when it wraps a frame in a box.
    pub fn render_rows(&self, mode: ColorMode) -> Vec<String> {
        let mut rows = Vec::with_capacity(self.h);
        for y in 0..self.h {
            let mut out: Vec<u8> = Vec::with_capacity(self.w * 8);
            let mut last_fg: Option<Rgb> = None;
            let mut last_bg: Option<Rgb> = None;
            for x in 0..self.w {
                let cell = self.cells[y * self.w + x];
                if mode.is_color() {
                    if cell.ch != ' ' && last_fg != Some(cell.fg) {
                        write_fg(&mut out, cell.fg, mode);
                        last_fg = Some(cell.fg);
                    }
                    if last_bg != cell.bg {
                        match cell.bg {
                            Some(bg) => write_bg(&mut out, bg, mode),
                            None => out.extend_from_slice(b"\x1b[40m"),
                        }
                        last_bg = cell.bg;
                    }
                }
                let mut buf = [0u8; 4];
                out.extend_from_slice(cell.ch.encode_utf8(&mut buf).as_bytes());
            }
            out.extend_from_slice(b"\x1b[0m");
            rows.push(String::from_utf8_lossy(&out).into_owned());
        }
        rows
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
        // The cursor already sits where the new cell starts, so the encoder
        // must not waste a `CUP` sequence on it.
        assert!(sink.contains(&b'b'), "{sink:?}");
        assert!(!sink.windows(4).any(|w| w == b"\x1b[2;"), "{sink:?}");
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

    #[test]
    fn background_is_diffed_and_emitted() {
        let mut c = Canvas::new(6, 2);
        let sky = Rgb::new(92, 148, 252);
        c.fill(0, 0, 6, 2, ' ', sky, Some(sky));
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::True).unwrap();
        let s = String::from_utf8_lossy(&sink).to_string();
        assert!(s.contains("48;2;92;148;252"), "{s:?}");

        sink.clear();
        c.present(&mut sink, ColorMode::True).unwrap();
        assert!(sink.is_empty(), "static background must not repaint");

        // painting the default background back must reset the terminal
        c.set_bg(2, 1, None);
        sink.clear();
        c.present(&mut sink, ColorMode::True).unwrap();
        let s = String::from_utf8_lossy(&sink).to_string();
        assert!(s.contains("\x1b[40m"), "{s:?}");
    }

    #[test]
    fn set_keeps_existing_background() {
        let mut c = Canvas::new(4, 1);
        let sky = Rgb::new(1, 2, 3);
        c.set_bg(1, 0, Some(sky));
        c.set(1, 0, '#', Rgb::WHITE);
        let cell = c.get(1, 0).unwrap();
        assert_eq!(cell.ch, '#');
        assert_eq!(cell.bg, Some(sky));
    }

    #[test]
    fn dropping_a_background_repaints_with_default() {
        // Playlist switching: the next scene paints no background at all, so
        // the cells the old scene had coloured must be repainted as default.
        let mut c = Canvas::new(4, 2);
        let sky = Rgb::new(1, 2, 3);
        c.fill(0, 0, 4, 2, ' ', sky, Some(sky));
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::True).unwrap();
        assert!(!sink.is_empty());

        c.clear();
        sink.clear();
        c.present(&mut sink, ColorMode::True).unwrap();
        let s = String::from_utf8_lossy(&sink).to_string();
        assert!(s.contains("\x1b[40m"), "{s:?}");
    }

    #[test]
    fn mono_mode_never_emits_background_codes() {
        let mut c = Canvas::new(4, 1);
        c.fill(0, 0, 4, 1, ' ', Rgb::WHITE, Some(Rgb::new(9, 9, 9)));
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::Mono).unwrap();
        let s = String::from_utf8_lossy(&sink).to_string();
        assert!(!s.contains("48;"), "{s:?}");
    }

    #[test]
    fn render_keeps_background_painted_columns() {
        let mut c = Canvas::new(8, 2);
        let sky = Rgb::new(10, 20, 30);
        c.fill(0, 0, 8, 2, ' ', sky, Some(sky));
        let out = c.render_ansi(ColorMode::True);
        let text = String::from_utf8_lossy(&out);
        let first = text.lines().next().unwrap_or("");
        assert!(first.contains("48;2;10;20;30"), "{first:?}");
        assert!(first.contains("m        "), "{first:?}");
    }

    #[test]
    fn color_slack_is_a_hard_error_bound() {
        let mut c = Canvas::new(3, 1);
        let base = Rgb::new(100, 100, 100);
        c.set(0, 0, 'a', base);
        c.set(1, 0, 'b', base);
        c.set(2, 0, 'c', base);
        let mut sink = Vec::new();
        c.present(&mut sink, ColorMode::True).unwrap();
        sink.clear();
        // Within slack: a colour-only drift must not cost any traffic.
        c.set(1, 0, 'b', Rgb::new(102, 97, 103));
        let opts = PresentOptions {
            color_slack: 3,
            ..PresentOptions::default()
        };
        c.present_with(&mut sink, ColorMode::True, opts).unwrap();
        assert_eq!(sink.len(), 0, "{sink:?}");

        // Beyond slack: the cell is repainted, and the emitted colour is the
        // requested one (max error is bounded by the slack).
        c.set(2, 0, 'c', Rgb::new(120, 120, 120));
        sink.clear();
        c.present_with(&mut sink, ColorMode::True, opts).unwrap();
        assert!(sink.windows(4).any(|w| w == b"38;2"), "{sink:?}");
    }

    /// A deliberately small ANSI terminal used to prove that the encoder's
    /// cursor tricks and `EL` erases rebuild the canvas exactly.
    struct Emu {
        w: usize,
        h: usize,
        ch: Vec<char>,
        fg: Vec<Option<Rgb>>,
        bg: Vec<Option<Rgb>>,
        cur: (usize, usize),
        live_fg: Option<Rgb>,
        live_bg: Option<Rgb>,
    }

    impl Emu {
        fn new(w: usize, h: usize) -> Self {
            Emu {
                w,
                h,
                ch: vec![' '; w * h],
                fg: vec![None; w * h],
                bg: vec![None; w * h],
                cur: (0, 0),
                live_fg: None,
                live_bg: None,
            }
        }

        fn put(&mut self, ch: char) {
            let (r, c) = self.cur;
            if r < self.h && c < self.w {
                let idx = r * self.w + c;
                self.ch[idx] = ch;
                self.fg[idx] = self.live_fg;
                self.bg[idx] = self.live_bg;
            }
            self.cur.1 = (c + 1).min(self.w);
        }

        fn sgr(&mut self, params: &[u8]) {
            let text = std::str::from_utf8(params).unwrap_or("");
            let nums: Vec<u32> = text
                .split(';')
                .map(|p| p.parse::<u32>().unwrap_or(0))
                .collect();
            let mut i = 0;
            while i < nums.len() {
                match nums[i] {
                    0 => {
                        self.live_fg = None;
                        self.live_bg = None;
                    }
                    38 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                        self.live_fg = Some(Rgb::new(
                            nums[i + 2] as u8,
                            nums[i + 3] as u8,
                            nums[i + 4] as u8,
                        ));
                        i += 4;
                    }
                    48 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                        self.live_bg = Some(Rgb::new(
                            nums[i + 2] as u8,
                            nums[i + 3] as u8,
                            nums[i + 4] as u8,
                        ));
                        i += 4;
                    }
                    40 | 49 => self.live_bg = None,
                    39 => self.live_fg = None,
                    _ => {}
                }
                i += 1;
            }
        }

        fn feed(&mut self, bytes: &[u8]) {
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == 0x1b {
                    if bytes.get(i + 1) != Some(&b'[') {
                        i += 2;
                        continue;
                    }
                    let mut j = i + 2;
                    while j < bytes.len() && !(0x40..=0x7e).contains(&bytes[j]) {
                        j += 1;
                    }
                    if j >= bytes.len() {
                        break;
                    }
                    let params = &bytes[i + 2..j];
                    match bytes[j] {
                        b'H' | b'f' => {
                            let text = std::str::from_utf8(params).unwrap_or("");
                            let mut parts = text.split(';');
                            let r: usize = parts.next().and_then(|p| p.parse().ok()).unwrap_or(1);
                            let c: usize = parts.next().and_then(|p| p.parse().ok()).unwrap_or(1);
                            self.cur = (r.saturating_sub(1), c.saturating_sub(1));
                        }
                        b'C' => {
                            let n = std::str::from_utf8(params)
                                .ok()
                                .and_then(|p| p.parse().ok())
                                .unwrap_or(1);
                            self.cur.1 = (self.cur.1 + n).min(self.w);
                        }
                        b'K' => {
                            let (r, c) = self.cur;
                            if r < self.h {
                                for x in c.min(self.w)..self.w {
                                    self.ch[r * self.w + x] = ' ';
                                    self.fg[r * self.w + x] = None;
                                    self.bg[r * self.w + x] = self.live_bg;
                                }
                            }
                        }
                        b'm' => self.sgr(params),
                        _ => {}
                    }
                    i = j + 1;
                } else if bytes[i] == b'\r' {
                    self.cur.1 = 0;
                    i += 1;
                } else {
                    let s = std::str::from_utf8(&bytes[i..]).ok();
                    if let Some(s) = s {
                        if let Some(ch) = s.chars().next() {
                            self.put(ch);
                            i += ch.len_utf8();
                            continue;
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    fn lcg(state: &mut u64) -> u32 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*state >> 33) as u32
    }

    #[test]
    fn encoder_geometry_survives_random_frames() {
        let mut seed = 0x1234_5678_9abc_def0u64;
        for case in 0..40 {
            let w = 1 + (lcg(&mut seed) % 18) as usize;
            let h = 1 + (lcg(&mut seed) % 7) as usize;
            let mut c = Canvas::new(w, h);
            let mut emu = Emu::new(w, h);
            let mut sink = Vec::new();
            for frame in 0..24 {
                let edits = 1 + (lcg(&mut seed) as usize) % (w * h + 1);
                for _ in 0..edits {
                    let x = (lcg(&mut seed) as usize) % w;
                    let y = (lcg(&mut seed) as usize) % h;
                    match lcg(&mut seed) % 5 {
                        0 => {
                            c.paint(x as isize, y as isize, ' ', Rgb::BLACK, None);
                        }
                        1 => {
                            let col = Rgb::new(
                                (lcg(&mut seed) % 256) as u8,
                                (lcg(&mut seed) % 256) as u8,
                                (lcg(&mut seed) % 256) as u8,
                            );
                            c.paint(
                                x as isize,
                                y as isize,
                                ' ',
                                col,
                                Some(Rgb::new(
                                    (lcg(&mut seed) % 256) as u8,
                                    (lcg(&mut seed) % 256) as u8,
                                    (lcg(&mut seed) % 256) as u8,
                                )),
                            );
                        }
                        _ => {
                            let glyphs = ['#', '@', 'x', 'o', '+'];
                            let ch = glyphs[(lcg(&mut seed) as usize) % glyphs.len()];
                            let col = Rgb::new(
                                (lcg(&mut seed) % 256) as u8,
                                (lcg(&mut seed) % 256) as u8,
                                (lcg(&mut seed) % 256) as u8,
                            );
                            c.set(x as isize, y as isize, ch, col);
                        }
                    }
                }
                if frame % 7 == 6 {
                    // Clearing everything exercises the `EL` tail path.
                    c.clear();
                }
                let slack = (lcg(&mut seed) % 9) as u8;
                let opts = PresentOptions {
                    color_slack: slack,
                    sync: frame % 2 == 0,
                    // Exercise both encoders: adaptive cursor moves plus `EL`,
                    // and the conservative absolute-addressing path.
                    erase_tail: frame % 3 != 0,
                    cursor_moves: frame % 4 != 0,
                };
                sink.clear();
                c.present_with(&mut sink, ColorMode::True, opts).unwrap();
                emu.feed(&sink);
                for y in 0..h {
                    for x in 0..w {
                        let idx = y * w + x;
                        let want = c.cells[idx];
                        let got_ch = emu.ch[idx];
                        assert_eq!(
                            got_ch, want.ch,
                            "case {case} frame {frame} at {x},{y}: {sink:?}"
                        );
                        match (want.bg, emu.bg[idx]) {
                            (None, None) => {}
                            (Some(a), Some(b)) => assert!(
                                near(a, b, slack),
                                "case {case} frame {frame} bg at {x},{y}: {a:?} vs {b:?} slack {slack}"
                            ),
                            (a, b) => panic!(
                                "case {case} frame {frame} bg at {x},{y}: {a:?} vs {b:?}"
                            ),
                        }
                        if want.ch != ' ' {
                            let got = emu.fg[idx].unwrap_or_else(|| {
                                panic!("case {case} frame {frame} at {x},{y}: no fg")
                            });
                            assert!(
                                near(want.fg, got, slack),
                                "case {case} frame {frame} fg at {x},{y}: {:?} vs {got:?} slack {slack}",
                                want.fg
                            );
                        }
                    }
                }
            }
        }
    }
}
