//! interstellar — a patient rendezvous at a rotating ring station.
//!
//! An eight-section ring station turns at the lower right.  Every ring section
//! is tied back to a central control room by a spoke, and the control room
//! itself turns with the ring.  A small four-armed ship slips in from the
//! upper left, joins the station's upper-left radial and spirals slowly toward
//! the hub while its roll turns with the ring; the two centres meet at
//! **20.0 s**.  Contact throws sparks; the docked stack burns, shrinks
//! toward a vanishing point, trailing a short echo of the wheel, and ends as a
//! single star that blinks twice.  The empty sky then dips to black and
//! recovers before a new pass begins.
//!
//! The long beat between entry and contact is deliberate: the ship closes the
//! distance very slowly, correcting its roll as it goes.  Nothing is a straight
//! zoom: the departure is a perspective recede with a vanishing point and a
//! glyph level-of-detail ladder.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

/// One full pass, including the closing dip.
const CYCLE: f32 = STAR_END + FADE_OUT + FADE_IN;
/// Station power-up and ring lighting sequence.
const POWER_END: f32 = 1.8;
/// The ship has joined the upper-left approach line.
const ENTRY_END: f32 = 10.5;
/// Contact.  This is the beat the whole scene is built around.
const DOCK_T: f32 = 20.0;
/// Engine ignition and the start of the recede.
const DEPART_START: f32 = 20.6;
/// The stack has collapsed to a point at the vanishing point.
const DEPART_END: f32 = 23.8;
/// The second star blink is gone; the closing dip starts here.
const STAR_END: f32 = 24.9;
/// Half a second down to black, half a second back up, then the loop.
const FADE_OUT: f32 = 0.5;
const FADE_IN: f32 = 0.5;

/// Ring angular speed, radians per second (one turn per ~17.5 s).
const OMEGA: f32 = 0.36;
/// The ship approaches down the station's upper-left radial.
const APPROACH_ANGLE: f32 = -2.30;
/// Distance from the hub when the ship joins the approach line, in radii.
const ENTRY_RADIUS: f32 = 2.35;
/// Terminal cells are about twice as tall as they are wide.
const CELL_ASPECT: f32 = 0.5;
/// Projected minor axis of the ring plane.
const PLANE_MINOR: f32 = 0.95;
/// Slight roll of the ring plane, so the ellipse reads as a 3D ring.
const TILT: f32 = -0.12;
/// Eight sections, joined to the hub by eight spokes.
const MODULES: usize = 8;
/// The ship is a small core with four tentacles.  Lengths are measured from
/// the centre in cells at scale 1: index, starboard, engine, port.  The
/// unequal lengths (and the gold index tip) make the roll legible.
const SHIP_ARM_LEN: [f32; 4] = [4.4, 3.1, 3.7, 3.3];
/// Compact hull core, in cells at scale 1.
const SHIP_CORE_FWD: f32 = 1.4;
const SHIP_CORE_LAT: f32 = 1.05;
/// The long docking arm sits at this fixed offset inside the rotating core
/// frame, so it always leaves through the same hexagon vertex.
const DOCK_ARM_OFFSET: f32 = APPROACH_ANGLE - OMEGA * DOCK_T;
/// The control block is drawn a little taller than the ring plane: at nine
/// cells wide a true ring-plane squash leaves only four rows, and the block's
/// turn would read as a static slab.
const CORE_ASPECT: f32 = 0.72;

const WHITE: Rgb = Rgb::hex(0xfffdf6);
const WINDOW: Rgb = Rgb::hex(0xffd9a0);
const PORT: Rgb = Rgb::hex(0x67e8f9);
const PLUME: Rgb = Rgb::hex(0x9be7ff);
const SPARK_HOT: Rgb = Rgb::hex(0xfff6e0);
const SPARK_GOLD: Rgb = Rgb::hex(0xffc857);
const SPARK_ORANGE: Rgb = Rgb::hex(0xff7a3c);
const SPARK_RED: Rgb = Rgb::hex(0x9a3412);
const STAR_COLORS: [Rgb; 5] = [
    Rgb::hex(0xffffff),
    Rgb::hex(0xcfe8ff),
    Rgb::hex(0x9cc9ff),
    Rgb::hex(0xffe6a8),
    Rgb::hex(0x8b9bb8),
];

struct Star {
    x: f32,
    y: f32,
    base: f32,
    phase: f32,
    speed: f32,
    color: Rgb,
    ch: char,
}

struct Spark {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    drag: f32,
    /// 0 = contact spray, 1 = engine ember.
    kind: u8,
}

pub struct InterstellarScene {
    rng: Rng,
    w: usize,
    h: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    stars: Vec<Star>,
    sparks: Vec<Spark>,
    /// Current sprite roll of the ship, radians.
    spin: f32,
    cycle: i64,
    dock_spawned: bool,
    shock_age: f32,
    dock_x: f32,
    dock_y: f32,
    vp_x: f32,
    vp_y: f32,
    entry_jx: f32,
    entry_jy: f32,
    /// How much of the approach is complete, 0..1.
    sync: f32,
    /// Per-module window phase, so the ring does not blink in unison.
    windows: [f32; MODULES],
}

impl InterstellarScene {
    pub fn new(seed: u64) -> Self {
        InterstellarScene {
            rng: Rng::new(seed ^ 0x1a57_e11a_c0de_0001),
            w: 0,
            h: 0,
            cx: 0.0,
            cy: 0.0,
            radius: 4.0,
            stars: Vec::new(),
            sparks: Vec::new(),
            spin: f32::NAN,
            cycle: i64::MIN,
            dock_spawned: false,
            shock_age: -1.0,
            dock_x: 0.0,
            dock_y: 0.0,
            vp_x: 0.0,
            vp_y: 0.0,
            entry_jx: 0.0,
            entry_jy: 0.0,
            sync: 0.0,
            windows: [0.0; MODULES],
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.cx = self.w as f32 * 0.72;
        self.cy = self.h as f32 * 0.60;
        self.radius = (self.w as f32 * 0.19)
            .min(self.h as f32 * 0.52)
            .clamp(3.0, 26.0);
        self.entry_jx = self.rng.range(-0.018, 0.018) * self.w as f32;
        self.entry_jy = self.rng.range(-0.008, 0.008) * self.h as f32;
        self.vp_x = self.w as f32 * 0.30 + self.rng.range(-0.035, 0.035) * self.w as f32;
        self.vp_y = self.h as f32 * 0.27 + self.rng.range(-0.025, 0.025) * self.h as f32;
        for w in self.windows.iter_mut() {
            *w = self.rng.range(0.0, TAU);
        }
        self.stars.clear();
        let n = ((self.w * self.h) / 26).clamp(50, 260);
        for _ in 0..n {
            let bright = self.rng.chance(0.07);
            let base = if bright {
                self.rng.range(0.62, 1.0)
            } else {
                self.rng.range(0.05, 0.48)
            };
            let ch = if base > 0.88 {
                '*'
            } else if base > 0.66 {
                '+'
            } else if self.rng.chance(0.35) {
                '\''
            } else {
                '.'
            };
            self.stars.push(Star {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                base,
                phase: self.rng.range(0.0, TAU),
                speed: self.rng.range(0.25, 1.4),
                color: STAR_COLORS[self.rng.below(STAR_COLORS.len())],
                ch,
            });
        }
        self.sparks.clear();
        self.shock_age = -1.0;
    }

    /// Polar point on the projected ring plane, relative to the station centre.
    #[inline]
    fn orbit_rel(&self, a: f32, rr: f32) -> (f32, f32) {
        let (sa, ca) = a.sin_cos();
        let u = ca * rr;
        let v = sa * rr * CELL_ASPECT * PLANE_MINOR;
        let (st, ct) = TILT.sin_cos();
        (u * ct - v * st, u * st + v * ct)
    }

    #[inline]
    fn orbit_point(&self, a: f32, rr: f32) -> (f32, f32) {
        let p = self.orbit_rel(a, rr);
        (self.cx + p.0, self.cy + p.1)
    }

    /// The ship approaches down the station's upper-left radial.
    #[inline]
    fn approach_angle(&self, t: f32) -> f32 {
        APPROACH_ANGLE + 0.025 * (t * 0.45 + self.windows[2]).sin()
    }

    /// Distance from the hub, in station radii.  The ship covers most of the
    /// distance early and then eases into the centre.
    fn ship_radius(&self, t: f32) -> f32 {
        if t < ENTRY_END {
            ENTRY_RADIUS
        } else {
            let u = ((t - ENTRY_END) / (DOCK_T - ENTRY_END)).clamp(0.0, 1.0);
            ENTRY_RADIUS * (1.0 - u).powf(1.35)
        }
    }

    /// A tiny drift while the ship is still far out; gone by contact.
    fn ship_drift(&self, t: f32) -> (f32, f32) {
        let fade = 1.0 - smoothstep(((t - (DOCK_T - 2.0)) / 1.5).clamp(0.0, 1.0));
        (
            (t * 1.3 + self.windows[0]).sin() * 0.55 * fade,
            (t * 1.1 + self.windows[1]).cos() * 0.38 * fade,
        )
    }

    /// Ship position before contact.  It enters at the top-left, joins the
    /// upper-left radial and spirals all the way into the hub.
    fn ship_pos(&self, t: f32) -> (f32, f32) {
        if t < ENTRY_END {
            let p0 = (
                self.w as f32 * 0.07 + self.entry_jx,
                self.h as f32 * 0.12 + self.entry_jy,
            );
            let p1 = (
                self.w as f32 * 0.22 + self.entry_jx * 0.5,
                self.h as f32 * 0.18 + self.entry_jy,
            );
            let p2 = (
                self.w as f32 * 0.36,
                self.h as f32 * 0.06 + self.entry_jy * 0.5,
            );
            let p3 = self.orbit_point(self.approach_angle(ENTRY_END), self.radius * ENTRY_RADIUS);
            let u = ((t - (POWER_END - 0.2)) / (ENTRY_END - (POWER_END - 0.2))).clamp(0.0, 1.0);
            let e = u * u * (3.0 - 2.0 * u);
            return bezier(p0, p1, p2, p3, e);
        }
        let p = self.orbit_point(self.approach_angle(t), self.radius * self.ship_radius(t));
        let d = self.ship_drift(t);
        (p.0 + d.0, p.1 + d.1)
    }

    /// Control room / docking target at the centre of the ring.
    #[inline]
    fn hub_pos(&self) -> (f32, f32) {
        (self.cx, self.cy)
    }

    /// The long docking-arm angle.  At contact it points up-left along the
    /// ship's approach radial.
    fn docking_arm_angle(&self, t: f32) -> f32 {
        OMEGA * t + (APPROACH_ANGLE - OMEGA * DOCK_T)
    }

    /// Ship roll target: a uniform spin at the ring's rate, then a nose-in
    /// lock for the final approach.
    fn ship_roll_target(&self, t: f32) -> f32 {
        let lock = smoothstep(((t - (DOCK_T - 1.6)) / 1.3).clamp(0.0, 1.0));
        let roll = OMEGA * t + self.windows[0];
        let inward = APPROACH_ANGLE + PI;
        roll + angle_delta(roll, inward) * lock
    }

    /// Perspective view of the station at time `t`: centre, radius, scale.
    fn view(&self, t: f32) -> (f32, f32, f32, f32) {
        let mut cx = self.cx;
        let mut cy = self.cy;
        let mut r = self.radius;
        let mut s = 1.0;
        if t >= DEPART_START {
            s = depart_scale(t);
            cx = self.vp_x + (self.cx - self.vp_x) * s;
            cy = self.vp_y + (self.cy - self.vp_y) * s;
            r = self.radius * s;
        }
        if (DOCK_T..DOCK_T + 0.5).contains(&t) {
            let amp = 1.5 * (-((t - DOCK_T) * 10.0)).exp();
            cx += (t * 93.0).sin() * amp;
            cy += (t * 71.0).cos() * amp;
        }
        (cx, cy, r, s)
    }

    fn module_lit(&self, i: usize, t: f32) -> f32 {
        if t >= POWER_END {
            return 1.0;
        }
        let start = 0.20 + i as f32 * 0.065;
        smoothstep((t - start) / 0.36).clamp(0.0, 1.0)
    }

    fn spawn_dock(&mut self, _t: f32) {
        let (cx, cy) = self.hub_pos();
        self.dock_x = cx;
        self.dock_y = cy;
        self.shock_age = 0.0;
        // Contact spray, thrown radially out of the control room.
        for _ in 0..46 {
            let ang = self.rng.range(0.0, TAU);
            let (sa, ca) = ang.sin_cos();
            let speed = self.rng.range(3.0, 19.0);
            let life = self.rng.range(0.4, 1.2);
            self.sparks.push(Spark {
                x: cx + self.rng.range(-0.6, 0.6),
                y: cy + self.rng.range(-0.5, 0.5),
                vx: ca * speed,
                vy: sa * speed * 0.8,
                life,
                max: life,
                drag: 1.5,
                kind: 0,
            });
        }
        // A halo of embers leaving the ring edge.
        for _ in 0..16 {
            let a = self.rng.range(0.0, TAU);
            let p = self.orbit_point(a, self.radius * 0.98);
            let dx = p.0 - self.cx;
            let dy = p.1 - self.cy;
            let l = (dx * dx + dy * dy).sqrt().max(0.001);
            let speed = self.rng.range(2.0, 7.0);
            let life = self.rng.range(0.5, 1.4);
            self.sparks.push(Spark {
                x: p.0,
                y: p.1,
                vx: dx / l * speed + self.rng.range(-1.5, 1.5),
                vy: dy / l * speed + self.rng.range(-1.0, 1.0),
                life,
                max: life,
                drag: 0.9,
                kind: 1,
            });
        }
        // Engine embers, thrown down-right as the stack burns away up-left.
        for k in 0..18 {
            let a = 1.05 + k as f32 * 0.055;
            let p = self.orbit_point(a, self.radius * 0.94);
            let life = self.rng.range(0.9, 1.9);
            self.sparks.push(Spark {
                x: p.0,
                y: p.1,
                vx: self.rng.range(3.0, 11.0),
                vy: self.rng.range(2.0, 8.0),
                life,
                max: life,
                drag: 1.2,
                kind: 1,
            });
        }
    }

    fn draw_stars(&self, canvas: &mut Canvas, t: f32, uni: bool) {
        for s in &self.stars {
            let tw = 0.58 + 0.42 * (t * s.speed + s.phase).sin().abs();
            let col = s.color.scale(s.base * tw);
            if col.luminance() < 0.02 {
                continue;
            }
            let x = s.x + (t * 0.06 + s.phase).sin() * 0.55;
            let ch = if uni {
                match s.ch {
                    '*' => '✦',
                    '+' => '✧',
                    _ => '·',
                }
            } else {
                s.ch
            };
            canvas.set(x as isize, s.y as isize, ch, col);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_section(
        &self,
        canvas: &mut Canvas,
        f: &Frame,
        t: f32,
        vcx: f32,
        vcy: f32,
        vr: f32,
        i: usize,
        a: f32,
        lit: f32,
        flash: f32,
        uni: bool,
    ) {
        if lit <= 0.02 {
            return;
        }
        let center = self.orbit_rel(a, vr);
        let t0 = self.orbit_rel(a - 0.035, vr);
        let t1 = self.orbit_rel(a + 0.035, vr);
        let (mut tx, mut ty) = (t1.0 - t0.0, t1.1 - t0.1);
        let tl = (tx * tx + ty * ty).sqrt().max(0.001);
        tx /= tl;
        ty /= tl;
        let (mut rx, mut ry) = center;
        let rl = (rx * rx + ry * ry).sqrt().max(0.001);
        rx /= rl;
        ry /= rl;
        let width = if vr >= 15.0 {
            7
        } else if vr >= 8.0 {
            5
        } else {
            3
        };
        let layers: &[f32] = if vr >= 8.0 {
            &[-1.5, -0.5, 0.5, 1.5]
        } else {
            &[-1.0, 0.0, 1.0]
        };
        let half = (width as f32 - 1.0) * 0.5;
        let hull = WHITE
            .lerp(f.theme.c(3), 0.38)
            .scale(lit * (1.0 + flash * 0.5));
        let win = WINDOW.scale(lit * (0.65 + 0.35 * (t * 1.4 + self.windows[i]).sin().abs()));
        // Every cabin gets the same three-cell-thick slab; no far-side block
        // is allowed to disappear into the background.
        for v in layers.iter() {
            for u in 0..width {
                let off = u as f32 - half;
                let x = vcx + center.0 + tx * off + rx * v;
                let y = vcy + center.1 + ty * off + ry * v;
                let centre_layer = (*v).abs() < 1.0;
                let lit_window = centre_layer && (u == width / 2 || u == 0);
                let (ch, col) = if lit_window {
                    (if uni { '▒' } else { '=' }, win)
                } else {
                    let shade = if (*v).abs() > 1.0 { 0.78 } else { 1.0 };
                    (if uni { '█' } else { '#' }, hull.scale(shade))
                };
                canvas.set(x.round() as isize, y.round() as isize, ch, col);
            }
        }
    }

    /// Map a point in the rotating core plane to a screen offset from the hub.
    fn core_offset(&self, spin: f32, lx: f32, ly: f32) -> (f32, f32) {
        let (sa, ca) = spin.sin_cos();
        let (st, ct) = TILT.sin_cos();
        let aspect = CORE_ASPECT;
        let (rx, ry) = (lx * ca - ly * aspect * sa, lx * sa + ly * aspect * ca);
        (rx * ct - ry * st, rx * st + ry * ct)
    }

    /// Inverse of [`Self::core_offset`].
    fn core_local(&self, spin: f32, dx: f32, dy: f32) -> (f32, f32) {
        let (sa, ca) = spin.sin_cos();
        let (st, ct) = TILT.sin_cos();
        let aspect = CORE_ASPECT;
        let vx = dx * ct + dy * st;
        let vy = -dx * st + dy * ct;
        (vx * ca + vy * sa, (-vx * sa + vy * ca) / aspect)
    }

    /// The central control block, built in the station's rotating frame.
    /// Its silhouette, windows, beacon and masts all turn with the ring.
    #[allow(clippy::too_many_arguments)]
    fn draw_core(
        &self,
        canvas: &mut Canvas,
        f: &Frame,
        vcx: f32,
        vcy: f32,
        core_r: f32,
        spin: f32,
        uni: bool,
    ) {
        let theme = f.theme;
        let hull = WHITE.lerp(theme.c(3), 0.32);
        let rim = theme.c(3);
        let apothem = core_r * 0.866_025_4;
        let extent = core_r.ceil() as isize + 1;
        let (hx, hy) = (vcx.round() as isize, vcy.round() as isize);
        for oy in -extent..=extent {
            for ox in -extent..=extent {
                let (lx, ly) = self.core_local(spin, ox as f32, oy as f32);
                let mut d = 0.0f32;
                for k in 0..3 {
                    let n = DOCK_ARM_OFFSET + PI / 6.0 + k as f32 * PI / 3.0;
                    d = d.max((lx * n.cos() + ly * n.sin()).abs());
                }
                let d = d / apothem;
                if d > 1.0 {
                    continue;
                }
                let (ch, col) = if d > 0.68 {
                    (if uni { '▓' } else { '#' }, rim)
                } else {
                    (if uni { '█' } else { '#' }, hull)
                };
                canvas.set(hx + ox, hy + oy, ch, col);
            }
        }
        // Windows are placed in the core frame, so the face keeps turning.
        for (rel, rr) in [(-0.55f32, 0.45f32), (0.45, 0.45), (2.60, 0.50)] {
            let a = DOCK_ARM_OFFSET + rel;
            let (ox, oy) = self.core_offset(spin, a.cos() * rr * core_r, a.sin() * rr * core_r);
            canvas.set(
                (vcx + ox).round() as isize,
                (vcy + oy).round() as isize,
                if uni { '▒' } else { '=' },
                WINDOW.scale(0.85),
            );
        }
        // Six explicit vertices make the block's turn legible even in ASCII.
        for k in 1..6 {
            let a = DOCK_ARM_OFFSET + k as f32 * PI / 3.0;
            let (ox, oy) = self.core_offset(spin, a.cos() * core_r * 0.92, a.sin() * core_r * 0.92);
            canvas.set(
                (vcx + ox).round() as isize,
                (vcy + oy).round() as isize,
                '+',
                rim.scale(1.25),
            );
        }
        // Gold index on the same vertex the long docking arm leaves from.
        let (ix, iy) = self.core_offset(
            spin,
            DOCK_ARM_OFFSET.cos() * core_r * 0.70,
            DOCK_ARM_OFFSET.sin() * core_r * 0.70,
        );
        canvas.set(
            (vcx + ix).round() as isize,
            (vcy + iy).round() as isize,
            if uni { '✦' } else { '*' },
            SPARK_GOLD,
        );
        // Two short masts on the rotating block instead of fixed verticals.
        if core_r > 3.0 {
            for side in [-1.0f32, 1.0] {
                let a = DOCK_ARM_OFFSET + side * FRAC_PI_2;
                let (sa, ca) = a.sin_cos();
                let (bx, by) = self.core_offset(spin, ca * core_r * 0.90, sa * core_r * 0.90);
                let (dir_x, dir_y) = self.core_offset(spin, ca, sa);
                let dl = (dir_x * dir_x + dir_y * dir_y).sqrt().max(0.001);
                let (nx, ny) = (dir_x / dl, dir_y / dl);
                let ch = shaft_glyph(nx, ny);
                for step in 0..=2 {
                    canvas.set(
                        (vcx + bx + nx * step as f32).round() as isize,
                        (vcy + by + ny * step as f32).round() as isize,
                        if step == 2 { '+' } else { ch },
                        theme.c(1).scale(1.35),
                    );
                }
            }
        }
        canvas.set(hx, hy, if uni { '◉' } else { '@' }, WHITE);
        canvas.add_light(hx, hy, theme.c(2), 0.5, '.');
    }

    fn draw_station(
        &self,
        canvas: &mut Canvas,
        f: &Frame,
        t: f32,
        view: (f32, f32, f32, f32),
        flash: f32,
        uni: bool,
    ) {
        let (vcx, vcy, vr, s) = view;
        let theme = f.theme;
        if vr < 0.55 {
            canvas.set(
                vcx.round() as isize,
                vcy.round() as isize,
                if uni { '✷' } else { '*' },
                WHITE.scale(0.7),
            );
            return;
        }
        if vr < 2.6 {
            // Distant LOD: a dotted ring with a hot core.
            let n = 14;
            for k in 0..n {
                let a = k as f32 / n as f32 * TAU + OMEGA * t;
                let p = self.orbit_rel(a, vr);
                let ch = if k % 3 == 0 { '+' } else { '.' };
                canvas.set(
                    (vcx + p.0).round() as isize,
                    (vcy + p.1).round() as isize,
                    ch,
                    theme.c(3).scale(0.45 + flash * 0.5),
                );
            }
            canvas.set(
                vcx.round() as isize,
                vcy.round() as isize,
                if uni { '◉' } else { '@' },
                WHITE,
            );
            return;
        }

        let spin = OMEGA * t;
        // A thin inner truss, so the gaps between sections read as one ring.
        let steps = ((vr * 4.0) as usize).clamp(28, 140);
        for k in (0..steps).step_by(3) {
            let a = k as f32 / steps as f32 * TAU;
            let p = self.orbit_rel(a, vr * 0.88);
            canvas.set(
                (vcx + p.0).round() as isize,
                (vcy + p.1).round() as isize,
                if uni { '·' } else { '.' },
                theme.c(1).scale(0.9),
            );
        }
        // A solid inner ring joins the eight cabins; the cabins sit on it as
        // thick bulges, so the station reads as one continuous wheel.
        let ring_steps = ((vr * 8.0) as usize).clamp(80, 260);
        for k in 0..ring_steps {
            let a = k as f32 / ring_steps as f32 * TAU;
            let p = self.orbit_rel(a, vr);
            let q = self.orbit_rel(a + 0.011, vr);
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let ch = if dx.abs() > dy.abs() * 1.6 {
                '-'
            } else if dy.abs() > dx.abs() * 1.6 {
                '|'
            } else if dx * dy > 0.0 {
                '\\'
            } else {
                '/'
            };
            canvas.set(
                (vcx + p.0).round() as isize,
                (vcy + p.1).round() as isize,
                ch,
                theme.c(3).scale(0.82),
            );
        }
        // Far half first.
        for i in 0..MODULES {
            let a = spin + i as f32 * TAU / MODULES as f32;
            if a.sin() < 0.0 {
                self.draw_section(
                    canvas,
                    f,
                    t,
                    vcx,
                    vcy,
                    vr,
                    i,
                    a,
                    self.module_lit(i, t),
                    flash,
                    uni,
                );
            }
        }
        // Eight struts: every ring section is tied back to the control room.
        let spoke_lit = smoothstep(t / 1.1);
        for i in 0..MODULES {
            let a = spin + i as f32 * TAU / MODULES as f32;
            let p0 = self.orbit_rel(a, vr * 0.16);
            let p1 = self.orbit_rel(a, vr * 0.94);
            let (x0, y0) = (vcx + p0.0, vcy + p0.1);
            let (x1, y1) = (vcx + p1.0, vcy + p1.1);
            let (dx, dy) = (x1 - x0, y1 - y0);
            let ch = if dx.abs() > dy.abs() * 1.6 {
                '-'
            } else if dy.abs() > dx.abs() * 1.6 {
                '|'
            } else if dx * dy > 0.0 {
                '\\'
            } else {
                '/'
            };
            let col = if a.sin() > 0.0 {
                theme.c(2).scale(0.48 * spoke_lit)
            } else {
                theme.c(1).scale(0.78 * spoke_lit)
            };
            let steps = dx.abs().max(dy.abs()).ceil().max(1.0) as i32;
            for step in 0..=steps {
                let u = step as f32 / steps as f32;
                canvas.set(
                    (x0 + dx * u).round() as isize,
                    (y0 + dy * u).round() as isize,
                    ch,
                    col,
                );
            }
        }
        // Control room: a hexagonal core that turns with the ring.
        let hx = vcx.round() as isize;
        let hy = vcy.round() as isize;
        let core_r = if vr > 14.0 {
            4.4
        } else if vr > 8.0 {
            3.3
        } else {
            2.4
        };
        self.draw_core(canvas, f, vcx, vcy, core_r, spin, uni);
        // A three-arm docking claw rotates with the whole station; the long
        // gold arm is the index the ship lines up with at contact.
        let arm_a = self.docking_arm_angle(t);
        let arm_base = core_r * 0.75;
        let arm_len = if vr > 14.0 { 6.0 } else { 4.0 };
        for k in 0..3 {
            let ang = arm_a + k as f32 * TAU / 3.0;
            let len: f32 = if k == 0 {
                arm_len + 1.5
            } else {
                arm_len * 0.55
            };
            let (ax, ay) = (ang.cos(), ang.sin() * 0.85);
            let ch = if ax.abs() > ay.abs() * 1.4 {
                '-'
            } else if ay.abs() > ax.abs() * 1.4 {
                '|'
            } else if ax * ay > 0.0 {
                '\\'
            } else {
                '/'
            };
            let col = if k == 0 { SPARK_GOLD } else { PORT.scale(0.55) };
            let steps = len.ceil().max(1.0) as i32;
            for step in 1..=steps {
                let rr = arm_base + step as f32;
                canvas.set(
                    (vcx + ax * rr).round() as isize,
                    (vcy + ay * rr).round() as isize,
                    if k == 0 && step == steps { '*' } else { ch },
                    col,
                );
            }
        }
        // The hub is the docking target; its beacon breathes as the ship closes.
        let urgency = ((t - (DOCK_T - 6.0)) / 6.0).clamp(0.0, 1.0);
        let pulse = 0.55 + 0.45 * ((t * (2.4 + urgency * 5.0)).sin().abs());
        canvas.add_light(hx, hy, PORT, 0.25 + 0.40 * pulse + flash * 0.6, '.');
        if (DOCK_T - 1.2..DOCK_T).contains(&t) && vr > 4.0 {
            canvas.set(
                hx,
                hy,
                if uni { '◈' } else { '+' },
                PORT.scale(0.8 + 0.5 * pulse),
            );
        }

        // Near half on top: the second four cabins.
        for i in 0..MODULES {
            let a = spin + i as f32 * TAU / MODULES as f32;
            if a.sin() >= 0.0 {
                self.draw_section(
                    canvas,
                    f,
                    t,
                    vcx,
                    vcy,
                    vr,
                    i,
                    a,
                    self.module_lit(i, t),
                    flash,
                    uni,
                );
            }
        }

        // Power-up scan: a single bead running around the ring.
        if t < POWER_END {
            let u = ((t - 0.25) / (POWER_END - 0.65)).clamp(0.0, 1.0);
            if u > 0.0 {
                let a = -PI * 0.5 + u * TAU * 1.15;
                let p = self.orbit_rel(a, vr);
                canvas.set(
                    (vcx + p.0).round() as isize,
                    (vcy + p.1).round() as isize,
                    if uni { '✦' } else { '*' },
                    WHITE,
                );
                for (j, back) in [0.10_f32, 0.19, 0.28].iter().enumerate() {
                    let q = self.orbit_rel(a - back, vr);
                    canvas.set(
                        (vcx + q.0).round() as isize,
                        (vcy + q.1).round() as isize,
                        if j == 2 { '.' } else { '+' },
                        PORT.scale(0.8 - j as f32 * 0.2),
                    );
                }
            }
        }
        let _ = s;
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_ship(
        &self,
        canvas: &mut Canvas,
        f: &Frame,
        sx: f32,
        sy: f32,
        spin: f32,
        power: f32,
        scale: f32,
        uni: bool,
    ) {
        let dx = spin.cos();
        let dy = spin.sin() * 0.55;
        let l = (dx * dx + dy * dy).sqrt().max(0.001);
        let (ux, uy) = (dx / l, dy / l);
        let (wx, wy) = (-uy, ux);
        let theme = f.theme;
        let at = |ox: f32, oy: f32| ((sx + ox).round() as isize, (sy + oy).round() as isize);
        let cf = SHIP_CORE_FWD * scale;
        let cs = SHIP_CORE_LAT * scale;
        let body = WHITE.lerp(theme.c(3), 0.38);
        let rim = theme.c(3);

        // A small solid core instead of the old saucer-wide hull.
        let extent = (cf + 1.0).ceil() as isize;
        for oy in -extent..=extent {
            for ox in -extent..=extent {
                let fwd = ox as f32 * ux + oy as f32 * uy;
                let lat = -ox as f32 * uy + oy as f32 * ux;
                let n = (fwd / cf).powi(2) + (lat / cs).powi(2);
                if n <= 1.0 {
                    let (ch, col) = if n > 0.58 {
                        (if uni { '█' } else { '#' }, rim)
                    } else {
                        (if uni { '█' } else { '#' }, body)
                    };
                    canvas.set(
                        (sx + ox as f32).round() as isize,
                        (sy + oy as f32).round() as isize,
                        ch,
                        col,
                    );
                }
            }
        }

        // Four tentacles.  Unequal lengths plus the gold index tip make the
        // roll read at a glance, even while the ship is still far out.
        let mut tips = [(0isize, 0isize); 4];
        for k in 0..4 {
            let ang = k as f32 * FRAC_PI_2;
            let (sa, ca) = ang.sin_cos();
            let (ax, ay) = (ux * ca + wx * sa, uy * ca + wy * sa);
            let len = SHIP_ARM_LEN[k] * scale;
            let start = if k % 2 == 0 { cf } else { cs };
            let tip_col = match k {
                0 => SPARK_GOLD,
                2 => PLUME,
                _ => PORT,
            };
            let steps = (len - start).max(0.0).ceil() as i32;
            for step in 1..=steps {
                let r = start + step as f32;
                if r > len + 0.01 {
                    break;
                }
                let outer = r > len * 0.78;
                let (ch, col) = if outer {
                    (if k == 0 { '+' } else { '.' }, tip_col.scale(0.8))
                } else {
                    (
                        shaft_glyph(ax, ay),
                        if k == 0 { body } else { rim.scale(0.8) },
                    )
                };
                canvas.set(
                    (sx + ax * r).round() as isize,
                    (sy + ay * r).round() as isize,
                    ch,
                    col,
                );
            }
            let tip = at(ax * len, ay * len);
            tips[k] = tip;
            match k {
                0 => {
                    canvas.set(
                        tip.0,
                        tip.1,
                        if uni { '◆' } else { nose_glyph(spin) },
                        SPARK_GOLD,
                    );
                    canvas.add_light(tip.0, tip.1, SPARK_GOLD, 0.45, '*');
                }
                1 => {}
                2 => canvas.set(tip.0, tip.1, if uni { '✦' } else { '*' }, PLUME),
                _ => canvas.set(tip.0, tip.1, '+', PORT.scale(0.75)),
            }
        }
        // Blinking navigation light on the starboard tentacle tip.
        let blink = ((f.t as f32) * 1.35).fract() < 0.42;
        canvas.set(
            tips[1].0,
            tips[1].1,
            '+',
            if blink {
                WINDOW.scale(0.95)
            } else {
                PORT.scale(0.5)
            },
        );

        // Cockpit and a short lit window strip on the compact core.
        let (cx, cy) = at(ux * cf * 0.45, uy * cf * 0.45);
        canvas.set(cx, cy, if uni { '◉' } else { '@' }, WHITE);
        for k in -1..=1 {
            let (x, y) = at(wx * k as f32 * cs * 0.55, wy * k as f32 * cs * 0.55);
            canvas.set(x, y, if uni { '▒' } else { '=' }, WINDOW.scale(0.85));
        }
        canvas.add_light(
            sx.round() as isize,
            sy.round() as isize,
            theme.c(2),
            0.35,
            '.',
        );

        // Engine glow, always a little alive.
        if power > 0.04 {
            let tail = SHIP_ARM_LEN[2] * scale;
            for j in 1..=3 {
                let k = j as f32;
                let flick = 0.55 + 0.45 * ((f.t as f32 * 22.0 + j as f32 * 1.7).sin().abs());
                let col = PLUME
                    .lerp(theme.c(2), k / 4.0)
                    .scale(power * flick * (1.0 - k / 4.5));
                let ch = if j == 1 {
                    '*'
                } else if j == 2 {
                    '+'
                } else {
                    '.'
                };
                canvas.set(
                    (sx - ux * (tail + 0.5 + k * 0.55)).round() as isize,
                    (sy - uy * (tail + 0.5 + k * 0.55)).round() as isize,
                    ch,
                    col,
                );
            }
            canvas.add_light(
                (sx - ux * (tail + 0.8)).round() as isize,
                (sy - uy * (tail + 0.8)).round() as isize,
                PLUME,
                0.45 * power,
                '.',
            );
        }
    }

    fn draw_speed_lines(&self, canvas: &mut Canvas, f: &Frame, t: f32, view: (f32, f32, f32, f32)) {
        let (vcx, vcy, vr, _) = view;
        let (mut dx, mut dy) = (vcx - self.vp_x, vcy - self.vp_y);
        let l = (dx * dx + dy * dy).sqrt().max(0.001);
        dx /= l;
        dy /= l;
        let (px, py) = (-dy, dx);
        let u = ((t - DEPART_START) / (DEPART_END - DEPART_START)).clamp(0.0, 1.0);
        let fade = (1.0 - u).powi(2);
        for k in 0..6 {
            let off = (k as f32 - 2.5) * vr * 0.35;
            for j in 1..=4 {
                let back = j as f32 * vr * 0.28;
                canvas.set(
                    (vcx + px * off - dx * back).round() as isize,
                    (vcy + py * off - dy * back).round() as isize,
                    '.',
                    f.theme.c(1).scale(fade * 0.55),
                );
            }
        }
    }

    fn draw_ghost(&self, canvas: &mut Canvas, f: &Frame, t: f32, view: (f32, f32, f32, f32)) {
        let (_vcx, _vcy, vr, s) = view;
        if vr < 3.0 {
            return;
        }
        let u = ((t - DEPART_START) / (DEPART_END - DEPART_START)).clamp(0.0, 1.0);
        if u > 0.82 {
            return;
        }
        let spin = OMEGA * t;
        let theme = f.theme;
        let fade = (1.0 - u).powf(0.8) * smoothstep(u / 0.12);
        // Three short, low-alpha copies of the wheel sit just behind the
        // receding station.  Each older copy is also turned back a little,
        // so the trail reads as motion rather than as a static echo.
        for (age, k, alpha) in [
            (1.0f32, 1.10f32, 0.22f32),
            (2.0, 1.17, 0.12),
            (3.0, 1.24, 0.06),
        ] {
            let gs = s * k;
            let gx = self.vp_x + (self.cx - self.vp_x) * gs;
            let gy = self.vp_y + (self.cy - self.vp_y) * gs;
            let gr = self.radius * gs;
            let alpha = alpha * fade;
            let lag = age * 0.045;
            let steps = ((gr * 2.6) as usize).clamp(20, 64);
            for j in 0..steps {
                let a = j as f32 / steps as f32 * TAU;
                let p = self.orbit_rel(a, gr);
                canvas.set(
                    (gx + p.0).round() as isize,
                    (gy + p.1).round() as isize,
                    '.',
                    theme.c(2).scale(alpha * 0.8),
                );
            }
            for i in 0..MODULES {
                let a = spin - lag + i as f32 * TAU / MODULES as f32;
                let c = self.orbit_rel(a, gr);
                let p0 = self.orbit_rel(a - 0.02, gr);
                let p1 = self.orbit_rel(a + 0.02, gr);
                let (mut tx, mut ty) = (p1.0 - p0.0, p1.1 - p0.1);
                let tl = (tx * tx + ty * ty).sqrt().max(0.001);
                tx /= tl;
                ty /= tl;
                for off in [-1.0f32, 0.0, 1.0] {
                    let (ch, col) = if off == 0.0 {
                        ('=', WINDOW.scale(alpha * 0.55))
                    } else {
                        ('#', WHITE.lerp(theme.c(3), 0.4).scale(alpha * 0.75))
                    };
                    canvas.set(
                        (gx + c.0 + tx * off).round() as isize,
                        (gy + c.1 + ty * off).round() as isize,
                        ch,
                        col,
                    );
                }
                for q in [0.28f32, 0.48, 0.68, 0.86] {
                    let p = self.orbit_rel(a, gr * q);
                    canvas.set(
                        (gx + p.0).round() as isize,
                        (gy + p.1).round() as isize,
                        '.',
                        theme.c(1).scale(alpha * 0.5),
                    );
                }
            }
        }
    }

    fn draw_plume(&self, canvas: &mut Canvas, f: &Frame, t: f32, view: (f32, f32, f32, f32)) {
        let (vcx, vcy, vr, _s) = view;
        if vr < 2.0 {
            return;
        }
        let u = ((t - DEPART_START) / (DEPART_END - DEPART_START)).clamp(0.0, 1.0);
        let strength = (1.0 - u).powi(2);
        if strength < 0.03 {
            return;
        }
        // The stack is moving up-left, so the exhaust leaves down-right.
        let (mut ax, mut ay) = (vcx - self.vp_x, vcy - self.vp_y);
        let l = (ax * ax + ay * ay).sqrt().max(0.001);
        ax /= l;
        ay /= l;
        let (px, py) = (-ay, ax);
        for j in 0..3 {
            let offset = (j as f32 - 1.0) * vr * 0.30;
            let ox = vcx + ax * vr * 0.52 + px * offset;
            let oy = vcy + ay * vr * 0.52 + py * offset;
            let len = (3.0 + 9.0 * strength) * (0.8 + 0.1 * j as f32);
            for k in 1..=len as usize {
                let fk = k as f32 / len;
                let col = if k <= 2 {
                    WHITE.lerp(PLUME, fk)
                } else {
                    PLUME.lerp(SPARK_GOLD, fk * 0.55)
                }
                .scale(strength * (1.0 - fk) * 1.15);
                let ch = if k == 1 {
                    '*'
                } else if k <= 4 {
                    '+'
                } else {
                    '.'
                };
                canvas.set(
                    (ox + ax * k as f32).round() as isize,
                    (oy + ay * k as f32).round() as isize,
                    ch,
                    col,
                );
            }
            canvas.add_light(
                ox.round() as isize,
                oy.round() as isize,
                PLUME,
                0.85 * strength,
                '*',
            );
        }
        let _ = f;
    }

    fn draw_shock(&self, canvas: &mut Canvas, t: f32) {
        if !self.dock_spawned || self.shock_age < 0.0 || self.shock_age > 1.15 {
            return;
        }
        let age = self.shock_age;
        let alpha = (1.0 - age / 1.15).powi(2);
        for ring in 0..2 {
            let rad = 1.5 + age * (14.0 + ring as f32 * 5.0);
            let a = alpha * (0.95 - ring as f32 * 0.45);
            for k in 0..34 {
                let ang = k as f32 / 34.0 * TAU;
                canvas.set(
                    (self.dock_x + ang.cos() * rad).round() as isize,
                    (self.dock_y + ang.sin() * rad * 0.55).round() as isize,
                    if k % 3 == 0 { '+' } else { '.' },
                    WHITE.scale(a),
                );
            }
        }
        canvas.add_light(
            self.dock_x.round() as isize,
            self.dock_y.round() as isize,
            WHITE,
            alpha * 1.5,
            '.',
        );
        let _ = t;
    }

    fn draw_sparks(&self, canvas: &mut Canvas, uni: bool) {
        for p in &self.sparks {
            let frac = (p.life / p.max).clamp(0.0, 1.0);
            let col = spark_color(frac);
            let ch = if frac > 0.68 {
                '*'
            } else if frac > 0.36 {
                '+'
            } else if frac > 0.12 {
                '.'
            } else {
                '\''
            };
            let ch = if uni {
                match ch {
                    '*' => '✦',
                    '+' => '✧',
                    _ => '·',
                }
            } else {
                ch
            };
            canvas.set(p.x.round() as isize, p.y.round() as isize, ch, col);
            if frac > 0.55 || p.kind == 1 {
                canvas.add_light(
                    p.x.round() as isize,
                    p.y.round() as isize,
                    col,
                    0.22 + 0.2 * frac,
                    '.',
                );
            }
        }
    }

    fn draw_star(&self, canvas: &mut Canvas, t: f32, uni: bool) {
        if t < DEPART_END {
            return;
        }
        let age = t - DEPART_END;
        if age < 0.0 {
            return;
        }
        let p1 = gauss(age, 0.22, 0.10);
        let p2 = gauss(age, 0.62, 0.13) * 1.25;
        let p = p1.max(p2).clamp(0.0, 1.0);
        if p < 0.025 {
            if age < 1.8 {
                let fade = (1.0 - age / 1.8).max(0.0) * 0.18;
                canvas.set(
                    self.vp_x.round() as isize,
                    self.vp_y.round() as isize,
                    '.',
                    WHITE.scale(fade),
                );
            }
            return;
        }
        let vx = self.vp_x.round() as isize;
        let vy = self.vp_y.round() as isize;
        let core = if uni { '✷' } else { '*' };
        canvas.set(vx, vy, core, WHITE);
        canvas.add_light(vx, vy, WHITE, 1.2 + 1.6 * p, '.');
        if p > 0.45 {
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                canvas.set(vx + dx, vy + dy, '+', WHITE.scale(p));
            }
        }
        let second = p2 > p1;
        let arms = if second { 4 } else { 3 };
        for i in 1..=arms {
            let a = (1.0 - i as f32 / (arms as f32 + 1.0)) * p;
            let ch = if i == 1 { '-' } else { '.' };
            canvas.set(vx + i, vy, ch, WHITE.scale(a));
            canvas.set(vx - i, vy, ch, WHITE.scale(a));
            let ch = if i == 1 { '|' } else { '.' };
            canvas.set(vx, vy + i, ch, WHITE.scale(a));
            canvas.set(vx, vy - i, ch, WHITE.scale(a));
        }
        if second && p2 > 0.55 {
            for i in 1..=3 {
                let a = p * (0.75 - i as f32 * 0.18);
                canvas.set(vx + i, vy + i, '\\', WHITE.scale(a));
                canvas.set(vx - i, vy - i, '\\', WHITE.scale(a));
                canvas.set(vx + i, vy - i, '/', WHITE.scale(a));
                canvas.set(vx - i, vy + i, '/', WHITE.scale(a));
            }
        }
        if p > 0.30 {
            let rr = 4.0 + age * 4.0;
            for k in 0..8 {
                let ang = k as f32 / 8.0 * TAU + 0.2;
                canvas.set(
                    (self.vp_x + ang.cos() * rr).round() as isize,
                    (self.vp_y + ang.sin() * rr * 0.6).round() as isize,
                    '.',
                    WHITE.scale(0.20 * p),
                );
            }
        }
    }
}

impl Scene for InterstellarScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let t = (f.t as f32).rem_euclid(CYCLE);
        if t < POWER_END {
            return Some("STATION ONLINE".to_string());
        }
        if t < DOCK_T {
            if t < ENTRY_END {
                return Some(format!("DOCK T-{:>4.1}s  INBOUND", DOCK_T - t));
            }
            return Some(format!(
                "DOCK T-{:>4.1}s  SYNC {:>3.0}%",
                DOCK_T - t,
                self.sync * 100.0
            ));
        }
        if t < DEPART_START {
            return Some("DOCKED  SPARKS".to_string());
        }
        if t < DEPART_END {
            return Some("BURN  DEPART".to_string());
        }
        if t < STAR_END {
            return Some("STAR  CONTACT LOST".to_string());
        }
        if t < STAR_END + FADE_OUT {
            return Some("FADING OUT".to_string());
        }
        Some("FADING IN".to_string())
    }

    fn update(&mut self, f: &Frame) {
        let t = (f.t as f32).rem_euclid(CYCLE);
        let dt = f.dt.clamp(0.0, 0.1);
        let idx = (f.t / CYCLE as f64).floor() as i64;
        if idx != self.cycle {
            self.cycle = idx;
            self.sparks.clear();
            self.shock_age = -1.0;
            self.dock_spawned = false;
            self.spin = f32::NAN;
        }
        // Approach progress: 0 at the entry radius, 1 at the hub.
        self.sync = (1.0 - self.ship_radius(t) / ENTRY_RADIUS).clamp(0.0, 1.0);

        if t >= DOCK_T && !self.dock_spawned {
            self.spawn_dock(t);
            self.dock_spawned = true;
        }

        // Roll: uniform, slow rotation at the ring's rate, then a smooth
        // nose-in lock for the final metre.  No spring: the rotation is even.
        if t < DOCK_T {
            self.spin = self.ship_roll_target(t);
        }

        if self.dock_spawned && self.shock_age >= 0.0 {
            self.shock_age += dt;
            if self.shock_age > 1.2 {
                self.shock_age = -1.0;
            }
        }
        for p in self.sparks.iter_mut() {
            let drag = (1.0 - p.drag * dt).clamp(0.0, 1.0);
            p.vx *= drag;
            p.vy *= drag;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.sparks.retain(|p| p.life > 0.0);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let t = (f.t as f32).rem_euclid(CYCLE);
        let uni = f.charset == Charset::Unicode;
        self.draw_stars(canvas, t, uni);

        let view = self.view(t);
        let flash = if (DOCK_T..DOCK_T + 0.5).contains(&t) {
            (1.0 - (t - DOCK_T) / 0.5).powi(2)
        } else {
            0.0
        };

        if (DEPART_START..DEPART_END).contains(&t) {
            self.draw_speed_lines(canvas, f, t, view);
            self.draw_ghost(canvas, f, t, view);
        }
        if t < DEPART_END {
            self.draw_station(canvas, f, t, view, flash, uni);
        }
        if (DEPART_START..DEPART_END).contains(&t) {
            self.draw_plume(canvas, f, t, view);
        }

        // Ship: flying until contact, then part of the shrinking stack.
        if (POWER_END - 0.2..DOCK_T).contains(&t) {
            let p = self.ship_pos(t);
            let scale = (view.2 / 12.0).clamp(0.85, 1.55);
            let power = if t < ENTRY_END {
                0.85 - 0.35 * smoothstep(((t - 4.0) / 3.5).clamp(0.0, 1.0))
            } else if t > DOCK_T - 1.6 {
                0.55
            } else {
                0.32
            };
            let spin = if self.spin.is_finite() {
                self.spin
            } else {
                0.0
            };
            self.draw_ship(canvas, f, p.0, p.1, spin, power, scale, uni);
            // Guide line from the ship to the hub during the final run.
            if t > ENTRY_END + 2.0 {
                let hub = self.hub_pos();
                let d = ((p.0 - hub.0).powi(2) + (p.1 - hub.1).powi(2)).sqrt();
                if d > 0.8 {
                    let n = ((d * 0.9).ceil() as usize).clamp(3, 14);
                    let fade = (1.0 - d / (view.2 * 1.6)).clamp(0.15, 0.8);
                    for k in 1..n {
                        let u = k as f32 / n as f32;
                        canvas.set(
                            lerp(p.0, hub.0, u).round() as isize,
                            lerp(p.1, hub.1, u).round() as isize,
                            '.',
                            PORT.scale(fade * 0.6),
                        );
                    }
                }
            }
        } else if t >= DOCK_T {
            let (vcx, vcy, vr, s) = view;
            if vr > 2.2 && s > 0.35 {
                canvas.set(
                    vcx.round() as isize,
                    vcy.round() as isize,
                    if s > 0.6 { '+' } else { '*' },
                    WHITE,
                );
            }
        }

        self.draw_shock(canvas, t);
        self.draw_sparks(canvas, uni);

        // Contact flash sits above the ship and sparks.
        if flash > 0.02 {
            let (vcx, vcy, _vr, _) = view;
            let x = vcx.round() as isize;
            let y = vcy.round() as isize;
            canvas.set(x, y, if uni { '✷' } else { '*' }, WHITE);
            canvas.add_light(x, y, WHITE, 2.0 * flash, '.');
            for (dx, dy) in [
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                canvas.set(x + dx, y + dy, '+', WHITE.scale(flash * 0.85));
            }
            for i in 2..=4 {
                let a = flash * (1.0 - i as f32 / 5.0);
                canvas.set(x + i, y, '-', WHITE.scale(a));
                canvas.set(x - i, y, '-', WHITE.scale(a));
                canvas.set(x, y + i, '|', WHITE.scale(a));
                canvas.set(x, y - i, '|', WHITE.scale(a));
            }
        }

        self.draw_star(canvas, t, uni);
        fade_scene(canvas, fade_level(t));
    }
}

fn smoothstep(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The scene stays at 1.0, dips to black over half a second at the end of
/// the star beat, then recovers over another half second before the loop.
fn fade_level(t: f32) -> f32 {
    let black = STAR_END + FADE_OUT;
    if t < STAR_END {
        1.0
    } else if t < black {
        1.0 - smoothstep((t - STAR_END) / FADE_OUT)
    } else {
        smoothstep((t - black) / FADE_IN)
    }
}

/// Dim every painted cell toward black.  The shared HUD is drawn after the
/// scene each frame, so its own rows are painted back at full brightness.
fn fade_scene(canvas: &mut Canvas, level: f32) {
    if level >= 0.999 {
        return;
    }
    let level = level.clamp(0.0, 1.0);
    for cell in &mut canvas.cells {
        if cell.ch != ' ' {
            cell.fg = cell.fg.scale(level);
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Shortest signed angle from `a` to `b`.
fn angle_delta(a: f32, b: f32) -> f32 {
    let mut d = (b - a).rem_euclid(TAU);
    if d > PI {
        d -= TAU;
    }
    d
}

fn bezier(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    (
        p0.0 * a + p1.0 * b + p2.0 * c + p3.0 * d,
        p0.1 * a + p1.1 * b + p2.1 * c + p3.1 * d,
    )
}

/// Line glyph for a shaft pointing along `(dx, dy)`.
fn shaft_glyph(dx: f32, dy: f32) -> char {
    if dx.abs() > dy.abs() * 1.6 {
        '-'
    } else if dy.abs() > dx.abs() * 1.6 {
        '|'
    } else if dx * dy > 0.0 {
        '\\'
    } else {
        '/'
    }
}

fn nose_glyph(spin: f32) -> char {
    let oct = ((spin / (TAU / 8.0)).round() as i32).rem_euclid(8);
    match oct {
        0 => '>',
        1 => '\\',
        2 => 'v',
        3 => '/',
        4 => '<',
        5 => '\\',
        6 => '^',
        _ => '/',
    }
}

fn spark_color(frac: f32) -> Rgb {
    let f = frac.clamp(0.0, 1.0);
    if f > 0.72 {
        SPARK_GOLD.lerp(SPARK_HOT, (f - 0.72) / 0.28)
    } else if f > 0.45 {
        SPARK_ORANGE.lerp(SPARK_GOLD, (f - 0.45) / 0.27)
    } else if f > 0.18 {
        SPARK_RED.lerp(SPARK_ORANGE, (f - 0.18) / 0.27)
    } else {
        SPARK_RED.scale(f / 0.18)
    }
}

fn gauss(age: f32, at: f32, width: f32) -> f32 {
    (-((age - at) / width).powi(2)).exp()
}

fn depart_scale(t: f32) -> f32 {
    let u = ((t - DEPART_START) / (DEPART_END - DEPART_START)).clamp(0.0, 1.0);
    let base = 1.0 / (1.0 + 8.0 * u * u);
    let collapse = 1.0 - 0.62 * smoothstep(((u - 0.82) / 0.18).clamp(0.0, 1.0));
    (base * collapse).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contact_is_at_twenty_seconds() {
        assert!((DOCK_T - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ship_meets_the_hub_at_contact() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let ship = s.ship_pos(DOCK_T);
        assert!((ship.0 - s.cx).abs() < 0.6, "x gap {}", ship.0 - s.cx);
        assert!((ship.1 - s.cy).abs() < 0.6, "y gap {}", ship.1 - s.cy);
    }

    #[test]
    fn ship_approaches_from_the_upper_left() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let entry = s.ship_pos(ENTRY_END);
        assert!(entry.0 < s.cx - 5.0, "entry should be left of the hub");
        assert!(entry.1 < s.cy - 5.0, "entry should be above the hub");
        let start = s.ship_pos(POWER_END);
        assert!(
            start.0 < s.w as f32 * 0.35,
            "entry should start at the left edge"
        );
        assert!(
            start.1 < s.h as f32 * 0.35,
            "entry should start near the top"
        );
        assert!(s.ship_radius(DOCK_T) < 0.001);
        assert!(s.ship_radius(DOCK_T - 0.2) < s.ship_radius(ENTRY_END));
    }

    #[test]
    fn all_eight_cabins_are_painted() {
        use crate::scene::Tune;
        use crate::theme;
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = Frame {
            t: 10.5,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: theme::find("ice").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        };
        let mut canvas = Canvas::new(120, 36);
        s.draw_station(&mut canvas, &f, 10.5, s.view(10.5), 0.0, false);
        let spin = OMEGA * 10.5;
        for i in 0..8 {
            let a = spin + i as f32 * TAU / 8.0;
            let p = s.orbit_point(a, s.radius);
            let mut found = false;
            for dy in -3..=3 {
                for dx in -3..=3 {
                    if let Some(c) =
                        canvas.get(p.0.round() as isize + dx, p.1.round() as isize + dy)
                    {
                        if c.ch == '#' || c.ch == '=' {
                            found = true;
                        }
                    }
                }
            }
            assert!(found, "cabin {i} is missing near {p:?}");
        }
    }

    #[test]
    fn docking_claw_and_ship_align_at_contact() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let arm = s.docking_arm_angle(DOCK_T);
        assert!(
            angle_delta(arm, APPROACH_ANGLE).abs() < 1e-3,
            "long arm should point along the approach at contact"
        );
        let roll = s.ship_roll_target(DOCK_T);
        assert!(
            angle_delta(roll, APPROACH_ANGLE + PI).abs() < 1e-3,
            "ship nose should point inward at contact"
        );
    }

    #[test]
    fn ship_hull_is_solid() {
        use crate::scene::Tune;
        use crate::theme;
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = Frame {
            t: 10.5,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: theme::find("ice").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        };
        let mut canvas = Canvas::new(120, 36);
        s.draw_ship(&mut canvas, &f, 60.0, 18.0, 0.7, 0.0, 1.2, false);
        let ink = canvas.cells.iter().filter(|c| c.ch != ' ').count();
        assert!(ink >= 14, "ship hull only {ink} cells");
    }

    fn test_frame(t: f32) -> Frame<'static> {
        use crate::scene::Tune;
        use crate::theme;
        Frame {
            t: t as f64,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: theme::find("ice").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        }
    }

    #[test]
    fn ship_has_a_small_core_and_four_tentacles() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = test_frame(10.5);
        let mut canvas = Canvas::new(120, 36);
        s.draw_ship(&mut canvas, &f, 60.0, 18.0, 0.0, 0.0, 1.2, false);
        for (dx, dy) in [(3isize, 0isize), (-3, 0), (0, 3), (0, -3)] {
            let mut found = false;
            for oy in -1..=1 {
                for ox in -1..=1 {
                    if canvas
                        .get(60 + dx + ox, 18 + dy + oy)
                        .is_some_and(|c| c.ch != ' ')
                    {
                        found = true;
                    }
                }
            }
            assert!(found, "tentacle at ({dx}, {dy}) is missing");
        }
        // Index arm points right at roll 0 and ends in the nose glyph.
        assert_eq!(canvas.get(65, 18).unwrap().ch, '>');
        let core_ink = (-2..=2)
            .flat_map(|oy| (-2..=2).map(move |ox| (ox, oy)))
            .filter(|(ox, oy)| canvas.get(60 + ox, 18 + oy).is_some_and(|c| c.ch != ' '))
            .count();
        assert!(core_ink >= 6, "compact core only {core_ink} cells");
    }

    #[test]
    fn ship_tentacles_move_with_the_roll() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = test_frame(10.5);
        let mut a = Canvas::new(120, 36);
        let mut b = Canvas::new(120, 36);
        s.draw_ship(&mut a, &f, 60.0, 18.0, 0.0, 0.0, 1.2, false);
        s.draw_ship(&mut b, &f, 60.0, 18.0, 0.7, 0.0, 1.2, false);
        let mut diff = 0;
        for y in 8..28 {
            for x in 48..72 {
                if a.get(x, y).unwrap().ch != b.get(x, y).unwrap().ch {
                    diff += 1;
                }
            }
        }
        assert!(diff >= 8, "roll changed only {diff} cells");
    }

    #[test]
    fn core_geometry_round_trips() {
        let s = InterstellarScene::new(7);
        let (ox, oy) = s.core_offset(0.7, 3.1, -1.4);
        let (lx, ly) = s.core_local(0.7, ox, oy);
        assert!((lx - 3.1).abs() < 1e-3, "lx {lx}");
        assert!((ly + 1.4).abs() < 1e-3, "ly {ly}");
    }

    #[test]
    fn station_core_turns_with_the_ring() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = test_frame(10.5);
        let mut a = Canvas::new(120, 36);
        let mut b = Canvas::new(120, 36);
        s.draw_core(&mut a, &f, s.cx, s.cy, 4.4, 0.0, false);
        s.draw_core(&mut b, &f, s.cx, s.cy, 4.4, 0.55, false);
        let (hx, hy) = (s.cx.round() as isize, s.cy.round() as isize);
        let mut diff = 0;
        for dy in -5..=5 {
            for dx in -6..=6 {
                if a.get(hx + dx, hy + dy).unwrap().ch != b.get(hx + dx, hy + dy).unwrap().ch {
                    diff += 1;
                }
            }
        }
        assert!(diff >= 5, "core turn changed only {diff} cells");
        assert_eq!(a.get(hx, hy).unwrap().ch, '@');
    }

    #[test]
    fn cycle_dips_to_black_and_recovers() {
        assert_eq!(fade_level(STAR_END), 1.0);
        assert!(fade_level(STAR_END + FADE_OUT) < 1e-5);
        assert!((fade_level(STAR_END + FADE_OUT * 0.5) - 0.5).abs() < 1e-4);
        assert!(fade_level(CYCLE - 0.001) > 0.999);

        let mut last = 1.0;
        let mut t = STAR_END;
        while t < STAR_END + FADE_OUT {
            let level = fade_level(t);
            assert!(level <= last + 1e-5, "dip grew at {t}");
            last = level;
            t += 0.05;
        }
        let mut last = 0.0;
        let mut t = STAR_END + FADE_OUT;
        while t <= CYCLE {
            let level = fade_level(t);
            assert!(level >= last - 1e-5, "recovery fell at {t}");
            last = level;
            t += 0.05;
        }
    }

    #[test]
    fn fade_dims_every_painted_cell() {
        let mut c = Canvas::new(20, 10);
        c.set(5, 5, '*', WHITE);
        c.set(5, 0, 'H', WHITE);
        c.set(5, 9, 'K', WHITE);
        fade_scene(&mut c, 0.25);
        let art = c.get(5, 5).unwrap().fg;
        assert!(art.luminance() < WHITE.luminance() * 0.3, "art {art:?}");
        assert_eq!(c.get(5, 5).unwrap().ch, '*');
        assert!(c.get(5, 0).unwrap().fg.luminance() < WHITE.luminance() * 0.3);
        assert!(c.get(5, 9).unwrap().fg.luminance() < WHITE.luminance() * 0.3);
    }

    #[test]
    fn departure_leaves_a_short_ghost_trail() {
        let mut s = InterstellarScene::new(7);
        s.resize(120, 36);
        let f = test_frame(21.5);
        let mut canvas = Canvas::new(120, 36);
        s.draw_ghost(&mut canvas, &f, 21.5, s.view(21.5));
        assert!(
            canvas.ink() >= 40,
            "ghost trail only {} cells",
            canvas.ink()
        );
        let mut late = Canvas::new(120, 36);
        s.draw_ghost(&mut late, &f, 23.7, s.view(23.7));
        assert_eq!(late.ink(), 0, "ghost trail must end before the star");
    }

    #[test]
    fn departure_scale_collapses_to_a_point() {
        assert!((depart_scale(DEPART_START) - 1.0).abs() < 1e-5);
        let end = depart_scale(DEPART_END);
        assert!(end < 0.07, "end scale {end}");
        let mut last = 1.0;
        let mut t = DEPART_START;
        while t <= DEPART_END {
            let s = depart_scale(t);
            assert!(s <= last + 1e-5, "scale must not grow at {t}");
            last = s;
            t += 0.1;
        }
    }
}
