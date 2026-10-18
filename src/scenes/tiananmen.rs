//! 光芒万丈 — Tiananmen gate tower radiating light.
//!
//! The gate is rebuilt from colour blocks instead of a few outline lines:
//! a tall red wall with five stone-trimmed arch gates, the long slogan
//! plaques, the central portrait, the red-columned upper hall with its
//! national emblem, and two tiers of tiled roofs with up-curled eaves.
//! Flags flank the gate and a simplified stone balustrade and plaza sit in
//! front of it.  Bridge detail is deliberately omitted.
//!
//! Light is a two-stage loop: first every beam grows outward from a source
//! hidden behind the roof ridge, then bright packets run from the source to
//! the tip of each beam, lighting the fan from the centre outwards.
//!
//! ASCII by default; `--unicode` uses solid blocks and box drawing.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

/// Terminal cells are roughly twice as tall as they are wide; all radial
/// geometry is corrected with this factor so beams look straight.
const ASPECT: f32 = 2.0;

const SKY_TOP: Rgb = Rgb::hex(0x070d1d);
const SKY_MID: Rgb = Rgb::hex(0x1a1830);
const SKY_LOW: Rgb = Rgb::hex(0x5a2214);
const HAZE: Rgb = Rgb::hex(0xff8a3d);

const RED: Rgb = Rgb::hex(0xd42222);
const RED_LIGHT: Rgb = Rgb::hex(0xe23b2e);
const RED_DARK: Rgb = Rgb::hex(0x8f1414);
const RED_DEEP: Rgb = Rgb::hex(0x5b1010);

const GOLD: Rgb = Rgb::hex(0xfbbf24);
const GOLD_LIGHT: Rgb = Rgb::hex(0xfde68a);
const GOLD_DEEP: Rgb = Rgb::hex(0xb45309);

const ORANGE: Rgb = Rgb::hex(0xd35400);
const ORANGE_LIGHT: Rgb = Rgb::hex(0xf97316);
const ORANGE_DARK: Rgb = Rgb::hex(0x9a3412);
const ORANGE_INK: Rgb = Rgb::hex(0x4a1409);

const STONE: Rgb = Rgb::hex(0xe7e5e4);
const STONE_DARK: Rgb = Rgb::hex(0xa8a29e);
const PAVING: Rgb = Rgb::hex(0x3f3b38);
const INK: Rgb = Rgb::hex(0x140d0b);
const WHITE: Rgb = Rgb::hex(0xfefce8);
const FLAG: Rgb = Rgb::hex(0xdc2626);
const SKIN: Rgb = Rgb::hex(0xd8a97a);
const UNIFORM: Rgb = Rgb::hex(0x4a5138);
const HAIR: Rgb = Rgb::hex(0x2b2118);

/// Short helper: pick the ASCII glyph by default, Unicode on request.
#[inline]
fn gl(uni: bool, ascii: char, unicode: char) -> char {
    if uni {
        unicode
    } else {
        ascii
    }
}

#[inline]
fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[derive(Clone, Copy, Debug)]
struct Ray {
    angle: f32,
    /// Length in visual units (one unit == one cell horizontally).
    len: f32,
    /// Signed index offset from the middle of the fan.
    order: f32,
    shimmer: f32,
}

#[derive(Clone, Copy, Debug)]
struct Spark {
    angle: f32,
    speed: f32,
    phase: f32,
    wobble: f32,
}

#[derive(Clone, Copy, Debug)]
struct Star {
    x: f32,
    y: f32,
    phase: f32,
    bright: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct Layout {
    cx: isize,
    roof_top: isize,
    roof_rows: isize,
    roof_half: isize,
    ridge_half: isize,
    hall_top: isize,
    hall_bot: isize,
    hall_half: isize,
    eave_top: isize,
    eave_half: isize,
    wall_top: isize,
    wall_bot: isize,
    wall_half: isize,
    src_x: f32,
    src_y: f32,
}

impl Layout {
    fn compute(w: usize, h: usize) -> Self {
        let wf = w as f32;
        let hf = h as f32;
        let cx = (w / 2) as isize;
        let hmax = (h as isize - 1).max(0);

        let roof_top = ((hf * 0.23).round() as isize).clamp(0, hmax);
        let wall_bot = ((hf * 0.84).round() as isize).clamp(roof_top, hmax);
        let gate_h = (wall_bot - roof_top + 1).max(1);

        let wall_h = ((gate_h as f32 * 0.44).round() as isize).clamp(1, gate_h);
        let wall_top = wall_bot - wall_h + 1;

        let upper_h = (wall_top - roof_top).max(1);
        let roof_rows = ((upper_h as f32 * 0.36).round() as isize).clamp(1, upper_h);
        let hall_top = roof_top + roof_rows;
        let avail = (wall_top - hall_top).max(0);
        let eave_rows = ((upper_h as f32 * 0.14).round() as isize)
            .clamp(1, 3)
            .min((avail.saturating_sub(1)).max(0));
        let eave_top = wall_top - eave_rows;
        let hall_bot = (eave_top - 1).max(hall_top);

        let wall_half = ((wf * 0.46).round() as isize).max(4);
        let hall_half = ((wall_half as f32 * 0.66).round() as isize).max(3);
        let roof_half = ((wall_half as f32 * 0.79).round() as isize).max(hall_half + 1);
        let ridge_half = ((roof_half as f32 * 0.60).round() as isize).max(3);
        let eave_half = ((wall_half as f32 * 0.74).round() as isize).max(hall_half + 1);

        Layout {
            cx,
            roof_top,
            roof_rows,
            roof_half,
            ridge_half,
            hall_top,
            hall_bot,
            hall_half,
            eave_top,
            eave_half,
            wall_top,
            wall_bot,
            wall_half,
            src_x: cx as f32 + 0.5,
            src_y: roof_top as f32 + 0.5,
        }
    }

    fn wall_h(&self) -> isize {
        (self.wall_bot - self.wall_top + 1).max(1)
    }
}

pub struct TiananmenScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    layout: Layout,
    rays: Vec<Ray>,
    sparks: Vec<Spark>,
    stars: Vec<Star>,
}

impl TiananmenScene {
    pub fn new(seed: u64) -> Self {
        let mut scene = TiananmenScene {
            seed,
            rng: Rng::new(seed ^ 0x5449_414e_414e),
            w: 0,
            h: 0,
            layout: Layout::default(),
            rays: Vec::new(),
            sparks: Vec::new(),
            stars: Vec::new(),
        };
        scene.rebuild(80, 24);
        scene
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.layout = Layout::compute(self.w, self.h);

        self.rays.clear();
        self.sparks.clear();
        self.stars.clear();

        let n = ((self.w as f32 * 0.078).round() as usize).clamp(7, 12);
        let centre = (n as f32 - 1.0) * 0.5;
        for i in 0..n {
            let f = if n > 1 {
                i as f32 / (n as f32 - 1.0)
            } else {
                0.5
            };
            // Upper half plane, with the two outermost beams kept a little
            // above the horizontal so they clear the roof line.
            let angle = -std::f32::consts::PI + 0.30 + f * (std::f32::consts::PI - 0.60);
            let len = self.ray_reach(angle) * self.rng.range(0.94, 1.0);
            self.rays.push(Ray {
                angle,
                len,
                order: i as f32 - centre,
                shimmer: self.rng.range(0.0, TAU),
            });
        }

        let spark_n = (self.w / 3).clamp(10, 46);
        for _ in 0..spark_n {
            self.sparks.push(Spark {
                angle: self.rng.range(-std::f32::consts::PI + 0.15, -0.15),
                speed: self.rng.range(2.5, 8.0),
                phase: self.rng.range(0.0, 220.0),
                wobble: self.rng.range(0.0, TAU),
            });
        }

        let star_n = (self.w * self.h / 150).clamp(10, 80);
        for _ in 0..star_n {
            self.stars.push(Star {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32 * 0.52),
                phase: self.rng.range(0.0, TAU),
                bright: self.rng.range(0.25, 1.0),
            });
        }
    }

    /// Distance from the light source to the frame edge along `angle`.
    fn ray_reach(&self, angle: f32) -> f32 {
        let ca = angle.cos();
        let sa = angle.sin();
        let (sx, sy) = (self.layout.src_x, self.layout.src_y);
        let mut reach = 260.0f32;
        if ca.abs() > 1e-4 {
            if ca < 0.0 {
                reach = reach.min((sx / -ca).max(0.0));
            } else {
                reach = reach.min(((self.w as f32 - sx) / ca).max(0.0));
            }
        }
        if sa.abs() > 1e-4 {
            // Screen y is measured in cells, the visual distance is 2x.
            reach = reach.min((sy * ASPECT / -sa).max(0.0));
        }
        reach.clamp(6.0, 260.0)
    }

    fn draw_sky(&self, canvas: &mut Canvas, uni: bool, _t: f32) {
        let l = &self.layout;
        let horizon = (self.h as f32 * 0.80).max(1.0);
        for y in 0..self.h {
            let v = (y as f32 / horizon).clamp(0.0, 1.0);
            let base = if v < 0.55 {
                SKY_TOP.lerp(SKY_MID, v / 0.55)
            } else {
                SKY_MID.lerp(SKY_LOW, (v - 0.55) / 0.45)
            };
            for x in 0..self.w {
                let dx = x as f32 - l.src_x;
                let dy = (y as f32 - l.src_y) * ASPECT;
                let d = (dx * dx + dy * dy).sqrt();
                let glow = (-d / (self.w as f32 * 0.18).max(8.0)).exp() * 0.32;
                let c = base.add(HAZE, glow);
                // A sparse dotted grid echoes the pixel grid of the icon.
                if (x + y) % 2 == 0 {
                    canvas.set(x as isize, y as isize, gl(uni, '.', '·'), c.scale(0.52));
                } else if x % 5 == 0 && y % 3 == 0 {
                    canvas.set(x as isize, y as isize, gl(uni, '+', '·'), c.scale(0.42));
                }
            }
        }
    }

    fn draw_stars(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        for s in &self.stars {
            let tw = 0.30 + 0.70 * (t * 1.6 + s.phase).sin().abs();
            if tw * s.bright < 0.34 {
                continue;
            }
            let x = s.x as isize;
            let y = s.y as isize;
            let ch = if s.bright > 0.8 {
                gl(uni, '+', '✦')
            } else {
                gl(uni, '.', '·')
            };
            canvas.set(x, y, ch, GOLD_LIGHT.scale(0.22 + 0.38 * tw * s.bright));
        }
    }

    fn draw_rays(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        let l = &self.layout;
        let build_end = 2.3f32;
        let period = 3.3f32;
        let travel = 1.40f32;

        for r in &self.rays {
            let ca = r.angle.cos();
            let sa = r.angle.sin();
            let slash = if ca.abs() < 0.22 {
                gl(uni, '|', '│')
            } else if ca < 0.0 {
                gl(uni, '/', '╱')
            } else {
                gl(uni, '\\', '╲')
            };
            let start = r.order.abs() * 0.045;
            let grow = smoothstep(((t - start) / 1.20).clamp(0.0, 1.0));
            if grow <= 0.001 {
                continue;
            }
            let formed = grow > 0.998;
            let dmax = r.len * grow;

            let mut has_pulse = false;
            let mut head = 0.0f32;
            if formed && t >= build_end {
                let since = t - build_end - r.order.abs() * 0.22;
                if since >= 0.0 {
                    let u = since.rem_euclid(period) / travel;
                    if u <= 1.0 {
                        has_pulse = true;
                        head = u * r.len;
                    }
                }
            }

            let mut d = 0.0f32;
            while d <= dmax {
                let x = l.src_x + ca * d;
                let y = l.src_y + sa * d * 0.5;
                if x < 0.0 || y < 0.0 || x >= self.w as f32 || y >= self.h as f32 {
                    d += 0.45;
                    continue;
                }
                let base = 0.15
                    + 0.04 * (d * 0.14 + t * 1.15 + r.shimmer).sin()
                    + 0.02 * (t * 0.7 + r.order).sin();
                let mut pulse = 0.0f32;
                if has_pulse {
                    let dd = d - head;
                    if dd <= 0.0 {
                        pulse = (dd / (4.0 + d * 0.05)).exp() * 1.25;
                    } else {
                        pulse = (-(dd / (1.6 + d * 0.012)).powi(2)).exp() * 1.75;
                    }
                }
                let tip = if !formed {
                    let left = dmax - d;
                    if left < 2.4 {
                        1.30 * (1.0 - left / 2.4)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let fade = (1.0 - (d / r.len).powi(5)).clamp(0.45, 1.0);
                let val = base * fade + pulse + tip;
                if val > 0.075 {
                    let col = if val > 1.0 {
                        GOLD.lerp(WHITE, ((val - 1.0) * 0.85).min(1.0))
                    } else {
                        GOLD_DEEP.lerp(GOLD, val.clamp(0.0, 1.0).powf(0.55))
                    };
                    let xx = x.round() as isize;
                    let yy = y.round() as isize;
                    canvas.set(xx, yy, slash, col);
                    // Soft one-cell halo around the shaft.
                    let halo = (val * 0.13).min(0.18);
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        canvas.add_light(xx + dx, yy + dy, GOLD_DEEP, halo, gl(uni, '.', '·'));
                    }
                    if val > 0.9 {
                        canvas.set(xx, yy, gl(uni, '#', '█'), col);
                    }
                    if has_pulse && (d - head).abs() < 0.45 {
                        canvas.set(xx, yy, gl(uni, '#', '█'), WHITE);
                        canvas.set(xx, yy - 1, gl(uni, '*', '✦'), GOLD_LIGHT);
                    }
                }
                d += 0.38;
            }

            if has_pulse && head >= r.len * 0.975 {
                let tx = (l.src_x + ca * r.len).round() as isize;
                let ty = (l.src_y + sa * r.len * 0.5).round() as isize;
                canvas.set(tx, ty, gl(uni, '*', '✦'), GOLD_LIGHT);
            }
        }
    }

    fn draw_source(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        let l = &self.layout;
        let flare = 0.70 + 0.30 * (t * 1.4).sin().abs();
        for dy in -4isize..=1 {
            for dx in -5isize..=5 {
                let dyf = dy as f32 * ASPECT;
                let d = (((dx * dx) as f32) + dyf * dyf).sqrt();
                if d > 5.2 {
                    continue;
                }
                let x = l.src_x as isize + dx;
                let y = l.src_y as isize + dy;
                let ch = if d < 2.2 {
                    gl(uni, '@', '█')
                } else if d < 4.4 {
                    gl(uni, '#', '▓')
                } else {
                    gl(uni, '*', '▒')
                };
                let c = WHITE.lerp(GOLD, (d / 5.2).clamp(0.0, 1.0)).scale(flare);
                canvas.set(x, y, ch, c);
            }
        }
        let y = l.src_y as isize - 1;
        for k in 1..=8 {
            let c = GOLD_LIGHT.scale(0.10 * (1.0 - k as f32 / 9.0));
            canvas.set_if_blank(l.cx - k, y, gl(uni, '-', '─'), c);
            canvas.set_if_blank(l.cx + k, y, gl(uni, '-', '─'), c);
        }
    }

    fn draw_sparks(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        let l = &self.layout;
        for s in &self.sparks {
            let reach = self.ray_reach_quiet(s.angle);
            let d = (s.phase + t * s.speed).rem_euclid(reach.max(8.0));
            let wobble = (t * 1.4 + s.wobble).sin() * 1.2;
            let x = l.src_x + s.angle.cos() * d - s.angle.sin() * wobble;
            let y = l.src_y + s.angle.sin() * d * 0.5 + s.angle.cos() * wobble * 0.5;
            let fade = (1.0 - d / reach.max(1.0)).clamp(0.0, 1.0);
            if fade <= 0.05 {
                continue;
            }
            let ch = if fade > 0.65 {
                gl(uni, '*', '✦')
            } else {
                gl(uni, '.', '·')
            };
            canvas.set(
                x.round() as isize,
                y.round() as isize,
                ch,
                GOLD_LIGHT.scale(0.18 + 0.55 * fade),
            );
        }
    }

    fn ray_reach_quiet(&self, angle: f32) -> f32 {
        let ca = angle.cos();
        let sa = angle.sin();
        let (sx, sy) = (self.layout.src_x, self.layout.src_y);
        let mut reach = 220.0f32;
        if ca.abs() > 1e-4 {
            if ca < 0.0 {
                reach = reach.min((sx / -ca).max(0.0));
            } else {
                reach = reach.min(((self.w as f32 - sx) / ca).max(0.0));
            }
        }
        if sa.abs() > 1e-4 {
            reach = reach.min((sy * ASPECT / -sa).max(0.0));
        }
        reach.clamp(6.0, 220.0)
    }

    fn draw_gate(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        let rim = rim_light(t);
        self.draw_wall(canvas, uni, rim);
        self.draw_hall(canvas, uni, rim);
        self.draw_lower_eave(canvas, uni, rim);
        self.draw_upper_roof(canvas, uni, rim);
    }

    fn draw_wall(&self, canvas: &mut Canvas, uni: bool, rim: f32) {
        let l = &self.layout;
        let wall_h = l.wall_h();
        let left = l.cx - l.wall_half;
        let right = l.cx + l.wall_half;

        for y in l.wall_top..=l.wall_bot {
            let depth = (y - l.wall_top) as f32 / wall_h as f32;
            let mut c = RED.lerp(RED_DARK, depth * 0.22);
            if (y - l.wall_top) % 3 == 0 {
                c = c.scale(0.94);
            }
            for x in left..=right {
                let edge = x == left || x == right;
                let mut col = if edge { RED_DARK } else { c };
                if (x / 2) % 7 == 0 {
                    col = col.scale(0.95);
                }
                let ch = if edge {
                    gl(uni, '|', '║')
                } else {
                    gl(uni, '#', '█')
                };
                canvas.set(x, y, ch, col);
            }
        }

        // Stone coping along the top of the wall.
        for x in left..=right {
            let c = if x % 4 == 0 { STONE } else { STONE_DARK };
            canvas.set(x, l.wall_top, gl(uni, '_', '▄'), c);
        }
        // Thin gold filet under the coping.
        if l.wall_top < l.wall_bot {
            for x in (left + 1)..right {
                if x % 3 != 0 {
                    canvas.set(x, l.wall_top + 1, gl(uni, '-', '▀'), GOLD_DEEP.scale(0.55));
                }
            }
        }

        // Long slogan plaques, portrait and the five arch gates need a
        // little room; drop the small details on tiny terminals.
        let portrait_h = if self.h >= 32 { 5 } else { 3 };
        let portrait_w = if self.w >= 100 { 9 } else { 7 };
        let portrait_y = l.wall_top + 1;
        let portrait_bot = (portrait_y + portrait_h - 1)
            .min(l.wall_bot - 2)
            .max(portrait_y);
        let portrait_left = l.cx - portrait_w / 2;
        let portrait_right = portrait_left + portrait_w - 1;

        let plaque_h = if self.h >= 46 {
            5
        } else if self.h >= 32 {
            4
        } else {
            3
        };
        let plaque_y = l.wall_top + 1;
        if self.w >= 64 && l.wall_h() >= 6 && plaque_y + plaque_h - 1 < portrait_bot + 1 {
            let gap = 2;
            let left_x0 = left + 2;
            let left_x1 = portrait_left - gap;
            let right_x0 = portrait_right + gap;
            let right_x1 = right - 2;
            if left_x1 - left_x0 >= 10 {
                draw_plaque(canvas, uni, left_x0, left_x1, plaque_y, plaque_h, false);
            }
            if right_x1 - right_x0 >= 10 {
                draw_plaque(canvas, uni, right_x0, right_x1, plaque_y, plaque_h, true);
            }
        }

        if l.wall_h() >= 4 && self.w >= 48 {
            draw_portrait(
                canvas,
                uni,
                portrait_left,
                portrait_right,
                portrait_y,
                portrait_bot,
                rim,
            );
        }

        // Five arch gates fill the lower part of the wall.
        let arch_top = (portrait_bot + 1).max(l.wall_top + wall_h / 2);
        if arch_top <= l.wall_bot && self.w >= 36 {
            let step = ((l.wall_half * 2 - 20) / 4).max(6);
            let aw = (step / 3).clamp(2, 9);
            for k in 0..5isize {
                let ax = l.cx + (k - 2) * step;
                draw_arch(
                    canvas,
                    uni,
                    ax,
                    aw,
                    arch_top,
                    l.wall_bot,
                    k == 2,
                    self.h >= 28,
                );
            }
        }
    }

    fn draw_hall(&self, canvas: &mut Canvas, uni: bool, rim: f32) {
        let l = &self.layout;
        if l.hall_bot < l.hall_top {
            return;
        }
        let left = l.cx - l.hall_half;
        let right = l.cx + l.hall_half;
        let rows = (l.hall_bot - l.hall_top + 1).max(1);

        // Deep shadow behind the colonnade.
        for y in l.hall_top..=l.hall_bot {
            for x in left..=right {
                canvas.set(x, y, gl(uni, ':', '▒'), INK.lerp(RED_DEEP, 0.30));
            }
        }

        // Gold bracket band under the roof.
        for x in left..=right {
            let ch = if (x - l.cx).rem_euclid(6) == 0 {
                gl(uni, '=', '█')
            } else {
                gl(uni, '-', '▄')
            };
            canvas.set(x, l.hall_top, ch, GOLD_DEEP.lerp(GOLD, 0.35 + 0.35 * rim));
        }

        // Red columns and gold capitals.
        let step = ((l.hall_half * 2) / 8).max(4);
        let body_top = l.hall_top + 1;
        let body_bot = (l.hall_bot - 1).max(body_top);
        let mut x = left + 1;
        while x < right {
            for dx in 0..2 {
                for y in body_top..=body_bot {
                    let c = if y == body_top {
                        GOLD.lerp(GOLD_LIGHT, 0.35 * rim)
                    } else {
                        RED_LIGHT.lerp(RED, (y - body_top) as f32 * 0.10)
                    };
                    canvas.set(x + dx, y, gl(uni, '#', '█'), c);
                }
            }
            // A hint of lattice in each bay.
            for bx in (x + 2)..(x + step - 1) {
                for y in body_top..=body_bot {
                    if (bx + y) % 3 == 0 {
                        canvas.set(bx, y, gl(uni, '.', '░'), RED_DEEP);
                    }
                }
            }
            x += step;
        }

        // Bottom beam.
        for x in left..=right {
            canvas.set(x, l.hall_bot, gl(uni, '=', '▄'), GOLD_DEEP.scale(0.85));
        }

        // National emblem above the central arch.
        let ey = l.hall_top + (rows / 2).max(1);
        let er = if self.w >= 100 { 2isize } else { 1 };
        for dy in -er..=er {
            for dx in -(er + 1)..=(er + 1) {
                if dx.abs() / 2 + dy.abs() <= er {
                    let edge = dx.abs() == er + 1 || dy.abs() == er;
                    canvas.set(
                        l.cx + dx,
                        ey + dy,
                        if edge {
                            gl(uni, '*', '▓')
                        } else {
                            gl(uni, '#', '█')
                        },
                        if edge { GOLD } else { RED_DARK },
                    );
                }
            }
        }
        canvas.set(l.cx, ey, gl(uni, '*', '★'), GOLD_LIGHT);
    }

    fn draw_upper_roof(&self, canvas: &mut Canvas, uni: bool, rim: f32) {
        let l = &self.layout;
        draw_roof(
            canvas,
            uni,
            l.cx,
            l.roof_top,
            l.roof_rows,
            l.roof_half,
            l.ridge_half,
            rim,
            true,
        );
    }

    fn draw_lower_eave(&self, canvas: &mut Canvas, uni: bool, rim: f32) {
        let l = &self.layout;
        let rows = l.wall_top - l.eave_top;
        if rows <= 0 {
            return;
        }
        draw_roof(
            canvas,
            uni,
            l.cx,
            l.eave_top,
            rows,
            l.eave_half,
            (l.eave_half as f32 * 0.80) as isize,
            rim * 0.8,
            false,
        );
    }

    fn draw_flags(&self, canvas: &mut Canvas, uni: bool, t: f32) {
        let l = &self.layout;
        let pole_top = l.roof_top + 3;
        let pole_bot = l.wall_bot + 1;
        let half = self.w as isize / 24;
        let fw = half.clamp(2, 6);
        for (px, dir) in [
            (l.cx - l.wall_half - 2, 1isize),
            (l.cx + l.wall_half + 2, -1isize),
        ] {
            if px < 0 || px >= self.w as isize {
                continue;
            }
            for y in pole_top..=pole_bot {
                canvas.set(px, y, gl(uni, '|', '│'), STONE_DARK.scale(0.80));
            }
            canvas.set(px, pole_top, gl(uni, '*', '◆'), GOLD_LIGHT);

            let top = pole_top + 1;
            for i in 0..fw {
                let xx = px + (i + 1) * dir;
                let wave = (t * 2.8 + i as f32 * 0.75).sin() * 1.1;
                let yy = top + (wave * 0.6).round() as isize + i / 3;
                let c = if i == fw - 1 { FLAG.scale(0.78) } else { FLAG };
                canvas.set(xx, yy, gl(uni, '=', '▰'), c);
                canvas.set(xx, yy + 1, gl(uni, '=', '▰'), c.scale(0.85));
            }
            // A small gold star in the upper corner of the flag.
            canvas.set(px + dir, top, gl(uni, '*', '★'), GOLD_LIGHT);
        }
    }

    fn draw_foreground(&self, canvas: &mut Canvas, uni: bool, _t: f32) {
        let l = &self.layout;
        let ty = l.wall_bot + 1;
        let h = self.h as isize;
        if ty >= h {
            return;
        }

        // Side wings: low white buildings with orange tiled roofs.  They sit
        // in front of the outer wall, below the arch band.
        let wing = (self.w as isize / 9).clamp(5, 15);
        for side in 0..2 {
            let (x0, x1) = if side == 0 {
                (0, wing)
            } else {
                (self.w as isize - wing - 1, self.w as isize - 1)
            };
            let top = (ty - 5).max(l.wall_top + 1);
            for x in x0..=x1 {
                let edge = (x - x0).min(x1 - x);
                let roof_start = top + if edge < (wing / 4).max(1) { 1 } else { 0 };
                for y in roof_start..=top + 1 {
                    let c = if y == top + 1 {
                        GOLD_DEEP.lerp(ORANGE_LIGHT, 0.35)
                    } else if edge < 2 {
                        ORANGE_DARK
                    } else {
                        ORANGE
                    };
                    canvas.set(x, y, gl(uni, '#', '█'), c);
                }
                for y in (top + 2)..ty {
                    let c = STONE_DARK.lerp(STONE, 0.20).scale(0.72);
                    canvas.set(x, y, gl(uni, '#', '█'), c);
                }
            }
            // Window slits (skip on small terminals where they turn into
            // big black holes).
            if self.w >= 100 {
                for x in (x0 + 2..x1 - 1).step_by(3) {
                    for y in (top + 3)..ty.min(top + 4) {
                        canvas.set(x, y, ' ', Rgb::BLACK);
                    }
                }
            }
        }

        // Plaza.
        for y in ty..h {
            let v = (y - ty) as f32 / (h - ty).max(1) as f32;
            for x in 0..self.w as isize {
                let centred = 1.0 - (x - l.cx).abs() as f32 / (self.w as f32 * 0.5);
                let c = PAVING
                    .lerp(Rgb::hex(0x5a544f), centred.max(0.0) * 0.32)
                    .scale(0.82 + v * 0.18);
                canvas.set(x, y, gl(uni, '#', '█'), c);
            }
        }

        // Central approach: a solid light stone ramp widening towards us.
        for y in ty..h {
            let v = (y - ty) as f32 / (h - ty).max(1) as f32;
            let half = (self.w as f32 * (0.075 + 0.155 * v)) as isize;
            let sheen = 0.70 + 0.30 * (1.0 - v);
            for x in (l.cx - half)..=(l.cx + half) {
                let edge = x == l.cx - half || x == l.cx + half;
                let c = if edge {
                    STONE_DARK
                } else {
                    Rgb::hex(0x736d67).scale(sheen)
                };
                canvas.set(x, y, gl(uni, '#', '█'), c);
            }
            if y % 2 == 0 {
                for x in (l.cx - half + 1)..(l.cx + half) {
                    canvas.add_light(x, y, STONE, 0.05, gl(uni, '.', '·'));
                }
            }
        }

        // One white stone rail with posts; the bridge detail is omitted.
        for x in 1..self.w as isize - 1 {
            let post = x % 6 == 0;
            canvas.set(
                x,
                ty,
                if post {
                    gl(uni, '+', '┬')
                } else {
                    gl(uni, '=', '▀')
                },
                STONE,
            );
            if post && ty + 1 < h {
                canvas.set(x, ty + 1, gl(uni, '|', '│'), STONE);
            }
        }
    }
}

fn rim_light(t: f32) -> f32 {
    let period = 3.3f32;
    let since = (t - 2.3).max(0.0);
    0.45 + 0.55 * (since * TAU / period).cos().max(0.0)
}

#[allow(clippy::too_many_arguments)]
fn draw_roof(
    canvas: &mut Canvas,
    uni: bool,
    cx: isize,
    y_top: isize,
    rows: isize,
    half_eave: isize,
    half_ridge: isize,
    rim: f32,
    ridge_ornaments: bool,
) {
    if rows <= 0 || half_eave <= 0 {
        return;
    }
    let ridge_half = half_ridge.clamp(1, half_eave);
    let eave_y = y_top + rows - 1;
    let eave_col = ORANGE_LIGHT.lerp(GOLD_LIGHT, 0.22 + 0.34 * rim);

    for dx in -half_eave..=half_eave {
        let ax = dx.abs();
        let top_off = if ax <= ridge_half {
            0
        } else {
            let span = (half_eave - ridge_half).max(1) as f32;
            (((ax - ridge_half) as f32 / span) * (rows - 2).max(1) as f32).round() as isize
        };
        let lift = if ax >= half_eave - 5 {
            let k = ((ax - (half_eave - 5)) as f32 / 5.0).clamp(0.0, 1.0);
            (k * k * 2.0).round() as isize
        } else {
            0
        };
        let y0 = y_top + top_off.min(rows - 1);
        let y1 = eave_y - lift.min(rows - 1);
        if y1 < y0 {
            continue;
        }
        for y in y0..=y1 {
            let c = if y == y1 {
                if ax >= half_eave - 5 {
                    GOLD_LIGHT.lerp(GOLD, 0.18)
                } else {
                    eave_col
                }
            } else if y == y0 && ridge_ornaments {
                if dx.rem_euclid(4) == 0 {
                    ORANGE_INK
                } else {
                    ORANGE_DARK
                }
            } else {
                // Vertical tile ridges, shaded from the ridge to the eave.
                let mut c = match dx.rem_euclid(4) {
                    0 => ORANGE_DARK,
                    2 => ORANGE_LIGHT,
                    _ => ORANGE,
                };
                let v = (y - y0) as f32 / (y1 - y0).max(1) as f32;
                c = c.scale(0.82 + 0.18 * v);
                c = c.scale(0.94 + 0.06 * rim);
                if (dx * 3 + y) % 19 == 0 {
                    c = GOLD_DEEP.scale(0.65);
                }
                c
            };
            let ch = if y == y1 {
                gl(uni, '=', '▄')
            } else {
                gl(uni, '#', '█')
            };
            canvas.set(cx + dx, y, ch, c);
        }
    }

    // Flying eave corners: three cells sweeping outward and upward.
    for k in 1..=3isize {
        let yy = eave_y - 1 - k / 2;
        let c = if k == 1 { GOLD_LIGHT } else { GOLD_DEEP };
        canvas.set(cx - half_eave - k, yy, gl(uni, '/', '╱'), c);
        canvas.set(cx + half_eave + k, yy, gl(uni, '\\', '╲'), c);
    }
    canvas.set(
        cx - half_eave - 3,
        eave_y - 2,
        gl(uni, '*', '◆'),
        GOLD_LIGHT,
    );
    canvas.set(
        cx + half_eave + 3,
        eave_y - 2,
        gl(uni, '*', '◆'),
        GOLD_LIGHT,
    );

    if ridge_ornaments {
        for xx in [cx - ridge_half - 1, cx + ridge_half + 1] {
            canvas.set(xx, y_top - 1, gl(uni, '#', '█'), GOLD_DEEP);
            canvas.set(xx, y_top - 2, gl(uni, '*', '◆'), GOLD_LIGHT);
        }
        canvas.set(cx, y_top - 1, gl(uni, '*', '◆'), GOLD);
        for dx in [-4isize, 4] {
            canvas.set(cx + dx, y_top - 1, gl(uni, '-', '▄'), GOLD_DEEP);
        }
    }
}

fn draw_plaque(
    canvas: &mut Canvas,
    uni: bool,
    x0: isize,
    x1: isize,
    y0: isize,
    h: isize,
    right: bool,
) {
    if x1 - x0 < 8 || h < 2 {
        return;
    }
    let y1 = y0 + h - 1;
    // White stone frame, red panel inside — the long plaques in the icon
    // read as white text on red, not as solid gold bars.
    for y in y0..=y1 {
        for x in x0..=x1 {
            let border = x == x0 || x == x1 || y == y0 || y == y1;
            let c = if border {
                STONE_DARK.lerp(STONE, 0.55)
            } else {
                RED_DARK.lerp(RED, 0.58).scale(0.92)
            };
            canvas.set(x, y, gl(uni, '#', '█'), c);
        }
    }
    for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
        canvas.set(x, y, gl(uni, '+', '◆'), GOLD_LIGHT);
    }

    // Nine blocky glyph clusters.  Real CJK glyphs are double width and
    // would corrupt the terminal cell grid, so the slogans are suggested
    // with 3x3 white masks (1x2 masks on short plaques).
    const LEFT5: [[u8; 3]; 9] = [
        [0b010, 0b111, 0b010],
        [0b101, 0b111, 0b101],
        [0b010, 0b010, 0b101],
        [0b111, 0b110, 0b101],
        [0b101, 0b111, 0b101],
        [0b010, 0b111, 0b010],
        [0b111, 0b101, 0b111],
        [0b111, 0b010, 0b010],
        [0b010, 0b111, 0b010],
    ];
    const RIGHT5: [[u8; 3]; 9] = [
        [0b101, 0b111, 0b111],
        [0b111, 0b010, 0b101],
        [0b010, 0b111, 0b010],
        [0b111, 0b110, 0b101],
        [0b010, 0b111, 0b010],
        [0b111, 0b101, 0b111],
        [0b111, 0b010, 0b101],
        [0b111, 0b010, 0b010],
        [0b010, 0b111, 0b010],
    ];
    const LEFT1: [u8; 9] = [0b11, 0b10, 0b01, 0b11, 0b10, 0b11, 0b01, 0b10, 0b11];
    const RIGHT1: [u8; 9] = [0b10, 0b11, 0b01, 0b11, 0b10, 0b11, 0b01, 0b11, 0b10];

    if h >= 4 {
        let gap = 1isize;
        let gw = 3isize;
        let span = 9 * gw + 8 * gap;
        if span > x1 - x0 - 3 {
            return;
        }
        let start = x0 + ((x1 - x0 + 1) - span) / 2;
        let masks = if right { &RIGHT5 } else { &LEFT5 };
        let text_rows = ((h - 2).clamp(1, 3)) as usize;
        for (gi, m) in masks.iter().enumerate() {
            let gx = start + gi as isize * (gw + gap);
            for (dy, bits) in m.iter().take(text_rows).enumerate() {
                for dx in 0..gw {
                    if bits & (1 << (gw - 1 - dx)) != 0 {
                        let c = if gi % 3 == 0 { GOLD_LIGHT } else { WHITE };
                        canvas.set(gx + dx, y0 + 1 + dy as isize, gl(uni, '#', '█'), c);
                    }
                }
            }
        }
    } else {
        let gap = 1isize;
        let gw = 2isize;
        let span = 9 * gw + 8 * gap;
        if span > x1 - x0 - 2 {
            return;
        }
        let start = x0 + ((x1 - x0 + 1) - span) / 2;
        let masks = if right { RIGHT1 } else { LEFT1 };
        let ty = y0 + h / 2;
        for (gi, bits) in masks.iter().enumerate() {
            let gx = start + gi as isize * (gw + gap);
            for dx in 0..gw {
                if bits & (1 << (gw - 1 - dx)) != 0 {
                    canvas.set(gx + dx, ty, gl(uni, '#', '█'), WHITE);
                }
            }
        }
    }
}

fn draw_portrait(
    canvas: &mut Canvas,
    uni: bool,
    x0: isize,
    x1: isize,
    y0: isize,
    y1: isize,
    rim: f32,
) {
    if x1 - x0 < 4 || y1 - y0 < 2 {
        return;
    }
    let cx = (x0 + x1) / 2;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let border = x == x0 || x == x1 || y == y0 || y == y1;
            let c = if border {
                GOLD.lerp(GOLD_LIGHT, 0.15 + 0.25 * rim)
            } else {
                Rgb::hex(0x3f3f2e)
            };
            canvas.set(x, y, gl(uni, '#', '█'), c);
        }
    }
    let inner_y0 = y0 + 1;
    let inner_y1 = y1 - 1;
    let inner_rows = (inner_y1 - inner_y0 + 1).max(1);
    let half_w = (x1 - x0) / 2;
    for y in inner_y0..=inner_y1 {
        let row = y - inner_y0;
        for x in (x0 + 1)..x1 {
            let dx = x - cx;
            let c = if inner_rows == 1 {
                // Three-row portrait on short terminals: hair, face, eyes.
                if dx.abs() >= half_w - 1 || dx.abs() == 1 {
                    HAIR
                } else {
                    SKIN
                }
            } else if row == 0 {
                HAIR
            } else if row >= inner_rows - 1 {
                // Shoulders and the white collar.
                if dx.abs() <= 1 {
                    WHITE
                } else if dx.abs() == 2 {
                    RED_DEEP
                } else {
                    UNIFORM
                }
            } else if row == inner_rows - 2 && inner_rows >= 4 {
                UNIFORM
            } else {
                // Face: hair at the temples, two dark eyes.
                if dx.abs() >= half_w - 1 || dx.abs() == 1 {
                    HAIR
                } else {
                    SKIN
                }
            };
            canvas.set(x, y, gl(uni, '#', '█'), c);
        }
    }
    // Thin gold halo around the frame so the portrait reads at small sizes.
    for x in (x0 - 1)..=(x1 + 1) {
        canvas.add_light(x, y0 - 1, GOLD, 0.10, gl(uni, '.', '·'));
        canvas.add_light(x, y1 + 1, GOLD, 0.10, gl(uni, '.', '·'));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_arch(
    canvas: &mut Canvas,
    uni: bool,
    ax: isize,
    aw: isize,
    top: isize,
    bottom: isize,
    central: bool,
    trim: bool,
) {
    let half = (aw / 2).max(1);
    let height = (bottom - top + 1).max(1);
    for dy in 0..height {
        let y = top + dy;
        let hw = if dy == 0 {
            (half - 2).max(0)
        } else if dy == 1 {
            (half - 1).max(0)
        } else {
            half
        };
        for dx in -hw..=hw {
            canvas.set(ax + dx, y, ' ', Rgb::BLACK);
        }
        if trim {
            let jamb = half + 1;
            canvas.set(ax - jamb, y, gl(uni, '|', '│'), STONE_DARK);
            canvas.set(ax + jamb, y, gl(uni, '|', '│'), STONE_DARK);
        }
    }
    if trim && !central {
        let hw = (half - 1).max(0);
        for dx in -hw..=hw {
            canvas.set(ax + dx, top - 1, gl(uni, '_', '▄'), STONE);
        }
        canvas.set(ax - half, top, gl(uni, '+', '▙'), STONE_DARK);
        canvas.set(ax + half, top, gl(uni, '+', '▟'), STONE_DARK);
    }
    if central {
        // Gold-framed central portal under the portrait.
        for y in top..=bottom {
            canvas.set(ax - half - 1, y, gl(uni, '|', '│'), GOLD);
            canvas.set(ax + half + 1, y, gl(uni, '|', '│'), GOLD);
        }
        // A small red sign above the doorway.
        for dx in -1..=1 {
            canvas.set(ax + dx, top, gl(uni, '#', '█'), RED_DEEP);
        }
        canvas.set(ax, top, gl(uni, '*', '◆'), GOLD_LIGHT);
        for dx in -1..=1 {
            canvas.set(ax + dx, bottom, gl(uni, '=', '▄'), GOLD_DEEP);
        }
    }
}

impl Scene for TiananmenScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {}

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let uni = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_sky(canvas, uni, t);
        self.draw_stars(canvas, uni, t);
        self.draw_rays(canvas, uni, t);
        self.draw_sparks(canvas, uni, t);
        self.draw_source(canvas, uni, t);
        self.draw_gate(canvas, uni, t);
        self.draw_flags(canvas, uni, t);
        self.draw_foreground(canvas, uni, t);
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorMode;
    use crate::scene::{EffectId, Tune};

    fn frame(t: f64, w: usize, h: usize, charset: Charset) -> Frame<'static> {
        let theme = EffectId::Tiananmen.default_theme();
        Frame {
            t,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme,
            charset,
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        }
    }

    #[test]
    fn rays_exist() {
        let mut s = TiananmenScene::new(7);
        s.resize(120, 36);
        assert!(s.rays.len() >= 7);
        assert!(!s.sparks.is_empty());
    }

    #[test]
    fn gate_fills_a_large_part_of_the_frame() {
        let mut s = TiananmenScene::new(7);
        s.resize(120, 36);
        let mut c = Canvas::new(120, 36);
        s.draw(&mut c, &frame(4.0, 120, 36, Charset::Ascii));
        assert!(c.ink() > 2500, "only {} cells inked", c.ink());
    }

    #[test]
    fn ascii_is_the_default_and_stays_ascii() {
        let mut s = TiananmenScene::new(7);
        s.resize(120, 36);
        let mut c = Canvas::new(120, 36);
        s.draw(&mut c, &frame(4.0, 120, 36, Charset::Ascii));
        for cell in &c.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }

    #[test]
    fn animation_changes_the_frame() {
        let mut s = TiananmenScene::new(7);
        s.resize(120, 36);
        let mut a = Canvas::new(120, 36);
        let mut b = Canvas::new(120, 36);
        s.draw(&mut a, &frame(1.0, 120, 36, Charset::Unicode));
        s.draw(&mut b, &frame(4.5, 120, 36, Charset::Unicode));
        let different = a
            .cells
            .iter()
            .zip(b.cells.iter())
            .filter(|(x, y)| x != y)
            .count();
        assert!(different > 200, "only {} cells changed", different);
        let _ = a.render_ansi(ColorMode::True);
    }
}
