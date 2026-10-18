//! Effect-to-effect transitions.
//!
//! A terminal cell can only change its glyph and its colour, so a transition
//! is built from those two moves.  `fade` dims the outgoing scene to black and
//! then brings the incoming one up; `dissolve` flips cells from one scene to
//! the other on an ordered-dither threshold, which reads as a cross-fade.
//!
//! Both scenes stay live for the whole transition: the outgoing scene keeps
//! advancing from the time it was left at, and the incoming scene starts at
//! `t = 0` immediately, so it has already warmed up (rain has columns falling,
//! fire has flames) by the time it becomes visible.

use crate::canvas::{Canvas, Cell};
use crate::cli::Charset;
use crate::color::Rgb;
use crate::scene::Scene;
use crate::theme::Theme;

/// How one effect hands over to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionMode {
    /// Instant cut (the pre-transition behaviour, with a short flash).
    None,
    /// Fade out to black, then fade the next effect in.
    Fade,
    /// Cross-dissolve the two scenes cell by cell.
    Dissolve,
}

impl TransitionMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "none" | "off" | "instant" | "cut" | "hard" => Some(Self::None),
            "fade" | "dip" | "blend" | "fade-out" => Some(Self::Fade),
            "dissolve" | "crossfade" | "cross-fade" | "melt" => Some(Self::Dissolve),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Fade => "fade",
            Self::Dissolve => "dissolve",
        }
    }
}

/// 8x8 ordered-dither matrix (Bayer), values 0..=63.
const BAYER8: [[u8; 8]; 8] = [
    [0, 32, 8, 40, 2, 34, 10, 42],
    [48, 16, 56, 24, 50, 18, 58, 26],
    [12, 44, 4, 36, 14, 46, 6, 38],
    [60, 28, 52, 20, 62, 30, 54, 22],
    [3, 35, 11, 43, 1, 33, 9, 41],
    [51, 19, 59, 27, 49, 17, 57, 25],
    [15, 47, 7, 39, 13, 45, 5, 37],
    [63, 31, 55, 23, 61, 29, 53, 21],
];

/// The scene that is fading out, plus its own clock and look.
pub struct Transition {
    pub mode: TransitionMode,
    pub duration: f32,
    pub elapsed: f32,
    pub from: Box<dyn Scene>,
    pub from_t: f64,
    pub from_seed: u64,
    pub from_theme: &'static Theme,
    pub from_charset: Charset,
}

impl Transition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mode: TransitionMode,
        duration: f32,
        from: Box<dyn Scene>,
        from_t: f64,
        from_seed: u64,
        from_theme: &'static Theme,
        from_charset: Charset,
    ) -> Self {
        Transition {
            mode,
            duration: duration.max(0.0),
            elapsed: 0.0,
            from,
            from_t,
            from_seed,
            from_theme,
            from_charset,
        }
    }

    /// 0 at the start of the transition, 1 when the new scene owns the frame.
    pub fn progress(&self) -> f32 {
        if self.duration <= f32::EPSILON {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Advance on wall-clock time; the outgoing scene advances on animation
    /// time, which is what freezes its content while paused.
    pub fn tick(&mut self, real_dt: f32, scene_dt: f32) {
        self.elapsed += real_dt.max(0.0);
        self.from_t += scene_dt.max(0.0) as f64;
    }

    /// Blend the two already-rendered scenes into `dst`.
    pub fn compose(&self, dst: &mut Canvas, a: &Canvas, b: &Canvas) {
        let p = self.progress();
        match self.mode {
            TransitionMode::None => {
                for i in 0..dst.cells.len() {
                    dst.cells[i] = b.cells.get(i).copied().unwrap_or(Cell::EMPTY);
                }
            }
            TransitionMode::Fade => compose_fade(dst, a, b, p),
            TransitionMode::Dissolve => compose_dissolve(dst, a, b, p),
        }
    }
}

/// Fade out during the first half, fade the replacement in during the second.
fn compose_fade(dst: &mut Canvas, a: &Canvas, b: &Canvas, p: f32) {
    let (src, k) = if p < 0.5 {
        (a, smoothstep(1.0 - p * 2.0))
    } else {
        (b, smoothstep((p - 0.5) * 2.0))
    };
    for i in 0..dst.cells.len() {
        let cell = src.cells.get(i).copied().unwrap_or(Cell::EMPTY);
        dst.cells[i] = dim(cell, k);
    }
}

/// Scale a cell towards black.  Fully dimmed cells become empty so the
/// terminal falls back to its own background instead of painting black cells.
fn dim(cell: Cell, k: f32) -> Cell {
    if k <= 0.004 || (cell.ch == ' ' && cell.bg.is_none()) {
        return Cell::EMPTY;
    }
    Cell {
        ch: cell.ch,
        fg: if cell.ch == ' ' {
            Rgb::BLACK
        } else {
            cell.fg.scale(k)
        },
        bg: if k < 0.04 {
            None
        } else {
            cell.bg.map(|bg| bg.scale(k))
        },
    }
}

/// Ordered-dither cross-fade.  The glyph comes from whichever scene wins the
/// cell, the colour is interpolated when both sides have a glyph there.
fn compose_dissolve(dst: &mut Canvas, a: &Canvas, b: &Canvas, p: f32) {
    let w = dst.w.max(1);
    for i in 0..dst.cells.len() {
        let threshold = (BAYER8[(i / w) & 7][(i % w) & 7] as f32 + 0.5) / 64.0;
        let ac = a.cells.get(i).copied().unwrap_or(Cell::EMPTY);
        let bc = b.cells.get(i).copied().unwrap_or(Cell::EMPTY);
        let mut cell = if p > threshold { bc } else { ac };
        if ac.ch != ' ' && bc.ch != ' ' {
            cell.fg = ac.fg.lerp(bc.fg, p);
        }
        cell.bg = match (ac.bg, bc.bg) {
            (Some(x), Some(y)) => Some(x.lerp(y, p)),
            _ => cell.bg,
        };
        dst.cells[i] = if cell.ch == ' ' && cell.bg.is_none() {
            Cell::EMPTY
        } else {
            cell
        };
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{EffectId, Frame, Tune};

    fn solid(w: usize, h: usize, ch: char, fg: Rgb) -> Canvas {
        let mut c = Canvas::new(w, h);
        for cell in c.cells.iter_mut() {
            *cell = Cell::new(ch, fg);
        }
        c
    }

    struct DummyScene;

    impl Scene for DummyScene {
        fn resize(&mut self, _w: usize, _h: usize) {}
        fn update(&mut self, _frame: &Frame) {}
        fn draw(&mut self, _canvas: &mut Canvas, _frame: &Frame) {}
    }

    fn transition(mode: TransitionMode, duration: f32) -> Transition {
        Transition::new(
            mode,
            duration,
            Box::new(DummyScene),
            0.0,
            1,
            EffectId::Rain.default_theme(),
            Charset::Ascii,
        )
    }

    #[test]
    fn parses_transition_modes() {
        assert_eq!(TransitionMode::parse("fade"), Some(TransitionMode::Fade));
        assert_eq!(TransitionMode::parse("FADE"), Some(TransitionMode::Fade));
        assert_eq!(
            TransitionMode::parse("crossfade"),
            Some(TransitionMode::Dissolve)
        );
        assert_eq!(TransitionMode::parse("none"), Some(TransitionMode::None));
        assert_eq!(TransitionMode::parse("off"), Some(TransitionMode::None));
        assert_eq!(TransitionMode::parse("sideways"), None);
    }

    #[test]
    fn fade_starts_on_the_old_scene_and_ends_on_the_new_one() {
        let a = solid(6, 4, 'A', Rgb::new(255, 0, 0));
        let b = solid(6, 4, 'B', Rgb::new(0, 0, 255));

        let mut dst = Canvas::new(6, 4);
        compose_fade(&mut dst, &a, &b, 0.0);
        assert!(dst
            .cells
            .iter()
            .all(|c| c.ch == 'A' && c.fg == a.cells[0].fg));

        compose_fade(&mut dst, &a, &b, 0.5);
        assert!(
            dst.cells.iter().all(|c| *c == Cell::EMPTY),
            "the dip to black happens exactly at the midpoint"
        );

        compose_fade(&mut dst, &a, &b, 1.0);
        assert!(dst
            .cells
            .iter()
            .all(|c| c.ch == 'B' && c.fg == b.cells[0].fg));
    }

    #[test]
    fn dissolve_mixes_both_scenes() {
        let a = solid(8, 8, 'A', Rgb::new(255, 0, 0));
        let b = solid(8, 8, 'B', Rgb::new(0, 0, 255));
        let mut dst = Canvas::new(8, 8);

        compose_dissolve(&mut dst, &a, &b, 0.0);
        assert!(dst.cells.iter().all(|c| c.ch == 'A'));

        compose_dissolve(&mut dst, &a, &b, 0.5);
        let old = dst.cells.iter().filter(|c| c.ch == 'A').count();
        let new = dst.cells.iter().filter(|c| c.ch == 'B').count();
        assert_eq!(old + new, 64);
        assert!(old > 0 && new > 0, "{old} old / {new} new");

        compose_dissolve(&mut dst, &a, &b, 1.0);
        assert!(dst.cells.iter().all(|c| c.ch == 'B'));
    }

    #[test]
    fn one_by_one_canvas_does_not_panic() {
        let a = solid(1, 1, 'A', Rgb::WHITE);
        let b = solid(1, 1, 'B', Rgb::WHITE);
        let mut dst = Canvas::new(1, 1);
        compose_fade(&mut dst, &a, &b, 0.25);
        compose_dissolve(&mut dst, &a, &b, 0.75);
    }

    #[test]
    fn blends_real_scenes_without_losing_ascii() {
        let theme = EffectId::Rain.default_theme();
        let (w, h) = (72, 24);
        let mut old = crate::scene::build(EffectId::Rain, 11);
        let mut new = crate::scene::build(EffectId::Fire, 12);
        old.resize(w, h);
        new.resize(w, h);
        let mut a = Canvas::new(w, h);
        let mut b = Canvas::new(w, h);
        let mut dst = Canvas::new(w, h);
        for step in 0..20 {
            let frame = Frame {
                t: step as f64 / 30.0,
                dt: 1.0 / 30.0,
                w,
                h,
                fps: 30,
                theme,
                charset: Charset::Ascii,
                seed: 11,
                speed: 1.0,
                tune: Tune::default(),
            };
            old.update(&frame);
            new.update(&frame);
            a.clear();
            old.draw(&mut a, &frame);
            b.clear();
            new.draw(&mut b, &frame);

            let p = step as f32 / 19.0;
            compose_fade(&mut dst, &a, &b, p);
            assert!(dst.cells.iter().all(|c| c.ch.is_ascii()));
            compose_dissolve(&mut dst, &a, &b, p);
            assert!(dst.cells.iter().all(|c| c.ch.is_ascii()));
        }
    }

    #[test]
    fn progress_and_finish_follow_wall_clock() {
        let mut tr = transition(TransitionMode::Fade, 0.5);
        assert_eq!(tr.progress(), 0.0);
        assert!(!tr.finished());
        tr.tick(0.25, 0.5);
        assert!((tr.progress() - 0.5).abs() < 1e-6);
        assert!((tr.from_t - 0.5).abs() < 1e-9);
        tr.tick(0.5, 0.0);
        assert!(tr.finished());
        assert_eq!(tr.progress(), 1.0);
    }

    #[test]
    fn empty_duration_is_immediately_done() {
        let tr = transition(TransitionMode::Fade, 0.0);
        assert!(tr.finished());
        assert_eq!(tr.progress(), 1.0);
    }

    #[test]
    fn dimming_keeps_ascii_only() {
        let a = solid(4, 2, 'A', Rgb::WHITE);
        let b = solid(4, 2, 'B', Rgb::WHITE);
        let mut dst = Canvas::new(4, 2);
        for step in 0..=10 {
            compose_fade(&mut dst, &a, &b, step as f32 / 10.0);
            assert!(dst.cells.iter().all(|c| c.ch.is_ascii()));
        }
    }
}
