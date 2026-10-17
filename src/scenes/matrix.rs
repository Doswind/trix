//! 黑客帝国 — a horizontal NASA/JPL intrusion console.
//!
//! Instead of vertical digital rain, this scene is built like a 90s movie
//! hacking terminal: horizontal tapes of hex, shell commands and telemetry
//! scroll across the screen, while a live intrusion console tracks firewall
//! and decryption progress against a NASA/JPL target.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash2, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

const TOKENS: &[&str] = &[
    "0x7F3A",
    "A4:2B",
    "DEADBEEF",
    "0xC0FFEE",
    "sudo",
    "root@dsn",
    "nmap",
    "-sS",
    "ssh",
    "decrypt",
    "auth",
    "bypass",
    "payload",
    "kernel",
    "trace",
    "NULL",
    "0x00",
    "::",
    "[]",
    "{}",
    "->",
    "if(auth)",
    "for(i=0;",
    "while(1)",
    "ret",
    "shell",
    "node",
    "TLS1.3",
    "GPS",
    "DSN",
    "JPL",
    "NASA",
    "VOYAGER",
    "ARTEMIS",
    "TELEMETRY",
    "UPLINK",
    "DOWNLINK",
    "SATCOM",
    "ORBIT",
    "APOLLO",
    "HUBBLE",
    "CASSINI",
    "0xDEAD",
    "0xBEEF",
    "chmod",
    "inject",
    "access",
    "mainframe",
    "handshake",
    "firewall",
    "rootkit",
    "packet",
    "socket",
    "buffer",
    "overflow",
    "exploit",
    "proxy",
    "tunnel",
];

const KANA_TOKENS: &[&str] = &[
    "ｱｸｾｽ",
    "ﾃﾞｰﾀ",
    "ﾊﾞｲﾊﾟｽ",
    "ﾈｯﾄﾜｰｸ",
    "ｼｽﾃﾑ",
    "ﾃﾚﾒﾄﾘ",
    "ﾊﾟｽﾜｰﾄﾞ",
    "ｱｰｶｲﾌﾞ",
    "ｼｸﾞﾅﾙ",
    "ｵﾝﾗｲﾝ",
    "ﾒｲﾝﾌﾚｰﾑ",
    "ｾｷｭﾘﾃｨ",
];

const STREAM_GLYPHS: &[u8] = b"0123456789ABCDEF#$%&*+=<>/\\|;:";

const LOGS: &[(&str, f64)] = &[
    ("verify credentials ......... OK", 0.6),
    ("2FA challenge .............. OK", 2.0),
    ("certificate chain .......... OK", 4.0),
    ("firewall bypass ............ 64%", 6.0),
    ("root shell established", 8.5),
    ("decrypting credentials ..... 81%", 11.0),
    ("ACCESS GRANTED", 14.0),
];

const CYCLE: f64 = 24.0;

struct Stream {
    x: f32,
    speed: f32,
    len: usize,
    glyph: usize,
    bright: f32,
}

struct Row {
    tape: Vec<char>,
    offset: f32,
    speed: f32,
    dim: f32,
    streams: Vec<Stream>,
}

#[derive(Clone, Copy)]
struct Panel {
    x: isize,
    y: isize,
    w: isize,
    h: isize,
    left: isize,
}

impl Panel {
    fn contains(&self, x: isize, y: isize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogoKind {
    Full,
    Compact,
    Tiny,
}

#[derive(Clone, Copy)]
struct Logo {
    x: isize,
    y: isize,
    w: isize,
    h: isize,
    sx: isize,
    sy: isize,
    kind: LogoKind,
}

impl Logo {
    fn contains(&self, x: isize, y: isize) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

pub struct MatrixScene {
    rng: Rng,
    seed: u64,
    w: usize,
    h: usize,
    rows: Vec<Row>,
    charset: Option<Charset>,
    glitch_t: f32,
    next_glitch: f64,
    glitch_rows: Vec<bool>,
}

impl MatrixScene {
    pub fn new(seed: u64) -> Self {
        MatrixScene {
            rng: Rng::new(seed ^ 0x0bad_c0de),
            seed,
            w: 0,
            h: 0,
            rows: Vec::new(),
            charset: None,
            glitch_t: 0.0,
            next_glitch: 8.0,
            glitch_rows: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let kana = self.charset == Some(Charset::Kana);
        self.rows.clear();
        for y in 0..self.h {
            let tape = self.make_tape(y, kana);
            let mut streams = Vec::new();
            let count = if y % 6 == 0 { 2 } else { 1 };
            for _ in 0..count {
                streams.push(self.make_stream(tape.len()));
            }
            let dim = if y % 7 == 3 {
                self.rng.range(0.11, 0.20)
            } else {
                self.rng.range(0.06, 0.15)
            };
            self.rows.push(Row {
                tape,
                offset: self.rng.range(0.0, 180.0),
                speed: self.rng.range(1.2, 6.5),
                dim,
                streams,
            });
        }
        self.glitch_rows = vec![false; self.h];
    }

    fn make_tape(&mut self, y: usize, kana: bool) -> Vec<char> {
        let mut tape = Vec::with_capacity(self.w * 2 + 80);
        let target = self.w * 2 + 60;
        let log_row = y % 7 == 3;
        while tape.len() < target {
            if log_row {
                let node = (self.rng.next_u32() & 0xffff) as u16;
                let line = format!(
                    "[ OK ] ACCESS NODE 0x{:04X} :: TELEMETRY STREAM {} :: ",
                    node,
                    self.rng.below(9) + 1
                );
                for ch in line.chars() {
                    if tape.len() < target {
                        tape.push(ch);
                    }
                }
                for ch in "PAYLOAD 0xDEADBEEF :: ".chars() {
                    if tape.len() < target {
                        tape.push(ch);
                    }
                }
            } else {
                let token = if kana && self.rng.chance(0.08) {
                    KANA_TOKENS[self.rng.below(KANA_TOKENS.len())]
                } else {
                    TOKENS[self.rng.below(TOKENS.len())]
                };
                for ch in token.chars() {
                    if tape.len() < target {
                        tape.push(ch);
                    }
                }
                if tape.len() < target {
                    tape.push(' ');
                }
            }
        }
        tape
    }

    fn make_stream(&mut self, tape_len: usize) -> Stream {
        Stream {
            x: self.rng.range(0.0, self.w as f32),
            speed: self.rng.range(5.0, 17.0),
            len: (self.rng.below(16) + 10).min(self.w.max(8)),
            glyph: self.rng.below(tape_len.max(1)),
            bright: self.rng.range(0.55, 1.0),
        }
    }

    fn set_charset(&mut self, charset: Charset) {
        if self.charset == Some(charset) {
            return;
        }
        self.charset = Some(charset);
        self.rebuild(self.w.max(1), self.h.max(1));
    }

    fn phase(&self, t: f64) -> f64 {
        let p = t % CYCLE;
        if p < 0.0 {
            p + CYCLE
        } else {
            p
        }
    }

    fn progress(&self, t: f64) -> f32 {
        ((self.phase(t) / 14.0) * 100.0).clamp(0.0, 100.0) as f32
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
        let dt = f.dt.clamp(0.0, 0.1);
        for row in self.rows.iter_mut() {
            row.offset += row.speed * dt;
            if row.offset >= row.tape.len() as f32 {
                row.offset -= row.tape.len() as f32;
            }
            for stream in row.streams.iter_mut() {
                stream.x -= stream.speed * dt;
                if stream.x + (stream.len as f32) < 0.0 {
                    let s = self.rng.range(6.0, 19.0);
                    let len = self.rng.below(16) + 10;
                    stream.x = self.w as f32 + self.rng.range(0.0, 24.0);
                    stream.speed = s;
                    stream.len = len.min(self.w.max(8));
                    stream.glyph = self.rng.below(row.tape.len().max(1));
                    stream.bright = self.rng.range(0.55, 1.0);
                }
            }
        }

        if f.t > self.next_glitch {
            self.glitch_t = self.rng.range(0.22, 0.4);
            self.next_glitch = f.t + self.rng.range(8.0, 15.0) as f64;
            for g in self.glitch_rows.iter_mut() {
                *g = false;
            }
            for _ in 0..(1 + self.rng.below(3)) {
                let y = self.rng.below(self.h);
                self.glitch_rows[y] = true;
            }
        }
        self.glitch_t = (self.glitch_t - dt).max(0.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        let w = self.w;
        let h = self.h;
        let panel = panel_rect(w, h);
        let logo = logo_rect(w, h);
        let glitching = self.glitch_t > 0.0;
        let phase = self.phase(f.t);
        let progress = self.progress(f.t);

        // Horizontal background tapes.
        for (y, row) in self.rows.iter().enumerate() {
            if y == 0 || y + 1 == h {
                continue;
            }
            let start = (row.offset as usize) % row.tape.len().max(1);
            let twinkle = 0.82 + 0.18 * (t * 2.3 + y as f32 * 0.7).sin();
            let col = theme.c(2).scale(row.dim * twinkle);
            for x in 0..w {
                if panel.is_some_and(|p| p.contains(x as isize, y as isize))
                    || logo.contains(x as isize, y as isize)
                {
                    continue;
                }
                let ch = row.tape[(start + x) % row.tape.len()];
                if ch != ' ' {
                    canvas.set(x as isize, y as isize, ch, col);
                }
            }

            // Bright horizontal data streams with a white head.
            for stream in &row.streams {
                let head = stream.x.floor() as isize;
                for i in 0..stream.len {
                    let x = head + i as isize;
                    if x < 0 || x >= w as isize {
                        continue;
                    }
                    if panel.is_some_and(|p| p.contains(x, y as isize))
                        || logo.contains(x, y as isize)
                    {
                        continue;
                    }
                    let idx = (stream.glyph + i * 7) % STREAM_GLYPHS.len();
                    let ch = STREAM_GLYPHS[idx] as char;
                    let fade = 1.0 - i as f32 / stream.len as f32;
                    let color = if i == 0 {
                        theme.c(4)
                    } else if i < 3 {
                        theme.hot()
                    } else {
                        theme
                            .c(3)
                            .lerp(theme.c(2), (i as f32 / stream.len as f32).min(1.0))
                            .scale(fade.powf(1.9) * stream.bright * 0.9)
                    };
                    if color.luminance() < 0.02 {
                        continue;
                    }
                    canvas.set(x, y as isize, ch, color);
                    if i == 0 {
                        canvas.add_light(x, y as isize, theme.c(3), 0.8, '.');
                        canvas.add_light(x + 1, y as isize, theme.c(2), 0.45, ':');
                        canvas.add_light(x + 2, y as isize, theme.c(2), 0.2, '.');
                    }
                }
            }
        }

        // A slow scanline sweeps down over the code.
        let scan_y = ((t * 3.0) as usize) % h.max(1);
        if scan_y > 0 && scan_y + 1 < h {
            for x in (0..w).step_by(2) {
                if panel.is_some_and(|p| p.contains(x as isize, scan_y as isize))
                    || logo.contains(x as isize, scan_y as isize)
                {
                    continue;
                }
                if hash2(x as i64, scan_y as i64, self.seed) & 3 == 0 {
                    canvas.add_light(x as isize, scan_y as isize, theme.c(2), 0.28, '-');
                }
            }
        }

        // Glitch bands outside the console.
        if glitching {
            for y in 0..h {
                if !self.glitch_rows.get(y).copied().unwrap_or(false) || y == 0 || y + 1 == h {
                    continue;
                }
                for x in (0..w).step_by(2) {
                    if panel.is_some_and(|p| p.contains(x as isize, y as isize))
                        || logo.contains(x as isize, y as isize)
                    {
                        continue;
                    }
                    let hsh = hash2(x as i64, y as i64 ^ 0x51, self.seed ^ 7);
                    let ch = if hsh & 3 == 0 {
                        '#'
                    } else if hsh & 3 == 1 {
                        '='
                    } else {
                        (b'0' + (hsh % 10) as u8) as char
                    };
                    let col = if hsh & 1 == 0 {
                        theme.c(4)
                    } else {
                        Rgb::hex(0xff4d4d)
                    };
                    canvas.set(x as isize, y as isize, ch, col);
                }
            }
        }

        self.draw_nasa_logo(canvas, theme, &logo, t);

        if let Some(p) = panel {
            self.draw_console(canvas, theme, p, t, phase, progress, glitching);
        }

        // HUD strips.
        canvas.clear_rect(0, 0, w as isize, 1);
        canvas.clear_rect(0, h as isize - 1, w as isize, 1);
        let title = if w >= 70 {
            " NASA SECURE MAINFRAME // AUTHORIZED PERSONNEL ONLY "
        } else {
            " NASA LOGIN "
        };
        canvas.put_str_gradient(0, 0, title, &[theme.c(2), theme.c(3), theme.hot()]);
        let right = "SESSION 0x7F3A-9C2B";
        let rx = w as isize - right.len() as isize - 1;
        if rx > title.len() as isize + 4 {
            canvas.put_str(rx, 0, right, theme.c(2).scale(0.8));
        }

        let status = if glitching {
            " [ !! TRACE DETECTED !! ]  SIGNAL LOST  [ RECONNECTING ] ".to_string()
        } else if progress >= 100.0 {
            " [ ACCESS GRANTED ]  SESSION 0x7F3A-9C2B  [ NASA-JPL ] ".to_string()
        } else {
            format!(
                " [ SECURE LOGIN ]  AUTH {:3.0}%  TRACE 00:00:00  [ NASA-JPL ] ",
                progress
            )
        };
        let sc = if glitching {
            Rgb::hex(0xff5a5a)
        } else if progress >= 100.0 {
            theme.hot()
        } else {
            theme.c(2)
        };
        canvas.put_str(1, h as isize - 1, &status, sc);
    }
}

impl MatrixScene {
    fn draw_nasa_logo(&self, canvas: &mut Canvas, theme: &Theme, logo: &Logo, t: f32) {
        let full = [
            "#   # ##### ##### #####",
            "##  # #   # #     #   #",
            "# # # ##### ##### #####",
            "#  ## #   #     # #   #",
            "#   # #   # ##### #   #",
        ];
        let compact = ["#  # ### ### ###", "## # # # #   # #", "# ## ### ### ###"];
        canvas.clear_rect(logo.x - 2, logo.y - 1, logo.w + 4, logo.h + 2);
        if logo.kind == LogoKind::Tiny {
            let text = "N A S A";
            for (i, ch) in text.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let x = logo.x + i as isize * 2;
                for k in 0..2 {
                    let sweep = ((t * 1.6 - i as f32 * 0.12).sin() * 0.5 + 0.5).powf(2.0);
                    let col = Rgb::hex(0xffffff)
                        .lerp(Rgb::hex(0x38bdf8), 0.18 + sweep * 0.32)
                        .lerp(Rgb::hex(0xef4444), sweep * 0.16);
                    canvas.set(x + k, logo.y, ch, col);
                }
            }
            return;
        }
        let (art, rows) = if logo.kind == LogoKind::Full {
            (&full[..], 5usize)
        } else {
            (&compact[..], 3usize)
        };
        let white = Rgb::hex(0xffffff);
        let blue = Rgb::hex(0x38bdf8);
        let red = Rgb::hex(0xef4444);
        for (r, line) in art.iter().enumerate().take(rows) {
            for (c, ch) in line.chars().enumerate() {
                if ch != '#' {
                    continue;
                }
                let sweep =
                    ((t * 1.7 - c as f32 * 0.10 + r as f32 * 0.05).sin() * 0.5 + 0.5).powf(2.0);
                let base = white
                    .lerp(blue, 0.10 + sweep * 0.28)
                    .lerp(red, sweep * 0.14);
                let base = base.scale(0.84 + 0.16 * (t * 3.0 + c as f32 * 0.1).sin().abs());
                for sy in 0..logo.sy {
                    for sx in 0..logo.sx {
                        let x = logo.x + c as isize * logo.sx + sx;
                        let y = logo.y + r as isize * logo.sy + sy;
                        canvas.set(x, y, '#', base);
                        if (c + r) % 2 == 0 {
                            canvas.add_light(x, y, base, 0.16, '.');
                        }
                    }
                }
            }
        }
        if logo.kind == LogoKind::Full {
            let sub = "NATIONAL AERONAUTICS AND SPACE ADMINISTRATION";
            if (logo.w as usize) >= 20 && (self.w as isize) > 20 {
                let sy = logo.y + logo.h;
                canvas.clear_rect(0, sy, self.w as isize, 1);
                let sx = ((self.w as isize - sub.len() as isize) / 2).max(1);
                canvas.put_str_gradient(
                    sx,
                    sy,
                    sub,
                    &[Rgb::hex(0x38bdf8), Rgb::hex(0xe2e8f0), Rgb::hex(0xef4444)],
                );
            }
        }
        let _ = theme;
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_console(
        &self,
        canvas: &mut Canvas,
        theme: &Theme,
        p: Panel,
        t: f32,
        phase: f64,
        progress: f32,
        glitching: bool,
    ) {
        let border = if glitching {
            Rgb::hex(0xff4d4d)
        } else {
            theme.c(2).scale(0.75)
        };
        canvas.clear_rect(p.x, p.y, p.w, p.h);
        canvas.rect(p.x, p.y, p.w, p.h, border);
        let title = " NASA BACKEND // SECURE LOGIN ";
        canvas.put_str(p.x + 1, p.y, title, theme.hot());
        let right = "SESSION 0x7F3A-9C2B ";
        let rx = p.x + p.w - right.len() as isize - 2;
        if rx > p.x + title.len() as isize + 2 {
            canvas.put_str(rx, p.y, right, theme.c(2).scale(0.8));
        }

        // left / right column divider
        let mid = p.x + p.left;
        canvas.vline(mid, p.y + 1, p.y + p.h - 2, '|', border);

        let lx = p.x + 2;
        let rxx = mid + 2;
        let top = p.y + 2;
        let label = theme.c(2).scale(0.75);
        let value = theme.c(3);
        let dim = theme.c(1).scale(1.8);

        // Backend login fields.
        canvas.put_str(lx, top, "OPERATOR", label);
        canvas.put_str(lx + 10, top, "root@dsn.jpl.nasa.gov", value);
        canvas.put_str(lx, top + 1, "PASSWORD", label);
        canvas.put_str(
            lx + 10,
            top + 1,
            "************************",
            theme.c(1).scale(1.3),
        );
        canvas.put_str(lx, top + 2, "2FA", label);
        canvas.put_str(lx + 10, top + 2, "739 204 // VERIFIED", value);

        let bar_y = top + 4;
        self.draw_bar(canvas, theme, lx, bar_y, "AUTH", progress, 24);
        self.draw_bar(
            canvas,
            theme,
            lx,
            bar_y + 1,
            "DECRYPT",
            (progress * 1.18).min(100.0),
            24,
        );

        // separator
        let sep_y = bar_y + 3;
        for x in lx..(mid - 1) {
            canvas.set(x, sep_y, '-', dim);
        }

        // Revealed log lines.
        let mut logs: Vec<(&str, usize)> = Vec::new();
        for (text, at) in LOGS {
            if phase + 0.001 >= *at {
                let elapsed = (phase - at).max(0.0);
                let typed = ((elapsed * 28.0) as usize).min(text.len());
                logs.push((text, typed));
            }
        }
        let show = if p.h >= 20 { 5 } else { 4 };
        let start = logs.len().saturating_sub(show);
        let log_y = sep_y + 1;
        for (i, (text, typed)) in logs[start..].iter().enumerate() {
            let y = log_y + i as isize;
            if y >= p.y + p.h - 3 {
                break;
            }
            let is_granted = text.contains("GRANTED");
            let col = if is_granted { theme.hot() } else { theme.c(3) };
            canvas.put_str(
                lx,
                y,
                ">",
                if is_granted { theme.hot() } else { theme.c(2) },
            );
            let shown: String = text.chars().take(*typed).collect();
            canvas.put_str(lx + 2, y, &shown, col);
            if *typed < text.len() {
                canvas.set(lx + 2 + shown.len() as isize, y, '_', theme.c(3));
            }
        }

        // Command prompt / status at the bottom of the left panel.
        let prompt_y = p.y + p.h - 2;
        if prompt_y > log_y {
            let (cmd, col) = if progress >= 100.0 {
                ("ACCESS GRANTED // root@dsn:~#", theme.hot())
            } else if phase > 6.0 {
                ("unlock /nasa/backend --force", theme.c(3))
            } else {
                ("login root@nasa.gov", theme.c(3))
            };
            canvas.put_str(lx, prompt_y, ">", theme.c(2));
            canvas.put_str(lx + 2, prompt_y, cmd, col);
            if (t * 2.0).fract() < 0.5 {
                canvas.set(lx + 2 + cmd.len() as isize, prompt_y, '#', theme.c(3));
            }
        }

        // Right-hand radar / uplink panel.
        canvas.put_str(rxx, top, "NASA SSO / DSN", label);
        let info_y = if p.h >= 20 {
            self.draw_radar(canvas, theme, rxx, top + 2, t, p);
            p.y + p.h - 6
        } else {
            canvas.put_str(rxx, top + 2, "[ DSN 70M ]", theme.c(3));
            top + 4
        };
        canvas.put_str(rxx, info_y, "SIGNAL 98%", theme.c(3));
        canvas.put_str(rxx, info_y + 1, "TRACE 00:00:00", theme.c(2).scale(0.9));
        canvas.put_str(
            rxx,
            info_y + 2,
            if progress >= 100.0 {
                "LOCK NASA-JPL"
            } else {
                "LOCK ACQUIRED"
            },
            if progress >= 100.0 {
                theme.hot()
            } else {
                theme.c(3)
            },
        );
        let bars: String = (0..(p.left.clamp(4, 20) - 2))
            .map(|i| if (i as f32 / 18.0) < 0.9 { '#' } else { '-' })
            .collect();
        if info_y + 3 < p.y + p.h - 1 {
            canvas.put_str(rxx, info_y + 3, &bars, theme.c(2).scale(0.8));
        }

        if glitching {
            let msg = "!! PACKET LOSS !!";
            let mx = p.x + p.w - msg.len() as isize - 2;
            canvas.put_str(mx, p.y + p.h - 2, msg, Rgb::hex(0xff5a5a));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_bar(
        &self,
        canvas: &mut Canvas,
        theme: &Theme,
        x: isize,
        y: isize,
        label: &str,
        percent: f32,
        width: usize,
    ) {
        canvas.put_str(x, y, label, theme.c(2).scale(0.8));
        let bx = x + 10;
        let pct = percent.clamp(0.0, 100.0);
        let filled = ((pct / 100.0) * width as f32).round() as usize;
        canvas.set(bx, y, '[', theme.c(2).scale(0.8));
        for i in 0..width {
            let ch = if i < filled { '#' } else { '.' };
            let col = if i < filled {
                theme.c(3)
            } else {
                theme.c(1).scale(1.7)
            };
            canvas.set(bx + 1 + i as isize, y, ch, col);
        }
        canvas.set(bx + 1 + width as isize, y, ']', theme.c(2).scale(0.8));
        canvas.put_str(
            bx + width as isize + 3,
            y,
            &format!("{:3.0}%", pct),
            theme.c(3),
        );
    }

    fn draw_radar(&self, canvas: &mut Canvas, theme: &Theme, x: isize, y: isize, t: f32, p: Panel) {
        let cx = x + 8;
        let cy = y + 3;
        let color = theme.c(2).scale(0.65);
        // ellipse outline
        for i in 0..48 {
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            let px = cx + (a.cos() * 7.0).round() as isize;
            let py = cy + (a.sin() * 3.0).round() as isize;
            if px > x && px < p.x + p.w - 2 {
                canvas.set(px, py, '.', color);
            }
        }
        canvas.set(cx, cy, '+', theme.c(3));
        // sweep line
        let a = t * 1.8;
        for r in 1..=7 {
            let px = cx + (a.cos() * r as f32).round() as isize;
            let py = cy + (a.sin() * r as f32 * 0.43).round() as isize;
            canvas.set(px, py, '-', theme.c(3).scale(0.8));
        }
        // blips
        for (i, (a0, r)) in [(0.7, 5.0f32), (2.1, 3.5), (4.3, 6.0), (5.5, 2.5)]
            .iter()
            .enumerate()
        {
            let a = a0 + t * 0.05;
            let px = cx + (a.cos() * r).round() as isize;
            let py = cy + (a.sin() * r * 0.43).round() as isize;
            canvas.set(px, py, if i % 2 == 0 { '*' } else { 'o' }, theme.c(3));
        }
        canvas.put_str(x, cy + 4, "DSN 70M", theme.c(1).scale(1.8));
    }
}

fn logo_kind(w: usize, h: usize) -> LogoKind {
    if w >= 96 && h >= 30 {
        LogoKind::Full
    } else if w >= 70 && h >= 24 {
        LogoKind::Compact
    } else {
        LogoKind::Tiny
    }
}

fn full_logo_sy(h: usize) -> isize {
    if h >= 44 {
        2
    } else {
        1
    }
}

fn logo_rect(w: usize, h: usize) -> Logo {
    let kind = logo_kind(w, h);
    match kind {
        LogoKind::Full => {
            let sx = if w >= 140 { 3 } else { 2 };
            let sy = full_logo_sy(h);
            let lw = 23 * sx;
            let lh = 5 * sy;
            Logo {
                x: (w as isize - lw) / 2,
                y: 2,
                w: lw,
                h: lh,
                sx,
                sy,
                kind,
            }
        }
        LogoKind::Compact => {
            let sx = 2;
            let lw = 23 * sx;
            Logo {
                x: (w as isize - lw) / 2,
                y: 1,
                w: lw,
                h: 3,
                sx,
                sy: 1,
                kind,
            }
        }
        LogoKind::Tiny => {
            let text = "N A S A";
            let lw = text.len() as isize * 2;
            Logo {
                x: (w as isize - lw) / 2,
                y: 1,
                w: lw,
                h: 1,
                sx: 2,
                sy: 1,
                kind,
            }
        }
    }
}

fn panel_rect(w: usize, h: usize) -> Option<Panel> {
    if w < 70 || h < 22 {
        return None;
    }
    let kind = logo_kind(w, h);
    let logo_h = match kind {
        LogoKind::Full => 5 * full_logo_sy(h),
        LogoKind::Compact => 3,
        LogoKind::Tiny => 1,
    };
    let y = match kind {
        LogoKind::Full => 2 + logo_h + 1,
        LogoKind::Compact => 4,
        LogoKind::Tiny => 3,
    };
    let bw = (w as isize - 10).min(84);
    let bh = (h as isize - y - 2).clamp(14, 20);
    if bw < 60 || bh < 14 {
        return None;
    }
    let x = (w as isize - bw) / 2;
    let left = (bw * 62 / 100).max(34);
    Some(Panel {
        x,
        y,
        w: bw,
        h: bh,
        left,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::EffectId;

    #[test]
    fn panel_fits() {
        for (w, h) in [(80, 24), (120, 36), (160, 48)] {
            if let Some(p) = panel_rect(w, h) {
                assert!(p.x >= 0 && p.y >= 0);
                assert!(p.x + p.w <= w as isize);
                assert!(p.y + p.h <= h as isize);
            }
        }
    }

    #[test]
    fn scene_has_horizontal_content() {
        let id = EffectId::Matrix;
        let mut scene = crate::scene::build(id, 7);
        let (w, h) = (120, 36);
        scene.resize(w, h);
        let theme = id.default_theme();
        let mut canvas = Canvas::new(w, h);
        for step in 0..220 {
            let t = step as f64 / 30.0;
            let frame = Frame {
                t,
                dt: 1.0 / 30.0,
                w,
                h,
                fps: 30,
                theme,
                charset: id.default_charset(),
                seed: 7,
                speed: 1.0,
            };
            scene.update(&frame);
            scene.draw(&mut canvas, &frame);
        }
        assert!(canvas.ink() > 1500);
    }
}
