//! Minimal Unix terminal driver: raw mode, size, non-blocking keys, cleanup.
//!
//! Deliberately uses `libc` directly instead of a terminal framework: the
//! binary stays small, start-up is a couple of syscalls, and there is no
//! event-loop machinery we do not need.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

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
        let (w, h) = query_size().unwrap_or((80, 24));
        Ok(Terminal {
            orig,
            raw_active: true,
            alt_screen,
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
        let (w, h) = query_size().unwrap_or((self.width, self.height));
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
    pub fn read_keys(&mut self) -> Vec<Key> {
        let mut buf = [0u8; 128];
        let n = unsafe { libc::read(0, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
        if n <= 0 {
            return Vec::new();
        }
        parse_keys(&buf[..n as usize])
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

fn query_size() -> Option<(usize, usize)> {
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

fn parse_keys(bytes: &[u8]) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0x1b {
            if i + 1 >= bytes.len() {
                keys.push(Key::Esc);
                i += 1;
                continue;
            }
            if bytes[i + 1] == b'[' || bytes[i + 1] == b'O' {
                if i + 2 >= bytes.len() {
                    keys.push(Key::Unknown);
                    i += 2;
                    continue;
                }
                let code = bytes[i + 2];
                match code {
                    b'A' => keys.push(Key::Up),
                    b'B' => keys.push(Key::Down),
                    b'C' => keys.push(Key::Right),
                    b'D' => keys.push(Key::Left),
                    b'H' => keys.push(Key::Home),
                    b'F' => keys.push(Key::End),
                    b'0'..=b'9' => {
                        // Extended sequence: ESC [ <n> ~
                        let mut n = (code - b'0') as u32;
                        let mut j = i + 3;
                        while j < bytes.len() && bytes[j].is_ascii_digit() {
                            n = n * 10 + (bytes[j] - b'0') as u32;
                            j += 1;
                        }
                        if j < bytes.len() && bytes[j] == b'~' {
                            match n {
                                1 | 7 => keys.push(Key::Home),
                                4 | 8 => keys.push(Key::End),
                                5 => keys.push(Key::PageUp),
                                6 => keys.push(Key::PageDown),
                                _ => keys.push(Key::Unknown),
                            }
                            i = j + 1;
                            continue;
                        }
                        keys.push(Key::Unknown);
                        i = j;
                        continue;
                    }
                    _ => keys.push(Key::Unknown),
                }
                i += 3;
                continue;
            }
            // Alt-<key> or a bare ESC followed by another key.
            keys.push(Key::Esc);
            i += 1;
            continue;
        }
        if b.is_ascii_graphic() || b == b' ' || b == b'\t' || b == 0x03 {
            keys.push(Key::Char(b as char));
        }
        i += 1;
    }
    keys
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
}
