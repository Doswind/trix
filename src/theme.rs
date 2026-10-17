//! Color themes shared by every effect.

use crate::color::Rgb;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub key: &'static str,
    pub label: &'static str,
    pub colors: [Rgb; 5],
}

impl Theme {
    #[inline]
    pub fn c(&self, i: usize) -> Rgb {
        self.colors[i.min(self.colors.len() - 1)]
    }

    /// Darkest anchor, usually used for haze or unlit structures.
    #[inline]
    pub fn shadow(&self) -> Rgb {
        self.colors[0]
    }

    #[inline]
    pub fn accent(&self) -> Rgb {
        self.colors[2]
    }

    #[inline]
    pub fn hot(&self) -> Rgb {
        self.colors[4]
    }

    /// Smoothly sample the palette as a loop. `phase` is in turns.
    pub fn sample(&self, phase: f32) -> Rgb {
        let n = self.colors.len();
        let p = phase.rem_euclid(1.0) * n as f32;
        let i = p.floor() as usize % n;
        let j = (i + 1) % n;
        self.colors[i].lerp(self.colors[j], p.fract())
    }
}

pub static THEMES: &[Theme] = &[
    Theme {
        key: "neon",
        label: "Neon cyan / violet",
        colors: [
            Rgb::hex(0x032b3a),
            Rgb::hex(0x0891b2),
            Rgb::hex(0x22d3ee),
            Rgb::hex(0x8b5cf6),
            Rgb::hex(0xe0f7ff),
        ],
    },
    Theme {
        key: "matrix",
        label: "Matrix green",
        colors: [
            Rgb::hex(0x02170c),
            Rgb::hex(0x0b6b3a),
            Rgb::hex(0x22c55e),
            Rgb::hex(0x86efac),
            Rgb::hex(0xf2fff6),
        ],
    },
    Theme {
        key: "ember",
        label: "Fire ember",
        colors: [
            Rgb::hex(0x1a0500),
            Rgb::hex(0x991b00),
            Rgb::hex(0xff6a00),
            Rgb::hex(0xffc21a),
            Rgb::hex(0xfff6d8),
        ],
    },
    Theme {
        key: "aurora",
        label: "Jade dragon / aurora",
        colors: [
            Rgb::hex(0x04131f),
            Rgb::hex(0x0f766e),
            Rgb::hex(0x22d3ee),
            Rgb::hex(0xfbbf24),
            Rgb::hex(0xfffbe8),
        ],
    },
    Theme {
        key: "ice",
        label: "Hologram ice",
        colors: [
            Rgb::hex(0x041424),
            Rgb::hex(0x0e5c88),
            Rgb::hex(0x38bdf8),
            Rgb::hex(0xa5f3fc),
            Rgb::hex(0xf2feff),
        ],
    },
    Theme {
        key: "plasma",
        label: "Plasma magenta",
        colors: [
            Rgb::hex(0x1a0326),
            Rgb::hex(0x8b1a6b),
            Rgb::hex(0xec4899),
            Rgb::hex(0xfb923c),
            Rgb::hex(0xfff1c8),
        ],
    },
    Theme {
        key: "gold",
        label: "Imperial gold",
        colors: [
            Rgb::hex(0x1c0d00),
            Rgb::hex(0x92400e),
            Rgb::hex(0xf59e0b),
            Rgb::hex(0xfde68a),
            Rgb::hex(0xfffdf2),
        ],
    },
    Theme {
        key: "chrome",
        label: "Monochrome chrome",
        colors: [
            Rgb::hex(0x0a0a0a),
            Rgb::hex(0x404040),
            Rgb::hex(0x9ca3af),
            Rgb::hex(0xe5e7eb),
            Rgb::hex(0xffffff),
        ],
    },
];

pub fn find(key: &str) -> Option<&'static Theme> {
    THEMES
        .iter()
        .find(|t| t.key.eq_ignore_ascii_case(key) || t.label.eq_ignore_ascii_case(key))
}

pub fn index_of(theme: &Theme) -> usize {
    THEMES.iter().position(|t| t.key == theme.key).unwrap_or(0)
}

pub fn next_theme(theme: &Theme) -> &'static Theme {
    let i = index_of(theme);
    &THEMES[(i + 1) % THEMES.len()]
}

pub fn theme_keys() -> String {
    THEMES.iter().map(|t| t.key).collect::<Vec<_>>().join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_and_cycle() {
        assert_eq!(find("NEON").unwrap().key, "neon");
        assert!(find("nope").is_none());
        let mut t = &THEMES[0];
        for _ in 0..=THEMES.len() {
            t = next_theme(t);
            assert!(t.colors[2].luminance() > 0.0);
        }
    }

    #[test]
    fn sampling_is_continuous() {
        let t = &THEMES[0];
        let a = t.sample(0.0);
        let b = t.sample(0.0001);
        assert!((a.r as i32 - b.r as i32).abs() < 5);
    }
}
