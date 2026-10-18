//! 舞动芭蕾 — a male and a female dancer joined at the hand, sweeping across
//! the whole stage.
//!
//! The couple is drawn as a procedural ballet skeleton: the woman in a tutu
//! and pointe shoes, the man in a fitted tunic.  Their inner arms always meet
//! at one glowing clasp, while the pair follows a Lissajous path that reaches
//! every corner of the screen.  Pose changes (arabesque, attitude, promenade)
//! are driven by slow sine curves; ribbons, spotlights and dust fill the stage.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise};
use crate::scene::{Frame, Scene};

const STAGE_TOP: Rgb = Rgb::hex(0x120a29);
const STAGE_MID: Rgb = Rgb::hex(0x2a1038);
const STAGE_LOW: Rgb = Rgb::hex(0x120b1d);
const CURTAIN: Rgb = Rgb::hex(0x4c0f24);
const CURTAIN_HL: Rgb = Rgb::hex(0x9d1c46);
const SPOT_WARM: Rgb = Rgb::hex(0xffe7b0);
const SPOT_COOL: Rgb = Rgb::hex(0x9fe8ff);
const FLOOR: Rgb = Rgb::hex(0x1d1430);
const FLOOR_HL: Rgb = Rgb::hex(0x6d4d8f);

const FEMALE: Rgb = Rgb::hex(0xf472b6);
const FEMALE_HL: Rgb = Rgb::hex(0xfbcfe8);
const FEMALE_DARK: Rgb = Rgb::hex(0x9d174d);
const MALE: Rgb = Rgb::hex(0x22d3ee);
const MALE_HL: Rgb = Rgb::hex(0xa5f3fc);
const MALE_DARK: Rgb = Rgb::hex(0x0e7490);
const SKIN: Rgb = Rgb::hex(0xf3c9a1);
const SKIN_HL: Rgb = Rgb::hex(0xffe4c8);
const SKIN_DARK: Rgb = Rgb::hex(0xbf8d67);
const HAIR: Rgb = Rgb::hex(0x2a1a16);
const HAIR_HL: Rgb = Rgb::hex(0x71452f);
const GOLD: Rgb = Rgb::hex(0xffd166);
const WHITE: Rgb = Rgb::hex(0xfffbef);

#[derive(Clone, Copy, Debug)]
struct P {
    x: f32,
    y: f32,
}

impl P {
    const fn new(x: f32, y: f32) -> Self {
        P { x, y }
    }
    fn add(self, o: P) -> P {
        P::new(self.x + o.x, self.y + o.y)
    }
    fn mul(self, k: f32) -> P {
        P::new(self.x * k, self.y * k)
    }
    fn rot(self, a: f32) -> P {
        let (s, c) = a.sin_cos();
        P::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
    fn lerp(self, o: P, t: f32) -> P {
        P::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
}

/// Screen transform shared by every limb in a frame.
#[derive(Clone, Copy)]
struct Xf {
    cx: f32,
    cy: f32,
    s: f32,
    ang: f32,
    unicode: bool,
}

impl Xf {
    fn pt(&self, p: P) -> (isize, isize) {
        let r = p.rot(self.ang);
        (
            (self.cx + r.x * self.s * 2.0).round() as isize,
            (self.cy + r.y * self.s).round() as isize,
        )
    }

    fn line(&self, canvas: &mut Canvas, a: P, b: P, ch: char, col: Rgb, thick: bool) {
        let (x0, y0) = self.pt(a);
        let (x1, y1) = self.pt(b);
        canvas.line(x0, y0, x1, y1, ch, col);
        if thick {
            let dx = (x1 - x0) as f32;
            let dy = (y1 - y0) as f32;
            let len = (dx * dx + dy * dy).sqrt().max(1.0);
            let ox = (-dy / len).round() as isize;
            let oy = (dx / len).round() as isize;
            if ox != 0 || oy != 0 {
                canvas.line(x0 + ox, y0 + oy, x1 + ox, y1 + oy, ch, col.scale(0.66));
            }
        }
    }

    fn quad(&self, canvas: &mut Canvas, a: P, b: P, c: P, d: P, col: Rgb) {
        // Fill a (possibly rotated) quad with many short scan lines.
        let steps = ((self.s * 18.0) as usize).clamp(10, 26);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let left = a.lerp(d, t);
            let right = b.lerp(c, t);
            let (x0, y0) = self.pt(left);
            let (x1, y1) = self.pt(right);
            canvas.line(x0, y0, x1, y1, self.block(), col);
        }
    }

    fn block(&self) -> char {
        if self.unicode {
            '█'
        } else {
            '#'
        }
    }

    fn shade(&self) -> char {
        if self.unicode {
            '▓'
        } else {
            '%'
        }
    }
}

pub struct BalletScene {
    seed: u64,
    w: usize,
    h: usize,
}

struct Pose {
    clasp: P,
    female_hip: P,
    female_neck: P,
    female_head: P,
    female_shoulder_l: P,
    female_shoulder_r: P,
    female_free_hand: P,
    female_free_elbow: P,
    female_inner_elbow: P,
    female_support_knee: P,
    female_support_foot: P,
    female_ext_knee: P,
    female_ext_foot: P,
    male_hip: P,
    male_neck: P,
    male_head: P,
    male_shoulder_l: P,
    male_shoulder_r: P,
    male_free_hand: P,
    male_free_elbow: P,
    male_inner_elbow: P,
    male_support_knee: P,
    male_support_foot: P,
    male_ext_knee: P,
    male_ext_foot: P,
    beat: f32,
}

impl BalletScene {
    pub fn new(seed: u64) -> Self {
        BalletScene { seed, w: 0, h: 0 }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn scale(&self) -> f32 {
        let by_h = self.h as f32 / 34.0;
        let by_w = self.w as f32 / 100.0;
        (by_h * by_w.clamp(0.78, 1.10)).clamp(0.52, 1.20)
    }

    /// Pair centre at time `t`; also used for the fading motion trail.
    fn centre_at(&self, t: f32) -> (f32, f32) {
        let s = self.scale();
        let w = self.w as f32;
        let h = self.h as f32;
        let mut x = w * 0.5 + w * 0.34 * (t * 0.44 + 0.5).sin() + w * 0.06 * (t * 0.94 + 1.2).sin();
        let mut y = h * 0.52 + h * 0.255 * (t * 0.52 + 1.1).sin() + h * 0.05 * (t * 1.03).sin();
        let mx = 10.5 * s + 2.0;
        let top = 6.6 * s + 1.0;
        let bottom = 8.7 * s + 1.0;
        x = x.clamp(mx.min(w * 0.5), (w - mx).max(w * 0.5));
        y = y.clamp(top.min(h * 0.5), (h - bottom).max(h * 0.5));
        (x, y)
    }

    fn pose(&self, t: f32) -> Pose {
        let beat = t * 0.72;
        let sway = (beat * 0.53).sin();
        let arabesque = 0.5 + 0.5 * (beat * 0.77 + 0.6).sin();
        let attitude = 0.5 + 0.5 * (beat * 0.63 + 2.4).sin();
        let female_lean = 0.18 * (beat * 0.63 + 0.2).sin();
        let male_lean = -0.10 * (beat * 0.57).sin();

        let clasp = P::new(0.15 * sway, 1.32 + 0.08 * sway);

        // Female dancer: slim torso, tutu, one leg en pointe and an arabesque.
        let female_hip = P::new(-4.05 + 0.22 * sway, 2.05);
        let female_chest = P::new(-4.05 + female_lean * 1.4, -0.95);
        let female_neck = P::new(-4.05 + female_lean * 2.1, -2.75);
        let female_head = P::new(-4.05 + female_lean * 2.6, -4.04);
        let female_shoulder_l = female_chest.add(P::new(-1.38, -0.52));
        let female_shoulder_r = female_chest.add(P::new(1.38, -0.52));

        let free_angle = -1.66 + 0.46 * (beat * 0.71).sin();
        let female_free_hand =
            female_shoulder_l.add(P::new(free_angle.cos(), free_angle.sin()).mul(3.25));
        let female_free_elbow = female_shoulder_l
            .lerp(female_free_hand, 0.52)
            .add(P::new(-0.55, 0.20));
        let female_inner_elbow = female_shoulder_r.lerp(clasp, 0.50).add(P::new(0.25, -0.72));

        let female_support_foot = P::new(-3.70 + 0.18 * sway, 8.02);
        let female_support_knee = female_hip
            .lerp(female_support_foot, 0.52)
            .add(P::new(0.18, 0.15));
        let female_ext_foot = P::new(-4.72 - 3.30 * arabesque, 7.80 - 4.45 * arabesque);
        let female_ext_knee = female_hip
            .lerp(female_ext_foot, 0.52)
            .add(P::new(0.20, -0.55 * arabesque));

        // Male dancer: broader shoulders, fitted tunic, a light supporting lunge.
        let male_hip = P::new(4.00 - 0.18 * sway, 2.25);
        let male_chest = P::new(4.00 + male_lean * 1.2, -1.05);
        let male_neck = P::new(4.00 + male_lean * 1.8, -2.92);
        let male_head = P::new(4.00 + male_lean * 2.3, -4.28);
        let male_shoulder_l = male_chest.add(P::new(-1.55, -0.62));
        let male_shoulder_r = male_chest.add(P::new(1.55, -0.62));

        let male_free_angle = -0.92 + 0.38 * (beat * 0.61 + 1.2).sin();
        let male_free_hand =
            male_shoulder_r.add(P::new(male_free_angle.cos(), male_free_angle.sin()).mul(3.35));
        let male_free_elbow = male_shoulder_r
            .lerp(male_free_hand, 0.52)
            .add(P::new(0.42, 0.18));
        let male_inner_elbow = male_shoulder_l.lerp(clasp, 0.50).add(P::new(-0.20, -0.70));

        let male_support_foot = P::new(2.85 - 0.20 * sway, 8.25);
        let male_support_knee = male_hip
            .lerp(male_support_foot, 0.52)
            .add(P::new(-0.15, 0.18));
        let male_ext_foot = P::new(6.10 + 1.45 * attitude, 7.95 - 2.70 * attitude);
        let male_ext_knee = male_hip.lerp(male_ext_foot, 0.52).add(P::new(0.25, -0.35));

        Pose {
            clasp,
            female_hip,
            female_neck,
            female_head,
            female_shoulder_l,
            female_shoulder_r,
            female_free_hand,
            female_free_elbow,
            female_inner_elbow,
            female_support_knee,
            female_support_foot,
            female_ext_knee,
            female_ext_foot,
            male_hip,
            male_neck,
            male_head,
            male_shoulder_l,
            male_shoulder_r,
            male_free_hand,
            male_free_elbow,
            male_inner_elbow,
            male_support_knee,
            male_support_foot,
            male_ext_knee,
            male_ext_foot,
            beat,
        }
    }

    fn draw_stage(&self, canvas: &mut Canvas, t: f32, unicode: bool, centre: (f32, f32)) {
        let w = self.w as f32;
        let h = self.h as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / h;
            let mut col = STAGE_TOP.lerp(STAGE_MID, smoothstep(v * 1.4));
            col = col.lerp(STAGE_LOW, smoothstep((v - 0.55) / 0.45) * 0.7);
            for x in 0..self.w as isize {
                let gu = (x as f32 + 0.5) / w;
                let grain = value_noise(gu * 4.0 + t * 0.01, v * 5.0, self.seed);
                let dim = col.scale(0.55 + grain * 0.45);
                let ch = if unicode { '·' } else { '.' };
                canvas.set(x, y, ch, dim.scale(0.72));
            }
        }

        // Velvet curtains at both edges.
        let curtain_w = (self.w as f32 * 0.055).clamp(2.0, 8.0);
        for y in 0..self.h as isize {
            let v = y as f32 / h;
            for x in 0..self.w as isize {
                let xf = x as f32;
                let left = xf < curtain_w;
                let right = xf >= w - curtain_w;
                if !left && !right {
                    continue;
                }
                let fold = (xf * 1.7 + (v * 11.0).sin() * 1.2 + t * 0.10).sin() * 0.5 + 0.5;
                let edge = if left {
                    1.0 - xf / curtain_w
                } else {
                    (xf - (w - curtain_w)) / curtain_w
                };
                let col = CURTAIN
                    .lerp(CURTAIN_HL, fold * 0.55 + edge * 0.20)
                    .scale(0.55 + edge * 0.45);
                canvas.set(
                    x,
                    y,
                    if unicode {
                        if fold > 0.55 {
                            '▓'
                        } else {
                            '▒'
                        }
                    } else if fold > 0.55 {
                        '%'
                    } else {
                        ':'
                    },
                    col,
                );
            }
        }

        // Two drifting spotlights.
        for (k, (base, col)) in [(0.27f32, SPOT_WARM), (0.73, SPOT_COOL)].iter().enumerate() {
            let sway = (t * 0.17 + k as f32 * 2.1).sin() * w * 0.035;
            let top_x = w * base + sway;
            for y in 0..(self.h as f32 * 0.92) as isize {
                let v = y as f32 / h;
                let half = 1.5 + v * w * 0.15;
                let x0 = (top_x - half) as isize;
                let x1 = (top_x + half) as isize;
                let fade = (1.0 - v * 0.85).max(0.0);
                for x in x0.max(0)..=x1.min(self.w as isize - 1) {
                    let dx = (x as f32 - top_x).abs() / half.max(1.0);
                    if dx <= 1.0 {
                        let a = (1.0 - dx * dx) * fade * 0.20;
                        canvas.add_light(x, y, *col, a, if unicode { '·' } else { '.' });
                    }
                }
            }
        }

        // Glow behind the dancers.
        let (px, py) = centre;
        let radius = self.h as f32 * 0.30;
        let r = radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                let d = ((dx as f32 * 0.5).powi(2) + (dy as f32).powi(2)).sqrt();
                if d < radius {
                    let a = (1.0 - d / radius).powf(2.0) * 0.10;
                    canvas.add_light(px as isize + dx, py as isize + dy, FEMALE_HL, a, ' ');
                    canvas.add_light(px as isize + dx, py as isize + dy, MALE_HL, a * 0.4, ' ');
                }
            }
        }

        // Stage floor.
        let floor_y = (h * 0.86) as isize;
        for y in floor_y..self.h as isize {
            let v = (y - floor_y) as f32 / h.max(1.0);
            for x in 0..self.w as isize {
                let board = (x as f32 * 0.16 + (y as f32 * 0.7).sin() * 0.8).sin();
                let fade = 1.0 - v * 2.5;
                let col = FLOOR
                    .lerp(FLOOR_HL, 0.12 + board.max(0.0) * 0.18)
                    .scale(fade.max(0.0));
                let ch = if (x as f32 + y as f32 * 7.0) as i32 % 9 == 0 {
                    if unicode {
                        '─'
                    } else {
                        '-'
                    }
                } else if unicode {
                    '·'
                } else {
                    '.'
                };
                canvas.set(x, y, ch, col);
            }
        }
        // Reflection of the pair's warm glow on the boards.
        let rx = px as isize;
        for dy in 0..3 {
            for dx in -8isize..=8 {
                let a = (1.0 - (dx.abs() as f32) / 9.0) * (0.18 - dy as f32 * 0.04);
                if a > 0.0 {
                    canvas.add_light(rx + dx, floor_y + dy, FLOOR_HL, a, ' ');
                }
            }
        }
    }

    fn draw_ribbon(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose, t: f32) {
        let start = pose.female_free_hand;
        let angle = (-1.66 + 0.46 * (pose.beat * 0.71).sin()) + 2.7;
        let dir = P::new(angle.cos(), angle.sin());
        let mut p = start;
        for k in 0..22 {
            let wobble = (t * 1.7 + k as f32 * 0.45).sin();
            p = p
                .add(dir.mul(0.34))
                .add(P::new(wobble * 0.16, wobble * 0.30));
            let (x, y) = xf.pt(p);
            let fade = 1.0 - k as f32 / 24.0;
            let col = FEMALE.lerp(GOLD, k as f32 / 26.0).scale(0.28 + fade * 0.55);
            let ch = if xf.unicode { '≈' } else { '~' };
            canvas.set(x, y, ch, col);
            if k % 7 == 0 {
                canvas.add_light(
                    x,
                    y,
                    FEMALE_HL,
                    0.35 * fade,
                    if xf.unicode { '·' } else { '.' },
                );
            }
        }
    }

    fn draw_dancer_female(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose, t: f32) {
        // Back leg and supporting leg.
        xf.line(
            canvas,
            pose.female_hip,
            pose.female_ext_knee,
            xf.shade(),
            FEMALE_DARK,
            true,
        );
        xf.line(
            canvas,
            pose.female_ext_knee,
            pose.female_ext_foot,
            xf.shade(),
            FEMALE,
            true,
        );
        xf.line(
            canvas,
            pose.female_hip,
            pose.female_support_knee,
            xf.shade(),
            FEMALE_DARK,
            true,
        );
        xf.line(
            canvas,
            pose.female_support_knee,
            pose.female_support_foot,
            xf.shade(),
            FEMALE,
            true,
        );
        // Pointe shoes.
        for foot in [pose.female_ext_foot, pose.female_support_foot] {
            let (fx, fy) = xf.pt(foot);
            canvas.set(fx, fy, if xf.unicode { '▸' } else { '>' }, FEMALE_HL);
        }

        // Tutu: ellipse of gauze plus radiating pleats.
        let hip = pose.female_hip;
        for k in 0..44 {
            let a = k as f32 / 44.0 * TAU;
            let r = 3.15 * (0.92 + 0.08 * (a * 5.0 + t * 2.0).sin());
            let p = hip.add(P::new(a.cos() * r, a.sin() * r * 0.30 + 0.18));
            let (x, y) = xf.pt(p);
            canvas.set(
                x,
                y,
                if xf.unicode { '░' } else { '.' },
                FEMALE_HL.scale(0.55 + 0.25 * (a * 3.0).sin().abs()),
            );
        }
        for k in 0..9 {
            let a = -0.35 + k as f32 / 8.0 * (TAU - 0.6);
            let p = hip.add(P::new(a.cos() * 2.9, a.sin() * 0.88 + 0.2));
            xf.line(
                canvas,
                hip.add(P::new(0.0, 0.10)),
                p,
                if xf.unicode { '╱' } else { '/' },
                FEMALE_HL.scale(0.52),
                false,
            );
        }
        // Bodice.
        let shoulder_mid = pose.female_shoulder_l.lerp(pose.female_shoulder_r, 0.5);
        xf.quad(
            canvas,
            pose.female_shoulder_l,
            pose.female_shoulder_r,
            pose.female_hip.add(P::new(1.0, 0.2)),
            pose.female_hip.add(P::new(-1.0, 0.2)),
            FEMALE_DARK,
        );
        xf.line(
            canvas,
            shoulder_mid,
            pose.female_hip,
            if xf.unicode { '│' } else { '|' },
            FEMALE.scale(0.82),
            false,
        );
        // Waistband.
        xf.line(
            canvas,
            pose.female_hip.add(P::new(-1.15, -0.10)),
            pose.female_hip.add(P::new(1.15, -0.10)),
            if xf.unicode { '═' } else { '=' },
            GOLD,
            false,
        );

        // Free arm raised into the air.
        xf.line(
            canvas,
            pose.female_shoulder_l,
            pose.female_free_elbow,
            xf.shade(),
            SKIN,
            false,
        );
        xf.line(
            canvas,
            pose.female_free_elbow,
            pose.female_free_hand,
            xf.shade(),
            SKIN_HL,
            false,
        );
        let (hx, hy) = xf.pt(pose.female_free_hand);
        canvas.set(hx, hy, if xf.unicode { '✦' } else { '*' }, GOLD);
        canvas.add_light(hx, hy, FEMALE_HL, 0.55, if xf.unicode { '·' } else { '.' });

        // Neck and head with a bun.
        xf.line(
            canvas,
            pose.female_neck,
            pose.female_head,
            xf.shade(),
            SKIN,
            false,
        );
        self.head(canvas, xf, pose.female_head, 1.10, false, true);

        // The inner arm reaching to the shared hand.
        xf.line(
            canvas,
            pose.female_shoulder_r,
            pose.female_inner_elbow,
            xf.shade(),
            SKIN,
            false,
        );
        xf.line(
            canvas,
            pose.female_inner_elbow,
            pose.clasp,
            xf.shade(),
            SKIN_HL,
            false,
        );
    }

    fn draw_dancer_male(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose) {
        // Legs.
        xf.line(
            canvas,
            pose.male_hip,
            pose.male_support_knee,
            xf.shade(),
            MALE_DARK,
            true,
        );
        xf.line(
            canvas,
            pose.male_support_knee,
            pose.male_support_foot,
            xf.shade(),
            MALE,
            true,
        );
        xf.line(
            canvas,
            pose.male_hip,
            pose.male_ext_knee,
            xf.shade(),
            MALE_DARK,
            true,
        );
        xf.line(
            canvas,
            pose.male_ext_knee,
            pose.male_ext_foot,
            xf.shade(),
            MALE,
            true,
        );
        for foot in [pose.male_support_foot, pose.male_ext_foot] {
            let (fx, fy) = xf.pt(foot);
            canvas.set(fx, fy, if xf.unicode { '▸' } else { '>' }, MALE_HL);
        }

        // Tunic / torso.
        xf.quad(
            canvas,
            pose.male_shoulder_l,
            pose.male_shoulder_r,
            pose.male_hip.add(P::new(1.15, 0.15)),
            pose.male_hip.add(P::new(-1.15, 0.15)),
            MALE_DARK,
        );
        xf.line(
            canvas,
            pose.male_shoulder_l.lerp(pose.male_shoulder_r, 0.5),
            pose.male_hip,
            if xf.unicode { '│' } else { '|' },
            MALE.scale(0.8),
            false,
        );
        xf.line(
            canvas,
            pose.male_hip.add(P::new(-1.2, 0.0)),
            pose.male_hip.add(P::new(1.2, 0.0)),
            if xf.unicode { '═' } else { '=' },
            MALE_HL.scale(0.85),
            false,
        );

        // Free arm.
        xf.line(
            canvas,
            pose.male_shoulder_r,
            pose.male_free_elbow,
            xf.shade(),
            SKIN,
            false,
        );
        xf.line(
            canvas,
            pose.male_free_elbow,
            pose.male_free_hand,
            xf.shade(),
            SKIN_HL,
            false,
        );
        let (hx, hy) = xf.pt(pose.male_free_hand);
        canvas.set(hx, hy, if xf.unicode { '✦' } else { '*' }, GOLD);
        canvas.add_light(hx, hy, MALE_HL, 0.40, if xf.unicode { '·' } else { '.' });

        // Neck and head.
        xf.line(
            canvas,
            pose.male_neck,
            pose.male_head,
            xf.shade(),
            SKIN,
            false,
        );
        self.head(canvas, xf, pose.male_head, 1.02, true, false);

        // Inner arm to the clasp.
        xf.line(
            canvas,
            pose.male_shoulder_l,
            pose.male_inner_elbow,
            xf.shade(),
            SKIN,
            false,
        );
        xf.line(
            canvas,
            pose.male_inner_elbow,
            pose.clasp,
            xf.shade(),
            SKIN_HL,
            false,
        );
    }

    fn head(
        &self,
        canvas: &mut Canvas,
        xf: &Xf,
        head: P,
        radius: f32,
        is_male: bool,
        is_female: bool,
    ) {
        let (cx, cy) = xf.pt(head);
        let r = (radius * xf.s).max(1.0);
        let ri = r.ceil() as isize;
        let rx = (r * 2.0).ceil() as isize;
        for dy in -ri..=ri {
            for dx in -rx..=rx {
                let vx = dx as f32 * 0.5;
                let vy = dy as f32;
                if vx * vx + vy * vy <= r * r {
                    let hair = if is_male {
                        vy < -r * 0.15
                    } else {
                        vy < -r * 0.30 || (vx < -r * 0.35 && vy < r * 0.55)
                    };
                    let col = if hair {
                        HAIR.lerp(HAIR_HL, 0.10)
                    } else {
                        SKIN.lerp(SKIN_HL, 0.25 + 0.15 * (-vy / r).max(0.0))
                    };
                    canvas.set(cx + dx, cy + dy, xf.block(), col);
                }
            }
        }
        // Face, turned toward the partner.
        let look = if is_male { -1 } else { 1 };
        canvas.set(cx + look, cy - 1, if xf.unicode { '·' } else { 'o' }, HAIR);
        canvas.set(cx + look, cy, if xf.unicode { '·' } else { 'o' }, HAIR);
        canvas.set(
            cx + look,
            cy + 2,
            if xf.unicode { '▁' } else { '_' },
            SKIN_DARK,
        );
        if is_female {
            // Bun and a small tiara.
            let bun = (cx - 2, cy - 1);
            canvas.set(bun.0, bun.1, xf.shade(), HAIR);
            canvas.set(bun.0, bun.1 + 1, xf.shade(), HAIR.scale(0.85));
            canvas.set(cx, cy - ri - 1, if xf.unicode { '✦' } else { '*' }, GOLD);
            canvas.add_light(
                cx,
                cy - ri - 1,
                GOLD,
                0.5,
                if xf.unicode { '·' } else { '.' },
            );
        } else {
            // A short, swept hair line.
            let top = cy - ri;
            for dx in -2..=2 {
                canvas.set(cx + dx, top, xf.shade(), HAIR_HL.scale(0.6));
            }
        }
    }

    fn draw_clasp(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose, t: f32) {
        let (x, y) = xf.pt(pose.clasp);
        let pulse = (t * 3.0).sin() * 0.5 + 0.5;
        canvas.set(x, y, if xf.unicode { '✦' } else { '*' }, WHITE);
        canvas.set(
            x - 1,
            y,
            if xf.unicode { '─' } else { '-' },
            GOLD.scale(0.9),
        );
        canvas.set(
            x + 1,
            y,
            if xf.unicode { '─' } else { '-' },
            GOLD.scale(0.9),
        );
        canvas.set(
            x,
            y - 1,
            if xf.unicode { '│' } else { '|' },
            GOLD.scale(0.8),
        );
        canvas.set(
            x,
            y + 1,
            if xf.unicode { '│' } else { '|' },
            GOLD.scale(0.8),
        );
        for r in 1..=3 {
            let rr = r as f32;
            let a = (1.0 - rr / 4.0) * (0.35 + pulse * 0.35);
            for k in 0..12 {
                let ang = k as f32 / 12.0 * TAU + t * 0.8;
                let px = x as f32 + ang.cos() * rr * 2.2;
                let py = y as f32 + ang.sin() * rr;
                canvas.add_light(
                    px.round() as isize,
                    py.round() as isize,
                    GOLD,
                    a,
                    if xf.unicode { '·' } else { '.' },
                );
            }
        }
        canvas.add_light(x, y, WHITE, 0.8, if xf.unicode { '·' } else { '.' });
    }

    fn draw_sparkles(&self, canvas: &mut Canvas, t: f32, unicode: bool) {
        let n = ((self.w * self.h) / 320).clamp(8, 34);
        for i in 0..n {
            let hx = hash01(i as i64, 13, self.seed);
            let hy = hash01(29, i as i64, self.seed);
            let drift = t * (0.10 + hx * 0.12);
            let x = (hx * self.w as f32 + (drift + hx * 7.0).sin() * 2.4).rem_euclid(self.w as f32);
            let y =
                (hy * self.h as f32 + (t * 0.12 + hy * 6.0).cos() * 1.8).rem_euclid(self.h as f32);
            let twinkle = (t * 1.5 + hx * 19.0).sin() * 0.5 + 0.5;
            let col = if hx > 0.5 { FEMALE_HL } else { MALE_HL };
            let ch = if unicode {
                if twinkle > 0.88 {
                    '✦'
                } else {
                    '·'
                }
            } else if twinkle > 0.88 {
                '*'
            } else {
                '.'
            };
            canvas.add_light(x as isize, y as isize, col, 0.25 + twinkle * 0.45, ch);
        }
    }
}

impl Scene for BalletScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {
        // All motion is analytic; nothing to integrate.
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        let (cx, cy) = self.centre_at(t);
        let s = self.scale();
        let angle = (t * 0.21 + 0.4).sin() * 0.32 + (t * 0.47).sin() * 0.10;
        let xf = Xf {
            cx,
            cy,
            s,
            ang: angle,
            unicode,
        };
        let pose = self.pose(t);

        self.draw_stage(canvas, t, unicode, (cx, cy));

        // Fading path of where the couple has been.
        for k in 1..=16 {
            let tt = t - k as f32 * 0.095;
            let (tx, ty) = self.centre_at(tt);
            let fade = 1.0 - k as f32 / 18.0;
            for d in 0..3 {
                let x = tx as isize + (d - 1);
                let y = ty as isize + if d == 1 { 1 } else { 0 };
                canvas.add_light(
                    x,
                    y,
                    FEMALE_HL,
                    fade * 0.16,
                    if unicode { '·' } else { '.' },
                );
            }
        }

        self.draw_ribbon(canvas, &xf, &pose, t);
        // Male in the back, female in front, then the shared hands.
        self.draw_dancer_male(canvas, &xf, &pose);
        self.draw_dancer_female(canvas, &xf, &pose, t);
        self.draw_clasp(canvas, &xf, &pose, t);
        self.draw_sparkles(canvas, t, unicode);

        // Tiny stage HUD.
        let title = if unicode {
            "✦ BALLET // PAS DE DEUX ✦"
        } else {
            "* BALLET // PAS DE DEUX *"
        };
        canvas.put_str(2, 0, title, FEMALE_HL.scale(0.8));
        let sep = if unicode { "·" } else { "|" };
        let status = format!("TEMPO {:.2}x  {sep}  HAND IN HAND", f.speed);
        let sx = (self.w as isize - status.chars().count() as isize - 2).max(2);
        canvas.put_str(sx, 0, &status, MALE_HL.scale(0.62));
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_stays_on_screen() {
        let mut s = BalletScene::new(7);
        s.resize(80, 24);
        for i in 0..500 {
            let t = i as f32 * 0.13;
            let (x, y) = s.centre_at(t);
            assert!(x > 0.0 && x < 80.0, "x={x}");
            assert!(y > 0.0 && y < 24.0, "y={y}");
        }
    }

    #[test]
    fn both_arms_reach_the_clasp() {
        let s = BalletScene::new(7);
        for step in 0..20 {
            let pose = s.pose(step as f32 * 0.7);
            let dist = |a: P| {
                let dx = a.x - pose.clasp.x;
                let dy = a.y - pose.clasp.y;
                (dx * dx + dy * dy).sqrt()
            };
            let female = dist(pose.female_shoulder_r);
            let male = dist(pose.male_shoulder_l);
            assert!((2.0..6.5).contains(&female), "female arm {female}");
            assert!((2.0..6.5).contains(&male), "male arm {male}");
        }
    }
}
