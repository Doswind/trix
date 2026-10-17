//! 黑客帝国 — green digital rain with katakana, glitches and a typewriter.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

static KANA: &[char] = &[
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾂ', 'ﾃ', 'ﾅ', 'ﾆ',
    'ﾇ', 'ﾈ', 'ﾊ', 'ﾋ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾗ', 'ﾘ', 'ﾜ', 'ﾝ', '0', '1', '2',
    '3', '4', '5', '6', '7', '8', '9', ':', '.', '"', '=', '*', '+', '-', '<', '>',
];

static ASCII: &[char] = &[
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ':', '.',
    '"', '=', '*', '+', '-', '<', '>', '|', '/', '\\', '$', '@', '#', '%', '&', '?', '!',
];

const MESSAGES: &[&str] = &[
    "Wake up, Neo...",
    "The Matrix has you...",
    "Follow the white rabbit.",
    "Knock, knock, Neo.",
];

struct Column {
    head: f32,
    speed: f32,
    len: usize,
    bright: f32,
    depth: f32,
}

pub struct MatrixScene {
    rng: Rng,
    w: usize,
    h: usize,
    cols: Vec<Column>,
    glyphs: Vec<char>,
    font: Vec<u8>,
    charset: Option<Charset>,
    glitch_t: f32,
    next_glitch: f64,
    glitch_rows: Vec<bool>,
    glitch_off: Vec<i8>,
    msg_active: bool,
    msg_index: usize,
    msg_typed: usize,
    msg_next_char: f64,
    msg_until: f64,
    msg_next: f64,
}

impl MatrixScene {
    pub fn new(seed: u64) -> Self {
        MatrixScene {
            rng: Rng::new(seed ^ 0x0bad_c0de),
            w: 0,
            h: 0,
            cols: Vec::new(),
            glyphs: KANA.to_vec(),
            font: Vec::new(),
            charset: None,
            glitch_t: 0.0,
            next_glitch: 8.0,
            glitch_rows: Vec::new(),
            glitch_off: Vec::new(),
            msg_active: false,
            msg_index: 0,
            msg_typed: 0,
            msg_next_char: 0.0,
            msg_until: 0.0,
            msg_next: 5.0,
        }
    }

    fn set_charset(&mut self, charset: Charset) {
        if self.charset == Some(charset) {
            return;
        }
        self.charset = Some(charset);
        self.glyphs = match charset {
            Charset::Kana => KANA.to_vec(),
            Charset::Ascii => ASCII.to_vec(),
        };
        self.rebuild(self.w.max(1), self.h.max(1));
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.font = vec![0; self.w * self.h];
        for cell in self.font.iter_mut() {
            *cell = self.rng.below(self.glyphs.len()) as u8;
        }
        self.cols.clear();
        for _ in 0..self.w {
            let depth = self.rng.f32();
            let len = (self.h as f32 * (0.35 + depth * 0.55)).max(6.0) as usize;
            self.cols.push(Column {
                head: self.rng.range(-(len as f32), h as f32),
                speed: self.rng.range(3.0, 14.0) * (0.65 + depth * 0.6),
                len: len.min(self.h),
                bright: 0.35 + depth * 0.75,
                depth,
            });
        }
        self.glitch_rows = vec![false; self.h];
        self.glitch_off = vec![0; self.h];
    }

    fn trigger_glitch(&mut self, t: f64) {
        self.glitch_t = self.rng.range(0.22, 0.42);
        self.next_glitch = t + self.rng.range(9.0, 19.0) as f64;
        for i in 0..self.h {
            self.glitch_rows[i] = false;
            self.glitch_off[i] = 0;
        }
        let bands = 1 + self.rng.below(3);
        for _ in 0..bands {
            let y = self.rng.below(self.h);
            let off = self.rng.range(-4.0, 4.0) as i8;
            for dy in 0..self.rng.below(3) + 1 {
                let yy = (y + dy).min(self.h - 1);
                self.glitch_rows[yy] = true;
                self.glitch_off[yy] = off;
            }
        }
    }

    fn glyph(&self, x: usize, y: usize) -> char {
        self.glyphs[self.font[y * self.w + x] as usize % self.glyphs.len()]
    }
}

fn matrix_color(theme: &Theme, f: f32) -> Rgb {
    // f: 0 at the tail, 1 at the head.
    if f > 0.9 {
        theme.c(4)
    } else if f > 0.55 {
        theme.c(3).lerp(theme.c(4), (f - 0.55) / 0.35)
    } else if f > 0.2 {
        theme.c(2).lerp(theme.c(3), (f - 0.2) / 0.35)
    } else {
        theme.c(1).lerp(theme.c(2), f / 0.2)
    }
}

impl Scene for MatrixScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        if self.charset != Some(f.charset) {
            self.set_charset(f.charset);
        }
        let dt = f.dt.clamp(0.0, 0.1).min(0.1);
        let hf = self.h as f32;
        for col in self.cols.iter_mut() {
            col.head += col.speed * dt;
            // Mutate a glyph in the visible trail now and then.
            if self.rng.chance(0.18) {
                let y = self.rng.below(self.h);
                let x = self.rng.below(self.w);
                self.font[y * self.w + x] = self.rng.below(self.glyphs.len()) as u8;
            }
            if col.head - col.len as f32 > hf {
                let depth = self.rng.f32();
                col.head = self.rng.range(-(col.len as f32) * 1.5, -1.0);
                col.speed = self.rng.range(3.0, 14.0) * (0.65 + depth * 0.6);
                col.depth = depth;
                col.bright = 0.35 + depth * 0.75;
                col.len = (self.h as f32 * (0.35 + depth * 0.55)).max(6.0) as usize;
                col.len = col.len.min(self.h);
            }
        }

        if f.t > self.next_glitch {
            self.trigger_glitch(f.t);
        }
        self.glitch_t = (self.glitch_t - dt).max(0.0);

        // Typewriter message.
        if self.msg_active {
            if self.msg_typed < MESSAGES[self.msg_index].len() && f.t >= self.msg_next_char {
                self.msg_typed += 1;
                self.msg_next_char = f.t + 0.055;
            }
            if self.msg_typed >= MESSAGES[self.msg_index].len() && f.t > self.msg_until {
                self.msg_active = false;
                self.msg_next = f.t + 10.0;
            }
        } else if f.t >= self.msg_next {
            self.msg_active = true;
            self.msg_index = (self.msg_index + 1) % MESSAGES.len();
            self.msg_typed = 0;
            self.msg_next_char = f.t;
            self.msg_until = f.t + 3.5;
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let glitching = self.glitch_t > 0.0;

        for (x, col) in self.cols.iter().enumerate() {
            let head_y = col.head.floor() as isize;
            let mut x_draw = x as isize;
            if glitching
                && self
                    .glitch_rows
                    .get(head_y.max(0) as usize)
                    .copied()
                    .unwrap_or(false)
            {
                x_draw += self.glitch_off[(head_y.max(0) as usize).min(self.h - 1)] as isize;
            }
            for d in 0..col.len {
                let y = head_y - d as isize;
                if y < 0 || y >= self.h as isize {
                    continue;
                }
                let fade = 1.0 - d as f32 / col.len as f32;
                let ch = self.glyph(x, y as usize);
                let scan = if (y as usize + (t * 9.0) as usize) % 7 == 0 {
                    0.82
                } else {
                    1.0
                };
                let mut color = if d == 0 {
                    theme.c(4)
                } else {
                    matrix_color(theme, fade)
                        .scale(fade.powf(1.35) * (0.62 + col.bright * 0.42) * scan)
                };
                if glitching && self.glitch_rows.get(y as usize).copied().unwrap_or(false) {
                    color = color.lerp(Rgb::new(255, 80, 90), 0.45);
                }
                if color.luminance() < 0.02 {
                    continue;
                }
                canvas.set(x_draw, y, ch, color);
                if d == 0 {
                    canvas.add_light(x_draw, y + 1, theme.c(2).scale(0.4), 0.8, ':');
                    canvas.add_light(x_draw, y - 1, theme.c(3).scale(0.18), 0.5, '.');
                }
            }
        }

        if glitching {
            // Sharp horizontal corruption bands.
            let mut bands = 0;
            for y in 0..self.h {
                if !self.glitch_rows[y] || bands >= 4 {
                    continue;
                }
                bands += 1;
                for x in (0..self.w).step_by(3) {
                    if (x + y) % 2 == 0 {
                        let ch = if (x / 3) % 2 == 0 { '=' } else { '#' };
                        canvas.set(
                            x as isize,
                            y as isize,
                            ch,
                            theme.c(4).lerp(Rgb::new(255, 50, 50), 0.35),
                        );
                    }
                }
            }
        }

        // Typewriter message in a small console.
        if self.msg_active && self.w >= 36 && self.h >= 12 {
            let text = MESSAGES[self.msg_index];
            let shown = &text[..self.msg_typed.min(text.len())];
            let bw = (text.len() + 8).min(self.w - 4) as isize;
            let x = 2isize;
            let y = self.h as isize - 5;
            canvas.clear_rect(x, y, bw, 3);
            let dim = theme.c(1);
            canvas.hline(x, x + bw - 1, y, '-', dim);
            canvas.hline(x, x + bw - 1, y + 2, '-', dim);
            canvas.set(x, y, '+', dim);
            canvas.set(x + bw - 1, y, '+', dim);
            canvas.set(x, y + 2, '+', dim);
            canvas.set(x + bw - 1, y + 2, '+', dim);
            canvas.put_str(x + 2, y + 1, ">", theme.c(2));
            canvas.put_str(x + 4, y + 1, shown, theme.c(4));
            let cursor_on = (t * 2.0).fract() < 0.55;
            if cursor_on {
                canvas.set(
                    x + 4 + shown.len() as isize,
                    y + 1,
                    if (t * 4.0).fract() < 0.5 { '#' } else { '_' },
                    theme.c(3),
                );
            }
        }
    }
}
