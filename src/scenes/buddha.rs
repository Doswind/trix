//! 佛光普照 — a meditating Buddha radiating light.
//!
//! A gold seated figure sits at the centre of a slowly turning mandala.  The
//! head has a ushnisha, long ears and a serene face; a layered halo and rays
//! fill the screen, while lotus petals and motes drift through the aura.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};
use crate::theme::Theme;

struct Petal {
    x: f32,
    y: f32,
    vy: f32,
    phase: f32,
    ch: char,
    color: Rgb,
}

struct Mote {
    x: f32,
    y: f32,
    phase: f32,
    speed: f32,
}

const GOLD: Rgb = Rgb::hex(0xfbbf24);
const PALE: Rgb = Rgb::hex(0xfff7d6);
const AMBER: Rgb = Rgb::hex(0xd97706);
const PINK: Rgb = Rgb::hex(0xf9a8d4);

/// Brightness above 1.0 would be lost when scaling gold (the channels clamp at
/// 255), so any extra boost is spent bleaching the hue towards pale light.
/// The current rings stay at or below full gold, but this keeps the helper
/// usable if the aura is brightened again.
fn ring_color(level: f32) -> Rgb {
    if level <= 1.0 {
        GOLD.scale(level)
    } else {
        GOLD.lerp(PALE, ((level - 1.0) * 0.9).min(1.0))
    }
}

/// Vertical squash of the mandala rings.  Terminal cells are about twice as
/// tall as wide, so ~0.55 reads as a circle on screen.
const RING_SQUASH: f32 = 0.55;

static BUDDHA_ART: [&str; 19] = [
    r#"                                  _"#,
    r#"                               _ooOoo_"#,
    r#"                              o8888888o"#,
    r#"                              88" . "88"#,
    r#"                              (| -_- |)"#,
    r#"                              O\  =  /O"#,
    r#"                           ____/`---'\____"#,
    r#"                         .'  \\|     |//  `."#,
    r#"                        /  \\|||  :  |||//  \"#,
    r#"                       /  _||||| -:- |||||_  \"#,
    r#"                       |   | \\\  -  /'| |   |"#,
    r#"                       | \_|  `\`---'//  |_/ |"#,
    r#"                       \  .-\__ `-. -'__/-.  /"#,
    r#"                     ___`. .'  /--.--\  `. .'___"#,
    r#"                  ."" '<  `.___\_<|>_/___.' _> \""."#,
    r#"                 | | :  `- \`. ;`. _/; .'/ /  .' ; |"#,
    r#"                 \  \ `-.   \_\_`. _.'_/_/  -' _.' /"#,
    r#"       ===========`-.`___`-.__\ \___  /__.-'_.'_.-'================"#,
    r#"                               `=--=-'"#,
];

pub struct BuddhaScene {
    rng: Rng,
    w: usize,
    h: usize,
    petals: Vec<Petal>,
    motes: Vec<Mote>,
    stars: Vec<(f32, f32, f32, f32)>,
    /// Reused ring buffer: `(rx, ry, brightness)` per mandala layer.
    rings: Vec<(f32, f32, f32)>,
    /// Innermost light-ring radius that clears the whole figure and its halos.
    ring_min_rx: f32,
    cx: f32,
    head_y: f32,
    hr: f32,
    lap_y: f32,
}

impl BuddhaScene {
    pub fn new(seed: u64) -> Self {
        BuddhaScene {
            rng: Rng::new(seed ^ 0xb0dd_4a11),
            w: 0,
            h: 0,
            petals: Vec::new(),
            motes: Vec::new(),
            stars: Vec::new(),
            rings: Vec::with_capacity(8),
            ring_min_rx: 0.0,
            cx: 0.0,
            head_y: 0.0,
            hr: 0.0,
            lap_y: 0.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.cx = self.w as f32 * 0.5;
        self.hr = (self.h as f32 * 0.135).clamp(2.4, 5.2);
        self.head_y = self.h as f32 * 0.275;
        self.lap_y = self.h as f32 * 0.835;
        self.petals.clear();
        self.motes.clear();
        self.stars.clear();
        let np = (self.w * self.h / 90).clamp(24, 90);
        for _ in 0..np {
            self.petals.push(Petal {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                vy: self.rng.range(1.0, 4.0),
                phase: self.rng.range(0.0, TAU),
                ch: if self.rng.chance(0.5) { '*' } else { '\'' },
                color: if self.rng.chance(0.65) { GOLD } else { PINK },
            });
        }
        let nm = (self.w * self.h / 45).clamp(60, 240);
        for _ in 0..nm {
            self.motes.push(Mote {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                phase: self.rng.range(0.0, TAU),
                speed: self.rng.range(0.3, 1.4),
            });
        }
        let ns = (self.w * self.h / 35).clamp(60, 260);
        for _ in 0..ns {
            self.stars.push((
                self.rng.range(0.0, self.w as f32),
                self.rng.range(0.0, self.h as f32),
                self.rng.range(0.0, TAU),
                self.rng.range(0.15, 0.7),
            ));
        }
        self.ring_min_rx = self.ring_clearance();
    }

    /// Figure art lines for the current canvas height.
    fn art_lines(&self) -> &'static [&'static str] {
        if self.h >= 30 {
            &BUDDHA_ART
        } else if self.h >= 22 {
            &BUDDHA_ART[..14]
        } else {
            &BUDDHA_ART[..8]
        }
    }

    /// Innermost light-ring radius that clears the painted figure.
    ///
    /// For every point that is actually drawn (the art glyphs, the tight head
    /// halo, the large radiant halo around the head and the lotus seat) this
    /// takes the radius at which an ellipse would pass through it; the largest
    /// of those radii encloses the whole figure, so a ring at that radius can
    /// never overlap the statue or the halo above its head.
    fn ring_clearance(&self) -> f32 {
        let lines = self.art_lines();
        let art_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as isize;
        let art_h = lines.len() as isize;
        let x = ((self.w as isize - 2 - art_w) / 2).max(0);
        let y = ((self.h as isize - art_h) / 2).max(1);
        let head_x = (x + art_w / 2) as f32;
        let head_y = (y + 4) as f32;
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;

        let mut need = 0.0f32;
        let mut mark = |px: f32, py: f32| {
            let dx = px - cx;
            let dy = (py - cy) / RING_SQUASH;
            need = need.max((dx * dx + dy * dy).sqrt());
        };

        // Every glyph of the seated figure, including its wide base line.
        for (r, line) in lines.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    mark((x + c as isize) as f32, (y + r as isize) as f32);
                }
            }
        }
        // Tight head halo (two layers) and the large radiant rings behind
        // head and body.
        let halo_rx = (self.h as f32 * 0.28).clamp(7.0, 12.0);
        let aura_r = (self.h as f32 * 0.12 + 3.0 * 2.4).min(self.h as f32 * 0.46);
        for k in 0..48 {
            let a = k as f32 / 48.0 * TAU;
            for layer in 0..2 {
                let l = layer as f32;
                let hrx = halo_rx + l * 1.4;
                let hry = halo_rx * 0.52 + l * 0.8;
                mark(head_x + a.cos() * hrx, head_y + a.sin() * hry);
            }
            mark(head_x + a.cos() * aura_r, head_y + a.sin() * aura_r * 0.9);
        }
        // Lotus seat fan plus the horizontal line under it.
        let ly = y + art_h + 1;
        for k in -7..=7 {
            let kf = k as f32;
            let lx = head_x + kf * 2.4;
            let lyy = (ly as f32 + kf.abs() * 0.35 + 2.0).min(self.h as f32 - 2.0);
            mark(lx, lyy);
        }
        mark(head_x - 14.0, ly as f32);
        mark(head_x + 14.0, ly as f32);

        need + 1.5
    }

    /// Number of outward-expanding mandala rings.  The first ring starts
    /// outside the figure and there are at most five layers.
    fn ring_count(&self, span: f32) -> usize {
        ((span / 3.0).round() as usize).clamp(3, 5)
    }

    /// Lay out the mandala rings innermost-first as `(rx, ry, brightness)`.
    ///
    /// The innermost ring starts just outside the figure, so it never crosses
    /// the statue or the halo above its head.  Every ring then advances with
    /// one shared expansion phase and sits exactly one radial step from its
    /// neighbours, so the gaps never change (等距扩散).  There are at most
    /// five layers, and brightness falls off from the inner to the outer
    /// layers.
    fn layout_rings(&self, t: f32, out: &mut Vec<(f32, f32, f32)>) {
        out.clear();
        let cy = self.h as f32 * 0.5;
        // Largest radius whose ellipse still touches the canvas corner; any
        // bigger ring would be completely off-screen.
        let max_rx = ((self.w as f32 * 0.5).powi(2) + (cy / RING_SQUASH).powi(2)).sqrt() * 0.99;
        let min_rx = self.ring_min_rx.min(max_rx - 2.0).max(2.0);
        let span = (max_rx - min_rx).max(1.0);
        let count = self.ring_count(span);
        let phase = (t * 0.045).rem_euclid(1.0);
        for i in 0..count {
            let u = (phase + i as f32 / count as f32).rem_euclid(1.0);
            let rx = min_rx + u * span;
            // Kept below the statue's own gold (full gold is 1.0): the inner
            // layer tops out at 0.6 and fades to ~0.2 at the outermost layer.
            let level = 0.6 * (1.0 - u * 0.65);
            out.push((rx, rx * RING_SQUASH, level));
        }
        out.sort_by(|a, b| a.0.total_cmp(&b.0));
    }

    fn draw_art_figure(&self, canvas: &mut Canvas, theme: &Theme, t: f32) {
        let lines = self.art_lines();
        let art_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as isize;
        let art_h = lines.len() as isize;
        let max_x = self.w as isize - 2;
        let x = ((max_x - art_w) / 2).max(0);
        let y = ((self.h as isize - art_h) / 2).max(1);
        let head_x = x + art_w / 2;
        let head_y = y + 4;
        // tight, unmistakable halo directly around the head
        let halo_rx = (self.h as f32 * 0.28).clamp(7.0, 12.0);
        let halo_ry = halo_rx * 0.52;
        for layer in 0..2 {
            let rx = halo_rx + layer as f32 * 1.4;
            let ry = halo_ry + layer as f32 * 0.8;
            let n = 90usize;
            let mut prev: Option<(isize, isize)> = None;
            for k in 0..=n {
                let a = k as f32 / n as f32 * TAU + t * 0.2;
                let x = (head_x as f32 + a.cos() * rx).round() as isize;
                let y = (head_y as f32 + a.sin() * ry).round() as isize;
                if let Some((px, py)) = prev {
                    canvas.line(px, py, x, y, if layer == 0 { '*' } else { '+' }, PALE);
                }
                prev = Some((x, y));
                if k % 6 == 0 {
                    canvas.add_light(x, y, GOLD, 0.5, '.');
                }
            }
        }
        // large radiant halo behind the head and body
        for ring in 0..4 {
            let rr = (self.h as f32 * 0.12 + ring as f32 * 2.4).min(self.h as f32 * 0.46);
            let n = 100usize;
            let mut prev: Option<(isize, isize)> = None;
            for k in 0..=n {
                let a = k as f32 / n as f32 * TAU + t * (0.10 + ring as f32 * 0.03);
                let px = (head_x as f32 + a.cos() * rr).round() as isize;
                let py = (head_y as f32 + a.sin() * rr * 0.9).round() as isize;
                if let Some((ox, oy)) = prev {
                    canvas.line(
                        ox,
                        oy,
                        px,
                        py,
                        if ring == 0 { '*' } else { '+' },
                        GOLD.scale(0.45 + ring as f32 * 0.12),
                    );
                }
                prev = Some((px, py));
                if k % 7 == 0 {
                    canvas.add_light(px, py, GOLD, 0.25, '.');
                }
            }
        }
        // rays
        for k in 0..24 {
            let a = k as f32 / 24.0 * TAU + t * 0.06;
            for s in (self.h as f32 * 0.14) as isize..(self.w as f32 * 0.32) as isize {
                if s % 3 != 0 {
                    continue;
                }
                let px = head_x + (a.cos() * s as f32).round() as isize;
                let py = head_y + (a.sin() * s as f32 * 0.75).round() as isize;
                canvas.set(px, py, '.', GOLD.scale(0.22));
            }
        }

        let ink = Rgb::hex(0x1c1917);
        let mut glow = 0usize;
        for (r, line) in lines.iter().enumerate() {
            for (c, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let row = r as f32;
                let col = match ch {
                    '8' | 'o' | 'O' => PALE.lerp(GOLD, 0.35),
                    '|' | '\\' | '/' => {
                        if row > 9.0 {
                            AMBER.lerp(Rgb::hex(0xfb923c), 0.5)
                        } else {
                            PALE.lerp(GOLD, 0.45)
                        }
                    }
                    '_' | '-' | '=' => {
                        if row < 7.0 && (ch == '-' || ch == '_') {
                            ink
                        } else {
                            GOLD.lerp(PALE, 0.25)
                        }
                    }
                    '"' | '\'' | '.' | ':' | '`' => PALE,
                    _ => GOLD,
                };
                let x = x + c as isize;
                let yy = y + r as isize;
                canvas.set(x, yy, ch, col);
                glow += 1;
                if glow % 3 == 0 {
                    canvas.add_light(x, yy, col, 0.28, '.');
                }
            }
        }
        // lotus seat below the figure
        let ly = y + art_h + 1;
        if ly < self.h as isize - 1 {
            for k in -7..=7 {
                let kf = k as f32;
                let lx = head_x + (kf * 2.4) as isize;
                let lyy = ly + (kf.abs() * 0.35) as isize;
                canvas.line(
                    head_x,
                    ly,
                    lx,
                    (lyy + 2).min(self.h as isize - 2),
                    if k % 2 == 0 { '/' } else { '\\' },
                    PINK.scale(0.85),
                );
                canvas.set(lx, (lyy + 2).min(self.h as isize - 2), '*', GOLD);
            }
            canvas.hline(
                (head_x - 14).max(0),
                (head_x + 14).min(self.w as isize - 1),
                ly,
                '=',
                GOLD.scale(0.6),
            );
        }
        let _ = theme;
    }
}

impl Scene for BuddhaScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        for p in self.petals.iter_mut() {
            p.y += p.vy * dt;
            p.x += (f.t as f32 * 0.7 + p.phase).sin() * dt * 5.0;
            if p.y >= self.h as f32 + 1.0 {
                p.y = -1.0;
                p.x = self.rng.range(0.0, self.w as f32);
            }
        }
        for m in self.motes.iter_mut() {
            m.y -= m.speed * dt;
            if m.y < -1.0 {
                m.y = self.h as f32 + 1.0;
                m.x = self.rng.range(0.0, self.w as f32);
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let theme = f.theme;
        let t = f.t as f32;
        // deep background stars
        for (x, y, phase, bright) in &self.stars {
            let tw = 0.4 + 0.6 * (t * 0.9 + phase).sin().abs();
            canvas.set(
                *x as isize,
                *y as isize,
                if *bright > 0.55 { '+' } else { '.' },
                theme.c(2).scale(bright * tw * 0.35),
            );
        }
        // motes
        for m in &self.motes {
            let tw = 0.4 + 0.6 * (t * 1.4 + m.phase).sin().abs();
            canvas.set(m.x as isize, m.y as isize, '.', GOLD.scale(0.25 * tw));
        }
        // petals
        for p in &self.petals {
            let sway = (t * 1.2 + p.phase).sin() * 1.2;
            canvas.set(
                (p.x + sway) as isize,
                p.y as isize,
                p.ch,
                p.color.scale(0.7),
            );
        }
        // Outward-expanding mandala rings.  They start outside the figure,
        // share one phase so the layers stay equidistant, and dim outwards.
        let mut rings = std::mem::take(&mut self.rings);
        self.layout_rings(t, &mut rings);
        let cx = self.cx;
        let cy = self.h as f32 * 0.5;
        for &(rx, ry, level) in &rings {
            let n = (rx * 4.0 + 40.0) as usize;
            let color = ring_color(level);
            let ch = if level >= 0.4 { '+' } else { '.' };
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU + t * 0.05;
                let x = (cx + a.cos() * rx).round() as isize;
                let y = (cy + a.sin() * ry).round() as isize;
                if x < 0 || y < 0 || x >= self.w as isize || y >= self.h as isize {
                    continue;
                }
                canvas.set(x, y, ch, color);
            }
        }
        self.rings = rings;
        self.draw_art_figure(canvas, theme, t);
        // HUD
        let title = ":: AMITABHA // RADIANT HALO ::";
        if self.w >= title.len() + 4 {
            canvas.put_str_gradient(2, 0, title, &[GOLD, PALE, theme.c(4)]);
        }
        let right = "LOTUS 108 // OM";
        if self.w >= right.len() + 10 {
            canvas.put_str(
                self.w as isize - right.len() as isize - 2,
                0,
                right,
                GOLD.scale(0.8),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn figure_fits() {
        let mut s = BuddhaScene::new(9);
        s.resize(100, 30);
        assert!(s.cx > 20.0 && s.cx < 80.0);
        assert!(s.head_y > 2.0 && s.lap_y < 30.0);
    }

    #[test]
    fn rings_are_equidistant_and_dim_outwards() {
        let mut s = BuddhaScene::new(3);
        s.resize(120, 36);
        assert!(s.ring_min_rx > 0.0);
        let mut rings = Vec::new();
        s.layout_rings(0.0, &mut rings);
        let count = rings.len();
        assert!(count > 3 && count <= 5);
        // The first layer clears the figure instead of crossing its head halo.
        assert!(rings[0].0 >= s.ring_min_rx);
        let gap = rings[1].0 - rings[0].0;
        for pair in rings.windows(2) {
            assert!((pair[1].0 - pair[0].0 - gap).abs() < 0.01, "uneven gap");
            assert!(pair[1].2 < pair[0].2, "outer ring must be dimmer");
        }
        // The rings stay below the statue's own gold; the rim is much dimmer.
        assert!((rings[0].2 - 0.6).abs() < 0.01);
        assert!(rings[count - 1].2 < 0.3);
    }
}
