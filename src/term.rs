//! Minimal Unix terminal driver: raw mode, size, non-blocking keys, cleanup.
//!
//! Deliberately uses `libc` directly instead of a terminal framework: the
//! binary stays small, start-up is a couple of syscalls, and there is no
//! event-loop machinery we do not need.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::canvas::Canvas;
use crate::color::ColorMode;

static SAVED_TERMIOS: Mutex<Option<libc::termios>> = Mutex::new(None);
static QUIT: AtomicBool = AtomicBool::new(false);
static PANIC_HOOK_SET: AtomicBool = AtomicBool::new(false);

extern "C" fn on_quit_signal(_sig: libc::c_int) {
    QUIT.store(true, Ordering::Relaxed);
}

pub fn quit_requested() -> bool {
    QUIT.load(Ordering::Relaxed)
}

pub fn is_tty(fd: i32) -> bool {
    unsafe { libc::isatty(fd) == 1 }
}

fn emergency_restore() {
    const RESTORE: &[u8] = b"\x1b[0m\x1b[?7h\x1b[?25h\x1b[?1049l";
    unsafe {
        libc::write(1, RESTORE.as_ptr() as *const libc::c_void, RESTORE.len());
        if let Ok(guard) = SAVED_TERMIOS.try_lock() {
            if let Some(t) = *guard {
                libc::tcsetattr(0, libc::TCSANOW, &t);
            }
        }
    }
}

fn install_panic_hook() {
    if PANIC_HOOK_SET.swap(true, Ordering::SeqCst) {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        emergency_restore();
        previous(info);
    }));
    unsafe {
        libc::signal(
            libc::SIGINT,
            on_quit_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGTERM,
            on_quit_signal as *const () as libc::sighandler_t,
        );
        libc::signal(
            libc::SIGHUP,
            on_quit_signal as *const () as libc::sighandler_t,
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Unknown,
}

pub struct Terminal {
    orig: libc::termios,
    raw_active: bool,
    alt_screen: bool,
    input: InputParser,
    pub width: usize,
    pub height: usize,
}

impl Terminal {
    pub fn enter(alt_screen: bool) -> io::Result<Self> {
        if !is_tty(0) || !is_tty(1) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "not a terminal (try --snapshot to render to a file)",
            ));
        }
        let mut orig: libc::termios = unsafe { std::mem::zeroed() };
        unsafe {
            if libc::tcgetattr(0, &mut orig) != 0 {
                return Err(io::Error::last_os_error());
            }
            if let Ok(mut guard) = SAVED_TERMIOS.lock() {
                *guard = Some(orig);
            }
            install_panic_hook();

            let mut raw = orig;
            libc::cfmakeraw(&mut raw);
            raw.c_cc[libc::VMIN] = 0;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(0, libc::TCSANOW, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }
            QUIT.store(false, Ordering::Relaxed);
            let write_result = {
                let mut out = io::stdout().lock();
                (|| -> io::Result<()> {
                    if alt_screen {
                        out.write_all(b"\x1b[?1049h")?;
                    }
                    out.write_all(b"\x1b[?25l\x1b[?7l\x1b[0m\x1b[40m\x1b[2J\x1b[H")?;
                    out.flush()
                })()
            };
            if let Err(e) = write_result {
                libc::tcsetattr(0, libc::TCSANOW, &orig);
                if let Ok(mut guard) = SAVED_TERMIOS.lock() {
                    *guard = None;
                }
                return Err(e);
            }
        }
        let (w, h) = size().unwrap_or((80, 24));
        Ok(Terminal {
            orig,
            raw_active: true,
            alt_screen,
            input: InputParser::default(),
            width: w,
            height: h,
        })
    }

    pub fn restore(&mut self) {
        if !self.raw_active {
            return;
        }
        self.raw_active = false;
        unsafe {
            libc::tcsetattr(0, libc::TCSANOW, &self.orig);
        }
        let mut out = io::stdout().lock();
        let _ = out.write_all(b"\x1b[0m\x1b[?7h\x1b[?25h");
        if self.alt_screen {
            let _ = out.write_all(b"\x1b[?1049l");
        }
        let _ = out.flush();
    }

    /// Re-query the terminal size. Returns true when it changed.
    pub fn refresh_size(&mut self) -> bool {
        let (w, h) = size().unwrap_or((self.width, self.height));
        if w != self.width || h != self.height {
            self.width = w;
            self.height = h;
            true
        } else {
            false
        }
    }

    /// Clear the screen and force the next canvas frame to repaint.
    pub fn clear(&mut self) {
        let mut out = io::stdout().lock();
        let _ = out.write_all(b"\x1b[0m\x1b[40m\x1b[2J\x1b[H");
        let _ = out.flush();
    }

    pub fn present(&mut self, canvas: &mut Canvas, mode: ColorMode) -> io::Result<usize> {
        let mut out = io::stdout().lock();
        canvas.present(&mut out, mode)
    }

    /// Non-blocking read of all currently available input bytes.
    ///
    /// Bytes go through a small state machine so escape sequences split
    /// across `read()` calls (slow SSH links, macOS Terminal, Linux consoles)
    /// are decoded as one key instead of a spurious `Esc`.
    pub fn read_keys(&mut self) -> Vec<Key> {
        let mut buf = [0u8; 256];
        let n = unsafe { libc::read(0, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n > 0 {
            self.input.push(&buf[..n as usize]);
        }
        self.input.drain(Instant::now())
    }

    pub fn sleep_until(&self, deadline: std::time::Instant) {
        let now = std::time::Instant::now();
        if deadline > now {
            std::thread::sleep(deadline - now);
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.restore();
        if let Ok(mut guard) = SAVED_TERMIOS.lock() {
            *guard = None;
        }
    }
}

pub fn size() -> Option<(usize, usize)> {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(1, libc::TIOCGWINSZ, &mut ws) == 0 {
            let w = ws.ws_col as usize;
            let h = ws.ws_row as usize;
            if w > 1 && h > 1 {
                return Some((w, h));
            }
        }
    }
    let w = std::env::var("COLUMNS").ok()?.parse().ok()?;
    let h = std::env::var("LINES").ok()?.parse().ok()?;
    if w > 1 && h > 1 {
        Some((w, h))
    } else {
        None
    }
}

/// How long to wait for the rest of an escape sequence before deciding that
/// a lone `ESC` really was the Escape key.
const ESC_TIMEOUT: Duration = Duration::from_millis(40);
/// How long to wait for the tail of a malformed / incomplete sequence.
const SEQ_TIMEOUT: Duration = Duration::from_millis(140);

enum Csi {
    /// A decoded key (possibly `Unknown`) and the number of bytes consumed.
    Key(Key, usize),
    /// `ESC [ 200 ~`: everything up to `ESC [ 201 ~` is pasted text.
    Paste,
    NeedMore,
}

/// Stateful byte -> key decoder.
///
/// Deliberately terminal-agnostic: it accepts both CSI (`ESC [`) and SS3
/// (`ESC O`) cursor keys, ignores modifiers and private parameter prefixes,
/// and never lets an `Alt`/`Option` chord or a mouse report turn into a stray
/// shortcut.
#[derive(Default)]
struct InputParser {
    buf: Vec<u8>,
    last_feed: Option<Instant>,
    paste: bool,
}

impl InputParser {
    fn push(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.buf.extend_from_slice(bytes);
        self.last_feed = Some(Instant::now());
    }

    fn drain(&mut self, now: Instant) -> Vec<Key> {
        let mut keys = Vec::new();
        loop {
            if self.buf.is_empty() {
                break;
            }
            if self.paste {
                if let Some(pos) = find_subslice(&self.buf, b"\x1b[201~") {
                    self.buf.drain(..pos + 6);
                    self.paste = false;
                    continue;
                }
                // Keep a small tail in case the end marker is split.
                let keep = self.buf.len().min(5);
                if self.buf.len() > keep {
                    self.buf.drain(..self.buf.len() - keep);
                }
                break;
            }
            match self.buf[0] {
                0x1b => match self.buf.get(1).copied() {
                    None => {
                        if self.timed_out(now, ESC_TIMEOUT) {
                            self.buf.remove(0);
                            keys.push(Key::Esc);
                            continue;
                        }
                        break;
                    }
                    Some(b'[') => match self.parse_csi(now) {
                        Csi::Key(key, used) => {
                            self.buf.drain(..used);
                            keys.push(key);
                        }
                        Csi::Paste => {
                            self.buf.drain(..6);
                            self.paste = true;
                        }
                        Csi::NeedMore => break,
                    },
                    Some(b'O') => {
                        if self.buf.len() < 3 {
                            if self.timed_out(now, SEQ_TIMEOUT) {
                                self.buf.clear();
                            }
                            break;
                        }
                        let key = match self.buf[2] {
                            b'A' => Key::Up,
                            b'B' => Key::Down,
                            b'C' => Key::Right,
                            b'D' => Key::Left,
                            b'H' => Key::Home,
                            b'F' => Key::End,
                            _ => Key::Unknown,
                        };
                        self.buf.drain(..3);
                        keys.push(key);
                    }
                    Some(b']' | b'P' | b'_' | b'^') => {
                        if let Some(used) = string_sequence_len(&self.buf) {
                            self.buf.drain(..used);
                        } else if self.timed_out(now, SEQ_TIMEOUT) {
                            self.buf.clear();
                        } else {
                            break;
                        }
                    }
                    Some(next) => {
                        // Alt / Option chord.  Treat it as "not a shortcut"
                        // instead of `Esc` + the key, which used to quit.
                        let used = 1 + utf8_len(next).unwrap_or(1);
                        if self.buf.len() < used {
                            if self.timed_out(now, SEQ_TIMEOUT) {
                                self.buf.clear();
                            }
                            break;
                        }
                        self.buf.drain(..used);
                        keys.push(Key::Unknown);
                    }
                },
                0x03 => {
                    self.buf.remove(0);
                    keys.push(Key::Char('\u{3}'));
                }
                0x00..=0x1f | 0x7f => {
                    // Enter/Tab/Backspace/other C0 controls are not shortcuts.
                    self.buf.remove(0);
                    keys.push(Key::Unknown);
                }
                _ => {
                    let n = utf8_len(self.buf[0]).unwrap_or(1);
                    if self.buf.len() < n {
                        if self.timed_out(now, SEQ_TIMEOUT) {
                            self.buf.clear();
                        }
                        break;
                    }
                    match std::str::from_utf8(&self.buf[..n]) {
                        Ok(s) => {
                            let ch = s.chars().next().unwrap_or('\0');
                            self.buf.drain(..n);
                            if !ch.is_control() {
                                keys.push(Key::Char(ch));
                            }
                        }
                        Err(_) => {
                            self.buf.remove(0);
                            keys.push(Key::Unknown);
                        }
                    }
                }
            }
        }
        keys
    }

    fn timed_out(&self, now: Instant, timeout: Duration) -> bool {
        match self.last_feed {
            Some(t) => now.saturating_duration_since(t) >= timeout,
            None => true,
        }
    }

    fn parse_csi(&self, now: Instant) -> Csi {
        let mut i = 2;
        while i < self.buf.len() && (0x30..=0x3f).contains(&self.buf[i]) {
            i += 1;
        }
        while i < self.buf.len() && (0x20..=0x2f).contains(&self.buf[i]) {
            i += 1;
        }
        if i >= self.buf.len() {
            if self.timed_out(now, SEQ_TIMEOUT) {
                return Csi::Key(Key::Unknown, i.max(1));
            }
            return Csi::NeedMore;
        }
        let final_byte = self.buf[i];
        if !(0x40..=0x7e).contains(&final_byte) {
            return Csi::Key(Key::Unknown, i + 1);
        }
        let used = i + 1;
        let params = std::str::from_utf8(&self.buf[2..i]).unwrap_or("");
        if final_byte == b'~' {
            let first = params.split(';').next().unwrap_or("");
            match first {
                "200" => return Csi::Paste,
                "201" => return Csi::Key(Key::Unknown, used),
                _ => {}
            }
            let key = match first.parse::<u32>().unwrap_or(0) {
                1 | 7 => Key::Home,
                4 | 8 => Key::End,
                5 => Key::PageUp,
                6 => Key::PageDown,
                _ => Key::Unknown,
            };
            return Csi::Key(key, used);
        }
        // Mouse reports (`ESC [ < ... M`) and focus events (`ESC [ I/O`) are
        // not shortcuts; modifiers on arrows (params like `1;2`) are ignored.
        if params.starts_with('<') && (final_byte == b'M' || final_byte == b'm') {
            return Csi::Key(Key::Unknown, used);
        }
        let key = match final_byte {
            b'A' => Key::Up,
            b'B' => Key::Down,
            b'C' => Key::Right,
            b'D' => Key::Left,
            b'H' => Key::Home,
            b'F' => Key::End,
            _ => Key::Unknown,
        };
        Csi::Key(key, used)
    }
}

fn utf8_len(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7f => Some(1),
        0xc0..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf7 => Some(4),
        _ => None,
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Length of an OSC/DCS/APC/PM string sequence, including its terminator.
fn string_sequence_len(bytes: &[u8]) -> Option<usize> {
    let mut i = 1;
    while i < bytes.len() {
        if bytes[i] == 0x07 {
            return Some(i + 1);
        }
        if bytes[i] == 0x1b && bytes.get(i + 1) == Some(&b'\\') {
            return Some(i + 2);
        }
        i += 1;
    }
    None
}

/// Convenience wrapper used by the unit tests below.
#[cfg(test)]
fn parse_keys(bytes: &[u8]) -> Vec<Key> {
    let mut parser = InputParser::default();
    parser.push(bytes);
    parser.drain(Instant::now() + Duration::from_millis(500))
}

/// Small helper for tests and non-interactive demo pacing.
pub fn fixed_frame(dt: f32) -> Duration {
    Duration::from_secs_f32(dt.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_keys() {
        assert_eq!(
            parse_keys(b"qn3"),
            vec![Key::Char('q'), Key::Char('n'), Key::Char('3')]
        );
    }

    #[test]
    fn parses_escape_sequences() {
        assert_eq!(parse_keys(b"\x1b[A"), vec![Key::Up]);
        assert_eq!(parse_keys(b"\x1b[6~"), vec![Key::PageDown]);
        assert_eq!(parse_keys(b"\x1b"), vec![Key::Esc]);
        assert_eq!(parse_keys(b"\x1bOH"), vec![Key::Home]);
    }

    #[test]
    fn ctrl_c_passes_through() {
        assert_eq!(parse_keys(&[0x03]), vec![Key::Char('\u{3}')]);
    }

    #[test]
    fn modified_arrows_do_not_leak_digits_or_letters() {
        assert_eq!(parse_keys(b"\x1b[1;2A"), vec![Key::Up]);
        assert_eq!(parse_keys(b"\x1b[1;5C"), vec![Key::Right]);
        assert_eq!(parse_keys(b"\x1b[1;3D"), vec![Key::Left]);
        assert_eq!(parse_keys(b"\x1b[1;2H"), vec![Key::Home]);
    }

    #[test]
    fn split_escape_sequence_is_not_escape() {
        let mut parser = InputParser::default();
        parser.push(b"\x1b");
        assert!(parser.drain(Instant::now()).is_empty());
        parser.push(b"[A");
        assert_eq!(parser.drain(Instant::now()), vec![Key::Up]);
    }

    #[test]
    fn lone_escape_fires_after_timeout() {
        let mut parser = InputParser::default();
        parser.push(b"\x1b");
        assert!(parser.drain(Instant::now()).is_empty());
        assert_eq!(
            parser.drain(Instant::now() + Duration::from_millis(100)),
            vec![Key::Esc]
        );
    }

    #[test]
    fn alt_chords_are_ignored_not_escape() {
        // macOS Option-as-Meta sends ESC + the key; it must never quit.
        assert_eq!(parse_keys(b"\x1bq"), vec![Key::Unknown]);
        assert_eq!(parse_keys("é".as_bytes()), vec![Key::Char('é')]);
    }

    #[test]
    fn mouse_and_paste_reports_are_ignored() {
        assert_eq!(parse_keys(b"\x1b[<0;10;5M"), vec![Key::Unknown]);
        assert_eq!(parse_keys(b"\x1b[<0;10;5m"), vec![Key::Unknown]);
        assert!(parse_keys(b"\x1b[200~q n 3\x1b[201~").is_empty());
    }

    #[test]
    fn osc_titles_and_focus_events_are_ignored() {
        assert!(parse_keys(b"\x1b]0;title\x07").is_empty());
        assert_eq!(parse_keys(b"\x1b[I"), vec![Key::Unknown]);
        assert_eq!(parse_keys(b"\x1b[O"), vec![Key::Unknown]);
    }

    #[test]
    fn uppercase_letters_survive_for_the_shortcut_layer() {
        // The app lowercases these, so Shift and Caps Lock keep working.
        assert_eq!(parse_keys(b"Q"), vec![Key::Char('Q')]);
        assert_eq!(parse_keys(b"N"), vec![Key::Char('N')]);
    }
}
