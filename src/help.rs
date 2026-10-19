//! The styled command help: overview, per-effect dossiers, catalog and
//! diagnostics.
//!
//! This is deliberately hand-rolled: the binary has no CLI or layout
//! dependency, and the help should look like the rest of the product instead
//! of a generic `--help` dump.  Every renderer degrades to plain, aligned
//! ASCII when stdout is not a TTY or colour is disabled.

use std::io::Write;

use crate::canvas::Canvas;
use crate::cli::{self, Charset, Options};
use crate::color::{self, ColorMode, Rgb};
use crate::hud;
use crate::meta::{Category, CharsetSupport, Tuning};
use crate::scene::{self, EffectId, Frame, Tune};
use crate::term;
use crate::theme::{self, Theme};

const TEXT: Rgb = Rgb::hex(0xcbd5e1);
const DIM: Rgb = Rgb::hex(0x94a3b8);
const FAINT: Rgb = Rgb::hex(0x475569);
const KEY: Rgb = Rgb::hex(0x67e8f9);
const HOT: Rgb = Rgb::hex(0xf0abfc);
const OK: Rgb = Rgb::hex(0x22c55e);
const ERR: Rgb = Rgb::hex(0xef4444);

const WORDMARK: [&str; 5] = [
    r" _____  ____   ___  __  __",
    r"|_   _||  _ \ |_ _| \ \/ /",
    r"  | |  | |_) | | |   \  / ",
    r"  | |  |  _ <  | |   /  \ ",
    r"  |_|  |_| \_\|___| /_/\_\",
];

#[derive(Clone, Copy)]
pub struct HelpStyle {
    pub width: usize,
    pub color: ColorMode,
    pub unicode: bool,
    pub tty: bool,
    pub theme: &'static Theme,
}

impl HelpStyle {
    pub fn detect(opts: &Options) -> Self {
        // `TRIX_HELP_TTY` lets the documentation tools render the full
        // TTY layout (preview included) into a pipe.
        let tty = term::is_tty(1) || std::env::var_os("TRIX_HELP_TTY").is_some();
        let width = term::size().map(|(w, _)| w).unwrap_or(80).clamp(40, 140);
        let color = match opts.color {
            Some(mode) => mode,
            None if tty => ColorMode::detect(),
            None => ColorMode::Mono,
        };
        let unicode = match opts.charset {
            Some(Charset::Unicode) => true,
            Some(Charset::Ascii) | Some(Charset::Kana) => false,
            None => tty && locale_utf8(),
        };
        Self {
            width,
            color,
            unicode,
            tty,
            theme: theme::find("neon").unwrap_or(&theme::THEMES[0]),
        }
    }

    pub fn plain() -> Self {
        Self {
            width: 80,
            color: ColorMode::Mono,
            unicode: false,
            tty: false,
            theme: &theme::THEMES[0],
        }
    }

    fn on(&self) -> bool {
        self.color.is_color()
    }
}

fn locale_utf8() -> bool {
    ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .filter_map(|key| std::env::var(key).ok())
        .any(|value| {
            let value = value.to_ascii_lowercase();
            value.contains("utf-8") || value.contains("utf8")
        })
}

fn paint(text: &str, rgb: Rgb, style: &HelpStyle) -> String {
    if !style.on() || text.is_empty() {
        return text.to_string();
    }
    let mut out = Vec::with_capacity(text.len() + 20);
    color::write_fg(&mut out, rgb, style.color);
    out.extend_from_slice(text.as_bytes());
    out.extend_from_slice(b"\x1b[0m");
    String::from_utf8_lossy(&out).into_owned()
}

fn bold(text: &str, rgb: Rgb, style: &HelpStyle) -> String {
    if !style.on() {
        return text.to_string();
    }
    let mut out = Vec::with_capacity(text.len() + 24);
    out.extend_from_slice(b"\x1b[1m");
    color::write_fg(&mut out, rgb, style.color);
    out.extend_from_slice(text.as_bytes());
    out.extend_from_slice(b"\x1b[0m");
    String::from_utf8_lossy(&out).into_owned()
}

fn dim(text: &str, style: &HelpStyle) -> String {
    paint(text, DIM, style)
}

fn faint(text: &str, style: &HelpStyle) -> String {
    paint(text, FAINT, style)
}

fn char_width(c: char) -> usize {
    let u = c as u32;
    match u {
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1FAFF => 2,
        _ => 1,
    }
}

/// Display width in terminal cells (CJK counts as two).
pub fn display_width(text: &str) -> usize {
    text.chars().map(char_width).sum()
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                for c2 in chars.by_ref() {
                    if ('@'..='~').contains(&c2) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                for c2 in chars.by_ref() {
                    if c2 == '\x07' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn visible_width(text: &str) -> usize {
    display_width(&strip_ansi(text))
}

fn pad(text: &str, width: usize) -> String {
    let w = visible_width(text);
    if w >= width {
        text.to_string()
    } else {
        let mut out = String::with_capacity(text.len() + (width - w));
        out.push_str(text);
        out.push_str(&" ".repeat(width - w));
        out
    }
}

fn truncate_plain(text: &str, width: usize) -> String {
    if display_width(text) <= width {
        return text.to_string();
    }
    if width <= 2 {
        return text.chars().take(width).collect();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = char_width(c);
        if used + w > width - 2 {
            break;
        }
        used += w;
        out.push(c);
    }
    out.push_str("..");
    out
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(12);
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut line = String::new();
        for word in raw.split_whitespace() {
            let w = display_width(word);
            if line.is_empty() {
                if w > width {
                    // Hard split very long words so the box never overflows.
                    let mut chunk = String::new();
                    let mut used = 0;
                    for c in word.chars() {
                        let cw = char_width(c);
                        if used + cw > width {
                            lines.push(std::mem::take(&mut chunk));
                            used = 0;
                        }
                        chunk.push(c);
                        used += cw;
                    }
                    if !chunk.is_empty() {
                        line = chunk;
                    }
                } else {
                    line.push_str(word);
                }
            } else if display_width(&line) + 1 + w <= width {
                line.push(' ');
                line.push_str(word);
            } else {
                lines.push(std::mem::take(&mut line));
                line.push_str(word);
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn section(title: &str, style: &HelpStyle) -> String {
    let rule = if style.unicode { "─" } else { "-" };
    let head = format!("  {}  ", bold(title, style.theme.c(3), style));
    let used = visible_width(&head);
    let remaining = style.width.saturating_sub(used + 2);
    format!("{}{}", head, faint(&rule.repeat(remaining), style))
}

struct Borders {
    tl: char,
    tr: char,
    bl: char,
    br: char,
    h: char,
    v: char,
}

fn borders(style: &HelpStyle) -> Borders {
    if style.unicode {
        Borders {
            tl: '╭',
            tr: '╮',
            bl: '╰',
            br: '╯',
            h: '─',
            v: '│',
        }
    } else {
        Borders {
            tl: '+',
            tr: '+',
            bl: '+',
            br: '+',
            h: '-',
            v: '|',
        }
    }
}

/// Wrap already-painted lines in a box.  `width` includes the borders.
fn boxed(lines: &[String], width: usize, style: &HelpStyle, title: Option<&str>) -> Vec<String> {
    let width = width.max(20);
    let inner = width - 4;
    let b = borders(style);
    let mut out = Vec::with_capacity(lines.len() + 2);
    let top = match title {
        Some(t) => {
            let t = truncate_plain(t, inner.saturating_sub(4));
            let head = format!("{} {} ", b.h, t);
            let used = visible_width(&head);
            let rest = width.saturating_sub(used + 2);
            format!(
                "{}{}{}{}",
                paint(&b.tl.to_string(), FAINT, style),
                paint(&head, style.theme.c(2), style),
                faint(&b.h.to_string().repeat(rest), style),
                paint(&b.tr.to_string(), FAINT, style)
            )
        }
        None => format!(
            "{}{}{}",
            paint(&b.tl.to_string(), FAINT, style),
            faint(&b.h.to_string().repeat(width - 2), style),
            paint(&b.tr.to_string(), FAINT, style)
        ),
    };
    out.push(top);
    for line in lines {
        out.push(format!(
            "{} {}{} {}",
            paint(&b.v.to_string(), FAINT, style),
            pad(line, inner),
            "",
            paint(&b.v.to_string(), FAINT, style)
        ));
    }
    out.push(format!(
        "{}{}{}",
        paint(&b.bl.to_string(), FAINT, style),
        faint(&b.h.to_string().repeat(width - 2), style),
        paint(&b.br.to_string(), FAINT, style)
    ));
    out
}

fn gradient(text: &str, style: &HelpStyle) -> String {
    if !style.on() {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len().max(2) - 1;
    let mut out = String::new();
    for (i, c) in chars.iter().enumerate() {
        let t = i as f32 / n as f32;
        let rgb = style.theme.c(1).lerp(style.theme.c(3), t);
        out.push_str(&paint(&c.to_string(), rgb, style));
    }
    out
}

fn header(style: &HelpStyle) -> Vec<String> {
    let logo = &WORDMARK;
    let info = [
        format!(
            "{}  {}",
            bold("trix-effects", style.theme.c(3), style),
            dim("terminal effects engine", style)
        ),
        format!(
            "{} effects  ·  {} themes  ·  zero dependencies",
            EffectId::ALL.len(),
            theme::THEMES.len()
        ),
        dim("github.com/Doswind/trix-effects", style),
        format!("v{}", cli::VERSION),
    ];
    let logo_w = logo.iter().map(|l| display_width(l)).max().unwrap_or(0);
    let info_w = info.iter().map(|l| visible_width(l)).max().unwrap_or(0);
    let version = format!("v{}", cli::VERSION);
    if style.width >= logo_w + 3 + info_w {
        let mut out = Vec::new();
        for (i, line) in logo.iter().enumerate() {
            let left = pad(&gradient(line, style), logo_w);
            let right = info.get(i).cloned().unwrap_or_default();
            out.push(format!(
                "{}{}",
                left,
                if right.is_empty() {
                    String::new()
                } else {
                    format!("   {}", right)
                }
            ));
        }
        for line in info.iter().skip(logo.len()) {
            out.push(format!("{}{}", " ".repeat(logo_w + 3), line));
        }
        out
    } else {
        let mut out = vec![gradient("T R I X", style)];
        if style.width >= 58 {
            out.push(format!(
                "{}  {}  {}",
                bold("trix-effects", style.theme.c(3), style),
                dim("terminal effects engine", style),
                version
            ));
        } else {
            out.push(format!(
                "{}  {}",
                bold("trix-effects", style.theme.c(3), style),
                version
            ));
            out.push(dim("terminal effects engine", style));
        }
        if style.width >= 48 {
            out.push(format!(
                "{} effects  ·  {} themes  ·  zero dependencies",
                EffectId::ALL.len(),
                theme::THEMES.len()
            ));
        } else {
            out.push(format!(
                "{} effects  ·  {} themes  ·  no deps",
                EffectId::ALL.len(),
                theme::THEMES.len()
            ));
        }
        out.push(dim("github.com/Doswind/trix-effects", style));
        out
    }
}

fn usage(style: &HelpStyle) -> Vec<String> {
    let mut out = vec![section("USAGE", style)];
    let rows = [
        ("trix [effect] [options]", "run one effect"),
        ("trix", "cycle all effects"),
        ("trix --list", "catalog + themes"),
        ("trix --help <effect>", "details + live preview"),
    ];
    let left = rows
        .iter()
        .map(|(l, _)| display_width(l))
        .max()
        .unwrap_or(0);
    for (cmd, desc) in rows {
        if style.width >= 62 {
            out.push(format!(
                "    {}   {}",
                pad(&paint(cmd, KEY, style), left + 2),
                dim(desc, style)
            ));
        } else {
            out.push(format!("    {}", paint(cmd, KEY, style)));
            out.push(format!("      {}", dim(desc, style)));
        }
    }
    out
}

fn preview_with_hud(style: &HelpStyle) -> Vec<String> {
    if !style.tty || style.width < 60 {
        return Vec::new();
    }
    let outer = style.width.clamp(44, 96);
    let w = outer - 4;
    let h = 11usize;
    let id = EffectId::Dragon;
    let theme = id.default_theme();
    let charset = id.default_charset();
    let seed = 0x7e51_a017;
    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let mut canvas = Canvas::new(w, h);
    let dt = 0.1f32;
    let mut t = 0.0f64;
    for _ in 0..50 {
        let frame = Frame {
            t,
            dt,
            w,
            h,
            fps: 10,
            theme,
            charset,
            seed,
            speed: 1.0,
            tune: Tune::default(),
        };
        scene.update(&frame);
        t += dt as f64;
    }
    let frame = Frame {
        t,
        dt,
        w,
        h,
        fps: 10,
        theme,
        charset,
        seed,
        speed: 1.0,
        tune: Tune::default(),
    };
    canvas.clear();
    scene.draw(&mut canvas, &frame);
    let status = scene.hud_status(&frame);
    hud::draw_header(&mut canvas, &frame, id, status.as_deref());
    hud::draw_keybar(&mut canvas, &frame, false);
    hud::draw_playlist(&mut canvas, &frame, id, 12.0, 4.0);
    let rows = canvas.render_rows(style.color);
    boxed(&rows, outer, style, Some("WHAT IT LOOKS LIKE"))
}

fn effect_grid(style: &HelpStyle) -> Vec<String> {
    let cols = if style.width >= 118 {
        3
    } else if style.width >= 72 {
        2
    } else {
        1
    };
    let gap = 2;
    let cell = (style.width.saturating_sub(4) - gap * (cols - 1)) / cols;
    let mut out = Vec::new();
    for cat in Category::ALL {
        let ids: Vec<EffectId> = EffectId::ALL
            .iter()
            .copied()
            .filter(|id| id.category() == cat)
            .collect();
        out.push(String::new());
        out.push(section(cat.title(), style));
        for chunk in ids.chunks(cols) {
            let mut line = String::new();
            for (i, id) in chunk.iter().enumerate() {
                let badge = id.charset_support().badge();
                let name_w = cell.saturating_sub(17).max(6);
                let name = truncate_plain(id.title_en(), name_w);
                let cell_text = format!(
                    "  {} {} {}",
                    pad(&paint(id.key(), id.default_theme().c(2), style), 10),
                    pad(&paint(&name, TEXT, style), name_w),
                    if id.charset_support() == CharsetSupport::Dual {
                        paint(badge, HOT, style)
                    } else {
                        faint(badge, style)
                    }
                );
                line.push_str(&pad(&cell_text, cell));
                if i + 1 < chunk.len() {
                    line.push_str(&" ".repeat(gap));
                }
            }
            out.push(line);
        }
    }
    out.push(String::new());
    if style.width >= 72 {
        out.push(format!(
            "  {}",
            faint(
                "A = ascii only   A+U = ascii default, --unicode available",
                style
            )
        ));
    } else {
        out.push(format!("  {}", faint("A = ascii only", style)));
        out.push(format!(
            "  {}",
            faint("A+U = ascii default + --unicode", style)
        ));
    }
    out
}

fn curated_options(style: &HelpStyle) -> Vec<String> {
    let mut out = vec![section("OPTIONS", style)];
    let groups: &[(&str, &[&str])] = &[
        (
            "PLAYBACK",
            &[
                "-e --effect",
                "--playlist",
                "--switch",
                "--transition",
                "--transition-secs",
                "--no-fade",
            ],
        ),
        (
            "RENDER",
            &[
                "-f --fps",
                "-s --speed",
                "-t --theme",
                "--charset",
                "--unicode",
                "--ascii",
                "--color",
                "--no-color",
                "--no-hud",
                "--no-alt",
            ],
        ),
        (
            "TUNING",
            &[
                "--density",
                "--spin",
                "--twinkle",
                "--meteors",
                "--beam-speed",
                "--beam-motion",
                "--trail",
                "--seed-count",
                "--seed-speed",
                "--street",
                "--tornado-count",
                "--tornado-speed",
                "--life-cycle",
                "--eye-motion",
                "--ghost",
                "--gallery",
                "--plain",
                "--jump",
            ],
        ),
        ("CAPTURE", &["--snapshot", "--frames", "--size", "--bench"]),
        ("OTHER", &["--seed", "--duration", "--completions"]),
    ];
    for (label, flags) in groups {
        let mut rest = flags.join("  ");
        let mut first = true;
        loop {
            let prefix = if first {
                format!("    {}  ", paint(label, style.theme.c(2), style))
            } else {
                format!("    {}  ", " ".repeat(label.len()))
            };
            let room = style.width.saturating_sub(visible_width(&prefix));
            if display_width(&rest) <= room {
                out.push(format!("{}{}", prefix, paint(&rest, KEY, style)));
                break;
            }
            // Break at the last flag that fits.
            let mut cut = 0;
            let mut used = 0;
            for (idx, part) in rest.split("  ").enumerate() {
                let w = display_width(part) + if idx == 0 { 0 } else { 2 };
                if used + w > room && idx > 0 {
                    break;
                }
                used += w;
                cut = idx + 1;
            }
            let parts: Vec<&str> = rest.split("  ").collect();
            let (head, tail) = parts.split_at(cut.max(1));
            out.push(format!("{}{}", prefix, paint(&head.join("  "), KEY, style)));
            rest = tail.join("  ");
            first = false;
            if rest.is_empty() {
                break;
            }
        }
    }
    out.push(format!(
        "  {}",
        dim("Full reference: trix --help all", style)
    ));
    out
}

fn theme_strip(style: &HelpStyle) -> Vec<String> {
    let mut out = vec![section("THEMES", style)];
    let mut line = "    ".to_string();
    for t in theme::THEMES {
        let entry = format!(
            "{}{}",
            paint(t.key, t.c(2), style),
            paint(" ●", t.c(4), style)
        );
        let gap = "  ";
        if visible_width(&line) + visible_width(&entry) + gap.len() > style.width {
            out.push(line);
            line = "    ".to_string();
        }
        line.push_str(&entry);
        line.push_str(gap);
    }
    out.push(line);
    out
}

fn keys_block(style: &HelpStyle) -> Vec<String> {
    let mut out = vec![section("KEYS", style)];
    let rows = [
        ("q", "quit"),
        ("space", "pause"),
        ("n/p", "next/prev"),
        ("t", "theme"),
        ("h/?", "help"),
        ("i", "info"),
        ("r", "restart"),
        ("+/-", "speed"),
    ];
    let mut line = "    ".to_string();
    for (key, desc) in rows {
        let entry = format!("{} {}", paint(key, KEY, style), dim(desc, style));
        let sep = if line.trim().is_empty() { "" } else { "  ·  " };
        if visible_width(&line) + visible_width(&entry) + visible_width(sep) > style.width {
            out.push(line.trim_end().to_string());
            line = "    ".to_string();
        }
        if !line.trim().is_empty() {
            line.push_str(sep);
        }
        line.push_str(&entry);
    }
    out.push(line.trim_end().to_string());
    let details =
        "1-9 0 [ ] jump to the first 12 effects · m e d f l b j c z k o v g y u w select more";
    for line in wrap(details, style.width.saturating_sub(6)) {
        out.push(format!("    {}", dim(&line, style)));
    }
    let note = "tunnel / helix / lights / volcano / tesseract / launch / plasma: no single-key shortcut, use trix <effect>";
    for line in wrap(note, style.width.saturating_sub(6)) {
        out.push(format!("    {}", dim(&line, style)));
    }
    out
}

fn try_block(style: &HelpStyle, width: usize) -> Vec<String> {
    let mut out = vec![section("TRY", style)];
    let cmds = [
        ("trix", "cycle every effect"),
        ("trix rain", "code rain"),
        ("trix dragon --theme gold", "golden dragon"),
        ("trix --playlist random --switch 8", "random playlist"),
        ("trix monalisa --unicode", "Unicode portrait"),
        (
            "trix rain --snapshot rain.ans --size 120x36 --frames 90",
            "write a frame",
        ),
    ];
    let left = cmds
        .iter()
        .map(|(c, _)| display_width(c))
        .max()
        .unwrap_or(0);
    for (cmd, desc) in cmds {
        if width >= 96 {
            out.push(format!(
                "    {}   {}",
                pad(&paint(cmd, KEY, style), left + 2),
                dim(desc, style)
            ));
        } else {
            let shown = truncate_plain(cmd, width.saturating_sub(6));
            out.push(format!("    {}", paint(&shown, KEY, style)));
        }
    }
    out
}

fn footer(style: &HelpStyle) -> Vec<String> {
    let mut out = vec![section("MORE", style)];
    if style.width >= 64 {
        out.push(format!(
            "    {}  ·  {}  ·  {}",
            paint("trix --list", KEY, style),
            paint("trix --help <effect>", KEY, style),
            paint("trix --help all", KEY, style)
        ));
    } else {
        for command in ["trix --list", "trix --help <effect>", "trix --help all"] {
            out.push(format!("    {}", paint(command, KEY, style)));
        }
    }
    for line in wrap(
        "docs/hud-and-keys.md  ·  github.com/Doswind/trix-effects",
        style.width.saturating_sub(6),
    ) {
        out.push(format!("    {}", dim(&line, style)));
    }
    for line in wrap(
        "If trix-effects makes your terminal nicer, a star helps others find it.",
        style.width.saturating_sub(6),
    ) {
        out.push(format!("    {}", paint(&line, OK, style)));
    }
    out
}

pub fn overview(style: &HelpStyle) -> String {
    let mut lines = header(style);
    lines.push(String::new());
    lines.extend(usage(style));
    let preview = preview_with_hud(style);
    if !preview.is_empty() {
        lines.push(String::new());
        lines.extend(preview);
    }
    lines.push(String::new());
    lines.extend(effect_grid(style));
    lines.push(String::new());
    lines.extend(curated_options(style));
    lines.push(String::new());
    lines.extend(theme_strip(style));
    lines.push(String::new());
    lines.extend(keys_block(style));
    lines.push(String::new());
    lines.extend(try_block(style, style.width));
    lines.push(String::new());
    lines.extend(footer(style));
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn common_rows() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        ("-f, --fps <1-240>", "target frame rate", "30"),
        ("-s, --speed <0.05-8>", "animation speed multiplier", "1.0"),
        ("-t, --theme <name>", "color theme", "effect default"),
        ("--charset <mode>", "kana | ascii | unicode", "ascii"),
        ("--unicode", "use Unicode glyphs", ""),
        ("--ascii", "force ASCII glyphs", ""),
        (
            "--color <mode>",
            "auto | truecolor | 256 | 16 | mono",
            "auto",
        ),
        ("--no-color", "alias for --color mono", ""),
    ]
}

fn option_rows(width: usize, style: &HelpStyle, rows: &[(&str, &str, &str)]) -> Vec<String> {
    let mut out = Vec::new();
    let left = rows
        .iter()
        .map(|(l, _, _)| display_width(l))
        .max()
        .unwrap_or(0);
    for (left_text, desc, default) in rows {
        let default = if default.is_empty() {
            String::new()
        } else {
            format!("  {}", faint(&format!("[{}]", default), style))
        };
        if width >= 66 {
            out.push(format!(
                "    {}  {}{}",
                pad(&paint(left_text, KEY, style), left + 2),
                paint(desc, TEXT, style),
                default
            ));
        } else {
            out.push(format!("    {}", paint(left_text, KEY, style)));
            let lines = wrap(desc, width.saturating_sub(6 + visible_width(&default)));
            let last = lines.len().saturating_sub(1);
            for (i, line) in lines.iter().enumerate() {
                out.push(format!(
                    "      {}{}",
                    paint(line, TEXT, style),
                    if i == last { &default } else { "" }
                ));
            }
        }
    }
    out
}

fn tuning_rows(tuning: &[Tuning], style: &HelpStyle, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for t in tuning {
        let flag = if t.value.is_empty() {
            t.flag.to_string()
        } else {
            format!("{} {}", t.flag, t.value)
        };
        let default = if t.default.is_empty() {
            String::new()
        } else {
            format!("  {}", faint(&format!("[{}]", t.default), style))
        };
        if width >= 66 {
            out.push(format!(
                "    {}  {}{}",
                pad(&paint(&flag, KEY, style), 26),
                paint(t.desc, TEXT, style),
                default
            ));
        } else {
            out.push(format!("    {}", paint(&flag, KEY, style)));
            let lines = wrap(t.desc, width.saturating_sub(6 + visible_width(&default)));
            let last = lines.len().saturating_sub(1);
            for (i, line) in lines.iter().enumerate() {
                out.push(format!(
                    "      {}{}",
                    paint(line, TEXT, style),
                    if i == last { &default } else { "" }
                ));
            }
        }
    }
    out
}

fn tuning_example(flag: &str) -> &'static str {
    match flag {
        "--density" => "1.5",
        "--spin" => "0.25",
        "--twinkle" => "2.0",
        "--meteors" => "8",
        "--beam-speed" => "1.2",
        "--beam-motion" => "jump",
        "--seed-count" => "180",
        "--seed-speed" => "1.4",
        "--street" => "vertical",
        "--tornado-count" => "5",
        "--tornado-speed" => "1.6",
        "--life-cycle" => "60",
        "--eye-motion" => "still",
        "--trail" => "1.5",
        _ => "",
    }
}

fn scene_preview(id: EffectId, w: usize, h: usize, style: &HelpStyle) -> Vec<String> {
    let theme = id.default_theme();
    let charset = id.default_charset();
    let seed = 0x7e51_a017;
    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let mut canvas = Canvas::new(w, h);
    let dt = 0.1f32;
    let frames = ((id.preview_secs() * 10.0) as usize).clamp(10, 400);
    let mut t = 0.0f64;
    for _ in 0..frames {
        let frame = Frame {
            t,
            dt,
            w,
            h,
            fps: 10,
            theme,
            charset,
            seed,
            speed: 1.0,
            tune: Tune::default(),
        };
        scene.update(&frame);
        t += dt as f64;
    }
    let frame = Frame {
        t,
        dt,
        w,
        h,
        fps: 10,
        theme,
        charset,
        seed,
        speed: 1.0,
        tune: Tune::default(),
    };
    canvas.clear();
    scene.draw(&mut canvas, &frame);
    canvas.render_rows(style.color)
}

pub fn effect(id: EffectId, base: &HelpStyle) -> String {
    let style = HelpStyle {
        theme: id.default_theme(),
        ..*base
    };
    let width = style.width;
    let inner = width.saturating_sub(4).max(24);
    let body_width = inner.saturating_sub(4).max(20);
    let title = format!("{}  ·  {}  ·  {}", id.key(), id.title_en(), id.title_zh());
    let mut body: Vec<String> = Vec::new();
    body.push(String::new());
    for line in wrap(id.about(), body_width) {
        body.push(format!("  {}", paint(&line, TEXT, &style)));
    }
    body.push(String::new());
    let meta = format!(
        "theme {}  ·  charset {}  ·  hud {}  ·  key {}",
        id.default_theme().key,
        id.charset_support().label(),
        if id.hud_mode() == scene::HudMode::Ambient {
            "ambient"
        } else {
            "full"
        },
        id.shortcut()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    for line in wrap(&meta, body_width) {
        body.push(format!("  {}", paint(&line, TEXT, &style)));
    }
    body.push(String::new());

    if style.tty && width >= 60 {
        let pw = body_width.clamp(40, 76);
        let rows = scene_preview(id, pw, 11, &style);
        body.extend(boxed(&rows, inner, &style, Some("PREVIEW")));
        body.push(String::new());
    }

    body.push(format!("  {}", bold("OPTIONS", style.theme.c(3), &style)));
    body.extend(option_rows(body_width, &style, common_rows()));
    let tuning = id.tuning();
    if !tuning.is_empty() {
        body.push(String::new());
        body.push(format!(
            "  {}",
            bold("EFFECT OPTIONS", style.theme.c(3), &style)
        ));
        body.extend(tuning_rows(tuning, &style, body_width));
    }

    body.push(String::new());
    body.push(format!("  {}", bold("TRY", style.theme.c(3), &style)));
    let mut examples = vec![
        format!("trix {}", id.key()),
        format!("trix {} --theme {}", id.key(), id.default_theme().key),
    ];
    if let Some(first) = tuning.first() {
        let value = tuning_example(first.flag);
        if value.is_empty() {
            examples.push(format!("trix {} {}", id.key(), first.flag));
        } else {
            examples.push(format!("trix {} {} {}", id.key(), first.flag, value));
        }
    }
    examples.push(format!(
        "trix {} --snapshot {}.ans --size 120x36 --frames 90",
        id.key(),
        id.key()
    ));
    for example in examples {
        for line in wrap(&example, body_width.saturating_sub(4)) {
            body.push(format!("    {}", paint(&line, KEY, &style)));
        }
    }

    body.push(String::new());
    let prev = id.prev();
    let next = id.next();
    let prev_text = match prev.shortcut() {
        Some(key) => format!("prev {} {}", prev.key(), key),
        None => format!("prev {}", prev.key()),
    };
    let next_text = match next.shortcut() {
        Some(key) => format!("next {} {}", next.key(), key),
        None => format!("next {}", next.key()),
    };
    if body_width >= 38 {
        body.push(format!(
            "  {}  ·  {}",
            paint(&prev_text, KEY, &style),
            paint(&next_text, KEY, &style)
        ));
    } else {
        body.push(format!("  {}", paint(&prev_text, KEY, &style)));
        body.push(format!("  {}", paint(&next_text, KEY, &style)));
    }
    body.push(String::new());

    let mut out = boxed(&body, width, &style, Some(&title)).join("\n");
    out.push('\n');
    out
}

struct OptRow {
    flags: &'static str,
    arg: &'static str,
    desc: &'static str,
    default: Option<&'static str>,
}

const PLAYBACK: &[OptRow] = &[
    OptRow {
        flags: "-e, --effect",
        arg: "<name>",
        desc: "effect to run; omit to cycle every effect",
        default: None,
    },
    OptRow {
        flags: "--playlist",
        arg: "<mode>",
        desc: "cycle | random (only when no effect is given)",
        default: Some("cycle"),
    },
    OptRow {
        flags: "--switch",
        arg: "<secs>",
        desc: "seconds per effect in the playlist",
        default: Some("12"),
    },
    OptRow {
        flags: "--transition",
        arg: "<mode>",
        desc: "fade | dissolve | none",
        default: Some("fade"),
    },
    OptRow {
        flags: "--transition-secs",
        arg: "<s>",
        desc: "length of the blend",
        default: Some("0.6"),
    },
    OptRow {
        flags: "--no-fade",
        arg: "",
        desc: "alias for --transition none",
        default: None,
    },
];

const RENDER: &[OptRow] = &[
    OptRow {
        flags: "-f, --fps",
        arg: "<1-240>",
        desc: "target frame rate",
        default: Some("30"),
    },
    OptRow {
        flags: "-s, --speed",
        arg: "<0.05-8>",
        desc: "animation speed multiplier",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--chase-speed",
        arg: "<f>",
        desc: "alias for --speed (Tom & Jerry)",
        default: None,
    },
    OptRow {
        flags: "-t, --theme",
        arg: "<name>",
        desc: "color theme",
        default: Some("effect default"),
    },
    OptRow {
        flags: "--charset",
        arg: "<mode>",
        desc: "kana | ascii | unicode",
        default: Some("ascii"),
    },
    OptRow {
        flags: "--unicode",
        arg: "",
        desc: "use Unicode glyphs where the effect supports them",
        default: None,
    },
    OptRow {
        flags: "--ascii",
        arg: "",
        desc: "force ASCII glyphs",
        default: None,
    },
    OptRow {
        flags: "--color",
        arg: "<mode>",
        desc: "auto | truecolor | 256 | 16 | mono",
        default: Some("auto"),
    },
    OptRow {
        flags: "--no-color",
        arg: "",
        desc: "alias for --color mono",
        default: None,
    },
    OptRow {
        flags: "--no-hud",
        arg: "",
        desc: "hide the top/bottom HUD (h/i/pause panels still work)",
        default: None,
    },
    OptRow {
        flags: "--no-alt",
        arg: "",
        desc: "do not use the alternate screen",
        default: None,
    },
];

const TUNING: &[OptRow] = &[
    OptRow {
        flags: "--density",
        arg: "<f>",
        desc: "beams/fireworks/galaxy/traffic density",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--spin",
        arg: "<f>",
        desc: "black hole rotation speed",
        default: Some("0.5"),
    },
    OptRow {
        flags: "--twinkle",
        arg: "<f>",
        desc: "galaxy twinkle rate",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--meteors",
        arg: "<n>",
        desc: "black hole / galaxy meteors",
        default: Some("3"),
    },
    OptRow {
        flags: "--beam-speed",
        arg: "<f>",
        desc: "beams horizontal speed",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--beam-motion",
        arg: "<m>",
        desc: "straight | jump",
        default: Some("straight"),
    },
    OptRow {
        flags: "--beam-jump",
        arg: "",
        desc: "alias for --beam-motion jump",
        default: None,
    },
    OptRow {
        flags: "--trail",
        arg: "<f>",
        desc: "DeepSeek Harness afterimage strength (0 = off)",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--seed-count",
        arg: "<n>",
        desc: "dandelion drifting seeds",
        default: Some("90"),
    },
    OptRow {
        flags: "--seed-speed",
        arg: "<f>",
        desc: "dandelion seed drift speed",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--street",
        arg: "<dir>",
        desc: "traffic street: horizontal | vertical",
        default: Some("horizontal"),
    },
    OptRow {
        flags: "--tornado-count",
        arg: "<n>",
        desc: "tornado count",
        default: Some("3"),
    },
    OptRow {
        flags: "--tornado-speed",
        arg: "<f>",
        desc: "tornado travel speed",
        default: Some("1.0"),
    },
    OptRow {
        flags: "--life-cycle",
        arg: "<secs>",
        desc: "life: seconds per 0-100 round",
        default: Some("120"),
    },
    OptRow {
        flags: "--eye-motion",
        arg: "<m>",
        desc: "demoneye: dart | still",
        default: Some("dart"),
    },
    OptRow {
        flags: "--eye-still",
        arg: "",
        desc: "alias for --eye-motion still",
        default: None,
    },
    OptRow {
        flags: "--ghost",
        arg: "",
        desc: "monalisa: flat pure-ASCII portrait",
        default: None,
    },
    OptRow {
        flags: "--gallery",
        arg: "",
        desc: "monalisa: soft gallery glow",
        default: None,
    },
    OptRow {
        flags: "--plain",
        arg: "",
        desc: "dragon: plain starfield (no mountains/cloud sea)",
        default: None,
    },
    OptRow {
        flags: "--jump",
        arg: "",
        desc: "Tom & Jerry: enable jumping",
        default: None,
    },
    OptRow {
        flags: "--no-jump",
        arg: "",
        desc: "Tom & Jerry: disable jumping",
        default: None,
    },
];

const CAPTURE: &[OptRow] = &[
    OptRow {
        flags: "--snapshot",
        arg: "<path>",
        desc: "render --frames steps to ANSI ('-' = stdout)",
        default: None,
    },
    OptRow {
        flags: "--frames",
        arg: "<n>",
        desc: "simulation steps before snapshot / benchmark",
        default: None,
    },
    OptRow {
        flags: "--size",
        arg: "<WxH>",
        desc: "offscreen canvas size, e.g. 120x36",
        default: None,
    },
    OptRow {
        flags: "--bench",
        arg: "",
        desc: "offscreen render benchmark (includes ANSI encoding)",
        default: None,
    },
];

const OTHER: &[OptRow] = &[
    OptRow {
        flags: "--completions",
        arg: "<shell>",
        desc: "print a bash / zsh / fish completion script",
        default: None,
    },
    OptRow {
        flags: "--seed",
        arg: "<u64>",
        desc: "deterministic random seed",
        default: None,
    },
    OptRow {
        flags: "--duration",
        arg: "<secs>",
        desc: "exit automatically after N seconds",
        default: None,
    },
    OptRow {
        flags: "-l, --list",
        arg: "[effect]",
        desc: "list effects and themes (or one effect)",
        default: None,
    },
    OptRow {
        flags: "-h, --help",
        arg: "[effect|all]",
        desc: "this help, an effect dossier, or the full option reference",
        default: None,
    },
    OptRow {
        flags: "-V, --version",
        arg: "",
        desc: "print version",
        default: None,
    },
];

fn option_table(rows: &[OptRow], width: usize, style: &HelpStyle) -> Vec<String> {
    let mut out = Vec::new();
    let left = rows
        .iter()
        .map(|r| display_width(r.flags) + r.arg.len() + 1)
        .max()
        .unwrap_or(20)
        .min(34);
    let max_desc = rows
        .iter()
        .map(|r| display_width(r.desc))
        .max()
        .unwrap_or(0);
    let max_default = rows
        .iter()
        .filter_map(|r| r.default)
        .map(|d| display_width(d) + 4)
        .max()
        .unwrap_or(0);
    let one_line = width >= left + 8 + max_desc + max_default;
    for row in rows {
        let left_text = if row.arg.is_empty() {
            row.flags.to_string()
        } else {
            format!("{} {}", row.flags, row.arg)
        };
        let default = match row.default {
            Some(d) => format!("  {}", faint(&format!("[{}]", d), style)),
            None => String::new(),
        };
        if one_line {
            out.push(format!(
                "    {}  {}{}",
                pad(&paint(&left_text, KEY, style), left + 2),
                paint(row.desc, TEXT, style),
                default
            ));
        } else {
            out.push(format!("    {}", paint(&left_text, KEY, style)));
            let lines = wrap(row.desc, width.saturating_sub(6 + visible_width(&default)));
            let last = lines.len().saturating_sub(1);
            for (i, line) in lines.iter().enumerate() {
                out.push(format!(
                    "      {}{}",
                    dim(line, style),
                    if i == last { &default } else { "" }
                ));
            }
        }
    }
    out
}

pub fn all_options(style: &HelpStyle) -> String {
    let mut lines = header(style);
    lines.push(String::new());
    lines.push(section("FULL OPTION REFERENCE", style));
    let groups: &[(&str, &[OptRow])] = &[
        ("PLAYBACK", PLAYBACK),
        ("RENDER", RENDER),
        ("EFFECT TUNING", TUNING),
        ("CAPTURE", CAPTURE),
        ("OTHER", OTHER),
    ];
    for (title, rows) in groups {
        lines.push(String::new());
        lines.push(format!("  {}", bold(title, style.theme.c(3), style)));
        lines.extend(option_table(rows, style.width, style));
    }
    lines.push(String::new());
    for line in wrap(
        "Aliases: --no-altscreen --utf8 --kana --vertical --bg --no-background --no-ghost --no-gallery --no-jump --eye-dart --fade --dissolve --crossfade --shuffle --interval --seeds --life-span --life-duration",
        style.width.saturating_sub(4),
    ) {
        lines.push(format!("  {}", dim(&line, style)));
    }
    lines.push(String::new());
    lines.extend(footer(style));
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn plain_list() -> String {
    let mut s = String::new();
    s.push_str("EFFECTS\n");
    for e in EffectId::ALL {
        s.push_str(&format!(
            "  {:<10} {:<28} {}\n",
            e.key(),
            e.title_en(),
            e.title_zh()
        ));
    }
    s.push_str("\nTHEMES\n");
    for t in theme::THEMES {
        s.push_str(&format!("  {:<10} {}\n", t.key, t.label));
    }
    s.push_str("\nRun `trix --help` for options.\n");
    s
}

fn pretty_list(style: &HelpStyle) -> String {
    let header = if style.width >= 54 {
        format!(
            "{}  {}  ·  {} effects  ·  {} themes",
            bold("trix-effects", style.theme.c(3), style),
            dim("catalog", style),
            EffectId::ALL.len(),
            theme::THEMES.len()
        )
    } else {
        format!(
            "{}  ·  {} effects",
            bold("trix-effects", style.theme.c(3), style),
            EffectId::ALL.len()
        )
    };
    let mut lines = vec![header, String::new()];
    for cat in Category::ALL {
        lines.push(section(cat.title(), style));
        for id in EffectId::ALL
            .iter()
            .copied()
            .filter(|e| e.category() == cat)
        {
            let name = pad(id.title_en(), 20);
            let zh = pad(id.title_zh(), 10);
            let key = pad(&paint(id.key(), id.default_theme().c(2), style), 10);
            let theme = pad(
                &paint(id.default_theme().key, id.default_theme().c(3), style),
                9,
            );
            let badge = if id.charset_support() == CharsetSupport::Dual {
                paint(id.charset_support().badge(), HOT, style)
            } else {
                faint(id.charset_support().badge(), style)
            };
            if style.width >= 70 {
                lines.push(format!(
                    "    {} {} {} {} {}",
                    key,
                    paint(&name, TEXT, style),
                    paint(&zh, TEXT, style),
                    theme,
                    badge
                ));
            } else {
                lines.push(format!(
                    "    {} {} {}",
                    key,
                    paint(name.trim_end(), TEXT, style),
                    badge
                ));
                lines.push(format!("              {}", dim(zh.trim_end(), style)));
            }
        }
        lines.push(String::new());
    }
    lines.push(section("THEMES", style));
    let mut line = "    ".to_string();
    for t in theme::THEMES {
        let entry = format!(
            "{} {}{}",
            paint(t.key, t.c(2), style),
            paint("●", t.c(4), style),
            paint(&format!(" ({})", t.label), DIM, style)
        );
        if visible_width(&line) + visible_width(&entry) > style.width {
            lines.push(line);
            line = "    ".to_string();
        }
        line.push_str(&entry);
        line.push_str("  ");
    }
    lines.push(line);
    lines.push(String::new());
    if style.width >= 64 {
        lines.push(format!(
            "  {}  ·  {}",
            dim("trix --help <effect> for details and a preview", style),
            paint("trix --help all", KEY, style)
        ));
    } else {
        lines.push(format!("  {}", dim("trix --help <effect>", style)));
        lines.push(format!("  {}", paint("trix --help all", KEY, style)));
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

pub fn list(style: &HelpStyle) -> String {
    if style.on() {
        pretty_list(style)
    } else {
        plain_list()
    }
}

pub fn error(message: &str, style: &HelpStyle) -> String {
    if !style.tty || !style.on() {
        return format!("trix: {message}\n");
    }
    let mark = if style.unicode { "✗" } else { "x" };
    let mut lines = message.lines();
    let first = lines.next().unwrap_or("");
    let mut out = format!(
        "  {} {}\n",
        paint(mark, ERR, style),
        paint(first, ERR, style)
    );
    for line in lines {
        out.push_str(&format!("    {}\n", paint(line, TEXT, style)));
    }
    out.push('\n');
    out.push_str(&format!(
        "    {}\n",
        paint("trix --help all  ·  trix --list", KEY, style)
    ));
    out
}

fn unknown_effect(name: &str, style: &HelpStyle) -> i32 {
    let message = cli::unknown_effect_message(name);
    eprint!("{}", error(&message, style));
    2
}

fn effect_word_list() -> String {
    EffectId::ALL
        .iter()
        .map(|e| e.key())
        .collect::<Vec<_>>()
        .join(" ")
}

fn theme_word_list() -> String {
    theme::THEMES
        .iter()
        .map(|t| t.key)
        .collect::<Vec<_>>()
        .join(" ")
}

fn flag_word_list() -> String {
    cli::PRIMARY_FLAGS.join(" ")
}

pub fn completions(shell: &str) -> String {
    let effects = effect_word_list();
    let themes = theme_word_list();
    let flags = flag_word_list();
    match shell {
        "bash" => format!(
            r#"# bash completion for trix-effects
_trix() {{
  local cur prev
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  prev="${{COMP_WORDS[COMP_CWORD-1]}}"
  case "$prev" in
    -e|--effect) COMPREPLY=( $(compgen -W "{effects}" -- "$cur") ); return 0 ;;
    -t|--theme) COMPREPLY=( $(compgen -W "{themes}" -- "$cur") ); return 0 ;;
    --playlist) COMPREPLY=( $(compgen -W "cycle random" -- "$cur") ); return 0 ;;
    --transition) COMPREPLY=( $(compgen -W "fade dissolve none" -- "$cur") ); return 0 ;;
    --charset) COMPREPLY=( $(compgen -W "kana ascii unicode" -- "$cur") ); return 0 ;;
    --color) COMPREPLY=( $(compgen -W "auto truecolor 256 16 mono" -- "$cur") ); return 0 ;;
    --street) COMPREPLY=( $(compgen -W "horizontal vertical" -- "$cur") ); return 0 ;;
    --eye-motion) COMPREPLY=( $(compgen -W "dart still" -- "$cur") ); return 0 ;;
    --beam-motion) COMPREPLY=( $(compgen -W "straight jump" -- "$cur") ); return 0 ;;
    --completions) COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") ); return 0 ;;
  esac
  if [[ "$cur" == -* ]]; then
    COMPREPLY=( $(compgen -W "{flags}" -- "$cur") )
  else
    COMPREPLY=( $(compgen -W "{effects}" -- "$cur") )
  fi
}}
complete -F _trix trix
"#
        ),
        "zsh" => format!(
            r#"#compdef trix
_trix() {{
  _arguments -s \
    '(-e --effect)'{{-e,--effect}}'[effect to run]:effect:({effects})' \
    '--playlist[cycle | random]:mode:(cycle random)' \
    '--switch[seconds per effect]:seconds:' \
    '--transition[fade | dissolve | none]:mode:(fade dissolve none)' \
    '--transition-secs[blend length]:seconds:' \
    '(-f --fps)'{{-f,--fps}}'[target frame rate]:fps:' \
    '(-s --speed)'{{-s,--speed}}'[speed multiplier]:speed:' \
    '(-t --theme)'{{-t,--theme}}'[color theme]:theme:({themes})' \
    '--charset[charset]:mode:(kana ascii unicode)' \
    '--unicode[use Unicode glyphs]' \
    '--ascii[force ASCII glyphs]' \
    '--color[color mode]:mode:(auto truecolor 256 16 mono)' \
    '--no-color[monochrome]' \
    '--no-hud[hide the HUD]' \
    '--no-alt[do not use the alternate screen]' \
    '--ghost[monalisa: pure-ASCII ghost]' \
    '--gallery[monalisa: gallery glow]' \
    '--plain[dragon: plain starfield]' \
    '--jump[Tom & Jerry: jump]' \
    '--no-jump[Tom & Jerry: no jump]' \
    '--snapshot[write ANSI]:path:_files' \
    '--frames[simulation steps]:count:' \
    '--size[canvas size]:size:' \
    '--bench[offscreen benchmark]' \
    '--seed[random seed]:seed:' \
    '--duration[exit after seconds]:seconds:' \
    '--list[list effects and themes]' \
    '--help[help]:topic:(all {effects})' \
    '--version[print version]' \
    '--completions[completion script]:shell:(bash zsh fish)' \
    '*:effect:({effects})'
}}
compdef _trix trix
"#
        ),
        _ => format!(
            r#"# fish completion for trix-effects
complete -c trix -f
complete -c trix -n '__fish_use_subcommand' -a '{effects}'
complete -c trix -l effect -xa '{effects}'
complete -c trix -l theme -xa '{themes}'
complete -c trix -l playlist -xa 'cycle random'
complete -c trix -l transition -xa 'fade dissolve none'
complete -c trix -l charset -xa 'kana ascii unicode'
complete -c trix -l color -xa 'auto truecolor 256 16 mono'
complete -c trix -l street -xa 'horizontal vertical'
complete -c trix -l eye-motion -xa 'dart still'
complete -c trix -l beam-motion -xa 'straight jump'
complete -c trix -l completions -xa 'bash zsh fish'
complete -c trix -l list -d 'list effects and themes'
complete -c trix -l help -d 'help'
complete -c trix -l version -d 'version'
complete -c trix -l no-hud -d 'hide the HUD'
complete -c trix -l no-color -d 'monochrome'
complete -c trix -l ghost -d 'monalisa ghost'
complete -c trix -l gallery -d 'monalisa gallery'
complete -c trix -l plain -d 'dragon plain'
complete -c trix -l jump -d "Tom & Jerry jump"
complete -c trix -l no-jump -d "Tom & Jerry no jump"
complete -c trix -l snapshot -r -d 'write ANSI'
complete -c trix -l frames -r -d 'simulation steps'
complete -c trix -l size -r -d 'canvas size'
complete -c trix -l bench -d 'benchmark'
complete -c trix -l seed -r -d 'seed'
complete -c trix -l duration -r -d 'seconds'
"#
        ),
    }
}

pub fn run_completions(shell: &str) -> i32 {
    print!("{}", completions(shell));
    0
}

pub fn run_help(opts: &Options) -> i32 {
    let style = HelpStyle::detect(opts);
    let text = match opts.effect.as_deref() {
        None => overview(&style),
        Some(name) if name.eq_ignore_ascii_case("all") => all_options(&style),
        Some(name) => match EffectId::parse(name) {
            Some(id) => effect(id, &style),
            None => return unknown_effect(name, &style),
        },
    };
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
    0
}

pub fn run_list(opts: &Options) -> i32 {
    let style = HelpStyle::detect(opts);
    let text = match opts.effect.as_deref() {
        Some(name) if !name.eq_ignore_ascii_case("all") => match EffectId::parse(name) {
            Some(id) => effect(id, &style),
            None => return unknown_effect(name, &style),
        },
        _ => list(&style),
    };
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    let _ = stdout.flush();
    0
}

pub fn run_version() -> i32 {
    println!("trix-effects {}", cli::VERSION);
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> HelpStyle {
        HelpStyle::plain()
    }

    #[test]
    fn overview_contains_every_effect_and_primary_sections() {
        let text = overview(&plain());
        for id in EffectId::ALL {
            assert!(text.contains(id.key()), "missing {}", id.key());
        }
        for heading in ["USAGE", "OPTIONS", "THEMES", "KEYS", "MORE"] {
            assert!(text.contains(heading), "missing {heading}");
        }
    }

    #[test]
    fn overview_lines_fit_the_requested_width() {
        for width in [40, 60, 80, 100, 120] {
            let style = HelpStyle { width, ..plain() };
            let text = overview(&style);
            for line in text.lines() {
                assert!(
                    visible_width(line) <= width,
                    "{} > {} in {:?}: {:?}",
                    visible_width(line),
                    width,
                    width,
                    line
                );
            }
        }
    }

    #[test]
    fn every_effect_has_a_dossier() {
        for id in EffectId::ALL {
            let text = effect(id, &plain());
            assert!(text.contains(id.key()), "{id:?} key");
            assert!(text.contains(id.title_en()), "{id:?} title");
            assert!(text.contains("OPTIONS"), "{id:?} options");
            assert!(text.contains("TRY"), "{id:?} try");
        }
    }

    #[test]
    fn dossiers_and_reference_fit_the_requested_width() {
        for width in [40, 60, 80, 100, 120] {
            let style = HelpStyle { width, ..plain() };
            for id in EffectId::ALL {
                for line in effect(id, &style).lines() {
                    assert!(
                        visible_width(line) <= width,
                        "{id:?} {} > {} in {:?}: {:?}",
                        visible_width(line),
                        width,
                        width,
                        line
                    );
                }
            }
            for line in all_options(&style).lines() {
                assert!(
                    visible_width(line) <= width,
                    "all_options {} > {} in {:?}: {:?}",
                    visible_width(line),
                    width,
                    width,
                    line
                );
            }
            for line in pretty_list(&style).lines() {
                assert!(
                    visible_width(line) <= width,
                    "list {} > {} in {:?}: {:?}",
                    visible_width(line),
                    width,
                    width,
                    line
                );
            }
        }
    }

    #[test]
    fn all_option_text_covers_every_effect_tuning_flag() {
        let text = super::all_options(&plain());
        for id in EffectId::ALL {
            for t in id.tuning() {
                assert!(text.contains(t.flag), "{id:?} missing {}", t.flag);
            }
        }
    }

    #[test]
    fn plain_list_keeps_the_script_format() {
        let text = list(&plain());
        assert!(text.starts_with("EFFECTS\n"));
        for id in EffectId::ALL {
            assert!(text.contains(&format!("  {:<10}", id.key())));
        }
    }

    #[test]
    fn completions_cover_every_effect() {
        for shell in ["bash", "zsh", "fish"] {
            let text = completions(shell);
            for id in EffectId::ALL {
                assert!(text.contains(id.key()), "{shell} missing {}", id.key());
            }
            assert!(!text.trim().is_empty());
        }
        assert!(completions("bash").contains("complete -F _trix trix"));
        assert!(completions("zsh").contains("#compdef trix"));
        assert!(completions("fish").contains("complete -c trix"));
    }

    #[test]
    fn no_ansi_in_mono_output() {
        let text = overview(&plain());
        assert!(!text.contains('\x1b'));
    }

    #[test]
    fn error_is_plain_off_a_tty() {
        let text = error("unknown option '--fsp' (did you mean '--fps'?)", &plain());
        assert_eq!(
            text,
            "trix: unknown option '--fsp' (did you mean '--fps'?)\n"
        );
    }
}
