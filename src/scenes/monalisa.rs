//! 蒙娜丽莎 — the Mona Lisa as a living oil painting.
//!
//! The whole screen becomes the painting: the sfumato landscape, the dark
//! dress, folded hands and the famously ambiguous smile are rebuilt from
//! coloured terminal cells.  The portrait breathes, her eyes wander and
//! blink, the smile shifts by a hair, and the varnish shimmers as if the
//! canvas were lit by a slow museum spotlight.

use std::f32::consts::PI;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise, Rng};
use crate::scene::{Frame, Scene};

// Sfumato landscape palette.
const SKY_TOP: Rgb = Rgb::hex(0x27332f);
const SKY_HORIZON: Rgb = Rgb::hex(0x8c9a80);
const CLOUD: Rgb = Rgb::hex(0xd6dfc2);
const FAR_MOUNTAIN: Rgb = Rgb::hex(0x5c6f63);
const FAR_LIGHT: Rgb = Rgb::hex(0x8b9a83);
const NEAR_MOUNTAIN: Rgb = Rgb::hex(0x3f5047);
const NEAR_LIGHT: Rgb = Rgb::hex(0x66785f);
const WATER: Rgb = Rgb::hex(0x52665e);
const WATER_LIGHT: Rgb = Rgb::hex(0xa8b79a);
const ROAD: Rgb = Rgb::hex(0x9d8c67);
const FOREGROUND: Rgb = Rgb::hex(0x2c2a1d);

// Figure palette.
const HAIR: Rgb = Rgb::hex(0x241a13);
const HAIR_HL: Rgb = Rgb::hex(0x5c4128);
const DRESS: Rgb = Rgb::hex(0x2b2519);
const DRESS_HL: Rgb = Rgb::hex(0x57472a);
const SKIN: Rgb = Rgb::hex(0xd8b487);
const SKIN_LIGHT: Rgb = Rgb::hex(0xf0d6a8);
const SKIN_SHADOW: Rgb = Rgb::hex(0x9f7853);
const LIP: Rgb = Rgb::hex(0x8e5748);
const LIP_LIGHT: Rgb = Rgb::hex(0xbd7f67);
const EYE_DARK: Rgb = Rgb::hex(0x33251b);
const BROW: Rgb = Rgb::hex(0x5a3d2a);

// Frame palette.
const GOLD: Rgb = Rgb::hex(0xe0aa3e);
const GOLD_DARK: Rgb = Rgb::hex(0x6b4712);
const GOLD_HOT: Rgb = Rgb::hex(0xffe9a8);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Sky,
    Mountain,
    Water,
    Dress,
    Hair,
    Skin,
    Detail,
}

struct Mood {
    t: f32,
    blink: f32,
    gaze_u: f32,
    gaze_v: f32,
    smile: f32,
    breath: f32,
}

pub struct MonaLisaScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    next_blink: f64,
    blink_start: f64,
    blink_len: f64,
}

impl MonaLisaScene {
    pub fn new(seed: u64) -> Self {
        MonaLisaScene {
            seed,
            rng: Rng::new(seed ^ 0x4d4f_4e41_4c49_5341),
            w: 0,
            h: 0,
            next_blink: 3.2,
            blink_start: -10.0,
            blink_len: 0.26,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.next_blink = self.rng.range(3.0, 6.5) as f64;
        self.blink_start = -10.0;
    }

    fn blink(&self, t: f64) -> f32 {
        if t < self.blink_start {
            return 0.0;
        }
        let p = (t - self.blink_start) / self.blink_len.max(0.01);
        if !(0.0..=1.0).contains(&p) {
            return 0.0;
        }
        (p as f32 * PI).sin().clamp(0.0, 1.0).powf(0.55)
    }

    fn background(&self, u: f32, v: f32, t: f32) -> (Rgb, Kind) {
        // Sky, softly scumbled with drifting clouds.
        let up = smoothstep((v + 1.0) / 1.35);
        let mut c = SKY_TOP.lerp(SKY_HORIZON, up * 0.92);
        let cloud = value_noise(u * 2.3 + t * 0.012, v * 4.7, self.seed ^ 0x11);
        c = c.add(CLOUD, (cloud - 0.48).max(0.0) * 0.15);
        let mut kind = Kind::Sky;

        // Two ridges of blue-green distance.
        let far = -0.18 + 0.070 * (u * 3.1 + 0.6).sin() + 0.032 * (u * 6.7 + 2.0).sin();
        if v > far {
            let d = ((v - far) / 0.55).clamp(0.0, 1.0);
            c = FAR_MOUNTAIN.lerp(FAR_LIGHT, (0.25 + d * 0.35) * (0.7 + cloud * 0.4));
            kind = Kind::Mountain;
        }
        let near = 0.03 + 0.085 * (u * 2.3 + 2.1).sin() + 0.038 * (u * 5.2 + 0.4).sin();
        if v > near {
            let d = ((v - near) / 0.7).clamp(0.0, 1.0);
            c = NEAR_MOUNTAIN.lerp(NEAR_LIGHT, 0.18 + d * 0.28);
            kind = Kind::Mountain;
        }

        // Water and the winding river on the right; the pale road on the left.
        if v > 0.30 {
            let wave = (u * 21.0 + v * 8.5 + t * 0.55).sin();
            c = WATER.lerp(WATER_LIGHT, 0.30 + 0.16 * wave);
            kind = Kind::Water;
        }
        let river = 0.58 + 0.08 * (v * 5.2 + 0.8).sin();
        if v > 0.10 && (u - river).abs() < 0.065 {
            c = WATER_LIGHT.scale(0.92 + 0.10 * (u * 30.0 + t).sin());
            kind = Kind::Water;
        }
        let road = -0.55 + 0.065 * (v * 4.2).sin();
        if v > 0.16 && (u - road).abs() < 0.026 {
            c = ROAD.scale(0.8);
            kind = Kind::Mountain;
        }

        // Aerial haze and dark foreground ledges at the lower corners.
        if v > 0.55 && u.abs() > 0.54 {
            c = c.lerp(FOREGROUND, 0.5);
            kind = Kind::Mountain;
        }
        let grain = value_noise(u * 11.0 + t * 0.01, v * 11.0, self.seed ^ 0x23);
        c = c.scale(0.90 + grain * 0.20);
        (c, kind)
    }

    fn dress_half(v: f32) -> f32 {
        if v < 0.02 {
            return 0.0;
        }
        let t = ((v - 0.02) / 0.98).clamp(0.0, 1.0);
        let shoulders = 0.31 + 0.30 * smoothstep(t * 1.3);
        (shoulders + 0.045 * (t * PI).sin()).min(0.92)
    }

    fn paint(&self, u: f32, v: f32, m: &Mood) -> (char, Rgb, Kind) {
        let (mut c, mut kind) = self.background(u, v, m.t);
        let breath = m.breath;
        let vv = v - breath * 0.004;

        // Dress: a dark pyramidal mass from shoulders to the bottom edge.
        let body_half = Self::dress_half(vv);
        let shoulder_mass = ellipse(u, vv, 0.0, 0.105, 0.47, 0.19) <= 1.0;
        if (vv > 0.02 && u.abs() <= body_half) || shoulder_mass {
            let fold = value_noise(u * 8.0, vv * 8.0 + m.t * 0.02, self.seed ^ 0x31);
            let lit = 0.55 + 0.45 * smoothstep((0.35 - u) / 0.9);
            let mut col = DRESS.lerp(DRESS_HL, lit * 0.55 + fold * 0.18);
            if (u * 13.0 + vv * 5.0 + m.t * 0.1).sin() > 0.93 {
                col = col.add(DRESS_HL, 0.28);
            }
            c = col;
            kind = Kind::Dress;
        }

        // Hair framing the face and falling to the shoulders.
        let hair = ellipse(u, vv, 0.0, -0.25, 0.345, 0.50);
        let side_locks = vv > -0.44 && vv < 0.22 && u.abs() > 0.20 && u.abs() < 0.355;
        if hair <= 1.0 || side_locks {
            let strand = value_noise(u * 16.0, vv * 26.0 + m.t * 0.03, self.seed ^ 0x41);
            let sheen = (u * 8.0 - vv * 11.0 + m.t * 0.15).sin() * 0.5 + 0.5;
            let mut col = HAIR.lerp(HAIR_HL, strand * 0.42 + sheen * 0.10);
            if hair > 0.82 && hair < 1.0 {
                col = col.scale(0.72 + sheen * 0.18);
            }
            c = col.scale(1.12);
            kind = Kind::Hair;
        }

        // Neck and the small triangle of chest above the neckline.
        let neck = u.abs() < 0.105 && vv > -0.055 && vv < 0.115;
        let chest = vv > 0.07 && vv < 0.20 && u.abs() < 0.16 * (1.0 - (vv - 0.07) / 0.28);
        if neck || chest {
            let shade = 0.72 + 0.28 * smoothstep((u + 0.10) / 0.20);
            c = SKIN_SHADOW.lerp(SKIN, shade);
            kind = Kind::Skin;
        }

        // Face: a soft oval with a slightly narrower, feminine jaw.
        let face_v = ((vv + 0.335) / 0.315).clamp(-1.0, 1.0);
        let jaw = 1.0 - 0.17 * face_v.max(0.0).powf(1.35);
        let face = ellipse(u, vv, 0.0, -0.335, 0.198 * jaw, 0.315);
        if face <= 1.0 {
            let light = (1.0 - face).clamp(0.0, 1.0);
            let lx = (-u * 0.95 - (vv + 0.33) * 0.25 + 0.80).clamp(0.0, 1.0);
            let mut col = SKIN_SHADOW.lerp(SKIN_LIGHT, lx * 0.86 + light * 0.26);
            col = col.lerp(SKIN_SHADOW, ((u + 0.19) / 0.38).clamp(0.0, 1.0) * 0.18);
            let brush = value_noise(u * 15.0 + m.t * 0.02, vv * 19.0, self.seed ^ 0x61);
            col = col.scale(0.94 + brush * 0.12);
            if ellipse(u, vv, -0.09, -0.29, 0.075, 0.060) <= 1.0
                || ellipse(u, vv, 0.10, -0.29, 0.070, 0.058) <= 1.0
            {
                col = col.lerp(Rgb::hex(0xc48a72), 0.14);
            }
            // Soft oval edge and cheek shading.
            if face > 0.80 {
                col = col.lerp(SKIN_SHADOW, (face - 0.80) / 0.20 * 0.32);
            }
            if (u > 0.09 && vv > -0.36) || (u < -0.11 && vv > -0.27) {
                col = col.lerp(SKIN_SHADOW, 0.20);
            }
            if vv < -0.46 {
                col = col.lerp(SKIN_SHADOW, 0.16);
            }
            c = col;
            kind = Kind::Skin;
        }

        // Folded hands resting across the lap.
        let hands = ellipse(u, vv, -0.12, 0.505, 0.195, 0.095) <= 1.0
            || ellipse(u, vv, 0.105, 0.552, 0.155, 0.078) <= 1.0
            || ellipse(u, vv, -0.30, 0.438, 0.115, 0.085) <= 1.0;
        if hands {
            let shade = 0.80 + 0.20 * smoothstep((u + 0.20) / 0.42);
            c = SKIN_SHADOW.lerp(SKIN_LIGHT, shade);
            kind = Kind::Skin;
        }

        // ---- face details -------------------------------------------------
        // Mona Lisa has no visible eyebrows; a faint brow shadow is all that
        // remains of them.
        for side in [-1.0f32, 1.0] {
            let du = u - side * 0.076;
            if du.abs() < 0.045 && (vv + 0.404).abs() < 0.006 {
                c = c.lerp(SKIN_SHADOW, 0.24);
            }
        }

        // Eyes.  They track slowly and close on a blink.
        for side in [-1.0f32, 1.0] {
            let du = u - side * 0.076;
            let dv = vv + 0.362;
            if ellipse(du, dv, 0.0, 0.0, 0.038, 0.018) <= 1.0 {
                if m.blink > 0.35 {
                    c = SKIN_SHADOW.scale(0.88);
                    kind = Kind::Detail;
                    if dv.abs() < 0.005 {
                        c = BROW.scale(0.55);
                    }
                } else {
                    c = Rgb::hex(0xcdb68f);
                    kind = Kind::Detail;
                    let pu = du - (side * 0.0 + m.gaze_u);
                    let pv = dv - m.gaze_v;
                    if ellipse(pu, pv, 0.0, 0.0, 0.016, 0.011) <= 1.0 {
                        c = EYE_DARK;
                    }
                    if ellipse(pu + 0.006, pv + 0.005, 0.0, 0.0, 0.004, 0.004) <= 1.0 {
                        c = Rgb::hex(0xfff2d0);
                    }
                    if dv < -0.012 {
                        c = c.scale(0.82);
                    }
                }
            }
            // Under-eye shadow.
            if du.abs() < 0.065 && dv > 0.014 && dv < 0.030 {
                c = c.lerp(SKIN_SHADOW, 0.28);
            }
        }

        // Nose: a soft ridge with a shadow on the right.
        if u.abs() < 0.033 && vv > -0.375 && vv < -0.235 {
            let ridge = 1.0 - (u.abs() / 0.033).powi(2);
            if u > 0.006 {
                c = c.lerp(SKIN_SHADOW, 0.18 * ridge);
            } else if u < -0.004 {
                c = c.lerp(SKIN_LIGHT, 0.20 * ridge);
            }
            if vv > -0.265 && vv < -0.242 && u.abs() > 0.010 && u.abs() < 0.026 {
                c = c.lerp(SKIN_SHADOW, 0.55);
            }
        }

        // The ambiguous smile: the corners lift by a hair over time.
        let smile_lift = 0.0035 * m.smile;
        let m_v = -0.188 - 0.012 * (u.abs() / 0.052).powf(1.6) + smile_lift;
        if u.abs() < 0.052 && (vv - m_v).abs() < 0.010 {
            c = LIP;
            kind = Kind::Detail;
            if vv > m_v + 0.003 {
                c = LIP_LIGHT;
            }
            if u.abs() > 0.055 {
                c = c.scale(0.86 + 0.14 * m.smile);
            }
        }

        // Hand and finger separation lines.
        if hands {
            let finger = (u * 19.0 + vv * 3.0).sin();
            if vv > 0.47 && vv < 0.60 && finger > 0.965 {
                c = c.lerp(SKIN_SHADOW, 0.55);
                kind = Kind::Detail;
            }
            if (vv - 0.505).abs() < 0.006 && u > -0.31 && u < 0.25 {
                c = c.lerp(SKIN_SHADOW, 0.40);
            }
        }

        // Varnish: a slow warm/green shimmer across the paint.
        let varnish = value_noise(u * 6.0 + m.t * 0.03, vv * 6.0, self.seed ^ 0x51);
        c = c.scale(0.96 + varnish * 0.08);
        let vig = (1.0 - (u * u * 0.10 + vv * vv * 0.12)).clamp(0.62, 1.0);
        c = c.scale(vig);
        (glyph(kind, c.luminance()), c, kind)
    }

    fn draw_frame(&self, canvas: &mut Canvas, f: &Frame, unicode: bool) {
        let w = self.w as isize;
        let h = self.h as isize;
        let t = f.t as f32;
        let glint = ((t * 0.55).sin() * 0.5 + 0.5).powf(5.0);
        let base = GOLD_DARK.lerp(GOLD, 0.55 + glint * 0.35);
        let hot = GOLD.lerp(GOLD_HOT, glint);

        for x in 0..w {
            let p = x as f32 / (w.max(2) - 1) as f32;
            let sheen = (p * 9.0 + t * 0.7).sin() * 0.5 + 0.5;
            let col = base.lerp(hot, sheen.powf(7.0));
            let ch = if unicode {
                if x % 11 == 0 {
                    '❖'
                } else {
                    '═'
                }
            } else if x % 11 == 0 {
                '*'
            } else {
                '-'
            };
            canvas.set(x, 0, ch, col);
            canvas.set(x, h - 1, ch, col.scale(0.82));
        }
        for y in 0..h {
            let p = y as f32 / (h.max(2) - 1) as f32;
            let sheen = (p * 7.0 - t * 0.5).sin() * 0.5 + 0.5;
            let col = base.lerp(hot, sheen.powf(7.0) * 0.8);
            let ch = if unicode { '║' } else { '|' };
            canvas.set(0, y, ch, col);
            canvas.set(w - 1, y, ch, col.scale(0.78));
        }
        let corners = if unicode {
            ['╔', '╗', '╚', '╝']
        } else {
            ['+', '+', '+', '+']
        };
        canvas.set(0, 0, corners[0], hot);
        canvas.set(w - 1, 0, corners[1], hot);
        canvas.set(0, h - 1, corners[2], hot.scale(0.9));
        canvas.set(w - 1, h - 1, corners[3], hot.scale(0.9));

        // Little brass plaque in the top frame.
        let title = if unicode {
            "✦ LA GIOCONDA · 1503 ✦"
        } else {
            "* MONA LISA * 1503 *"
        };
        let tx = (w - title.chars().count() as isize - 6).max(3);
        canvas.put_str(tx, 0, title, GOLD_HOT.scale(0.72));
        let sig = "L. da Vinci";
        canvas.put_str(3, h - 1, sig, GOLD.scale(0.62));
    }

    fn sx(&self, u: f32) -> isize {
        ((u * 0.5 + 0.5) * self.w as f32).round() as isize
    }

    fn sy(&self, v: f32) -> isize {
        ((v * 0.5 + 0.5) * self.h as f32).round() as isize
    }

    /// Crisp facial features are drawn on top of the soft sfumato shading so
    /// that the portrait stays readable even on a small terminal.
    fn draw_face(&self, canvas: &mut Canvas, m: &Mood, unicode: bool) {
        let solid = if unicode { '█' } else { '#' };
        let eye_v = -0.362;
        let ey = self.sy(eye_v + m.breath * 0.002);
        let eye_half = ((0.036 * self.w as f32 * 0.5) as isize).max(1);
        let gaze = ((m.gaze_u * self.w as f32 * 0.5).round() as isize).clamp(-1, 1);

        for side in [-1.0f32, 1.0] {
            let ex = self.sx(side * 0.076);
            if m.blink > 0.35 {
                // The terminal has no tinted background: use full cells so
                // the closed lid does not punch black holes in the face.
                for dx in -eye_half - 1..=eye_half + 1 {
                    canvas.set(ex + dx, ey, solid, SKIN_SHADOW.scale(0.72));
                }
                canvas.set(
                    ex,
                    ey + 1,
                    if unicode { '·' } else { '.' },
                    SKIN_SHADOW.scale(0.65),
                );
            } else {
                // Only the pupil is painted dark; the soft sclera comes from
                // the underlying oil shading, so the eye never becomes a bar.
                let pupil_x = (ex + gaze).clamp(ex - eye_half + 1, ex + eye_half - 1);
                canvas.set(pupil_x, ey, solid, EYE_DARK);
                canvas.set(
                    pupil_x - 1,
                    ey,
                    if unicode { '·' } else { '.' },
                    Rgb::hex(0xfff4da).scale(0.85),
                );
            }
        }

        // Nose: a faint sfumato shadow between the eyes and the mouth.
        let nx = self.sx(0.0);
        let nose_y = self.sy(-0.265 + m.breath * 0.002);
        canvas.set(nx + 1, nose_y, solid, SKIN_SHADOW.scale(0.62));
        canvas.set(nx + 2, nose_y, solid, SKIN_SHADOW.scale(0.42));
        canvas.set(nx - 1, nose_y, solid, SKIN_LIGHT.scale(0.86));
        // The half-smile.  The corners rise by a single cell very slowly.
        let my = self.sy(-0.188 + m.breath * 0.002);
        let mouth_half = ((0.048 * self.w as f32 * 0.5) as isize).max(1);
        let lift = if m.smile > 0.55 { 1 } else { 0 };
        for dx in -mouth_half..=mouth_half {
            let row = if dx.abs() == mouth_half && lift > 0 {
                my - 1
            } else {
                my
            };
            let lip = if row == my {
                LIP_LIGHT.scale(0.82)
            } else {
                LIP.scale(0.88)
            };
            canvas.set(nx + dx, row, solid, lip);
        }
        // The lower lip catches only a whisper of light.
        canvas.set(
            nx - 1,
            my + 1,
            if unicode { '·' } else { '.' },
            LIP_LIGHT.scale(0.58),
        );
        // Corner dimples sharpen the ambiguity of the expression.
        canvas.set(
            nx - mouth_half - 1,
            my - lift,
            if unicode { '·' } else { '.' },
            LIP.scale(0.42),
        );
        canvas.set(
            nx + mouth_half + 1,
            my - lift,
            if unicode { '·' } else { '.' },
            LIP.scale(0.42),
        );
    }

    /// Hair strands, veil sheen, collar and folded hands.
    fn draw_details(&self, canvas: &mut Canvas, m: &Mood, unicode: bool) {
        let cx = self.sx(0.0);
        // Centre parting, disappearing into the dark hair.
        for k in 0..6 {
            let y = self.sy(-0.735 + k as f32 * 0.012);
            canvas.set(cx, y, if unicode { '·' } else { '.' }, HAIR_HL.scale(0.34));
            canvas.add_light(cx, y, HAIR_HL, 0.10, if unicode { '·' } else { '.' });
        }
        // Loose strands falling down both sides of the face.
        for side in [-1.0f32, 1.0] {
            for k in 0..16 {
                let f = k as f32 / 15.0;
                let u = side * (0.305 + 0.018 * (f * 9.0 + m.t * 0.4).sin());
                let v = -0.49 + f * 0.78;
                let col = HAIR_HL.scale(0.26 + 0.12 * (f * 3.0).sin().abs());
                let (x, y) = (self.sx(u), self.sy(v));
                canvas.set(x, y, if unicode { '░' } else { ':' }, col);
                if k % 4 == 0 {
                    canvas.add_light(x, y, HAIR_HL, 0.18, if unicode { '·' } else { '.' });
                }
            }
        }
        // The thin veil catches the museum light along the crown.
        for k in 0..20 {
            let a = -2.55 + k as f32 / 19.0 * 2.35;
            let u = a.cos() * 0.32;
            let v = -0.25 + a.sin() * 0.47;
            let col = Rgb::hex(0x8f9a84).scale(0.26 + 0.14 * (k as f32 * 0.7 + m.t).sin().abs());
            canvas.set(self.sx(u), self.sy(v), if unicode { '·' } else { '.' }, col);
        }
        // Collar and folded hands are left to the soft shading; crisp edges
        // here read as holes in the paint at terminal resolution.
    }

    fn draw_motes(&self, canvas: &mut Canvas, t: f32, unicode: bool) {
        let n = ((self.w * self.h) / 260).clamp(6, 26);
        for i in 0..n {
            let base_x = hash01(i as i64, 7, self.seed);
            let base_y = hash01(11, i as i64, self.seed);
            let drift = (t * (0.05 + base_x * 0.04) + base_x * 9.0).sin() * 1.6;
            let x = (base_x * self.w as f32 + drift + t * 0.22).rem_euclid(self.w as f32);
            let y = (base_y * self.h as f32 + (t * 0.16 + base_y * 12.0).sin() * 1.2)
                .rem_euclid(self.h as f32);
            let glow = 0.20 + 0.35 * ((t * 0.9 + base_x * 17.0).sin() * 0.5 + 0.5);
            canvas.add_light(
                x as isize,
                y as isize,
                GOLD_HOT,
                glow * 0.35,
                if unicode { '·' } else { '.' },
            );
        }
    }
}

impl Scene for MonaLisaScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        if f.t >= self.next_blink {
            self.blink_start = f.t;
            self.blink_len = self.rng.range(0.20, 0.34) as f64;
            self.next_blink = f.t + self.rng.range(3.4, 8.2) as f64;
            // Occasionally the painting blinks twice, very softly.
            if self.rng.chance(0.22) {
                self.next_blink = f.t + self.blink_len + 0.28;
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        let blink = self.blink(t as f64);
        let smile = (t * 0.24).sin() * 0.5 + 0.5;
        let gaze_cells = (t * 0.37).sin() * 1.35 + (t * 0.91).sin() * 0.45;
        let gaze_rows = (t * 0.53).sin() * 0.45;
        let mood = Mood {
            t,
            blink,
            gaze_u: gaze_cells * 2.0 / self.w.max(1) as f32,
            gaze_v: gaze_rows * 2.0 / self.h.max(1) as f32,
            smile,
            breath: (t * 0.82).sin(),
        };

        let cx = self.w as f32 * 0.5;
        let cy = self.h as f32 * 0.5;
        for y in 0..self.h {
            let v = (y as f32 + 0.5 - cy) * 2.0 / self.h.max(1) as f32;
            for x in 0..self.w {
                let u = (x as f32 + 0.5 - cx) * 2.0 / self.w.max(1) as f32;
                let (mut ch, col, kind) = self.paint(u, v, &mood);
                if !unicode {
                    ch = ascii_glyph(kind, ch);
                }
                canvas.set(x as isize, y as isize, ch, col);
            }
        }

        self.draw_face(canvas, &mood, unicode);
        self.draw_details(canvas, &mood, unicode);

        // A slow museum spotlight blooms over the face.
        let fx = (self.w as f32 * 0.5) as isize;
        let fy = (self.h as f32 * 0.42) as isize;
        for dy in -2..=2 {
            for dx in -4..=4 {
                let d = (dx * dx) as f32 * 0.25 + (dy * dy) as f32;
                if d < 9.0 {
                    canvas.add_light(
                        fx + dx,
                        fy + dy,
                        Rgb::hex(0xffe6b0),
                        0.055 * (1.0 - d / 9.0),
                        ' ',
                    );
                }
            }
        }

        self.draw_motes(canvas, t, unicode);
        self.draw_frame(canvas, f, unicode);
    }
}

fn ellipse(u: f32, v: f32, cx: f32, cy: f32, rx: f32, ry: f32) -> f32 {
    let x = (u - cx) / rx.max(0.0001);
    let y = (v - cy) / ry.max(0.0001);
    x * x + y * y
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn glyph(kind: Kind, lum: f32) -> char {
    match kind {
        Kind::Sky => {
            if lum < 0.10 {
                '·'
            } else {
                '░'
            }
        }
        Kind::Mountain => {
            if lum < 0.13 {
                '▒'
            } else {
                '▓'
            }
        }
        Kind::Water => '≈',
        Kind::Dress => {
            if lum < 0.10 {
                '▒'
            } else {
                '▓'
            }
        }
        Kind::Hair => '▓',
        Kind::Skin => {
            if lum < 0.40 {
                '▒'
            } else {
                '█'
            }
        }
        Kind::Detail => '█',
    }
}

fn ascii_glyph(kind: Kind, ch: char) -> char {
    match kind {
        Kind::Sky => {
            if ch == '·' {
                '.'
            } else {
                ':'
            }
        }
        Kind::Mountain => {
            if ch == '▒' {
                ':'
            } else {
                '='
            }
        }
        Kind::Water => '~',
        Kind::Dress => '#',
        Kind::Hair => '%',
        Kind::Skin => {
            if ch == '▒' {
                '+'
            } else {
                '#'
            }
        }
        Kind::Detail => '*',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_frame() {
        let mut s = MonaLisaScene::new(7);
        s.resize(80, 24);
        assert_eq!(s.w, 80);
        assert!(s.next_blink > 0.0);
    }

    #[test]
    fn blink_closes_and_opens() {
        let mut s = MonaLisaScene::new(7);
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
        assert!(s.blink(1.1) > 0.5);
        assert_eq!(s.blink(9.0), 0.0);
    }
}
