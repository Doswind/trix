//! Color primitives and terminal color-space conversion.

use std::io::Write;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);
    pub const WHITE: Rgb = Rgb::new(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn hex(v: u32) -> Self {
        Self {
            r: ((v >> 16) & 0xff) as u8,
            g: ((v >> 8) & 0xff) as u8,
            b: (v & 0xff) as u8,
        }
    }

    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.rem_euclid(1.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let i = (h * 6.0).floor();
        let f = h * 6.0 - i;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));
        let (r, g, b) = match i as i32 % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };
        Self::new(
            (r * 255.0 + 0.5) as u8,
            (g * 255.0 + 0.5) as u8,
            (b * 255.0 + 0.5) as u8,
        )
    }

    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        let f = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t + 0.5) as u8;
        Rgb::new(f(self.r, other.r), f(self.g, other.g), f(self.b, other.b))
    }

    pub fn scale(self, f: f32) -> Rgb {
        let f = f.max(0.0);
        Rgb::new(
            (self.r as f32 * f).min(255.0) as u8,
            (self.g as f32 * f).min(255.0) as u8,
            (self.b as f32 * f).min(255.0) as u8,
        )
    }

    pub fn add(self, other: Rgb, f: f32) -> Rgb {
        Rgb::new(
            (self.r as f32 + other.r as f32 * f).min(255.0) as u8,
            (self.g as f32 + other.g as f32 * f).min(255.0) as u8,
            (self.b as f32 + other.b as f32 * f).min(255.0) as u8,
        )
    }

    pub fn blend(self, other: Rgb, t: f32) -> Rgb {
        self.lerp(other, t)
    }

    pub fn luminance(self) -> f32 {
        (0.2126 * self.r as f32 + 0.7152 * self.g as f32 + 0.0722 * self.b as f32) / 255.0
    }

    /// Rotate hue while keeping saturation/value roughly intact.
    pub fn hue_rotate(self, turns: f32) -> Rgb {
        let (h, s, v) = self.to_hsv();
        Rgb::from_hsv(h + turns, s, v)
    }

    pub fn to_hsv(self) -> (f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d <= f32::EPSILON {
            0.0
        } else if max == r {
            ((g - b) / d).rem_euclid(6.0) / 6.0
        } else if max == g {
            ((b - r) / d + 2.0) / 6.0
        } else {
            ((r - g) / d + 4.0) / 6.0
        };
        let s = if max <= f32::EPSILON { 0.0 } else { d / max };
        (h, s, max)
    }

    /// xterm-256 palette index (6x6x6 cube + grayscale ramp).
    pub fn ansi256(self) -> u8 {
        let q = |c: u8| -> u8 {
            // Nearest level of the standard 0,95,135,175,215,255 ramp.
            const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
            let mut best = 0;
            let mut best_d = i32::MAX;
            for (i, &l) in LEVELS.iter().enumerate() {
                let d = (l as i32 - c as i32).abs();
                if d < best_d {
                    best_d = d;
                    best = i as u8;
                }
            }
            best
        };
        let (qr, qg, qb) = (q(self.r), q(self.g), q(self.b));
        let cube = 16 + 36 * qr + 6 * qg + qb;
        // Compare against the grayscale ramp.
        if self.r == self.g && self.g == self.b {
            let v = self.r as i32;
            let gi = if v < 8 {
                0
            } else if v > 238 {
                23
            } else {
                ((v - 8) / 10).clamp(0, 23) as u8
            };
            let gray = 232 + gi;
            let gv = 8 + gi as i32 * 10;
            if (gv - v).abs() + 2 < (self.r as i32 - [0, 95, 135, 175, 215, 255][qr as usize]).abs()
            {
                return gray;
            }
        }
        cube
    }

    /// Closest of the 16 classic ANSI colors.
    pub fn ansi16(self) -> u8 {
        const PALETTE: [(u8, u8, u8); 16] = [
            (0, 0, 0),
            (128, 0, 0),
            (0, 128, 0),
            (128, 128, 0),
            (0, 0, 128),
            (128, 0, 128),
            (0, 128, 128),
            (192, 192, 192),
            (128, 128, 128),
            (255, 0, 0),
            (0, 255, 0),
            (255, 255, 0),
            (0, 0, 255),
            (255, 0, 255),
            (0, 255, 255),
            (255, 255, 255),
        ];
        let mut best = 0;
        let mut best_d = i32::MAX;
        for (i, &(r, g, b)) in PALETTE.iter().enumerate() {
            let d = (r as i32 - self.r as i32).pow(2)
                + (g as i32 - self.g as i32).pow(2)
                + (b as i32 - self.b as i32).pow(2);
            if d < best_d {
                best_d = d;
                best = i as u8;
            }
        }
        best
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMode {
    True,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => None,
            "true" | "truecolor" | "24bit" | "24-bit" | "rgb" => Some(ColorMode::True),
            "256" | "256color" | "ansi256" => Some(ColorMode::Ansi256),
            "16" | "16color" | "ansi16" | "ansi" => Some(ColorMode::Ansi16),
            "mono" | "none" | "off" | "ascii" => Some(ColorMode::Mono),
            _ => None,
        }
    }

    pub fn detect() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            return ColorMode::Mono;
        }
        if let Ok(ct) = std::env::var("COLORTERM") {
            let ct = ct.to_ascii_lowercase();
            if ct.contains("truecolor") || ct.contains("24bit") {
                return ColorMode::True;
            }
        }
        if let Ok(tp) = std::env::var("TERM_PROGRAM") {
            if matches!(
                tp.as_str(),
                "iTerm.app" | "WezTerm" | "Apple_Terminal" | "vscode" | "Hyper" | "ghostty"
            ) {
                return ColorMode::True;
            }
        }
        match std::env::var("TERM") {
            Ok(t) if t.contains("truecolor") || t.contains("direct") => ColorMode::True,
            Ok(t) if t.contains("256color") => ColorMode::Ansi256,
            _ => ColorMode::Ansi16,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ColorMode::True => "truecolor",
            ColorMode::Ansi256 => "256",
            ColorMode::Ansi16 => "16",
            ColorMode::Mono => "mono",
        }
    }

    pub fn is_color(self) -> bool {
        !matches!(self, ColorMode::Mono)
    }
}

/// Write the escape sequence selecting `color` as the foreground color.
pub fn write_fg(out: &mut Vec<u8>, color: Rgb, mode: ColorMode) {
    match mode {
        ColorMode::True => {
            let _ = write!(out, "\x1b[38;2;{};{};{}m", color.r, color.g, color.b);
        }
        ColorMode::Ansi256 => {
            let _ = write!(out, "\x1b[38;5;{}m", color.ansi256());
        }
        ColorMode::Ansi16 => {
            let i = color.ansi16();
            let code = if i < 8 { 30 + i } else { 90 + (i - 8) };
            let _ = write!(out, "\x1b[{}m", code);
        }
        ColorMode::Mono => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_roundtrip() {
        for &(h, s, v) in &[(0.0, 1.0, 1.0), (0.33, 0.8, 0.7), (0.75, 0.4, 0.9)] {
            let rgb = Rgb::from_hsv(h, s, v);
            let (h2, s2, v2) = rgb.to_hsv();
            assert!((h - h2).abs() < 0.02, "{h} vs {h2}");
            assert!((s - s2).abs() < 0.02);
            assert!((v - v2).abs() < 0.02);
        }
    }

    #[test]
    fn basic_colors() {
        assert_eq!(Rgb::from_hsv(0.0, 1.0, 1.0), Rgb::new(255, 0, 0));
        assert_eq!(Rgb::from_hsv(1.0 / 3.0, 1.0, 1.0), Rgb::new(0, 255, 0));
        assert_eq!(Rgb::from_hsv(2.0 / 3.0, 1.0, 1.0), Rgb::new(0, 0, 255));
        assert_eq!(Rgb::new(255, 255, 255).ansi16(), 15);
        assert_eq!(Rgb::new(0, 0, 0).ansi16(), 0);
    }

    #[test]
    fn ansi256_grayscale() {
        assert_eq!(Rgb::new(0, 0, 0).ansi256(), 16); // black cube entry
        let g = Rgb::new(128, 128, 128).ansi256();
        assert!((232..=255).contains(&g), "got {g}");
    }

    #[test]
    fn blend_endpoints() {
        let a = Rgb::new(10, 20, 30);
        let b = Rgb::new(200, 100, 50);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
    }
}
