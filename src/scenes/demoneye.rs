//! 恶魔之眼 — the whole screen is one enormous, watchful eye.
//!
//! A bloodshot sclera fills an almond lid that reaches almost every edge of
//! the terminal.  The iris darts to random targets with a saccadic ease,
//! occasionally locking straight onto the viewer; the slit pupil breathes and
//! the lids slam shut at unpredictable intervals.  Heavy red shadow, pulsing
//! veins and a blink shockwave supply the oppressive atmosphere.

use std::f32::consts::{PI, TAU};

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise, Rng};
use crate::scene::{Frame, Scene};

const BG_DEEP: Rgb = Rgb::hex(0x030104);
const BG_RED: Rgb = Rgb::hex(0x24060c);
const BG_GLOW: Rgb = Rgb::hex(0x7f1d1d);
const SKIN: Rgb = Rgb::hex(0x2a0d13);
const SKIN_LIGHT: Rgb = Rgb::hex(0x521c22);
const SKIN_DARK: Rgb = Rgb::hex(0x120307);
const CREASE: Rgb = Rgb::hex(0x090104);
const SCLERA: Rgb = Rgb::hex(0xd8c3b3);
const SCLERA_WARM: Rgb = Rgb::hex(0xefd9c4);
const SCLERA_EDGE: Rgb = Rgb::hex(0x9c4a45);
const VEIN: Rgb = Rgb::hex(0x7f1d1d);
const VEIN_HOT: Rgb = Rgb::hex(0xdc2626);
const IRIS_DEEP: Rgb = Rgb::hex(0x3f0608);
const IRIS_MID: Rgb = Rgb::hex(0xb91c1c);
const IRIS_HOT: Rgb = Rgb::hex(0xf97316);
const IRIS_RIM: Rgb = Rgb::hex(0x1a0203);
const PUPIL: Rgb = Rgb::hex(0x050002);
const GLINT: Rgb = Rgb::hex(0xfff3df);
const LASH: Rgb = Rgb::hex(0x140306);

pub struct DemonEyeScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    veins: Vec<Vec<(f32, f32)>>,
    next_blink: f64,
    blink_start: f64,
    blink_len: f64,
    blink_count: u32,
    /// Queued gaze events for the current blink window: "look away" and
    /// "return to the centre" alternate, so the eye never stares off-centre
    /// for long.
    gaze_times: [f64; 4],
    gaze_centers: [bool; 4],
    gaze_count: usize,
    gaze_next: usize,
    gaze: (f32, f32),
    target: (f32, f32),
    saccade_from: (f32, f32),
    saccade_start: f64,
    saccade_len: f64,
    /// Corneal highlight offset from the eyeball centre, in iris radii.
    glint: (f32, f32),
    glint_from: (f32, f32),
    glint_target: (f32, f32),
    glint_start: f64,
    glint_len: f64,
    next_glint: f64,
    stare: f32,
    dilate: f32,
    next_dilate: f64,
    dilate_target: f32,
}

impl DemonEyeScene {
    pub fn new(seed: u64) -> Self {
        DemonEyeScene {
            seed,
            rng: Rng::new(seed ^ 0x4445_4d4f_4e45_5945),
            w: 0,
            h: 0,
            veins: Vec::new(),
            next_blink: 2.6,
            blink_start: -10.0,
            blink_len: 0.32,
            blink_count: 0,
            gaze_times: [f64::INFINITY; 4],
            gaze_centers: [true; 4],
            gaze_count: 0,
            gaze_next: 0,
            gaze: (0.0, 0.0),
            target: (0.0, 0.0),
            saccade_from: (0.0, 0.0),
            saccade_start: -10.0,
            saccade_len: 0.1,
            glint: (-0.18, -0.34),
            glint_from: (-0.18, -0.34),
            glint_target: (-0.18, -0.34),
            glint_start: 0.0,
            glint_len: 0.1,
            next_glint: 0.8,
            stare: 0.0,
            dilate: 1.0,
            next_dilate: 2.0,
            dilate_target: 1.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.next_blink = self.rng.range(8.0, 15.0) as f64;
        self.blink_start = -10.0;
        self.veins.clear();
        self.make_veins();
        // Opening dart plan for the very first blink window.
        self.plan_gaze(0.0, self.next_blink);
    }

    /// Plan the gaze for one inter-blink window: one or two glances, each a
    /// hard dart followed by a fast return to the centre inside a second.
    /// Two glances are only used when they can sit at least five seconds
    /// apart, otherwise the eye just watches the viewer for the rest of the
    /// window.
    fn plan_gaze(&mut self, now: f64, gap: f64) {
        self.gaze_count = 0;
        self.gaze_next = 0;
        // The short gap of a double blink is the lid's, not the iris's.
        if gap <= 0.9 {
            return;
        }
        let end = now + gap - 0.10;
        let two = self.rng.chance(0.50);
        let (lo, hi) = if two { (0.06, 0.22) } else { (0.12, 0.42) };
        let out = now + gap * self.rng.range(lo, hi) as f64;
        let back = out + self.rng.range(0.45, 0.85) as f64;
        if back > end {
            return;
        }
        self.push_gaze(out, false);
        self.push_gaze(back, true);
        if two {
            let out2 = out + self.rng.range(5.0, 5.6) as f64;
            let back2 = out2 + self.rng.range(0.45, 0.85) as f64;
            if out2 > back + 0.3 && back2 <= end {
                self.push_gaze(out2, false);
                self.push_gaze(back2, true);
            }
        }
    }

    fn push_gaze(&mut self, t: f64, centre: bool) {
        if self.gaze_count < self.gaze_times.len() {
            self.gaze_times[self.gaze_count] = t;
            self.gaze_centers[self.gaze_count] = centre;
            self.gaze_count += 1;
        }
    }

    fn make_veins(&mut self) {
        for i in 0..20 {
            let top = self.rng.chance(0.68);
            let u0 = self.rng.range(-0.94, 0.94);
            let lid = (1.0 - u0 * u0).max(0.0).powf(0.55);
            let v0 = if top { -lid * 0.96 } else { lid * 0.96 };
            let end_u = u0 * self.rng.range(0.12, 0.35) + self.rng.range(-0.06, 0.06);
            let end_v = if top {
                -self.rng.range(0.02, 0.22)
            } else {
                self.rng.range(0.02, 0.22)
            };
            let phase = self.rng.range(0.0, TAU);
            let mut pts = Vec::new();
            let steps = 16;
            for k in 0..=steps {
                let f = k as f32 / steps as f32;
                let wobble = (f * 7.0 + phase + i as f32).sin() * 0.018 * (1.0 - f);
                let bend = (f * PI).sin() * self.rng.range(-0.02, 0.02);
                let u = u0 + (end_u - u0) * f + wobble + bend;
                let v = v0 + (end_v - v0) * f + (f * 9.0 + phase).cos() * 0.012 * (1.0 - f);
                pts.push((u, v));
            }
            self.veins.push(pts);
            // Branch off the main capillary.
            if self.rng.chance(0.70) {
                let at = 5 + self.rng.below(5);
                let branch_start = self.veins.last().and_then(|v| v.get(at)).copied();
                if let Some((bu, bv)) = branch_start {
                    let dir = self.rng.sign();
                    let mut bpts = vec![(bu, bv)];
                    for k in 1..=7 {
                        let f = k as f32 / 7.0;
                        bpts.push((
                            bu + dir * f * self.rng.range(0.04, 0.10),
                            bv + f * self.rng.range(0.02, 0.07) * if top { 1.0 } else { -1.0 },
                        ));
                    }
                    self.veins.push(bpts);
                }
            }
        }
    }

    fn blink(&self, t: f64) -> f32 {
        if t < self.blink_start {
            return 0.0;
        }
        let p = (t - self.blink_start) / self.blink_len.max(0.01);
        if !(0.0..=1.0).contains(&p) {
            return 0.0;
        }
        let p = p as f32;
        if p < 0.30 {
            smoothstep(p / 0.30)
        } else if p < 0.52 {
            1.0
        } else {
            1.0 - smoothstep((p - 0.52) / 0.48)
        }
    }

    fn geometry(&self) -> (f32, f32, f32, f32) {
        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;
        let ex = self.w as f32 * 0.485;
        let ey = self.h as f32 * 0.445;
        (cx, cy, ex, ey)
    }

    fn opening(u: f32, open: f32) -> f32 {
        let lid = (1.0 - u * u).max(0.0).powf(0.55);
        lid * open
    }

    /// Where the corneal highlight darts next: anywhere on the iris, so the
    /// white light sweeps around the way it did on the original eye.
    fn pick_glint(&mut self) -> (f32, f32) {
        let a = self.rng.range(0.0, TAU);
        let r = self.rng.range(0.14, 0.55);
        (r * a.cos(), r * a.sin())
    }

    /// Both highlights in eyeball-local units (fractions of the iris radius):
    /// the spark and its small companion, at the original offsets.
    fn glint_offsets(&self) -> ((f32, f32), (f32, f32)) {
        (
            on_cornea(self.glint),
            on_cornea((self.glint.0 + 0.31, self.glint.1 + 0.56)),
        )
    }

    fn draw_veins(&self, canvas: &mut Canvas, t: f32, open: f32, blink: f32, unicode: bool) {
        if blink > 0.55 {
            return;
        }
        let (cx, cy, ex, ey) = self.geometry();
        for (i, vein) in self.veins.iter().enumerate() {
            let bright = 0.20 + 0.14 * (t * 2.1 + i as f32 * 1.7).sin().abs();
            let hot = if i % 5 == 0 { VEIN_HOT } else { VEIN };
            for (k, &(u, v)) in vein.iter().enumerate() {
                let o = Self::opening(u, open);
                if v.abs() >= o + 0.04 {
                    continue;
                }
                let x = (cx + u * ex).round() as isize;
                let y = (cy + v * ey).round() as isize;
                // Capillaries are dotted, not drawn as hard diagonal strokes.
                canvas.set(
                    x,
                    y,
                    if k % 4 == 0 {
                        if unicode {
                            '·'
                        } else {
                            '.'
                        }
                    } else {
                        ':'
                    },
                    hot.scale(bright),
                );
                if k % 6 == 0 && self.w > 90 {
                    canvas.set(
                        x + 1,
                        y,
                        if unicode { '·' } else { '.' },
                        hot.scale(bright * 0.45),
                    );
                }
            }
        }
    }

    fn draw_lashes(&self, canvas: &mut Canvas, t: f32, open: f32, blink: f32, unicode: bool) {
        if open < 0.18 || blink > 0.7 {
            return;
        }
        let (cx, cy, ex, ey) = self.geometry();
        let step = (self.w as f32 / 22.0).max(4.0) as isize;
        let mut x = (cx - ex) as isize;
        while x <= (cx + ex) as isize {
            let u = (x as f32 - cx) / ex;
            let lid = Self::opening(u, open);
            if lid > 0.10 {
                let y = (cy - lid * ey).round() as isize;
                // A short fan of lashes: one to three cells, pointing outward.
                let len = (1.0 + (1.0 - u.abs()) * 1.8).round() as isize;
                let dir = if u < 0.0 { -1 } else { 1 };
                let col = LASH.lerp(VEIN, 0.10 + 0.08 * (t * 0.7).sin().abs());
                for k in 1..=len {
                    canvas.set(
                        x + dir * (k / 2),
                        y - k,
                        if unicode {
                            if dir < 0 {
                                '╲'
                            } else {
                                '╱'
                            }
                        } else if dir < 0 {
                            '\\'
                        } else {
                            '/'
                        },
                        col.scale(1.0 - k as f32 * 0.18),
                    );
                }
            }
            x += step;
        }
    }

    fn draw_shockwave(&self, canvas: &mut Canvas, t: f32, unicode: bool) {
        let p = (t as f64 - self.blink_start) / self.blink_len.max(0.01);
        if !(0.0..=2.6).contains(&p) {
            return;
        }
        let p = p as f32;
        let r = (p * 1.2).clamp(0.0, 1.4);
        let fade = (1.0 - p / 2.6).clamp(0.0, 1.0) * 0.38;
        let (cx, cy, ex, ey) = self.geometry();
        let steps = 120;
        for k in 0..steps {
            let a = k as f32 / steps as f32 * TAU;
            let x = cx + a.cos() * ex * r;
            let y = cy + a.sin() * ey * r * 0.78;
            let col = BG_GLOW.lerp(VEIN_HOT, 0.35 + 0.35 * (a * 5.0).sin().abs());
            canvas.add_light(
                x.round() as isize,
                y.round() as isize,
                col,
                fade,
                if unicode { '·' } else { '.' },
            );
        }
    }

    fn draw_hud(&self, canvas: &mut Canvas, t: f32, blink: f32) {
        let title = "DEMON EYE // THE WATCHER";
        canvas.put_str(2, 0, title, VEIN_HOT.scale(0.62));
        let state = if blink > 0.75 {
            "LID CLOSED".to_string()
        } else if self.stare > 0.7 {
            "IT SEES YOU".to_string()
        } else {
            "SCANNING".to_string()
        };
        let status = format!("{state}  |  BLINK {:02}", self.blink_count);
        let x = (self.w as isize - status.chars().count() as isize - 2).max(2);
        canvas.put_str(x, 0, &status, VEIN.scale(0.78));
        let h = self.h as isize;
        let pulse = (t * 1.8).sin() * 0.5 + 0.5;
        let footer = "DO NOT LOOK AWAY";
        let fx = (self.w as isize - footer.len() as isize) / 2;
        canvas.put_str(fx.max(1), h - 1, footer, VEIN.scale(0.35 + pulse * 0.35));
    }
}

impl Scene for DemonEyeScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        // Blinks: clustered doubles keep the rhythm unpredictable.
        if f.t >= self.next_blink {
            self.blink_start = f.t;
            self.blink_len = self.rng.range(0.24, 0.48) as f64;
            // A long, unhurried rhythm: one blink every eight to fifteen
            // seconds, now and then doubled up by a quick second one.
            self.next_blink = f.t + self.rng.range(8.0, 15.0) as f64;
            self.blink_count = self.blink_count.wrapping_add(1);
            if self.rng.chance(0.30) {
                self.next_blink = f.t + self.blink_len + self.rng.range(0.20, 0.40) as f64;
            }
            // One or two glances for the window that just opened.
            if !f.tune.eye_still {
                let gap = self.next_blink - f.t;
                self.plan_gaze(f.t, gap);
            }
        }

        let dt = f.dt.clamp(0.0, 0.08);
        let k = 1.0 - (-dt * 9.5).exp();
        if f.tune.eye_still {
            // `--eye-still`: the eyeball is nailed to the centre, only the
            // lids, the pupil and the cornea light keep moving.
            self.gaze = (0.0, 0.0);
            self.target = (0.0, 0.0);
            self.gaze_count = 0;
            self.gaze_next = 0;
        } else {
            // Gaze events: glance away, then come back to the centre. Each
            // change is a hard saccade, the whole eyeball snapping over in
            // about a tenth of a second.
            if self.gaze_next < self.gaze_count && f.t >= self.gaze_times[self.gaze_next] {
                self.target = if self.gaze_centers[self.gaze_next] {
                    (0.0, 0.0)
                } else {
                    let (mut gx, mut gy) =
                        (self.rng.range(-0.78, 0.78), self.rng.range(-0.72, 0.72));
                    // A glance is never a shrug: always look somewhere clearly.
                    let r = (gx * gx + gy * gy).sqrt();
                    if r < 0.32 {
                        let scale = 0.32 / r.max(1e-3);
                        gx *= scale;
                        gy *= scale;
                    }
                    (gx, gy)
                };
                self.saccade_from = self.gaze;
                self.saccade_start = f.t;
                self.saccade_len = self.rng.range(0.07, 0.14) as f64;
                self.gaze_next += 1;
            }
            let snap = if self.saccade_len > 1e-4 {
                ((f.t - self.saccade_start) / self.saccade_len).clamp(0.0, 1.0) as f32
            } else {
                1.0
            };
            let snap = smoothstep(snap);
            self.gaze.0 = self.saccade_from.0 + (self.target.0 - self.saccade_from.0) * snap;
            self.gaze.1 = self.saccade_from.1 + (self.target.1 - self.saccade_from.1) * snap;
        }

        // The cornea light lives on its own clock: every second or so it
        // darts to a new spot on the iris, exactly like the original eye.
        if f.t >= self.next_glint {
            self.glint_from = self.glint;
            self.glint_target = self.pick_glint();
            self.glint_start = f.t;
            self.glint_len = self.rng.range(0.09, 0.18) as f64;
            self.next_glint = f.t + self.rng.range(0.55, 1.60) as f64;
        }
        let gp = if self.glint_len > 1e-4 {
            ((f.t - self.glint_start) / self.glint_len).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        let gp = smoothstep(gp);
        self.glint.0 = self.glint_from.0 + (self.glint_target.0 - self.glint_from.0) * gp;
        self.glint.1 = self.glint_from.1 + (self.glint_target.1 - self.glint_from.1) * gp;

        let near_center = 1.0 - (self.gaze.0 * self.gaze.0 + self.gaze.1 * self.gaze.1).sqrt();
        self.stare += (near_center.clamp(0.0, 1.0) - self.stare) * (k * 0.6);

        // Pupil dilation changes on its own schedule.
        if f.t >= self.next_dilate {
            self.dilate_target = self.rng.range(0.86, 1.28);
            self.next_dilate = f.t + self.rng.range(0.9, 2.7) as f64;
        }
        self.dilate += (self.dilate_target - self.dilate) * (k * 0.35);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        let blink = self.blink(f.t);
        let open = (1.0 - blink).max(0.0);
        let (cx, cy, ex, ey) = self.geometry();
        // The eyeball itself travels with the gaze; `ox` / `oy` are that
        // offset expressed in the same aspect-corrected units as dx / dy.
        // A still eye (`--eye-still`) keeps the iris dead centre, bob included.
        let (gaze_x, gaze_y) = if f.tune.eye_still {
            (0.0, 0.0)
        } else {
            (
                self.gaze.0 * ex * 0.62,
                self.gaze.1 * ey * 0.56 + (t * 1.7).sin() * 0.35,
            )
        };
        let ox = gaze_x * 0.5;
        let oy = gaze_y;
        let iris_r = (ey * 0.64).max(3.0);
        let pulse = 1.0 + 0.045 * (t * 1.6).sin() + self.stare * 0.06;
        let pupil_w = (iris_r * 0.095 * self.dilate * pulse).max(0.55);
        let pupil_h = (iris_r * 0.43 * self.dilate * pulse).max(1.4);
        let (glint, glint2) = self.glint_offsets();
        let glint_off = (glint.0 * iris_r, glint.1 * iris_r);
        let glint2_off = (glint2.0 * iris_r, glint2.1 * iris_r);

        for y in 0..self.h as isize {
            for x in 0..self.w as isize {
                let dx = (x as f32 + 0.5 - cx) * 0.5;
                let dy = y as f32 + 0.5 - cy;
                let u = dx / (ex * 0.5);
                let v = dy / ey;
                let lid_outer = (1.0 - u * u).max(0.0).powf(0.50);
                let opening = Self::opening(u, open);

                // Outer shadow / flesh / smoke.
                let su = (x as f32 + 0.5 - cx) / (self.w as f32 * 0.5);
                let sv = (y as f32 + 0.5 - cy) / (self.h as f32 * 0.5);
                let smoke = value_noise(su * 2.8 + t * 0.025, sv * 2.6 - t * 0.015, self.seed);
                let radial = (u * u * 0.65 + v * v * 0.45).sqrt();
                let glow = (1.0 - (radial - 0.75).abs() * 1.7).clamp(0.0, 1.0);
                let mut col = BG_DEEP.lerp(BG_RED, smoke.powf(1.8) * 0.55 + glow * 0.30);
                let mut ch = if unicode { '░' } else { '.' };
                let near_skin = u.abs() < 1.10 && v.abs() < (lid_outer + 0.14).max(0.07);

                if near_skin && v.abs() >= opening {
                    // Eyelid flesh, with a heavy crease and a shadowed brow.
                    let upper = v < 0.0;
                    let from_edge = (v.abs() - opening).max(0.0);
                    let side = 1.0 - u.abs() * 0.35;
                    let shade = (0.32 - from_edge * 0.55).clamp(0.0, 1.0) * side;
                    let mut skin = SKIN.lerp(SKIN_LIGHT, shade * 0.75);
                    if from_edge > 0.10 && from_edge < 0.18 {
                        skin = skin.lerp(CREASE, 0.52);
                    }
                    if upper && from_edge > 0.32 {
                        skin = skin.lerp(SKIN_DARK, 0.38);
                    }
                    if !upper && from_edge > 0.22 {
                        skin = skin.lerp(SKIN_DARK, 0.25);
                    }
                    if blink > 0.55 && v.abs() < 0.022 {
                        skin = CREASE;
                    }
                    let grain = value_noise(u * 9.0, v * 9.0 + t * 0.01, self.seed ^ 0x77);
                    col = skin.scale(0.88 + grain * 0.24);
                    ch = if unicode {
                        if from_edge < 0.05 || skin.luminance() < 0.10 {
                            '▒'
                        } else {
                            '▓'
                        }
                    } else {
                        '#'
                    };
                } else if v.abs() < opening {
                    // Eyeball.
                    let edge = (v.abs() / opening.max(0.001)).clamp(0.0, 1.0);
                    let side_red = (u.abs() / 1.0).powf(2.0) * 0.55;
                    let mut sclera = SCLERA.lerp(SCLERA_WARM, (1.0 - edge) * 0.35);
                    sclera = sclera.lerp(SCLERA_EDGE, edge.powf(2.0) * 0.55 + side_red);
                    let tex = value_noise(u * 11.0, v * 11.0 + t * 0.02, self.seed ^ 0x91);
                    sclera = sclera.scale(0.90 + tex * 0.16);
                    if v < -opening * 0.72 {
                        sclera = sclera.lerp(SCLERA_EDGE, 0.20);
                    }
                    // Coordinates relative to the (moving) eyeball centre.
                    let bdx = dx - ox;
                    let bdy = dy - oy;
                    let d = (bdx * bdx + bdy * bdy).sqrt();
                    let rn = d / iris_r;
                    if rn < 1.0 {
                        let ang = bdy.atan2(bdx);
                        let fiber = 0.5
                            + 0.5
                                * (ang * 43.0
                                    + value_noise(rn * 7.0, ang * 3.0, self.seed ^ 0x35) * 4.0
                                    + t * 0.35)
                                    .sin();
                        let mut iris = IRIS_DEEP.lerp(IRIS_HOT, (1.0 - rn).powf(1.65));
                        iris = iris.lerp(IRIS_MID, 0.25 + fiber * 0.35);
                        iris = iris.scale(0.72 + fiber * 0.42 * (0.45 + 0.55 * (1.0 - rn)));
                        if rn > 0.88 {
                            iris = iris.lerp(IRIS_RIM, (rn - 0.88) / 0.12 * 0.85);
                        }
                        if rn < 0.34 {
                            iris = iris.lerp(IRIS_HOT, 0.30);
                        }
                        // Slit pupil.
                        let slit = 1.0 - 0.18 * (bdy / pupil_h).abs();
                        let pu = bdx / (pupil_w * slit.max(0.25));
                        let pv = bdy / pupil_h;
                        if pu * pu + pv * pv < 1.0 {
                            iris = PUPIL;
                        } else if pu * pu + pv * pv < 1.45 {
                            iris = iris.lerp(IRIS_HOT, 0.25);
                        }
                        // Corneal glints, drawn exactly as the original eye
                        // did — and never over the pupil or its hot rim, so
                        // the black slit always stays black.
                        if pu * pu + pv * pv > 1.55 {
                            let gd =
                                ((bdx - glint_off.0).powi(2) + (bdy - glint_off.1).powi(2)).sqrt();
                            if gd < iris_r * 0.20 {
                                iris = GLINT;
                            } else if gd < iris_r * 0.30 {
                                iris = iris.lerp(GLINT, 0.30);
                            }
                            // A second, smaller catchlight.
                            let gd2 = ((bdx - glint2_off.0).powi(2) + (bdy - glint2_off.1).powi(2))
                                .sqrt();
                            if gd2 < iris_r * 0.07 {
                                iris = GLINT.scale(0.85);
                            }
                        }
                        col = iris;
                        ch = if unicode {
                            if iris == PUPIL {
                                '█'
                            } else {
                                '▓'
                            }
                        } else if iris == PUPIL {
                            '@'
                        } else {
                            '*'
                        };
                    } else {
                        col = sclera;
                        ch = if unicode { '░' } else { '.' };
                    }
                }

                // Soft red rim around the whole eye.
                if radial > 0.85 && radial < 1.45 {
                    let rim = (1.0 - (radial - 1.05).abs() * 1.8).clamp(0.0, 1.0);
                    col = col.add(BG_GLOW, rim * 0.12);
                    if ch == ' ' {
                        ch = if unicode { '·' } else { '.' };
                    }
                }
                canvas.set(x, y, ch, col);
            }
        }

        self.draw_veins(canvas, t, open, blink, unicode);
        self.draw_lashes(canvas, t, open, blink, unicode);

        // Vein-like arcs of shadow around the socket.
        for k in 0..5 {
            let r = 1.12 + k as f32 * 0.08;
            let col = BG_GLOW.scale(0.10 + 0.04 * k as f32);
            let steps = 90;
            for i in 0..steps {
                let a = i as f32 / steps as f32 * TAU;
                let x = cx + a.cos() * ex * r;
                let y = cy + a.sin() * ey * r * 0.82;
                if (i / 3) % 3 == 0 {
                    canvas.add_light(
                        x.round() as isize,
                        y.round() as isize,
                        col,
                        0.15,
                        if unicode { '·' } else { '.' },
                    );
                }
            }
        }

        // Floating embers / motes in the dark.
        let n = ((self.w * self.h) / 420).clamp(6, 24);
        for i in 0..n {
            let hx = hash01(i as i64, 3, self.seed);
            let hy = hash01(5, i as i64, self.seed);
            let x =
                (hx * self.w as f32 + (t * 0.18 + hx * 8.0).sin() * 2.2).rem_euclid(self.w as f32);
            let y =
                (hy * self.h as f32 + (t * 0.12 + hy * 5.0).cos() * 1.6).rem_euclid(self.h as f32);
            let a = 0.10 + 0.20 * (t * 1.2 + hx * 17.0).sin().abs();
            canvas.add_light(
                x as isize,
                y as isize,
                VEIN_HOT,
                a,
                if unicode { '·' } else { '.' },
            );
        }

        self.draw_shockwave(canvas, t, unicode);
        self.draw_hud(canvas, t, blink);
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Keep a cornea highlight near the middle of the iris.
fn on_cornea(p: (f32, f32)) -> (f32, f32) {
    let r = p.0.hypot(p.1);
    if r > 0.62 {
        (p.0 / r * 0.62, p.1 / r * 0.62)
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blinks_randomly_and_opens() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        s.next_blink = 1.0;
        let frame = Frame {
            t: 1.0,
            dt: 0.1,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune: Default::default(),
        };
        s.update(&frame);
        assert!(s.blink(1.12) > 0.5);
        assert_eq!(s.blink(9.0), 0.0);
    }

    #[test]
    fn glances_out_and_back_between_blinks() {
        let mut s = DemonEyeScene::new(3);
        s.resize(80, 24);
        let mut pairs = 0;
        for gap in [8.0, 11.0, 15.0] {
            for _ in 0..64 {
                s.plan_gaze(10.0, gap);
                assert!(s.gaze_count == 2 || s.gaze_count == 4, "{}", s.gaze_count);
                assert!(!s.gaze_centers[0], "a window opens with a glance");
                assert!(
                    s.gaze_centers[s.gaze_count - 1],
                    "and closes back on the centre"
                );
                let mut prev = 10.0;
                for i in 0..s.gaze_count {
                    let at = s.gaze_times[i];
                    assert!(at > prev, "gaze events stay ordered");
                    prev = at;
                }
                assert!(prev <= 10.0 + gap - 0.099);
                // Every glance turns back within a second.
                let mut i = 0;
                while i + 1 < s.gaze_count {
                    assert!(!s.gaze_centers[i] && s.gaze_centers[i + 1]);
                    let hold = s.gaze_times[i + 1] - s.gaze_times[i];
                    assert!((0.45..=0.85).contains(&hold), "hold was {hold}");
                    i += 2;
                }
                if s.gaze_count == 4 {
                    // Two glances means at least five seconds between them.
                    assert!(s.gaze_times[2] - s.gaze_times[0] >= 5.0);
                    pairs += 1;
                }
            }
        }
        assert!(pairs > 0, "two-glance windows should happen");
        // The tight gap of a double blink gets no glance of its own.
        s.plan_gaze(10.0, 0.5);
        assert_eq!(s.gaze_count, 0);
    }

    #[test]
    fn blinks_are_eight_to_fifteen_seconds_apart() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        let frame = |t: f64| Frame {
            t,
            dt: 0.1,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune: Default::default(),
        };
        let mut last = s.blink_start;
        let mut gaps: Vec<f64> = Vec::new();
        let mut t = 0.0f64;
        while t < 420.0 {
            s.update(&frame(t));
            if s.blink_start != last {
                if last > -1.0 {
                    gaps.push(s.blink_start - last);
                }
                last = s.blink_start;
            }
            t += 0.1;
        }
        assert!(gaps.len() > 15, "only {} blinks", gaps.len());
        let mut doubles = 0;
        for gap in &gaps {
            if *gap > 2.0 {
                assert!((8.0..=15.0).contains(gap), "slow blink gap {gap}");
            } else {
                doubles += 1;
                assert!((0.4..=1.0).contains(gap), "double blink gap {gap}");
            }
        }
        assert!(doubles > 0, "some blinks should be doubled");
        assert!(gaps.len() - doubles > 10);
    }

    #[test]
    fn a_glance_snaps_out_and_returns_to_the_centre() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        s.gaze = (0.0, 0.0);
        s.gaze_count = 2;
        s.gaze_next = 0;
        s.gaze_times[0] = 0.0;
        s.gaze_centers[0] = false;
        s.gaze_times[1] = 0.6;
        s.gaze_centers[1] = true;
        let frame = |t: f64| Frame {
            t,
            dt: 0.05,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune: Default::default(),
        };
        s.update(&frame(0.0));
        // A glance is a snap, not a drift, and it goes somewhere.
        assert!(s.saccade_len <= 0.15);
        let away = s.target;
        assert!(away.0.hypot(away.1) >= 0.32);
        let mut t = 0.0f64;
        while t < 0.4 {
            t += 0.05;
            s.update(&frame(t));
            assert!(s.gaze.0.abs() <= 0.95 && s.gaze.1.abs() <= 0.95);
        }
        assert!((s.gaze.0 - away.0).abs() < 1e-4);
        assert!((s.gaze.1 - away.1).abs() < 1e-4);
        while t < 0.75 {
            t += 0.05;
            s.update(&frame(t));
        }
        assert_eq!(s.gaze, (0.0, 0.0), "the eyeball is back home");
    }

    #[test]
    fn eye_still_keeps_the_eyeball_centred() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        let tune = crate::scene::Tune {
            eye_still: true,
            ..Default::default()
        };
        let frame = |t: f64| Frame {
            t,
            dt: 0.1,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune,
        };
        for k in 0..240 {
            s.update(&frame(k as f64 * 0.1));
            assert_eq!(s.gaze, (0.0, 0.0));
            assert_eq!(s.gaze_count, 0);
        }
    }

    #[test]
    fn cornea_glints_keep_darting_around_the_iris() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        let tune = crate::scene::Tune {
            eye_still: true,
            ..Default::default()
        };
        let frame = |t: f64| Frame {
            t,
            dt: 0.1,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune,
        };
        let base = s.glint;
        let mut travel = 0.0f32;
        let mut moved = 0;
        for k in 1..=200 {
            s.update(&frame(k as f64 * 0.1));
            let (main, small) = s.glint_offsets();
            for p in [main, small] {
                assert!(p.0.hypot(p.1) <= 0.6301, "glint left the iris: {p:?}");
            }
            let d = (main.0 - base.0).hypot(main.1 - base.1);
            travel = travel.max(d);
            if d > 0.02 {
                moved += 1;
            }
        }
        // The white light keeps sweeping the iris even with a still eyeball.
        assert!(travel > 0.35, "glint only travelled {travel}");
        assert!(moved > 100, "glint moved in only {moved} frames");
    }

    #[test]
    fn cornea_glints_never_cover_the_pupil() {
        let mut s = DemonEyeScene::new(7);
        s.resize(80, 24);
        let tune = crate::scene::Tune {
            eye_still: true,
            ..Default::default()
        };
        let frame = Frame {
            t: 1.0,
            dt: 0.05,
            w: 80,
            h: 24,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Unicode,
            seed: 7,
            speed: 1.0,
            tune,
        };
        // Park the white light dead centre, right on top of the slit pupil.
        s.glint = (0.0, 0.0);
        s.glint_from = (0.0, 0.0);
        s.glint_target = (0.0, 0.0);
        s.next_glint = f64::INFINITY;
        let mut canvas = Canvas::new(80, 24);
        s.draw(&mut canvas, &frame);
        let centre = canvas.get(40, 12).expect("in bounds");
        assert_eq!(centre.ch, '█');
        assert_eq!(centre.fg, PUPIL, "the pupil must stay black");
        // The light itself is still drawn on the iris just beside it.
        s.glint = (-0.32, -0.34);
        s.glint_from = (-0.32, -0.34);
        s.glint_target = (-0.32, -0.34);
        let mut canvas = Canvas::new(80, 24);
        s.draw(&mut canvas, &frame);
        assert!(
            (0..24).any(|y| (0..80).any(|x| canvas.get(x, y).is_some_and(|c| c.fg == GLINT))),
            "the corneal highlight disappeared"
        );
    }
}
