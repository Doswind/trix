//! vibe — a Claude Code session that types its own source file.
//!
//! The scene reads `src/scenes/vibe.rs` from the working tree when it is
//! available (so edits show up on the next turn) and falls back to a
//! compile-time copy when the binary is installed without sources.  The source
//! is normalised to ASCII, split into short windows and typed character by
//! character with a small hand-written Rust syntax highlighter.
//!
//! The chrome follows Claude Code's transcript vocabulary: a prompt, assistant
//! and tool lines, a tree result, a rotating spinner with token count, an input
//! box and the bypass-permissions footer.  ASCII is the default; `--unicode`
//! swaps in the Claude Code-style `❯ ◉ └ ✻ ▶▶` glyphs.

use std::fmt::Write as _;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::scene::{Frame, Scene};

const PROMPTS: &[&str] = &[
    "Add a new scene that types its own source.",
    "Make the banner glow without losing the ASCII fallback.",
    "Add a test for the reserved HUD rows.",
    "Refactor the highlighter and loop the source file.",
    "Run the checks and fix the warnings.",
];

const VERBS: &[&str] = &[
    "Vibing",
    "Cerebrating",
    "Ionizing",
    "Cogitating",
    "Pondering",
    "Noodling",
    "Percolating",
    "Simmering",
    "Brewing",
    "Churning",
    "Crunching",
    "Working",
];

const PLACEHOLDERS: &[&str] = &[
    "Type a message...",
    "? for shortcuts",
    "Try \"explain this scene\"",
];

const SPINNERS_ASCII: &[char] = &['*', '+', 'x', 'o'];
const SPINNERS_UNICODE: &[char] = &['✻', '✽', '✢', '✳', '✶'];

const CODE_CPS: f32 = 105.0;
const PROSE_CPS: f32 = 78.0;
const MAX_LINES: usize = 900;

#[derive(Clone, Copy)]
struct Palette {
    text: Rgb,
    dim: Rgb,
    keyword: Rgb,
    ty: Rgb,
    string: Rgb,
    number: Rgb,
    mac: Rgb,
    comment: Rgb,
    func: Rgb,
    accent: Rgb,
    add: Rgb,
    success: Rgb,
}

const PALETTE: Palette = Palette {
    text: Rgb::hex(0xd8d3c8),
    dim: Rgb::hex(0x6b6b6b),
    keyword: Rgb::hex(0xc792ea),
    ty: Rgb::hex(0x82aaff),
    string: Rgb::hex(0xe5c07b),
    number: Rgb::hex(0xf78c6c),
    mac: Rgb::hex(0x89ddff),
    comment: Rgb::hex(0x5c6773),
    func: Rgb::hex(0xffd580),
    accent: Rgb::hex(0xd97757),
    add: Rgb::hex(0x98c379),
    success: Rgb::hex(0x7fb069),
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Prompt,
    Assistant,
    Tool,
    Result,
    Code,
    Status,
    Blank,
}

#[derive(Clone, Copy)]
struct StyledChar {
    ch: char,
    fg: Rgb,
}

struct ScriptLine {
    kind: Kind,
    prefix: Vec<StyledChar>,
    body: Vec<StyledChar>,
    cps: f32,
    pause: f32,
}

struct Source {
    lines: Vec<String>,
    path: &'static str,
}

pub struct VibeScene {
    w: usize,
    h: usize,
    source: Source,
    lines: Vec<ScriptLine>,
    current: usize,
    visible: f32,
    pause_left: f32,
    scroll: usize,
    turn_start: f64,
    turn_start_line: usize,
    turn_len: usize,
    total_chars: usize,
    prompt_idx: usize,
    window_idx: usize,
    cwd: String,
    branch: String,
    charset: Option<Charset>,
    // layout
    compact: bool,
    content_top: isize,
    content_bottom: isize,
    spinner_y: isize,
    input_top: isize,
    footer_y: isize,
    max_code_w: usize,
}

impl VibeScene {
    pub fn new(_seed: u64) -> Self {
        VibeScene {
            w: 0,
            h: 0,
            source: load_source(),
            lines: Vec::new(),
            current: 0,
            visible: 0.0,
            pause_left: 0.0,
            scroll: 0,
            turn_start: 0.0,
            turn_start_line: 0,
            turn_len: 0,
            total_chars: 0,
            prompt_idx: 0,
            window_idx: 0,
            cwd: current_cwd(),
            branch: git_branch(),
            charset: None,
            compact: false,
            content_top: 3,
            content_bottom: 3,
            spinner_y: 3,
            input_top: 3,
            footer_y: 3,
            max_code_w: 0,
        }
    }

    fn layout(&mut self) {
        let w = self.w as isize;
        let h = self.h as isize;
        self.compact = w < 50 || h < 18;
        if self.compact {
            self.content_top = 2;
            self.spinner_y = (h - 3).max(2);
            self.content_bottom = (self.spinner_y - 1).max(self.content_top);
            self.input_top = 0;
            self.footer_y = 0;
        } else {
            self.footer_y = h - 3;
            let input_bottom = self.footer_y - 1;
            self.input_top = input_bottom - 2;
            self.spinner_y = self.input_top - 1;
            self.content_top = 3;
            self.content_bottom = (self.spinner_y - 1).max(self.content_top);
        }
        self.max_code_w = self.w.saturating_sub(2);
    }

    fn content_h(&self) -> usize {
        (self.content_bottom - self.content_top + 1).max(1) as usize
    }

    fn refresh_source(&mut self) {
        let fresh = load_source();
        if fresh.lines.len() >= 20 {
            self.source = fresh;
        }
    }

    fn build_turn(&mut self, t: f64, charset: Charset) {
        self.refresh_source();
        self.charset = Some(charset);
        self.turn_start = t;
        self.turn_start_line = self.lines.len();
        let unicode = charset == Charset::Unicode;
        let prompt = PROMPTS[self.prompt_idx % PROMPTS.len()];
        let prompt = prompt.to_string();
        self.prompt_idx = self.prompt_idx.wrapping_add(1);

        self.push_prose(
            Kind::Prompt,
            prompt_prefix(unicode),
            PALETTE.accent,
            &prompt,
            PALETTE.text,
            PROSE_CPS,
            0.45,
        );
        self.push_blank();
        self.push_prose(
            Kind::Assistant,
            assistant_prefix(unicode),
            PALETTE.accent,
            "I'll read the scene source and wire it into the registry.",
            PALETTE.text,
            PROSE_CPS,
            0.45,
        );
        let path = self.source.path;
        self.push_tool("Read", path, unicode);
        self.push_result(&format!("Read {} lines", self.source.lines.len()), unicode);
        self.push_blank();

        let windows = 6;
        let win_len = 10;
        let usable = self.source.lines.len().max(1);
        for w in 0..windows {
            let start = ((w * usable / windows) + self.window_idx * 7) % usable;
            self.push_tool("Update", path, unicode);
            let mut count = 0;
            for j in 0..win_len {
                let idx = (start + j) % usable;
                let text = self.source.lines[idx].clone();
                self.push_code(idx + 1, &text);
                count += 1;
            }
            self.push_result(&format!("Updated {path} (+{count} lines)"), unicode);
            if w % 2 == 1 {
                self.push_tool("Bash", "cargo check", unicode);
                self.push_result("Finished dev profile [unoptimized] in 0.87s", unicode);
            }
            self.push_blank();
        }

        let done = if unicode {
            "Worked for 38s · done"
        } else {
            "Worked for 38s - done"
        };
        self.push_prose(
            Kind::Status,
            status_prefix(unicode),
            PALETTE.success,
            done,
            PALETTE.success,
            0.0,
            2.6,
        );

        self.window_idx = self.window_idx.wrapping_add(1);
        self.turn_len = self.lines.len() - self.turn_start_line;
        self.current = self.turn_start_line;
        self.visible = 0.0;
        self.pause_left = 0.0;
        self.scroll = self
            .current
            .saturating_sub(self.content_h().saturating_sub(1));
        self.cap_lines();
    }

    #[allow(clippy::too_many_arguments)]
    fn push_prose(
        &mut self,
        kind: Kind,
        prefix: &str,
        prefix_color: Rgb,
        body: &str,
        body_color: Rgb,
        cps: f32,
        pause: f32,
    ) {
        let prefix_len = prefix.chars().count();
        let max = self.max_code_w.saturating_sub(prefix_len);
        let body = truncate_chars(body, max);
        self.lines.push(ScriptLine {
            kind,
            prefix: styled(prefix, prefix_color),
            body: styled(&body, body_color),
            cps,
            pause,
        });
    }

    fn push_blank(&mut self) {
        self.lines.push(ScriptLine {
            kind: Kind::Blank,
            prefix: Vec::new(),
            body: Vec::new(),
            cps: 0.0,
            pause: 0.12,
        });
    }

    fn push_tool(&mut self, name: &str, arg: &str, unicode: bool) {
        let mut body = Vec::new();
        for ch in name.chars() {
            body.push(StyledChar {
                ch,
                fg: PALETTE.func,
            });
        }
        body.push(StyledChar {
            ch: '(',
            fg: PALETTE.dim,
        });
        let arg_color = if arg.contains("cargo") {
            PALETTE.string
        } else {
            PALETTE.ty
        };
        for ch in arg.chars() {
            body.push(StyledChar { ch, fg: arg_color });
        }
        body.push(StyledChar {
            ch: ')',
            fg: PALETTE.dim,
        });
        let body = truncate_styled(body, self.max_code_w.saturating_sub(2));
        self.lines.push(ScriptLine {
            kind: Kind::Tool,
            prefix: styled(tool_prefix(unicode), PALETTE.accent),
            body,
            cps: 0.0,
            pause: 0.30,
        });
    }

    fn push_result(&mut self, text: &str, unicode: bool) {
        let prefix = result_prefix(unicode);
        let max = self.max_code_w.saturating_sub(prefix.chars().count());
        let body = truncate_chars(text, max);
        self.lines.push(ScriptLine {
            kind: Kind::Result,
            prefix: styled(prefix, PALETTE.dim),
            body: styled(&body, PALETTE.dim),
            cps: 0.0,
            pause: 0.18,
        });
    }

    fn push_code(&mut self, line_no: usize, text: &str) {
        let mut gutter = String::new();
        let _ = write!(&mut gutter, "{line_no:>4} ");
        let mut prefix = styled(&gutter, PALETTE.dim);
        prefix.push(StyledChar {
            ch: '+',
            fg: PALETTE.add,
        });
        prefix.push(StyledChar {
            ch: ' ',
            fg: PALETTE.dim,
        });
        let max = self.max_code_w.saturating_sub(prefix.len());
        let text = truncate_chars(text, max);
        self.lines.push(ScriptLine {
            kind: Kind::Code,
            prefix,
            body: highlight_rust(&text),
            cps: CODE_CPS,
            pause: 0.035,
        });
    }

    fn cap_lines(&mut self) {
        if self.lines.len() <= MAX_LINES {
            return;
        }
        let drop = self.lines.len() - MAX_LINES;
        self.lines.drain(0..drop);
        self.current = self.current.saturating_sub(drop);
        self.scroll = self.scroll.saturating_sub(drop);
        self.turn_start_line = self.turn_start_line.saturating_sub(drop);
    }

    fn draw_header(&self, canvas: &mut Canvas, unicode: bool) {
        let w = self.w as isize;
        if w < 8 {
            return;
        }
        let title = if unicode {
            "✻ Claude Code"
        } else {
            "* Claude Code"
        };
        canvas.put_str(2, 1, title, PALETTE.accent);
        let mut x = 2 + title.chars().count() as isize + 2;
        let branch_w = if self.branch.is_empty() {
            0
        } else {
            self.branch.chars().count() as isize + 2
        };
        let room = w - 2 - x - branch_w;
        if room > 6 {
            let cwd = format!("~/{}", self.cwd);
            let shown = truncate_chars(&cwd, room as usize);
            canvas.put_str(x, 1, &shown, PALETTE.dim);
            x += shown.chars().count() as isize;
        }
        if !self.branch.is_empty() {
            let bx = w - 2 - self.branch.chars().count() as isize;
            if bx > x + 2 {
                canvas.put_str(bx, 1, &self.branch, PALETTE.dim.scale(0.8));
            }
        }
    }

    fn draw_transcript(&mut self, canvas: &mut Canvas, f: &Frame, unicode: bool) {
        let view = self.content_h();
        let focus = self.current.min(self.lines.len().saturating_sub(1));
        if focus < self.scroll {
            self.scroll = focus;
        }
        if focus >= self.scroll + view {
            self.scroll = (focus + 1).saturating_sub(view);
        }
        let max_scroll = self.lines.len().saturating_sub(view);
        let start = self.scroll.min(max_scroll).min(focus);
        // Only the lines that have started typing are visible.  Everything
        // below the typing head stays blank until the cursor gets there.
        let last = (self.current + 1).min(self.lines.len());
        let t = f.t as f32;
        for (row, idx) in (start..last).take(view).enumerate() {
            let y = self.content_top + row as isize;
            let line = &self.lines[idx];
            let active = idx == self.current;
            let visible = if active {
                self.visible.floor().max(0.0) as usize
            } else {
                line.body.len()
            };
            draw_line(canvas, 1, y, line, visible, active, t, unicode);
        }
    }

    fn draw_spinner(&self, canvas: &mut Canvas, f: &Frame, unicode: bool) {
        if self.spinner_y < 1 || self.spinner_y > self.h as isize - 3 {
            return;
        }
        let done = self.current >= self.lines.len()
            || self
                .lines
                .get(self.current)
                .is_some_and(|line| line.kind == Kind::Status);
        let text = if done {
            if unicode {
                "✻ Worked for 38s · done".to_string()
            } else {
                "* Worked for 38s - done".to_string()
            }
        } else {
            let elapsed = (f.t - self.turn_start).max(0.0) as u32;
            let tokens = self.total_chars / 4;
            let w = self.w.max(1);
            let glyph = if unicode {
                SPINNERS_UNICODE[(f.t * 10.0) as usize % SPINNERS_UNICODE.len()]
            } else {
                SPINNERS_ASCII[(f.t * 10.0) as usize % SPINNERS_ASCII.len()]
            };
            let verb = VERBS[(elapsed as usize / 4) % VERBS.len()];
            let mut cmd = String::new();
            if unicode {
                let _ = write!(
                    &mut cmd,
                    "{glyph} {verb}… ({elapsed}s · ↓ {tokens} tokens · esc to interrupt)"
                );
            } else {
                let _ = write!(
                    &mut cmd,
                    "{glyph} {verb}... ({elapsed}s . {tokens} tokens . esc to interrupt)"
                );
            }
            truncate_chars(&cmd, w.saturating_sub(4))
        };
        let color = if done {
            PALETTE.success
        } else {
            PALETTE.accent
        };
        canvas.put_str(2, self.spinner_y, &text, color.scale(0.9));
    }

    fn draw_input(&self, canvas: &mut Canvas, f: &Frame, unicode: bool) {
        if self.input_top < 3 || self.input_top + 2 > self.h as isize - 4 {
            return;
        }
        let bx = 2;
        let bw = self.w as isize - 4;
        if bw < 8 {
            return;
        }
        let border = PALETTE.dim.scale(0.75);
        let top = self.input_top;
        let bottom = top + 2;
        let (hchar, vchar) = if unicode { ('─', '│') } else { ('-', '|') };
        canvas.hline(bx + 1, bx + bw - 2, top, hchar, border);
        canvas.hline(bx + 1, bx + bw - 2, bottom, hchar, border);
        canvas.vline(bx, top + 1, bottom - 1, vchar, border);
        canvas.vline(bx + bw - 1, top + 1, bottom - 1, vchar, border);
        let (tl, tr, bl, br) = if unicode {
            ('╭', '╮', '╰', '╯')
        } else {
            ('+', '+', '+', '+')
        };
        canvas.set(bx, top, tl, border);
        canvas.set(bx + bw - 1, top, tr, border);
        canvas.set(bx, bottom, bl, border);
        canvas.set(bx + bw - 1, bottom, br, border);

        let y = top + 1;
        let prompt = if unicode { "❯" } else { ">" };
        canvas.put_str(bx + 2, y, prompt, PALETTE.accent);
        if (f.t as f32 * 2.0).rem_euclid(1.0) < 0.55 {
            canvas.set(bx + 4, y, if unicode { '█' } else { '#' }, PALETTE.accent);
        }
        let ph = PLACEHOLDERS[(f.t as usize / 6) % PLACEHOLDERS.len()];
        let px = bx + 6;
        if px + ph.chars().count() as isize <= bx + bw - 2 {
            canvas.put_str(px, y, ph, PALETTE.dim.scale(0.65));
        }
    }

    fn draw_footer(&self, canvas: &mut Canvas, unicode: bool) {
        if self.footer_y < 1 || self.footer_y > self.h as isize - 3 {
            return;
        }
        let text = if unicode {
            "▶▶ bypass permissions on (shift+tab to cycle) · esc to interrupt"
        } else {
            ">> bypass permissions on (shift+tab to cycle) - esc to interrupt"
        };
        let shown = truncate_chars(text, self.w.saturating_sub(6));
        canvas.put_str(2, self.footer_y, &shown, PALETTE.dim.scale(0.6));
        let ctx = "Context left 87%";
        let ctx_w = ctx.chars().count() as isize;
        let right = self.w as isize - 2 - ctx_w;
        if right > 2 + shown.chars().count() as isize + 2 {
            canvas.put_str(right, self.footer_y, ctx, PALETTE.dim.scale(0.5));
        }
    }
}

impl Scene for VibeScene {
    fn resize(&mut self, w: usize, h: usize) {
        let w = w.max(1);
        let h = h.max(1);
        if w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        self.layout();
        self.lines.clear();
        self.current = 0;
        self.visible = 0.0;
        self.pause_left = 0.0;
        self.scroll = 0;
        self.turn_start = 0.0;
        self.turn_start_line = 0;
        self.turn_len = 0;
        self.charset = None;
    }

    fn update(&mut self, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let dt = f.dt.clamp(0.0, 0.1);
        if self.charset != Some(f.charset) {
            self.charset = Some(f.charset);
            self.lines.clear();
            self.current = 0;
            self.visible = 0.0;
            self.scroll = 0;
            self.pause_left = 0.0;
            self.turn_start_line = 0;
            self.turn_len = 0;
        }
        if self.lines.is_empty() || self.current >= self.lines.len() {
            if self.pause_left > 0.0 {
                self.pause_left -= dt;
                return;
            }
            // A new loop starts from a clean transcript: clear the previous
            // session before the next prompt starts typing.
            if !self.lines.is_empty() {
                self.lines.clear();
                self.current = 0;
                self.visible = 0.0;
                self.scroll = 0;
                self.turn_start_line = 0;
                self.turn_len = 0;
                self.total_chars = 0;
            }
            self.build_turn(f.t, f.charset);
            return;
        }
        if self.pause_left > 0.0 {
            self.pause_left -= dt;
            return;
        }
        let cps = self.lines[self.current].cps;
        let len = self.lines[self.current].body.len();
        if cps <= 0.0 {
            self.visible = len as f32;
        } else {
            self.visible += cps * dt;
        }
        if self.visible >= len as f32 {
            self.visible = len as f32;
            self.total_chars += len;
            self.pause_left = self.lines[self.current].pause;
            self.current += 1;
            self.visible = 0.0;
            self.scroll = self
                .current
                .saturating_sub(self.content_h().saturating_sub(1));
            self.cap_lines();
        }
    }

    fn hud_status(&self, _f: &Frame) -> Option<String> {
        let done = self.current.saturating_sub(self.turn_start_line);
        let pct = (done * 100)
            .checked_div(self.turn_len)
            .unwrap_or(0)
            .min(100);
        let tokens = self.total_chars / 4;
        let label = if tokens >= 1000 {
            format!("{:.1}k", tokens as f32 / 1000.0)
        } else {
            tokens.to_string()
        };
        Some(format!("EDIT {pct:02}% | {label} tok"))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let unicode = f.charset == Charset::Unicode;
        self.draw_header(canvas, unicode);
        if !self.compact {
            canvas.hline(
                1,
                self.w as isize - 2,
                2,
                if unicode { '─' } else { '-' },
                PALETTE.dim.scale(0.45),
            );
        }
        self.draw_transcript(canvas, f, unicode);
        self.draw_spinner(canvas, f, unicode);
        if !self.compact {
            self.draw_input(canvas, f, unicode);
            self.draw_footer(canvas, unicode);
        }
    }
}

// ---------------------------------------------------------------------------
// Claude Code prefixes.
// ---------------------------------------------------------------------------

fn prompt_prefix(unicode: bool) -> &'static str {
    if unicode {
        "❯ "
    } else {
        "> "
    }
}

fn assistant_prefix(unicode: bool) -> &'static str {
    if unicode {
        "● "
    } else {
        "* "
    }
}

fn tool_prefix(unicode: bool) -> &'static str {
    if unicode {
        "◉ "
    } else {
        "* "
    }
}

fn result_prefix(unicode: bool) -> &'static str {
    if unicode {
        "  └ "
    } else {
        "  L "
    }
}

fn status_prefix(unicode: bool) -> &'static str {
    if unicode {
        "✻ "
    } else {
        "* "
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_line(
    canvas: &mut Canvas,
    x: isize,
    y: isize,
    line: &ScriptLine,
    visible: usize,
    active: bool,
    t: f32,
    unicode: bool,
) {
    let mut cx = x;
    let dim = if active { 1.0 } else { 0.88 };
    for sc in &line.prefix {
        canvas.set(cx, y, sc.ch, sc.fg.scale(dim));
        cx += 1;
    }
    for sc in line.body.iter().take(visible) {
        canvas.set(cx, y, sc.ch, sc.fg.scale(dim));
        cx += 1;
    }
    if active && (t * 2.0).rem_euclid(1.0) < 0.55 {
        canvas.set(cx, y, if unicode { '█' } else { '#' }, PALETTE.accent);
    }
}

// ---------------------------------------------------------------------------
// Styling helpers.
// ---------------------------------------------------------------------------

fn styled(text: &str, fg: Rgb) -> Vec<StyledChar> {
    text.chars()
        .map(|ch| StyledChar { ch, fg })
        .collect::<Vec<_>>()
}

fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    if max <= 2 {
        return text.chars().take(max).collect();
    }
    let mut out: String = text.chars().take(max - 1).collect();
    out.push('>');
    out
}

fn truncate_styled(mut text: Vec<StyledChar>, max: usize) -> Vec<StyledChar> {
    if text.len() <= max {
        text
    } else {
        text.truncate(max.saturating_sub(1));
        text.push(StyledChar {
            ch: '>',
            fg: PALETTE.dim,
        });
        text
    }
}

fn sanitize_ascii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        let mapped = match ch {
            '—' | '–' | '−' => '-',
            '“' | '”' => '"',
            '‘' | '’' => '\'',
            '…' => '.',
            '·' => '.',
            '，' => ',',
            '。' => '.',
            '：' => ':',
            '；' => ';',
            '！' => '!',
            '？' => '?',
            '（' => '(',
            '）' => ')',
            '【' => '[',
            '】' => ']',
            '《' => '<',
            '》' => '>',
            '、' => ',',
            '～' => '~',
            '\t' => ' ',
            c if c.is_ascii() => c,
            _ => '?',
        };
        out.push(mapped);
    }
    out
}

fn prepare_lines(text: &str) -> Vec<String> {
    let all: Vec<String> = text
        .lines()
        .map(|raw| {
            let expanded = raw.replace('\t', "    ");
            let ascii = sanitize_ascii(&expanded);
            ascii
                .chars()
                .take(400)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect();
    let mut main = Vec::new();
    let mut in_test = false;
    for line in &all {
        if line.trim_start().starts_with("#[cfg(test)]") {
            in_test = true;
        }
        if !in_test {
            main.push(line.clone());
        }
    }
    let chosen = if main.len() >= 24 { main } else { all };
    let mut out: Vec<String> = Vec::new();
    for line in chosen {
        if line.is_empty() && out.last().is_some_and(|l| l.is_empty()) {
            continue;
        }
        out.push(line);
    }
    while out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    if out.is_empty() {
        out.push("// source unavailable".to_string());
    }
    out
}

fn load_source() -> Source {
    for path in [
        "src/scenes/vibe.rs",
        "../src/scenes/vibe.rs",
        "../../src/scenes/vibe.rs",
    ] {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > 256 * 1024 {
                continue;
            }
        }
        if let Ok(text) = std::fs::read_to_string(path) {
            let lines = prepare_lines(&text);
            if lines.len() >= 12 {
                return Source { lines, path };
            }
        }
    }
    Source {
        lines: prepare_lines(include_str!("vibe.rs")),
        path: "src/scenes/vibe.rs",
    }
}

fn current_cwd() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .map(|s| sanitize_ascii(&s))
        .unwrap_or_else(|| "trix-effects".to_string())
}

fn git_branch() -> String {
    for path in [".git/HEAD", "../.git/HEAD", "../../.git/HEAD"] {
        if let Ok(text) = std::fs::read_to_string(path) {
            let text = text.trim();
            if let Some(rest) = text.strip_prefix("ref: refs/heads/") {
                return sanitize_ascii(rest);
            }
            if text.len() >= 7 {
                return sanitize_ascii(&text.chars().take(7).collect::<String>());
            }
        }
    }
    String::new()
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use",
    "where", "while", "yield",
];

fn highlight_rust(text: &str) -> Vec<StyledChar> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<StyledChar> = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            for &c in &chars[i..] {
                out.push(StyledChar {
                    ch: c,
                    fg: PALETTE.comment,
                });
            }
            break;
        }
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let mut j = i + 2;
            while j < chars.len() {
                if chars[j] == '*' && j + 1 < chars.len() && chars[j + 1] == '/' {
                    j += 2;
                    break;
                }
                j += 1;
            }
            for &c in &chars[i..j] {
                out.push(StyledChar {
                    ch: c,
                    fg: PALETTE.comment,
                });
            }
            i = j;
            continue;
        }
        if ch == '"' {
            let start = i;
            i += 1;
            while i < chars.len() {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let end = i.min(chars.len());
            for &c in &chars[start..end] {
                out.push(StyledChar {
                    ch: c,
                    fg: PALETTE.string,
                });
            }
            continue;
        }
        if ch == '\'' {
            if i + 2 < chars.len() && chars[i + 2] == '\'' {
                for &c in &chars[i..i + 3] {
                    out.push(StyledChar {
                        ch: c,
                        fg: PALETTE.string,
                    });
                }
                i += 3;
                continue;
            }
            if i + 3 < chars.len() && chars[i + 1] == '\\' && chars[i + 3] == '\'' {
                for &c in &chars[i..i + 4] {
                    out.push(StyledChar {
                        ch: c,
                        fg: PALETTE.string,
                    });
                }
                i += 4;
                continue;
            }
            out.push(StyledChar {
                ch: '\'',
                fg: PALETTE.ty,
            });
            i += 1;
            while i < chars.len() && is_ident_char(chars[i]) {
                out.push(StyledChar {
                    ch: chars[i],
                    fg: PALETTE.ty,
                });
                i += 1;
            }
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
            {
                i += 1;
            }
            for &c in &chars[start..i] {
                out.push(StyledChar {
                    ch: c,
                    fg: PALETTE.number,
                });
            }
            continue;
        }
        if is_ident_start(ch) {
            let start = i;
            while i < chars.len() && is_ident_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let fg = if KEYWORDS.contains(&word.as_str()) {
                PALETTE.keyword
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                PALETTE.ty
            } else if i < chars.len() && chars[i] == '!' {
                PALETTE.mac
            } else if i < chars.len() && chars[i] == '(' {
                PALETTE.func
            } else {
                PALETTE.text
            };
            for &c in &chars[start..i] {
                out.push(StyledChar { ch: c, fg });
            }
            continue;
        }
        out.push(StyledChar {
            ch,
            fg: PALETTE.dim,
        });
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_normalisation_strips_cjk_and_tests() {
        let mut text = String::new();
        for _ in 0..30 {
            text.push_str("fn filler() {}\n");
        }
        text.push_str("\tlet x = 1; // 中文\n#[cfg(test)]\nmod tests {}\n");
        let lines = prepare_lines(&text);
        assert!(lines.iter().all(|line| line.is_ascii()));
        assert!(lines.iter().all(|line| !line.contains("mod tests")));
        assert!(lines.iter().any(|line| line.contains("    let x = 1;")));
    }

    #[test]
    fn highlighter_marks_comments_strings_keywords_and_numbers() {
        let line = highlight_rust("let s = \"hi\"; let n = 42; // note");
        let color_of = |target: char| {
            line.iter()
                .find(|sc| sc.ch == target)
                .map(|sc| sc.fg)
                .unwrap()
        };
        assert_eq!(color_of('l'), PALETTE.keyword);
        assert_eq!(color_of('"'), PALETTE.string);
        assert_eq!(color_of('/'), PALETTE.comment);
        assert_eq!(color_of('4'), PALETTE.number);
    }

    #[test]
    fn future_lines_stay_blank_until_typed() {
        let mut scene = VibeScene::new(7);
        scene.resize(120, 36);
        let mut canvas = Canvas::new(120, 36);
        for i in 0..3 {
            let frame = Frame {
                t: i as f64 / 30.0,
                dt: 1.0 / 30.0,
                w: 120,
                h: 36,
                fps: 30,
                theme: crate::theme::find("gold").unwrap(),
                charset: Charset::Ascii,
                seed: 7,
                speed: 1.0,
                tune: crate::scene::Tune::default(),
            };
            scene.update(&frame);
            scene.draw(&mut canvas, &frame);
        }
        let plain = canvas.render_plain();
        assert!(plain.contains("Add a"), "prompt should be typing");
        assert!(
            !plain.contains("I'll read the scene source"),
            "future transcript lines must stay blank:\n{plain}"
        );
    }

    #[test]
    fn loop_restart_clears_the_previous_transcript() {
        let mut scene = VibeScene::new(7);
        scene.resize(120, 36);
        scene.build_turn(0.0, Charset::Ascii);
        let first_len = scene.lines.len();
        assert!(first_len > 10);
        scene.current = scene.lines.len();
        scene.pause_left = 0.0;
        let frame = Frame {
            t: 60.0,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: crate::theme::find("gold").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        assert_eq!(scene.current, 0);
        assert_eq!(scene.turn_start_line, 0);
        assert_eq!(scene.total_chars, 0);
        assert!(scene.lines.len() < first_len * 2, "old transcript was kept");
    }

    #[test]
    fn script_contains_code_and_loops() {
        let mut scene = VibeScene::new(7);
        scene.resize(120, 36);
        let frame = Frame {
            t: 0.0,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: crate::theme::find("gold").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        assert!(!scene.lines.is_empty());
        assert!(scene.lines.iter().any(|line| line.kind == Kind::Code));
        assert!(scene.turn_len > 10);
    }

    #[test]
    fn default_ascii_output_is_ascii_only() {
        let mut scene = VibeScene::new(7);
        scene.resize(80, 24);
        let mut canvas = Canvas::new(80, 24);
        for step in 0..30 {
            let frame = Frame {
                t: step as f64 / 30.0,
                dt: 1.0 / 30.0,
                w: 80,
                h: 24,
                fps: 30,
                theme: crate::theme::find("gold").unwrap(),
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

    #[test]
    fn animation_keeps_writing_diff_cells() {
        let mut scene = VibeScene::new(7);
        scene.resize(120, 36);
        let mut canvas = Canvas::new(120, 36);
        let mut sink = Vec::new();
        let mut total = 0usize;
        for i in 0..60 {
            let frame = Frame {
                t: i as f64 / 30.0,
                dt: 1.0 / 30.0,
                w: 120,
                h: 36,
                fps: 30,
                theme: crate::theme::find("gold").unwrap(),
                charset: Charset::Ascii,
                seed: 7,
                speed: 1.0,
                tune: crate::scene::Tune::default(),
            };
            scene.update(&frame);
            canvas.clear();
            scene.draw(&mut canvas, &frame);
            sink.clear();
            total += canvas
                .present(&mut sink, crate::color::ColorMode::True)
                .unwrap();
        }
        assert!(total > 200, "diff output was only {total} bytes");
    }

    #[test]
    fn tiny_layout_draws_without_panicking() {
        let mut scene = VibeScene::new(7);
        scene.resize(20, 8);
        let mut canvas = Canvas::new(20, 8);
        let frame = Frame {
            t: 3.0,
            dt: 1.0 / 30.0,
            w: 20,
            h: 8,
            fps: 30,
            theme: crate::theme::find("gold").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        };
        scene.update(&frame);
        scene.draw(&mut canvas, &frame);
        assert!(canvas.ink() > 0);
    }
}
