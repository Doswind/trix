//! Shared heads-up display.
//!
//! Every effect gets the same chrome, so the user always knows where to look
//! and which keys are live:
//!
//! ```text
//!   Code Rain  // rain                    1.00x | neon | ascii      row 0
//!   ... scene artwork ...
//!   playlist 03/26  Code Rain  next 8s                              row h-2
//!   q quit | space pause | n/p next | t theme | h help     i info   row h-1
//! ```
//!
//! `rain`, `fire` and `monalisa` are *ambient*: they keep their deliberately
//! text-free artwork and skip the header, but they still get the key bar and
//! playlist row so the controls never disappear.
//!
//! Scenes must not draw text into row 0 or row h-1.  Row h-2 is reserved for
//! the playlist indicator; diegetic labels belong in rows 1..=h-3.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::ColorMode;
use crate::scene::{EffectId, Frame};

pub const HEADER_ROW: isize = 0;

/// Bottom row: the always-on key bar.
pub fn keybar_row(h: usize) -> isize {
    h as isize - 1
}

/// Row above the key bar: playlist progress.
pub fn playlist_row(h: usize) -> isize {
    h as isize - 2
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    None,
    Help,
    Info,
}

fn width(text: &str) -> usize {
    text.chars().count()
}

fn truncate(text: &str, max: usize) -> String {
    if width(text) <= max {
        return text.to_string();
    }
    if max <= 2 {
        return text.chars().take(max).collect();
    }
    let mut out: String = text.chars().take(max - 2).collect();
    out.push_str("..");
    out
}

fn charset_label(charset: Charset) -> &'static str {
    match charset {
        Charset::Kana => "kana",
        Charset::Ascii => "ascii",
        Charset::Unicode => "unicode",
    }
}

fn join_width(parts: &[String]) -> usize {
    if parts.is_empty() {
        0
    } else {
        parts.iter().map(|s| width(s)).sum::<usize>() + (parts.len() - 1) * 3
    }
}

/// Persistent top bar: effect name and key on the left, live telemetry on
/// the right.  Only drawn for `HudMode::Full` effects.
pub fn draw_header(canvas: &mut Canvas, frame: &Frame, id: EffectId, scene_status: Option<&str>) {
    if frame.h == 0 || frame.w < 8 {
        return;
    }
    let y = HEADER_ROW;
    canvas.clear_rect(0, y, frame.w as isize, 1);
    let theme = frame.theme;

    let title = id.title_en();
    let key = id.key();
    let badge = format!(" // {key}");

    let mut candidates: Vec<String> = Vec::new();
    if let Some(status) = scene_status {
        let status = status.trim();
        if !status.is_empty() {
            candidates.push(status.to_string());
        }
    }
    candidates.push(format!("{:.2}x", frame.speed));
    candidates.push(theme.key.to_string());
    candidates.push(charset_label(frame.charset).to_string());

    // Drop the least important right-hand segment first until the title and
    // key badge fit.  The title is only truncated after everything else has
    // been dropped.
    loop {
        let right_w = join_width(&candidates);
        let left_max = frame.w as isize - 2 - width(&badge) as isize - right_w as isize - 3;
        if width(title) as isize <= left_max || candidates.is_empty() {
            break;
        }
        candidates.remove(0);
    }

    let right_w = join_width(&candidates);
    let left_max =
        (frame.w as isize - 2 - width(&badge) as isize - right_w as isize - 3).max(4) as usize;
    let shown_title = truncate(title, left_max);
    canvas.put_str_gradient(2, y, &shown_title, &[theme.c(2), theme.c(3), theme.hot()]);

    let x = 2 + width(&shown_title) as isize;
    if x + width(&badge) as isize + right_w as isize + 3 <= frame.w as isize {
        canvas.put_str(x + 2, y, &badge, theme.c(2).scale(0.62));
    }

    if !candidates.is_empty() {
        let text = candidates.join(" | ");
        let rx = (frame.w as isize - width(&text) as isize - 2).max(x + 2);
        if rx + width(&text) as isize <= frame.w as isize {
            canvas.put_str(rx, y, &text, theme.c(2).scale(0.8));
        }
    }
}

/// Persistent bottom bar.  This is the single source of truth for runtime
/// keys; scenes no longer draw their own hints.
pub fn draw_keybar(canvas: &mut Canvas, frame: &Frame, paused: bool) {
    if frame.h < 2 || frame.w < 16 {
        return;
    }
    let y = keybar_row(frame.h);
    canvas.clear_rect(0, y, frame.w as isize, 1);
    let theme = frame.theme;

    let text = if frame.w >= 86 {
        if paused {
            "q quit  |  space resume  |  n/p next  |  t theme  |  +/- speed  |  h help"
        } else {
            "q quit  |  space pause  |  n/p next  |  t theme  |  +/- speed  |  h help"
        }
    } else if frame.w >= 62 {
        if paused {
            "q quit  |  space resume  |  n/p next  |  h help"
        } else {
            "q quit  |  space pause  |  n/p next  |  h help"
        }
    } else if frame.w >= 40 {
        "q quit  |  n/p next  |  h help"
    } else {
        "q quit  |  h help"
    };
    canvas.put_str(2, y, text, theme.c(2).scale(0.55));

    let info = "i info";
    let ix = frame.w as isize - width(info) as isize - 2;
    if ix > 2 + width(text) as isize + 3 {
        canvas.put_str(ix, y, info, theme.c(2).scale(0.45));
    }
}

/// Playlist progress on the reserved row above the key bar.
pub fn draw_playlist(
    canvas: &mut Canvas,
    frame: &Frame,
    id: EffectId,
    switch_secs: f32,
    elapsed: f32,
) {
    if frame.h < 4 || frame.w < 36 {
        return;
    }
    let y = playlist_row(frame.h);
    let total = EffectId::ALL.len();
    let left = (switch_secs - elapsed).max(0.0) as u32;
    let text = format!(
        "playlist {}/{}  {}  next {}s ",
        id.index() + 1,
        total,
        id.title_en(),
        left
    );
    let x = (frame.w as isize - width(&text) as isize).max(1);
    canvas.clear_rect(x, y, frame.w as isize - x, 1);
    canvas.put_str(x, y, &text, frame.theme.c(2).scale(0.6));
}

/// Center-screen modal while the animation is paused.
pub fn draw_pause(canvas: &mut Canvas, frame: &Frame) {
    let text = " PAUSED  //  space to resume ";
    let bw = width(text) as isize + 4;
    if frame.w as isize <= bw || frame.h < 6 {
        return;
    }
    let x = ((frame.w as isize - bw) / 2).max(0);
    let y = frame.h as isize / 2 - 1;
    canvas.clear_rect(x, y, bw, 3);
    canvas.rect(x, y, bw, 3, frame.theme.c(2).scale(0.8));
    canvas.put_str(x + 3, y + 1, text, frame.theme.hot());
}

/// Center-screen help panel.  Every row here must match the key dispatch in
/// `app.rs`; this panel is the canonical in-app reference.
pub fn draw_help(canvas: &mut Canvas, frame: &Frame, id: EffectId, speed: f32, paused: bool) {
    let w = frame.w as isize;
    let h = frame.h as isize;
    let theme = frame.theme;
    if w < 34 || h < 10 {
        canvas.clear_rect(0, 1, w, 1);
        canvas.put_str(2, 1, "h / ? / Esc close help   i info   q quit", theme.c(2));
        return;
    }
    let bw = 78.min(w - 4).max(32);
    let bh = 24.min(h - 2).max(8);
    let x = (w - bw) / 2;
    let y = (h - bh) / 2;
    canvas.clear_rect(x, y, bw, bh);
    canvas.rect(x, y, bw, bh, theme.c(2).scale(0.65));
    canvas.put_str_gradient(
        x + 2,
        y + 1,
        "TRIX-TUI // KEYBOARD & EFFECTS",
        &[theme.c(2), theme.c(3), theme.hot()],
    );

    let rows: &[(&str, String)] = &[
        ("q / Ctrl-C", "quit".into()),
        ("Esc", "close panel, then quit".into()),
        (
            "space",
            if paused {
                "resume".into()
            } else {
                "pause".into()
            },
        ),
        ("n / p", "next / previous effect".into()),
        ("arrows", "next / previous effect".into()),
        ("1-9 0 [ ]", "first 12 effects".into()),
        ("m / e / d", "Mario / ECG / Dandelion".into()),
        ("f l b j", "Fish / Landscape / Bund / Gate".into()),
        ("c z k", "Traffic / Tornado / Storm".into()),
        ("o v g y", "Mona / Ballet / Eye / Life".into()),
        ("t", "cycle theme".into()),
        ("+ / -", format!("speed  {speed:.2}x")),
        ("r", "restart current effect".into()),
        ("i", "info / parameters".into()),
        ("h / ?", "toggle this panel".into()),
    ];
    for (i, (key, desc)) in rows.iter().enumerate() {
        let row = y + 3 + i as isize;
        if row >= y + bh - 3 {
            break;
        }
        let desc = truncate(desc, (bw - 16).max(1) as usize);
        canvas.put_str(x + 2, row, key, theme.c(3));
        canvas.put_str(x + 15, row, &desc, theme.c(2).scale(0.95));
    }

    if bw >= 74 {
        canvas.put_str(x + 44, y + 2, "EFFECTS", theme.c(1).scale(1.5));
        let half = (EffectId::ALL.len() + 1) / 2;
        for (i, e) in EffectId::ALL.iter().enumerate() {
            let (col, erow) = if i < half { (0, i) } else { (1, i - half) };
            let ex = x + 44 + col as isize * 16;
            let ey = y + 3 + erow as isize;
            if ey >= y + bh - 3 {
                break;
            }
            let marker = if *e == id { ">" } else { " " };
            let color = if *e == id {
                theme.hot()
            } else {
                e.default_theme().c(2)
            };
            canvas.put_str(ex, ey, marker, color);
            canvas.put_str(ex + 2, ey, e.key(), color);
            let badge = e.charset_support().badge();
            canvas.put_str(
                ex + 13,
                ey,
                badge,
                if e.charset_support() == crate::meta::CharsetSupport::Dual {
                    theme.c(3).scale(0.8)
                } else {
                    theme.c(1).scale(0.8)
                },
            );
        }
        canvas.put_str(
            x + 44,
            y + bh - 2,
            "A ascii   A+U dual",
            theme.c(1).scale(1.2),
        );
    }

    let summary = truncate(
        &format!("{} · {}", id.title_en(), id.title_zh()),
        (bw - 4).max(1) as usize,
    );
    canvas.put_str(x + 2, y + bh - 3, &summary, theme.c(3).scale(0.85));
    let footer = truncate(
        "trix --help <effect> for details  |  Esc closes panels",
        (bw - 3).max(1) as usize,
    );
    canvas.put_str(x + 2, y + bh - 2, &footer, theme.c(1).scale(1.4));
}

pub struct PlaylistInfo {
    pub index: usize,
    pub total: usize,
    pub switch_secs: f32,
    pub remaining: u32,
    pub transition: &'static str,
}

pub struct Info<'a> {
    pub effect: EffectId,
    /// Measured frames per second.
    pub fps: f32,
    /// Current adaptive ceiling (equal to the requested `--fps` when the
    /// governor is not throttling).
    pub target_fps: f32,
    /// Terminal traffic averaged over the last half second.
    pub ansi_kib_s: f32,
    pub frame_ms: f32,
    pub mode: ColorMode,
    pub status: Option<&'a str>,
    pub playlist: Option<PlaylistInfo>,
    pub params: &'a [(String, String)],
}

/// On-demand `i` panel: runtime state plus the CLI knobs the current effect
/// actually uses.  This replaces the old one-line status strip and the CLI
/// reminders that used to live in scene footers.
pub fn draw_info(canvas: &mut Canvas, frame: &Frame, info: &Info) {
    let w = frame.w as isize;
    let h = frame.h as isize;
    let theme = frame.theme;
    if w < 30 || h < 8 {
        canvas.clear_rect(0, 1, w, 1);
        let line = format!(
            "i close | {} | {:.2}x | {} | {}",
            info.effect.key(),
            frame.speed,
            theme.key,
            charset_label(frame.charset)
        );
        let shown: String = line.chars().take(frame.w).collect();
        canvas.put_str(0, 1, &shown, theme.c(2));
        return;
    }

    let mut lines: Vec<(Option<&'static str>, String)> = Vec::new();
    lines.push((
        Some("EFFECT"),
        format!("{} ({})", info.effect.title_en(), info.effect.key()),
    ));
    lines.push((
        Some("RUNNING"),
        format!(
            "{:.0} fps (target {:.0}) | {:.1} ms | {:.0} KiB/s | {:.2}x | {} | {} | {}x{} | {}",
            info.fps,
            info.target_fps,
            info.frame_ms,
            info.ansi_kib_s,
            frame.speed,
            theme.key,
            charset_label(frame.charset),
            frame.w,
            frame.h,
            info.mode.label()
        ),
    ));
    lines.push((Some("SEED"), frame.seed.to_string()));
    if let Some(status) = info.status {
        let status = status.trim();
        if !status.is_empty() {
            lines.push((Some("STATUS"), status.to_string()));
        }
    }
    if let Some(p) = &info.playlist {
        lines.push((
            Some("PLAYLIST"),
            format!(
                "{}/{} | {:.1}s | {} | next {}s",
                p.index + 1,
                p.total,
                p.switch_secs,
                p.transition,
                p.remaining
            ),
        ));
    }
    lines.push((None, String::new()));
    lines.push((Some("PARAMS"), String::new()));
    for (flag, value) in info.params {
        lines.push((None, format!("  {flag} {value}")));
    }
    lines.push((None, String::new()));
    lines.push((
        None,
        "KEYS  i / Esc close  |  h help  |  q quit".to_string(),
    ));

    let max_line = lines
        .iter()
        .map(|(label, value)| label.map(|l| width(l) + 10).unwrap_or(2) + width(value))
        .max()
        .unwrap_or(40);
    let bw = (max_line as isize + 6)
        .clamp(40, 78)
        .min((w - 4).max(24))
        .max(24);
    let bh = (lines.len() as isize + 4).min(h - 2).max(6);
    let x = (w - bw) / 2;
    let y = (h - bh) / 2;
    canvas.clear_rect(x, y, bw, bh);
    canvas.rect(x, y, bw, bh, theme.c(2).scale(0.65));
    canvas.put_str_gradient(
        x + 2,
        y + 1,
        "TRIX // INFO",
        &[theme.c(2), theme.c(3), theme.hot()],
    );

    let limit = y + bh - 1;
    let value_max = (bw - 13).max(1) as usize;
    let plain_max = (bw - 3).max(1) as usize;
    for (i, (label, value)) in lines.iter().enumerate() {
        let row = y + 3 + i as isize;
        if row >= limit {
            break;
        }
        match label {
            Some(label) => {
                let value = truncate(value, value_max);
                canvas.put_str(x + 2, row, label, theme.c(1).scale(1.4));
                canvas.put_str(x + 12, row, &value, theme.c(2).scale(0.95));
            }
            None => {
                let value = truncate(value, plain_max);
                canvas.put_str(x + 2, row, &value, theme.c(2).scale(0.9));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Tune;
    use crate::theme;

    fn frame<'a>(w: usize, h: usize, theme: &'a crate::theme::Theme) -> Frame<'a> {
        Frame {
            t: 0.0,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme,
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        }
    }

    #[test]
    fn header_fits_many_widths() {
        let theme = theme::find("neon").unwrap();
        for w in [8, 20, 40, 80, 160] {
            let mut canvas = Canvas::new(w, 24);
            let f = frame(w, 24, theme);
            draw_header(
                &mut canvas,
                &f,
                EffectId::Dragon,
                Some("CLOUD SEA // FLIGHT NOMINAL"),
            );
        }
    }

    #[test]
    fn keybar_fits_many_widths() {
        let theme = theme::find("neon").unwrap();
        for w in [16, 24, 40, 62, 100] {
            let mut canvas = Canvas::new(w, 12);
            let f = frame(w, 12, theme);
            draw_keybar(&mut canvas, &f, false);
            draw_keybar(&mut canvas, &f, true);
        }
    }

    #[test]
    fn info_panel_fits_small_screens() {
        let theme = theme::find("neon").unwrap();
        let params = vec![("--charset".to_string(), "ascii".to_string())];
        for (w, h) in [(20, 6), (40, 10), (80, 24), (160, 60)] {
            let mut canvas = Canvas::new(w, h);
            let f = frame(w, h, theme);
            let info = Info {
                effect: EffectId::Rain,
                fps: 30.0,
                target_fps: 30.0,
                ansi_kib_s: 128.0,
                frame_ms: 1.2,
                mode: ColorMode::True,
                status: None,
                playlist: None,
                params: &params,
            };
            draw_info(&mut canvas, &f, &info);
        }
    }

    #[test]
    fn truncate_uses_double_dot() {
        assert_eq!(truncate("abcdef", 4), "ab..");
        assert_eq!(truncate("abc", 4), "abc");
    }

    fn row(canvas: &Canvas, y: usize) -> String {
        canvas.cells[y * canvas.w..(y + 1) * canvas.w]
            .iter()
            .map(|c| c.ch)
            .collect()
    }

    #[test]
    fn header_shows_title_and_key() {
        let theme = theme::find("neon").unwrap();
        let mut canvas = Canvas::new(100, 24);
        let f = frame(100, 24, theme);
        draw_header(&mut canvas, &f, EffectId::Rain, Some("STARS 42"));
        let line = row(&canvas, 0);
        assert!(line.contains("Code Rain"), "{line:?}");
        assert!(line.contains("// rain"), "{line:?}");
        assert!(line.contains("STARS 42"), "{line:?}");
        assert!(line.contains("1.00x"), "{line:?}");
    }

    #[test]
    fn keybar_shows_quit_and_info() {
        let theme = theme::find("neon").unwrap();
        let mut canvas = Canvas::new(100, 24);
        let f = frame(100, 24, theme);
        draw_keybar(&mut canvas, &f, false);
        let line = row(&canvas, 23);
        assert!(line.contains("q quit"), "{line:?}");
        assert!(line.contains("space pause"), "{line:?}");
        assert!(line.contains("i info"), "{line:?}");
    }

    #[test]
    fn info_panel_shows_effect_and_params() {
        let theme = theme::find("neon").unwrap();
        let mut canvas = Canvas::new(100, 24);
        let f = frame(100, 24, theme);
        let params = vec![
            ("--seed-count".to_string(), "90".to_string()),
            ("--seed-speed".to_string(), "1.00".to_string()),
        ];
        let info = Info {
            effect: EffectId::Dandelion,
            fps: 30.0,
            target_fps: 30.0,
            ansi_kib_s: 128.0,
            frame_ms: 1.2,
            mode: ColorMode::True,
            status: Some("SEEDS 090 | WIND 1.0x"),
            playlist: None,
            params: &params,
        };
        draw_info(&mut canvas, &f, &info);
        let all: String = (0..24).map(|y| row(&canvas, y)).collect();
        assert!(all.contains("Dandelion (dandelion)"), "{all:?}");
        assert!(all.contains("SEEDS 090"), "{all:?}");
        assert!(all.contains("--seed-count 90"), "{all:?}");
        assert!(all.contains("target 30"), "{all:?}");
        assert!(all.contains("KiB/s"), "{all:?}");
        assert!(all.contains("i / Esc close"), "{all:?}");
    }

    #[test]
    fn pause_and_playlist_show_expected_words() {
        let theme = theme::find("neon").unwrap();
        let mut canvas = Canvas::new(100, 24);
        let f = frame(100, 24, theme);
        draw_pause(&mut canvas, &f);
        let all: String = (0..24).map(|y| row(&canvas, y)).collect();
        assert!(all.contains("PAUSED"), "{all:?}");
        assert!(all.contains("space to resume"), "{all:?}");

        let mut canvas = Canvas::new(100, 24);
        draw_playlist(&mut canvas, &f, EffectId::Rain, 12.0, 3.0);
        let line = row(&canvas, 22);
        assert!(line.contains("playlist 1/32"), "{line:?}");
        assert!(line.contains("next 9s"), "{line:?}");
    }

    #[test]
    fn help_panel_fits_small_and_large_screens() {
        let theme = theme::find("neon").unwrap();
        for (w, h) in [(34, 10), (40, 12), (80, 24), (160, 60)] {
            let mut canvas = Canvas::new(w, h);
            let f = frame(w, h, theme);
            draw_help(&mut canvas, &f, EffectId::Dragon, 1.0, false);
        }
    }
}
