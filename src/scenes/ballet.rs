//! 冰上芭蕾 — ICE BALLET: a male and a female skater gliding hand in hand
//! across a rink.
//!
//! The pair is drawn as a procedural ballet skeleton: the woman in a tutu and
//! skate boots, the man in a fitted tunic.  Both dancers are animated by a
//! 48 second choreography (glide, crossover, arabesque, lift, death spiral,
//! jump, spread eagle, spin, finale).  Every key pose is expressed as a handful
//! of hand and skate targets and solved with two bone IK, so the joints always
//! stay anatomical and the two inner hands can meet exactly while the rest of
//! the body keeps moving.
//!
//! On top of the keyframes a secondary layer adds breathing, skating
//! footwork, arm float and delayed head motion; the couple's glide path drives
//! the body lean and the engraved tracings left on the ice.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise};
use crate::scene::{Frame, Scene};

const WALL_TOP: Rgb = Rgb::hex(0x070a12);
const RINK_MID: Rgb = Rgb::hex(0x1d3c5c);
const RINK_NEAR: Rgb = Rgb::hex(0x060d18);
const BOARD: Rgb = Rgb::hex(0x123049);
const BOARD_HL: Rgb = Rgb::hex(0x3f7fa8);
const ICE_LINE: Rgb = Rgb::hex(0xbfe9ff);
const ICE_GLOW: Rgb = Rgb::hex(0x7fd4ff);
const AUD_FAR: Rgb = Rgb::hex(0x2b3d55);
const AUD_MID: Rgb = Rgb::hex(0x3d5672);
const AUD_NEAR: Rgb = Rgb::hex(0x5a7794);
const ARENA_WARM: Rgb = Rgb::hex(0xffd9a0);
const ARENA_COOL: Rgb = Rgb::hex(0x8fdcff);
const SPOT_WARM: Rgb = Rgb::hex(0xffe7b0);
const SPOT_COOL: Rgb = Rgb::hex(0x9fe8ff);

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

/// Local `y` of the ice surface underneath the pair.
const ICE_Y: f32 = 8.0;
/// Length of one full choreography loop, in seconds.
const LOOP: f32 = 39.0;

/// Playback rate of the whole scene (choreography, glide and secondary
/// motion).  1.0 is the authored tempo; the staged version runs twice as
/// fast.  `--speed` multiplies on top of this.
const TEMPO: f32 = 2.0;

/// Arms and legs follow the torso with this delay (scene seconds).  It is
/// what gives the dance its follow-through: the body leads, the limbs arrive
/// a moment later, so nothing moves as one rigid piece.
const LIMB_LAG: f32 = 0.13;

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
    fn sub(self, o: P) -> P {
        P::new(self.x - o.x, self.y - o.y)
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
    fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    fn dist(self, o: P) -> f32 {
        self.sub(o).len()
    }
    fn unit(self) -> P {
        let l = self.len();
        if l < 1e-5 {
            P::new(0.0, 1.0)
        } else {
            self.mul(1.0 / l)
        }
    }
    fn perp(self) -> P {
        P::new(-self.y, self.x)
    }
    fn dot(self, o: P) -> f32 {
        self.x * o.x + self.y * o.y
    }
}

/// Body metrics of one dancer, in local units.
#[derive(Clone, Copy)]
struct Rig {
    spine: f32,
    chest_k: f32,
    neck: f32,
    sh_half: f32,
    up: f32,
    fore: f32,
    thigh: f32,
    shin: f32,
    hip_half: f32,
    hip_y: f32,
    head_r: f32,
    skirt: f32,
}

const FEMALE_RIG: Rig = Rig {
    spine: 4.85,
    chest_k: 0.80,
    neck: 1.18,
    sh_half: 1.22,
    up: 1.80,
    fore: 1.70,
    thigh: 2.90,
    shin: 2.90,
    hip_half: 0.70,
    hip_y: 0.12,
    head_r: 1.05,
    skirt: 2.35,
};

const MALE_RIG: Rig = Rig {
    spine: 5.05,
    chest_k: 0.82,
    neck: 1.22,
    sh_half: 1.48,
    up: 1.92,
    fore: 1.80,
    thigh: 3.00,
    shin: 3.00,
    hip_half: 0.76,
    hip_y: 0.12,
    head_r: 1.00,
    skirt: 0.0,
};

/// Two bone IK: returns the middle joint plus the (possibly clamped) end joint.
fn limb(a: P, target: P, l1: f32, l2: f32, bend: P) -> (P, P) {
    let d = a.dist(target);
    let dir = target.sub(a).unit();
    let dc = d.clamp((l1 - l2).abs().max(0.08), l1 + l2 - 0.015);
    let along = (l1 * l1 - l2 * l2 + dc * dc) / (2.0 * dc);
    let h = (l1 * l1 - along * along).max(0.0).sqrt();
    let side = if dir.perp().dot(bend) >= 0.0 {
        1.0
    } else {
        -1.0
    };
    let mid = a.add(dir.mul(along)).add(dir.perp().mul(side * h));
    (mid, a.add(dir.mul(dc)))
}

/// One key pose of one dancer: root, spine, head and the four limb targets.
#[derive(Clone, Copy)]
struct DKey {
    root: (f32, f32),
    lean: f32,
    roll: f32,
    head: f32,
    out: (f32, f32),
    inn: (f32, f32),
    l: (f32, f32),
    r: (f32, f32),
}

#[derive(Clone, Copy)]
struct Key {
    spin: f32,
    f: DKey,
    m: DKey,
}

#[allow(clippy::too_many_arguments)]
const fn dk(
    root: (f32, f32),
    lean: f32,
    roll: f32,
    head: f32,
    out: (f32, f32),
    inn: (f32, f32),
    l: (f32, f32),
    r: (f32, f32),
) -> DKey {
    DKey {
        root,
        lean,
        roll,
        head,
        out,
        inn,
        l,
        r,
    }
}

/// The choreography: one key every 4 seconds, looping.
///
/// `l` is the foot on the pair's outer side (towards -x for the woman, +x for
/// the man); `r` is the foot on the partner side.  The two `inn` points are the
/// hands that meet: whenever they coincide the couple is holding.
const KEYS: [Key; 13] = [
    // 0 · entrance glide, arms swept back
    Key {
        spin: 0.0,
        f: dk(
            (-3.55, 2.20),
            0.05,
            0.04,
            0.02,
            (-6.60, 1.10),
            (0.10, 1.05),
            (-4.35, 8.00),
            (-2.85, 7.45),
        ),
        m: dk(
            (3.55, 2.25),
            -0.05,
            -0.04,
            -0.02,
            (6.50, 0.90),
            (0.10, 1.05),
            (2.85, 7.45),
            (4.35, 8.00),
        ),
    },
    // 1 · crossover push, knees deep, arms swinging through
    Key {
        spin: 0.0,
        f: dk(
            (-3.70, 2.55),
            0.10,
            0.06,
            0.06,
            (-6.35, -1.30),
            (0.05, 1.35),
            (-4.60, 8.00),
            (-2.10, 7.55),
        ),
        m: dk(
            (3.70, 2.60),
            -0.08,
            -0.06,
            -0.04,
            (6.30, -1.05),
            (0.05, 1.35),
            (2.10, 7.55),
            (4.60, 8.00),
        ),
    },
    // 2 · she goes to arabesque, he braces and holds
    Key {
        spin: 0.15,
        f: dk(
            (-3.45, 2.35),
            -0.30,
            0.10,
            -0.16,
            (-7.15, -1.10),
            (-0.95, -0.55),
            (-8.80, 0.10),
            (-3.05, 8.00),
        ),
        m: dk(
            (3.50, 2.45),
            -0.12,
            -0.06,
            -0.10,
            (6.30, 0.80),
            (-0.95, -0.55),
            (2.55, 8.00),
            (4.30, 7.90),
        ),
    },
    // 3 · promenade: she holds an attitude, he carries her round by the hand
    Key {
        spin: 0.30,
        f: dk(
            (-3.15, 2.15),
            -0.12,
            0.10,
            -0.16,
            (-6.40, -2.30),
            (0.10, -1.25),
            (-6.70, 4.30),
            (-2.75, 8.00),
        ),
        m: dk(
            (3.55, 2.45),
            -0.14,
            -0.06,
            0.14,
            (6.30, -0.70),
            (0.10, -1.25),
            (2.70, 8.00),
            (4.35, 7.85),
        ),
    },
    // 4 · lift: he holds her overhead, both of her feet clear of the ice
    Key {
        spin: 0.35,
        f: dk(
            (-1.00, -3.90),
            0.08,
            0.08,
            -0.15,
            (-3.90, -9.60),
            (0.20, -5.40),
            (-3.60, -3.40),
            (-0.10, -1.60),
        ),
        m: dk(
            (2.60, 2.35),
            -0.10,
            -0.05,
            0.26,
            (1.60, -2.60),
            (-1.00, -3.90),
            (1.60, 8.00),
            (4.00, 8.00),
        ),
    },
    // 5 · landing, both hands join for a standing spin
    Key {
        spin: 0.55,
        f: dk(
            (-3.55, 2.15),
            0.14,
            0.05,
            0.10,
            (-7.05, -1.85),
            (0.55, 0.10),
            (-4.15, 8.00),
            (-2.95, 7.70),
        ),
        m: dk(
            (3.55, 2.30),
            -0.16,
            -0.05,
            -0.12,
            (7.15, -1.65),
            (0.55, 0.10),
            (2.95, 7.70),
            (4.15, 8.00),
        ),
    },
    // 6 · death spiral: she is low and arched, he pivots in a lunge
    Key {
        spin: 0.45,
        f: dk(
            (-2.35, 4.30),
            -1.05,
            0.14,
            -0.30,
            (-8.20, 4.90),
            (-1.60, 0.05),
            (-4.85, 8.00),
            (-1.30, 7.85),
        ),
        m: dk(
            (2.90, 2.70),
            0.16,
            -0.08,
            0.18,
            (7.40, 1.00),
            (-1.60, 0.05),
            (1.85, 8.00),
            (4.75, 8.00),
        ),
    },
    // 7 · rise, crossovers the other way
    Key {
        spin: 0.25,
        f: dk(
            (-3.75, 2.30),
            0.08,
            -0.05,
            0.04,
            (-6.30, -1.55),
            (0.05, 1.25),
            (-2.20, 7.60),
            (-4.80, 8.00),
        ),
        m: dk(
            (3.75, 2.30),
            -0.06,
            0.05,
            -0.04,
            (6.45, -1.05),
            (0.05, 1.25),
            (4.80, 8.00),
            (2.30, 7.60),
        ),
    },
    // 8 · sissonne: both airborne, legs open, arms lifted
    Key {
        spin: 0.10,
        f: dk(
            (-3.55, 0.90),
            0.05,
            0.05,
            0.0,
            (-6.90, -2.60),
            (0.20, -0.30),
            (-9.30, -0.35),
            (0.60, 5.10),
        ),
        m: dk(
            (3.60, 0.95),
            -0.05,
            -0.05,
            -0.02,
            (7.00, -2.40),
            (0.20, -0.30),
            (3.60, 5.20),
            (8.10, -0.60),
        ),
    },
    // 9 · landing, one foot glide, arms wide
    Key {
        spin: 0.35,
        f: dk(
            (-3.60, 2.70),
            0.06,
            0.04,
            0.05,
            (-7.10, -0.80),
            (0.20, 1.30),
            (-4.55, 8.00),
            (-2.45, 6.10),
        ),
        m: dk(
            (3.60, 2.35),
            -0.05,
            -0.04,
            -0.02,
            (7.30, -0.50),
            (0.20, 1.30),
            (2.85, 8.00),
            (4.40, 8.00),
        ),
    },
    // 10 · Ina Bauer: she arches back, blades spread on the ice
    Key {
        spin: 0.20,
        f: dk(
            (-3.20, 3.20),
            -0.55,
            0.10,
            -0.18,
            (-8.60, 1.20),
            (-1.36, -0.10),
            (-7.55, 8.00),
            (-0.90, 8.00),
        ),
        m: dk(
            (3.30, 2.50),
            -0.10,
            -0.05,
            0.06,
            (5.90, 1.20),
            (-1.36, -0.10),
            (2.50, 8.00),
            (4.20, 8.00),
        ),
    },
    // 11 · joined spin, hands high
    Key {
        spin: 0.75,
        f: dk(
            (-3.30, 2.15),
            0.18,
            0.06,
            0.14,
            (-6.75, -2.15),
            (0.35, -0.60),
            (-3.95, 8.00),
            (-2.65, 7.75),
        ),
        m: dk(
            (3.30, 2.20),
            -0.18,
            -0.06,
            -0.14,
            (6.85, -1.95),
            (0.35, -0.60),
            (2.65, 7.75),
            (3.95, 8.00),
        ),
    },
    // 12 · finale, both arms up in a V
    Key {
        spin: 0.30,
        f: dk(
            (-3.55, 2.10),
            -0.06,
            0.05,
            -0.10,
            (-6.90, -2.40),
            (0.60, -1.60),
            (-4.05, 8.00),
            (-3.05, 7.90),
        ),
        m: dk(
            (3.55, 2.10),
            0.06,
            -0.05,
            -0.08,
            (7.10, -2.30),
            (0.60, -1.60),
            (3.05, 7.90),
            (4.05, 8.00),
        ),
    },
];

const STEP: f32 = LOOP / KEYS.len() as f32;

const FIGURES: [&str; 13] = [
    "GLIDE",
    "PUSH",
    "ARABESQUE",
    "PROMENADE",
    "LIFT",
    "SPIN RISE",
    "DEATH SPIRAL",
    "PUSH",
    "SISSONNE",
    "ONE FOOT",
    "INA BAUER",
    "JOINT SPIN",
    "FINALE",
];

fn cr(p0: f32, p1: f32, p2: f32, p3: f32, s: f32) -> f32 {
    let m1 = (p2 - p0) * 0.5;
    let m2 = (p3 - p1) * 0.5;
    let s2 = s * s;
    let s3 = s2 * s;
    (2.0 * s3 - 3.0 * s2 + 1.0) * p1
        + (s3 - 2.0 * s2 + s) * m1
        + (-2.0 * s3 + 3.0 * s2) * p2
        + (s3 - s2) * m2
}

fn crp(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32), s: f32) -> (f32, f32) {
    (cr(a.0, b.0, c.0, d.0, s), cr(a.1, b.1, c.1, d.1, s))
}

fn blend_dk(a: DKey, b: DKey, c: DKey, d: DKey, s: f32) -> DKey {
    DKey {
        root: crp(a.root, b.root, c.root, d.root, s),
        lean: cr(a.lean, b.lean, c.lean, d.lean, s),
        roll: cr(a.roll, b.roll, c.roll, d.roll, s),
        head: cr(a.head, b.head, c.head, d.head, s),
        out: crp(a.out, b.out, c.out, d.out, s),
        inn: crp(a.inn, b.inn, c.inn, d.inn, s),
        l: crp(a.l, b.l, c.l, d.l, s),
        r: crp(a.r, b.r, c.r, d.r, s),
    }
}

/// Choreography sampled at time `t` (looping, C1 continuous).
fn keys_at(time: f32) -> Key {
    let n = KEYS.len() as i64;
    let u = time.rem_euclid(LOOP) / STEP;
    let i = u.floor() as i64;
    let s = u - i as f32;
    let idx = |j: i64| -> usize { (((j % n) + n) % n) as usize };
    let k0 = KEYS[idx(i - 1)];
    let k1 = KEYS[idx(i)];
    let k2 = KEYS[idx(i + 1)];
    let k3 = KEYS[idx(i + 2)];
    Key {
        spin: cr(k0.spin, k1.spin, k2.spin, k3.spin, s),
        f: blend_dk(k0.f, k1.f, k2.f, k3.f, s),
        m: blend_dk(k0.m, k1.m, k2.m, k3.m, s),
    }
}

/// A solved dancer, in local units.
#[derive(Clone, Copy)]
struct Dancer {
    hip_l: P,
    hip_r: P,
    chest: P,
    neck: P,
    head: P,
    sh_out: P,
    sh_in: P,
    elb_out: P,
    hand_out: P,
    elb_in: P,
    hand_in: P,
    knee_l: P,
    foot_l: P,
    knee_r: P,
    foot_r: P,
    head_r: f32,
    skirt: f32,
    skirt_spin: f32,
}

struct Pose {
    f: Dancer,
    m: Dancer,
    clasp: P,
    held: f32,
    bbox: (P, P),
}

/// Horizontal squeeze applied to the authored stage: keeps the couple close
/// enough to read as one silhouette on a wide terminal.
const XSQUEEZE: f32 = 0.94;

fn solve_dancer(rig: &Rig, d: &DKey, out_side: f32, spin: f32) -> Dancer {
    // Every horizontal offset goes through `sq`, so the whole body is
    // squeezed uniformly and the IK solutions stay valid.
    let sq = |p: P| P::new(p.x * XSQUEEZE, p.y);
    let root = P::new(d.root.0 * XSQUEEZE, d.root.1);
    let up = P::new(d.lean.sin(), -d.lean.cos());
    let axis = P::new(d.lean.cos(), d.lean.sin());
    let chest = root.add(sq(up.mul(rig.spine * rig.chest_k)));
    let neck = root.add(sq(up.mul(rig.spine)));
    let hdir = P::new((d.lean + d.head).sin(), -(d.lean + d.head).cos());
    let head = neck.add(sq(hdir.mul(rig.neck)));

    let sh_l = chest
        .add(sq(axis.mul(-rig.sh_half)))
        .add(P::new(0.0, -d.roll));
    let sh_r = chest
        .add(sq(axis.mul(rig.sh_half)))
        .add(P::new(0.0, d.roll));
    let (sh_out, sh_in) = if out_side < 0.0 {
        (sh_l, sh_r)
    } else {
        (sh_r, sh_l)
    };

    let out_t = P::new(d.out.0 * XSQUEEZE, d.out.1);
    let inn_t = P::new(d.inn.0 * XSQUEEZE, d.inn.1);
    // Elbows fly outwards and slightly up, like a ballroom hold.
    let out_bend = P::new(
        if out_side < 0.0 { -0.8 } else { 0.8 },
        if out_t.y < sh_out.y { -0.9 } else { 0.7 },
    );
    let in_bend = P::new(
        if out_side < 0.0 { -0.3 } else { 0.3 },
        if inn_t.y < sh_in.y { -1.0 } else { 0.6 },
    );
    let (elb_out, hand_out) = limb(sh_out, out_t, rig.up, rig.fore, out_bend);
    let (elb_in, hand_in) = limb(sh_in, inn_t, rig.up, rig.fore, in_bend);

    let hip_l = root
        .add(sq(axis.mul(-rig.hip_half)))
        .add(P::new(0.0, rig.hip_y));
    let hip_r = root
        .add(sq(axis.mul(rig.hip_half)))
        .add(P::new(0.0, rig.hip_y));
    let (knee_l, foot_l) = limb(
        hip_l,
        P::new(d.l.0 * XSQUEEZE, d.l.1),
        rig.thigh,
        rig.shin,
        P::new(-1.0, 0.25),
    );
    let (knee_r, foot_r) = limb(
        hip_r,
        P::new(d.r.0 * XSQUEEZE, d.r.1),
        rig.thigh,
        rig.shin,
        P::new(1.0, 0.25),
    );

    Dancer {
        hip_l,
        hip_r,
        chest,
        neck,
        head,
        sh_out,
        sh_in,
        elb_out,
        hand_out,
        elb_in,
        hand_in,
        knee_l,
        foot_l,
        knee_r,
        foot_r,
        head_r: rig.head_r,
        skirt: rig.skirt * XSQUEEZE,
        skirt_spin: spin,
    }
}

fn smooth01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn smoothstep(x: f32) -> f32 {
    smooth01(x)
}

fn bbox_of(f: &Dancer, m: &Dancer) -> (P, P) {
    let mut min = P::new(f32::MAX, f32::MAX);
    let mut max = P::new(f32::MIN, f32::MIN);
    for d in [f, m] {
        let pad = d.head_r.max(d.skirt) + 0.6;
        for p in [
            d.hip_l, d.hip_r, d.chest, d.neck, d.head, d.sh_out, d.sh_in, d.elb_out, d.hand_out,
            d.elb_in, d.hand_in, d.knee_l, d.foot_l, d.knee_r, d.foot_r,
        ] {
            min = P::new(min.x.min(p.x), min.y.min(p.y));
            max = P::new(max.x.max(p.x), max.y.max(p.y));
        }
        min = P::new(min.x, min.y.min(d.head.y - d.head_r));
        max = P::new(max.x + pad * 0.0, max.y.max(d.head.y + d.head_r));
        max = P::new(max.x, max.y);
    }
    // Skirts widen the silhouette on the female side.
    let skirt = f.skirt;
    min = P::new(min.x.min(f.hip_l.x - skirt), min.y);
    max = P::new(max.x.max(f.hip_r.x + skirt), max.y);
    (min, max)
}

/// Screen placement of the pair at time `t`.
#[derive(Clone, Copy)]
struct Stage {
    cx: f32,
    cy: f32,
    s: f32,
    ang: f32,
}

/// Screen transform shared by every limb in a frame.
#[derive(Clone, Copy)]
struct Xf {
    cx: f32,
    cy: f32,
    s: f32,
    ang: f32,
    unicode: bool,
    dim: f32,
    ghost: bool,
    /// Screen row of the mirror plane used by the ice reflection.
    mirror: f32,
    mirror_k: f32,
}

impl Xf {
    fn pt(&self, p: P) -> (isize, isize) {
        let r = p.rot(self.ang);
        let mut y = self.cy + r.y * self.s;
        if self.mirror != 0.0 {
            y = self.mirror + (self.mirror - y) * self.mirror_k;
        }
        (
            (self.cx + r.x * self.s * 2.0).round() as isize,
            y.round() as isize,
        )
    }

    fn tint(&self, c: Rgb) -> Rgb {
        if self.dim >= 0.999 {
            c
        } else {
            c.scale(self.dim)
        }
    }

    fn line(&self, canvas: &mut Canvas, a: P, b: P, ch: char, col: Rgb, thick: bool) {
        let (x0, y0) = self.pt(a);
        let (x1, y1) = self.pt(b);
        let col = self.tint(col);
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
        if self.ghost {
            return;
        }
        let steps = ((self.s * 18.0) as usize).clamp(8, 24);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let left = a.lerp(d, t);
            let right = b.lerp(c, t);
            let (x0, y0) = self.pt(left);
            let (x1, y1) = self.pt(right);
            canvas.line(x0, y0, x1, y1, self.block(), self.tint(col));
        }
    }

    fn block(&self) -> char {
        if self.unicode {
            '█'
        } else {
            '#'
        }
    }

    /// Sparse glyph used for the soft fringe around limbs.
    fn fringe(&self) -> char {
        if self.unicode {
            '░'
        } else {
            ':'
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

#[derive(Clone, Copy)]
struct Look {
    main: Rgb,
    hl: Rgb,
    side: f32,
    bun: bool,
}

const FEMALE_LOOK: Look = Look {
    main: FEMALE,
    hl: FEMALE_HL,
    side: -1.0,
    bun: true,
};

const MALE_LOOK: Look = Look {
    main: MALE,
    hl: MALE_HL,
    side: 1.0,
    bun: false,
};

pub struct BalletScene {
    seed: u64,
    w: usize,
    h: usize,
}

impl BalletScene {
    pub fn new(seed: u64) -> Self {
        BalletScene { seed, w: 0, h: 0 }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
    }

    fn base_scale(&self) -> f32 {
        let by_h = self.h as f32 / 26.0;
        let by_w = self.w as f32 / 84.0;
        (by_h * by_w.clamp(0.82, 1.12)).clamp(0.55, 1.45)
    }

    /// The pair's glide path: a rink-sized track with long lobes and a slow
    /// depth change, so the couple sweeps the whole rink instead of a corner.
    fn path(&self, t: f32) -> (f32, f32, f32) {
        let w = self.w as f32;
        let h = self.h as f32;
        // The pair sweeps the whole rink: two lobes across, one in depth, at
        // twice the original travelling speed.
        let u = t * 0.28;
        let px = 0.5 + 0.300 * u.sin() + 0.060 * (2.0 * u + 1.1).sin();
        let py = 0.500 + 0.155 * (2.0 * u + 2.4).sin() + 0.035 * (1.7 * u).sin();
        let base = self.base_scale();
        // Nearer the camera (lower on screen) reads bigger.
        let depth = 0.80 + 0.46 * smooth01(py * 1.3 - 0.30);
        (
            (px * w).clamp(w * 0.13, w * 0.87),
            (py * h).clamp(h * 0.29, h * 0.78),
            base * depth,
        )
    }

    fn stage_at(&self, t: f32, spin: f32) -> Stage {
        let (cx, cy, s) = self.path(t);
        // Lean into the turn: curvature of the glide path drives the roll.
        let a = self.path(t - 0.28);
        let b = self.path(t);
        let c = self.path(t + 0.28);
        let v1 = P::new(b.0 - a.0, b.1 - a.1);
        let v2 = P::new(c.0 - b.0, c.1 - b.1);
        let cross = v1.x * v2.y - v1.y * v2.x;
        let scale = (v1.len() * v2.len()).max(1.0);
        let turn = (cross / scale).clamp(-1.0, 1.0);
        let ang = turn * 0.16 + 0.02 * (t * 0.7).sin() + spin * 0.05 * (t * 2.6).sin();
        Stage { cx, cy, s, ang }
    }

    fn fit(&self, st: Stage, pose: &Pose) -> Stage {
        let w = self.w as f32;
        let h = self.h as f32;
        let (lo, hi) = pose.bbox;
        let mut s = st.s;
        let span_x = (hi.x - lo.x).max(0.5) * 2.2;
        if span_x * s > w - 3.0 {
            s = (w - 3.0) / span_x;
        }
        let span_y = (hi.y - lo.y).max(0.5);
        if span_y * s > h - 3.0 {
            s = (h - 3.0) / span_y;
        }
        s = s.max(0.30);
        let left = (-lo.x).max(0.0) * 2.0 * s + 1.5;
        let right = hi.x.max(0.0) * 2.0 * s + 1.5;
        let cx = st.cx.clamp(left, (w - right).max(left));
        let top = (h * 0.26).max(4.0) + 1.0;
        let mut up = (-lo.y).max(0.0) * s;
        let down = hi.y.max(0.0) * s + 1.0;
        // A tall pose (the lift) is shrunk rather than pushed into the crowd.
        if top + up + down > h - 1.0 {
            s *= ((h - 1.0 - top) / (up + down).max(1.0)).clamp(0.55, 1.0);
            up *= s;
        }
        let cy = st.cy.clamp(top + up, (h - down).max(top + up));
        Stage {
            cx,
            cy,
            s,
            ang: st.ang,
        }
    }

    /// Secondary motion: breathing, skating footwork, arm float, head delay.
    fn secondary(&self, k: &mut Key, t: f32) {
        let breathe = (t * 1.9).sin();
        let step = t * 2.6;
        for (d, phase) in [(&mut k.f, 0.0f32), (&mut k.m, 1.9f32)] {
            d.root.1 += 0.09 * (t * 1.8 + phase).sin();
            d.lean += 0.035 * (t * 1.45 + phase * 0.7).sin();
            d.head += 0.10 * (t * 2.4 + phase).sin();
            d.roll += 0.04 * breathe;

            // Port de bras: the free arm traces a slow circle.
            let wob = t * 1.6 + phase;
            d.out.0 += 0.32 * wob.sin();
            d.out.1 += 0.42 * (wob * 1.35).cos();
            d.inn.0 += 0.07 * (wob * 0.8).sin();
            d.inn.1 += 0.09 * (wob * 0.9).cos();

            // Skate footwork: each blade leaves and returns to the ice.
            let low = d.root.1 > 1.6;
            if low {
                let l_lift = smooth01((step + phase).sin()) * 0.85;
                let r_lift = smooth01((step + phase + TAU * 0.5).sin()) * 0.85;
                if d.l.1 > ICE_Y - 0.55 {
                    d.l.1 -= l_lift;
                    d.l.0 += 0.30 * (step + phase).cos();
                }
                if d.r.1 > ICE_Y - 0.55 {
                    d.r.1 -= r_lift;
                    d.r.0 += 0.30 * (step + phase + TAU * 0.5).cos();
                }
            }
        }

        // Keep the shared hands welded whenever they are close.
        let df = P::new(k.f.inn.0, k.f.inn.1);
        let dm = P::new(k.m.inn.0, k.m.inn.1);
        if df.dist(dm) < 0.9 {
            let mid = df.lerp(dm, 0.5);
            let wob = 0.05 * (t * 2.2).sin();
            k.f.inn = (mid.x, mid.y + wob);
            k.m.inn = (mid.x, mid.y + wob);
        }
    }

    fn pose_from(&self, k: &Key, t: f32) -> Pose {
        let mut kk = *k;
        self.secondary(&mut kk, t);
        // Limbs trail the torso: sample the choreography slightly in the past
        // for the four hand / foot targets only.
        let mut lag = keys_at(t - LIMB_LAG);
        self.secondary(&mut lag, t);
        kk.f.out = lag.f.out;
        kk.f.inn = lag.f.inn;
        kk.f.l = lag.f.l;
        kk.f.r = lag.f.r;
        kk.m.out = lag.m.out;
        kk.m.inn = lag.m.inn;
        kk.m.l = lag.m.l;
        kk.m.r = lag.m.r;
        let f = solve_dancer(&FEMALE_RIG, &kk.f, -1.0, kk.spin);
        let m = solve_dancer(&MALE_RIG, &kk.m, 1.0, kk.spin);
        let clasp = f.hand_in.lerp(m.hand_in, 0.5);
        let gap = f.hand_in.dist(m.hand_in);
        Pose {
            f,
            m,
            clasp,
            held: 1.0 - (gap / 1.1).clamp(0.0, 1.0),
            bbox: bbox_of(&f, &m),
        }
    }

    fn frame_at(&self, t: f32) -> (Stage, Pose) {
        let k = keys_at(t);
        let pose = self.pose_from(&k, t);
        let st = self.fit(self.stage_at(t, k.spin), &pose);
        (st, pose)
    }

    // ---- rink -------------------------------------------------------------

    fn draw_rink(&self, canvas: &mut Canvas, t: f32, unicode: bool, cx: f32, ice_y: f32) {
        let w = self.w as f32;
        let h = self.h as f32;
        let horizon = (h * 0.26).max(4.0);
        let wall = horizon as isize;
        let ice_rows = (self.h as f32 - horizon).max(1.0);

        // Backdrop: the far end of the arena, then the ice sheet.  One smooth
        // cold gradient with a little grain keeps the whole frame alive.
        for y in 0..self.h as isize {
            let v = y as f32 / h;
            let mut col = WALL_TOP.lerp(RINK_MID, smoothstep(v * 2.2));
            col = col.lerp(RINK_NEAR, smoothstep((v - 0.62) / 0.38) * 0.55);
            for x in 0..self.w as isize {
                let gu = (x as f32 + 0.5) / w;
                let grain = value_noise(gu * 4.5 + t * 0.01, v * 6.0, self.seed);
                let dim = col.scale(0.52 + grain * 0.48);
                let mut c = dim.scale(0.85);
                if (y as f32) > horizon {
                    let depth = ((y as f32 - horizon) / ice_rows).clamp(0.0, 1.0);
                    let sheen = ((x as f32 * 0.10 + y as f32 * 0.42 + t * 0.16).sin() * 0.5 + 0.5)
                        * (1.0 - depth * 0.5);
                    c = c.scale(0.86 + sheen * 0.30);
                }
                canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(0.78));
            }
        }

        self.draw_audience(canvas, t, unicode, horizon);

        // Ceiling lights.
        let lamps = (self.w / 6).clamp(4, 22) as i64;
        for i in 0..lamps {
            let hx = hash01(i, 3, self.seed);
            let lx = (hx * w).round() as isize;
            let ly = (hash01(i, 11, self.seed) * 1.6) as isize;
            let tw = 0.55 + 0.45 * (t * 0.9 + hx * 12.0).sin();
            let col = if hx > 0.5 { ARENA_WARM } else { ARENA_COOL };
            canvas.set(
                lx,
                ly,
                if tw > 0.75 {
                    if unicode {
                        '✦'
                    } else {
                        '*'
                    }
                } else if unicode {
                    '·'
                } else {
                    '.'
                },
                col.scale(0.30 + tw * 0.50),
            );
            canvas.add_light(lx, ly, col, 0.22 * tw, ' ');
        }

        // Far boards: a bright rail with a kick plate under it.
        canvas.hline(0, self.w as isize - 1, wall, '-', ICE_GLOW.scale(0.50));
        for x in 0..self.w as isize {
            canvas.add_light(x, wall + 1, ICE_GLOW, 0.14, ' ');
            canvas.add_light(x, wall - 1, ICE_GLOW, 0.06, ' ');
        }

        // Rink boards down both sides, matching the old stage curtains.
        let board_w = (w * 0.055).clamp(2.0, 9.0);
        for y in wall + 1..self.h as isize {
            let v = y as f32 / h;
            for x in 0..self.w as isize {
                let xf = x as f32;
                let left = xf < board_w;
                let right = xf >= w - board_w;
                if !left && !right {
                    continue;
                }
                let edge = if left {
                    1.0 - xf / board_w
                } else {
                    (xf - (w - board_w)) / board_w
                };
                let fold = (xf * 1.7 + (v * 11.0).sin() * 1.2 + t * 0.10).sin() * 0.5 + 0.5;
                let col = BOARD
                    .lerp(BOARD_HL, fold * 0.50 + edge * 0.20)
                    .scale(0.50 + edge * 0.45);
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

        // Reflections of the ceiling rig, running down the ice.
        for (k, base) in [0.16f32, 0.38, 0.62, 0.84].iter().enumerate() {
            let cx = w * base + (t * 0.2 + k as f32).sin() * 1.4;
            let cold = k % 2 == 0;
            let col = if cold { SPOT_COOL } else { SPOT_WARM };
            for y in wall + 1..self.h as isize {
                let v = (y as f32 - horizon) / ice_rows;
                let half = 1.2 + v * w * 0.030;
                let fade = (1.0 - v).max(0.0) * (0.10 + v * 0.24);
                let x0 = (cx - half) as isize;
                let x1 = (cx + half) as isize;
                for x in x0.max(0)..=x1.min(self.w as isize - 1) {
                    let d = ((x as f32 - cx) / half).abs();
                    if d <= 1.0 {
                        canvas.add_light(x, y, col, (1.0 - d * d) * fade, ' ');
                    }
                }
            }
        }

        // Warm pool of light on the ice under the couple.
        let r = (h * 0.14) as isize;
        let px = cx.round() as isize;
        let py = ice_y.round() as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                let d = ((dx as f32 * 0.45).powi(2) + (dy as f32 * 0.55).powi(2)).sqrt();
                if d < r as f32 {
                    let a = (1.0 - d / r as f32).powi(2) * 0.11;
                    canvas.add_light(px + dx, py + dy, SPOT_WARM, a, ' ');
                }
            }
        }

        // Two follow spots.
        for (k, col) in [(0usize, SPOT_WARM), (1, SPOT_COOL)].iter() {
            let sway = (t * 0.23 + *k as f32 * 2.4).sin() * w * 0.06;
            let top_x = w * (0.30 + 0.40 * *k as f32) + sway;
            for y in 0..(h * 0.95) as isize {
                let v = y as f32 / h;
                let half = 1.4 + v * w * 0.13;
                let x0 = (top_x - half) as isize;
                let x1 = (top_x + half) as isize;
                let fade = (1.0 - v * 0.9).max(0.0);
                for x in x0.max(0)..=x1.min(self.w as isize - 1) {
                    let d = (x as f32 - top_x).abs() / half.max(1.0);
                    if d <= 1.0 {
                        canvas.add_light(x, y, *col, (1.0 - d * d) * fade * 0.09, ' ');
                    }
                }
            }
        }
    }

    /// The crowd: three tiers of spectators who sway, wave and occasionally
    /// fire a camera flash when the couple hits a big move.
    fn draw_audience(&self, canvas: &mut Canvas, t: f32, unicode: bool, horizon: f32) {
        let wall = horizon.round() as isize;
        if wall < 6 {
            return;
        }
        let w = self.w as f32;
        let rows = [2isize, 2 + (wall - 4) / 2, wall - 2];
        let hot = matches!(
            self.figure(t),
            "LIFT" | "SISSONNE" | "JOINT SPIN" | "SPIN RISE"
        );
        let flash_p = if hot { 0.975 } else { 0.995 };
        for (tier, &py) in rows.iter().enumerate() {
            let far = tier == 0;
            let step = if far { 2 } else { 3 };
            let (base, bright) = match tier {
                0 => (AUD_FAR, 0.55),
                1 => (AUD_MID, 0.72),
                _ => (AUD_NEAR, 0.95),
            };
            let n = (w / step as f32).ceil() as i64;
            for i in 0..n {
                let hx = hash01(i, tier as i64 * 11 + 3, self.seed);
                // A few empty seats keep the rows from looking like wallpaper.
                if hash01(i, tier as i64 * 5 + 41, self.seed) > 0.88 {
                    continue;
                }
                let phase = hx * TAU;
                let x0 = (i as f32 + 0.5) * step as f32;
                let sway = (t * 0.85 + phase * 1.9).sin();
                let wave = (x0 * 0.13 - t * 1.6).sin();
                let mut lift = (t * 1.25 + phase * 2.1).sin() * 0.45 + wave * 0.55;
                if hot {
                    lift += 0.18;
                }
                let cheer = lift > 0.42;
                let px = (x0 + sway * 0.8).round() as isize;
                if px < 0 || px >= self.w as isize {
                    continue;
                }
                if hash01(i, (t * 3.0) as i64, self.seed) > flash_p {
                    canvas.set(px, py, '*', WHITE);
                    canvas.add_light(px, py, WHITE, 0.55, ' ');
                    continue;
                }
                let col = base.scale(bright * (0.84 + 0.16 * sway));
                if far {
                    canvas.set(px, py, if unicode { '·' } else { '.' }, col.scale(0.75));
                } else if tier == 1 {
                    canvas.set(
                        px,
                        py,
                        if cheer {
                            'Y'
                        } else if unicode {
                            '•'
                        } else {
                            'o'
                        },
                        if cheer { ARENA_WARM.scale(0.85) } else { col },
                    );
                } else {
                    let body = py + 1;
                    if cheer {
                        canvas.set(px - 1, py, '\\', col);
                        canvas.set(px + 1, py, '/', col);
                        canvas.set(
                            px,
                            py - 1,
                            if unicode { '●' } else { 'o' },
                            ARENA_WARM.scale(0.95),
                        );
                        if body < wall {
                            canvas.set(px, body, if unicode { '╵' } else { '|' }, col);
                        }
                    } else {
                        canvas.set(px, py, if unicode { '●' } else { 'o' }, col);
                        if body < wall {
                            canvas.set(px, body, if unicode { '╵' } else { '|' }, col.scale(0.8));
                        }
                    }
                }
            }
        }
    }

    /// Blade tracings    /// Blade tracings: the arcs the pair has just carved into the ice.
    fn draw_tracings(&self, canvas: &mut Canvas, t: f32, unicode: bool) {
        let n = 16usize;
        let mut left: Vec<(isize, isize)> = Vec::with_capacity(n);
        let mut right: Vec<(isize, isize)> = Vec::with_capacity(n);
        for k in (1..=n).rev() {
            let tt = t - k as f32 * (1.0 / 13.0);
            let st = self.stage_at(tt, 0.0);
            let xf = Xf {
                cx: st.cx,
                cy: st.cy,
                s: st.s,
                ang: st.ang,
                unicode,
                dim: 1.0,
                ghost: false,
                mirror: 0.0,
                mirror_k: 0.0,
            };
            let gap = 2.3;
            left.push(xf.pt(P::new(-gap, ICE_Y)));
            right.push(xf.pt(P::new(gap, ICE_Y)));
        }
        for (i, arc) in [&left, &right].into_iter().enumerate() {
            for k in 1..arc.len() {
                let age = 1.0 - k as f32 / arc.len() as f32;
                let col = ICE_LINE
                    .lerp(ICE_GLOW, 0.4)
                    .scale(0.14 + age * 0.40 + i as f32 * 0.02);
                let (x0, y0) = arc[k - 1];
                let (x1, y1) = arc[k];
                let dx = x1 - x0;
                let dy = y1 - y0;
                let ch = if unicode {
                    if dx.abs() <= 1 && dy.abs() > 0 {
                        '│'
                    } else if (dx > 0) == (dy > 0) {
                        '╲'
                    } else {
                        '╱'
                    }
                } else if dx.abs() <= 1 && dy.abs() > 0 {
                    '|'
                } else if (dx > 0) == (dy > 0) {
                    '\\'
                } else {
                    '/'
                };
                canvas.line(x0, y0, x1, y1, ch, col);
            }
            for k in (0..arc.len()).step_by(5) {
                canvas.add_light(arc[k].0, arc[k].1, ICE_GLOW, 0.14, ' ');
            }
        }
    }

    /// Soft pool of light and a vertical smear where each skater touches the ice.
    fn draw_ice_glow(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose) {
        for (d, col) in [(&pose.f, FEMALE_HL), (&pose.m, MALE_HL)] {
            for foot in [d.foot_l, d.foot_r] {
                if foot.y < ICE_Y - 0.9 {
                    continue;
                }
                let (fx, fy) = xf.pt(P::new(foot.x, ICE_Y));
                let rf = (3.4 * xf.s).max(2.0);
                let r = rf as isize;
                for dy in 0..=(rf * 0.6) as isize {
                    for dx in -r..=r {
                        let dd = ((dx as f32 / r as f32).powi(2)
                            + (dy as f32 / (r as f32 * 0.6)).powi(2))
                        .sqrt();
                        if dd <= 1.0 {
                            canvas.add_light(fx + dx, fy + dy, col, (1.0 - dd) * 0.16, ' ');
                        }
                    }
                }
                // Blade mark.
                canvas.hline(
                    fx - 2,
                    fx + 2,
                    fy,
                    if xf.unicode { '─' } else { '-' },
                    ICE_LINE.scale(0.5),
                );
            }
        }
    }

    fn draw_reflection(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose, _t: f32) {
        // A full mirrored body reads as a second dancer in ASCII, so the ice
        // only keeps a soft, squashed smear of colour under each skater.
        for (d, col, dark) in [(&pose.f, FEMALE, FEMALE_DARK), (&pose.m, MALE, MALE_DARK)] {
            for foot in [d.foot_l, d.foot_r] {
                let (fx, fy) = xf.pt(P::new(foot.x, ICE_Y));
                for dy in 1..=3isize {
                    let a = 0.26 - dy as f32 * 0.07;
                    for dx in -2..=2isize {
                        let k = 1.0 - (dx.abs() as f32) * 0.28 - dy as f32 * 0.16;
                        if k <= 0.0 {
                            continue;
                        }
                        canvas.add_light(fx + dx, fy + dy, col, a * k, ' ');
                    }
                }
            }
            // Faint silhouette hint under the hips.
            let hip = d.hip_l.lerp(d.hip_r, 0.5);
            let (hx, hy) = xf.pt(P::new(hip.x, ICE_Y));
            for dy in 1..=2isize {
                for dx in -1..=1isize {
                    canvas.add_light(hx + dx, hy + dy, dark, 0.10 - dy as f32 * 0.03, ' ');
                }
            }
        }
    }

    // ---- dancers ----------------------------------------------------------

    /// A limb drawn as a bright core with a dim, sparse fringe: the fringe is
    /// what makes the silhouette read as soft instead of as a hard stick.
    fn soft_limb(&self, canvas: &mut Canvas, xf: &Xf, a: P, b: P, col: Rgb) {
        let (x0, y0) = xf.pt(a);
        let (x1, y1) = xf.pt(b);
        let dx = (x1 - x0) as f32;
        let dy = (y1 - y0) as f32;
        let len = (dx * dx + dy * dy).sqrt().max(1.0);
        let ox = (-dy / len).round() as isize;
        let oy = (dx / len).round() as isize;
        if ox != 0 || oy != 0 {
            for k in [-1isize, 1] {
                canvas.line(
                    x0 + ox * k,
                    y0 + oy * k,
                    x1 + ox * k,
                    y1 + oy * k,
                    xf.fringe(),
                    xf.tint(col.scale(0.40)),
                );
            }
        }
        xf.line(canvas, a, b, xf.shade(), col, true);
    }

    fn draw_dancer(&self, canvas: &mut Canvas, xf: &Xf, d: &Dancer, look: &Look, t: f32) {
        // Legs, far leg first.
        self.soft_limb(canvas, xf, d.hip_l, d.knee_l, look.main.scale(0.72));
        self.soft_limb(canvas, xf, d.knee_l, d.foot_l, look.main);
        self.soft_limb(canvas, xf, d.hip_r, d.knee_r, look.main.scale(0.78));
        self.soft_limb(canvas, xf, d.knee_r, d.foot_r, look.main);

        // Skates.
        for foot in [d.foot_l, d.foot_r] {
            let (fx, fy) = xf.pt(foot);
            canvas.hline(
                fx - 2,
                fx + 1,
                fy,
                if xf.unicode { '═' } else { '=' },
                xf.tint(look.hl.scale(0.9)),
            );
            canvas.set(
                fx + 2,
                fy,
                if xf.unicode { '▸' } else { '>' },
                xf.tint(look.hl),
            );
            canvas.add_light(fx, fy, ICE_GLOW, 0.20, ' ');
        }

        // Bodice / tunic, with a fringe along the shoulders.
        xf.quad(
            canvas,
            d.sh_out,
            d.sh_in,
            d.hip_r.add(P::new(0.0, 0.20)),
            d.hip_l.add(P::new(0.0, 0.20)),
            look.main.scale(0.74),
        );
        xf.line(
            canvas,
            d.sh_out.lerp(d.sh_in, 0.5),
            d.hip_l.lerp(d.hip_r, 0.5),
            if xf.unicode { '│' } else { '|' },
            look.main.scale(0.82),
            false,
        );
        let waist = d.hip_l.lerp(d.hip_r, 0.5);
        xf.line(
            canvas,
            waist.add(P::new(-1.10, -0.05)),
            waist.add(P::new(1.10, -0.05)),
            if xf.unicode { '═' } else { '=' },
            look.hl.scale(0.85),
            false,
        );

        if look.bun && !xf.ghost {
            self.draw_skirt(canvas, xf, d, t);
        }

        // Arms: thin, no fringe (they would blur together).
        for (a, b) in [(d.sh_out, d.elb_out), (d.sh_in, d.elb_in)] {
            xf.line(canvas, a, b, xf.shade(), SKIN, false);
        }
        for (a, b) in [(d.elb_out, d.hand_out), (d.elb_in, d.hand_in)] {
            xf.line(canvas, a, b, xf.shade(), SKIN_HL, false);
        }
        for hand in [d.hand_out, d.hand_in] {
            let (hx, hy) = xf.pt(hand);
            canvas.set(hx, hy, if xf.unicode { '✦' } else { '*' }, xf.tint(GOLD));
            canvas.add_light(hx, hy, look.hl, 0.30, ' ');
        }

        // Neck and head.
        xf.line(canvas, d.neck, d.head, xf.shade(), SKIN, false);
        self.head(canvas, xf, d, look);

        // Ribbon only when the free arm is lifted.
        if look.bun && d.hand_out.y < d.neck.y + 0.4 {
            self.draw_ribbon(canvas, xf, d, t);
        }
    }

    /// Tutu: a gauze ellipse plus radiating pleats, with a soft rim.
    fn draw_skirt(&self, canvas: &mut Canvas, xf: &Xf, d: &Dancer, t: f32) {
        let hip = d.hip_l.lerp(d.hip_r, 0.5);
        let flare = 1.0 + d.skirt_spin * 0.14 + 0.05 * (t * 3.0).sin();
        for k in 0..56 {
            let a = k as f32 / 56.0 * TAU;
            let r = d.skirt * flare * (0.90 + 0.10 * (a * 5.0 + t * 2.2).sin());
            let p = hip.add(P::new(a.cos() * r, a.sin() * r * 0.30 + 0.20));
            let (x, y) = xf.pt(p);
            let rim = (a.sin() * 0.5 + 0.5).abs();
            canvas.set(
                x,
                y,
                if xf.unicode { '░' } else { '.' },
                xf.tint(FEMALE_HL.scale(0.42 + 0.30 * rim)),
            );
        }
        for k in 0..6 {
            let a = -0.35 + k as f32 / 5.0 * (TAU - 0.6);
            let p = hip.add(P::new(a.cos() * d.skirt * 0.93, a.sin() * 0.90 + 0.2));
            xf.line(
                canvas,
                hip.add(P::new(0.0, 0.10)),
                p,
                if xf.unicode { '╱' } else { '/' },
                FEMALE_HL.scale(0.46),
                false,
            );
        }
    }

    fn draw_ribbon(&self, canvas: &mut Canvas, xf: &Xf, d: &Dancer, t: f32) {
        let dir = d.hand_out.sub(d.elb_out).unit();
        let mut p = d.hand_out;
        for k in 0..16 {
            let wob = (t * 1.7 + k as f32 * 0.45).sin();
            p = p
                .add(dir.mul(0.32))
                .add(dir.perp().mul(wob * 0.24))
                .add(P::new(0.0, 0.12));
            let (x, y) = xf.pt(p);
            let fade = 1.0 - k as f32 / 17.0;
            let col = FEMALE.lerp(GOLD, k as f32 / 17.0).scale(0.20 + fade * 0.50);
            canvas.set(x, y, if xf.unicode { '≈' } else { '~' }, xf.tint(col));
        }
    }

    fn head(&self, canvas: &mut Canvas, xf: &Xf, d: &Dancer, look: &Look) {
        let (cx, cy) = xf.pt(d.head);
        let r = (d.head_r * xf.s).max(1.0);
        let ri = r.ceil() as isize;
        let rx = (r * 2.0).ceil() as isize;
        for dy in -ri..=ri {
            for dx in -rx..=rx {
                let vx = dx as f32 * 0.5;
                let vy = dy as f32;
                if vx * vx + vy * vy <= r * r {
                    let hair = if look.bun {
                        vy < -r * 0.30 || (vx < -r * 0.35 && vy < r * 0.55)
                    } else {
                        vy < -r * 0.15
                    };
                    let col = if hair {
                        HAIR.lerp(HAIR_HL, 0.10)
                    } else {
                        SKIN.lerp(SKIN_HL, 0.25 + 0.15 * (-vy / r).max(0.0))
                    };
                    canvas.set(cx + dx, cy + dy, xf.block(), xf.tint(col));
                }
            }
        }
        // Face, turned toward the partner.
        let look_dir = -look.side as isize;
        canvas.set(
            cx + look_dir,
            cy - 1,
            if xf.unicode { '·' } else { 'o' },
            xf.tint(HAIR),
        );
        canvas.set(
            cx + look_dir,
            cy,
            if xf.unicode { '·' } else { 'o' },
            xf.tint(HAIR),
        );
        canvas.set(
            cx + look_dir,
            cy + 2,
            if xf.unicode { '▁' } else { '_' },
            xf.tint(SKIN_DARK),
        );
        if look.bun {
            let bun = (cx - 2, cy - 1);
            canvas.set(bun.0, bun.1, xf.shade(), xf.tint(HAIR));
            canvas.set(bun.0, bun.1 + 1, xf.shade(), xf.tint(HAIR.scale(0.85)));
            canvas.set(
                cx,
                cy - ri - 1,
                if xf.unicode { '✦' } else { '*' },
                xf.tint(GOLD),
            );
            canvas.add_light(cx, cy - ri - 1, GOLD, 0.45, ' ');
        } else {
            let top = cy - ri;
            for dx in -1..=1 {
                canvas.set(cx + dx, top, xf.shade(), xf.tint(HAIR_HL.scale(0.6)));
            }
        }
    }

    fn draw_clasp(&self, canvas: &mut Canvas, xf: &Xf, pose: &Pose, t: f32) {
        // During the lift his hand is on her waist, not in hers.
        if pose.held < 0.15 {
            return;
        }
        let (x, y) = xf.pt(pose.clasp);
        let held = pose.held;
        let pulse = (t * 3.0).sin() * 0.5 + 0.5;
        let main = WHITE.scale(0.55 + held * 0.45);
        canvas.set(x, y, if xf.unicode { '✦' } else { '*' }, main);
        let arm = GOLD.scale(0.45 + held * 0.45);
        for (dx, dy, ch) in [
            (-1, 0, if xf.unicode { '─' } else { '-' }),
            (1, 0, if xf.unicode { '─' } else { '-' }),
            (0, -1, if xf.unicode { '│' } else { '|' }),
            (0, 1, if xf.unicode { '│' } else { '|' }),
        ] {
            canvas.set(x + dx, y + dy, ch, arm);
        }
        for r in 1..=3 {
            let rr = r as f32;
            let a = (1.0 - rr / 4.0) * (0.25 + pulse * 0.30) * (0.25 + held * 0.75);
            for k in 0..12 {
                let ang = k as f32 / 12.0 * TAU + t * 0.8;
                let px = x as f32 + ang.cos() * rr * 2.2;
                let py = y as f32 + ang.sin() * rr;
                canvas.add_light(px.round() as isize, py.round() as isize, GOLD, a, ' ');
            }
        }
        canvas.add_light(x, y, WHITE, 0.30 + held * 0.55, ' ');
    }

    fn draw_sparkles(&self, canvas: &mut Canvas, t: f32, unicode: bool) {
        let n = ((self.w * self.h) / 340).clamp(8, 32);
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
            canvas.add_light(x as isize, y as isize, col, 0.18 + twinkle * 0.36, ch);
        }
    }

    fn figure(&self, t: f32) -> &'static str {
        let i = (t.rem_euclid(LOOP) / STEP).floor() as usize;
        FIGURES[i % FIGURES.len()]
    }
}

impl Scene for BalletScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(format!(
            "TEMPO {:.2}x | {}",
            f.speed * TEMPO,
            self.figure(f.t as f32 * TEMPO)
        ))
    }

    fn update(&mut self, _f: &Frame) {
        // All motion is analytic; nothing to integrate.
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32 * TEMPO;
        let (stage, pose) = self.frame_at(t);
        let xf = Xf {
            cx: stage.cx,
            cy: stage.cy,
            s: stage.s,
            ang: stage.ang,
            unicode,
            dim: 1.0,
            ghost: false,
            mirror: 0.0,
            mirror_k: 0.0,
        };

        self.draw_rink(canvas, t, unicode, xf.cx, xf.cy + ICE_Y * xf.s);
        self.draw_tracings(canvas, t, unicode);
        self.draw_ice_glow(canvas, &xf, &pose);
        self.draw_reflection(canvas, &xf, &pose, t);

        // The man skates behind, the woman in front, then their hands.
        self.draw_dancer(canvas, &xf, &pose.m, &MALE_LOOK, t);
        self.draw_dancer(canvas, &xf, &pose.f, &FEMALE_LOOK, t);
        self.draw_clasp(canvas, &xf, &pose, t);
        self.draw_sparkles(canvas, t, unicode);

        // Ice spray kicked up by the blades.
        for (d, k) in [(pose.f, 0), (pose.m, 1)] {
            for foot in [d.foot_l, d.foot_r] {
                if foot.y < ICE_Y - 0.8 {
                    continue;
                }
                let (fx, fy) = xf.pt(P::new(foot.x, ICE_Y));
                for j in 0..3 {
                    let n = hash01(j + k * 3, (t * 6.0) as i64, self.seed);
                    let sx = fx + (n * 5.0 - 2.5).round() as isize;
                    let sy = fy - (1.0 + n * 2.0) as isize;
                    canvas.add_light(sx, sy, ICE_LINE, 0.30 * (1.0 - n), ' ');
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_stays_on_screen() {
        let mut s = BalletScene::new(7);
        s.resize(80, 24);
        for i in 0..600 {
            let t = i as f32 * 0.11;
            let (stage, pose) = s.frame_at(t);
            let (lo, hi) = pose.bbox;
            assert!(
                lo.x * 2.0 * stage.s + stage.cx > -1.0,
                "left {lo:?} cx {} s {}",
                stage.cx,
                stage.s
            );
            assert!(hi.x * 2.0 * stage.s + stage.cx < 81.0, "right {hi:?}");
            assert!(lo.y * stage.s + stage.cy > -1.0, "top {lo:?}");
            assert!(hi.y * stage.s + stage.cy < 25.0, "bottom {hi:?}");
        }
    }

    #[test]
    fn hands_reach_their_targets() {
        // Every authored key must be anatomically reachable, otherwise the IK
        // clamps and the hands stop matching the choreography.
        for (i, k) in KEYS.iter().enumerate() {
            for (which, dk, rig, side) in [
                ("female", &k.f, &FEMALE_RIG, -1.0),
                ("male", &k.m, &MALE_RIG, 1.0),
            ] {
                let d = solve_dancer(rig, dk, side, k.spin);
                let reach = rig.up + rig.fore;
                for (name, target, hand) in [
                    ("out", P::new(dk.out.0 * XSQUEEZE, dk.out.1), d.hand_out),
                    ("in", P::new(dk.inn.0 * XSQUEEZE, dk.inn.1), d.hand_in),
                ] {
                    assert!(
                        hand.dist(target) < 0.12,
                        "key {i} {which} {name} unreachable: target {target:?} hand {hand:?} (reach {reach})"
                    );
                }
                let leg = rig.thigh + rig.shin;
                for (name, target, foot) in [
                    ("l", P::new(dk.l.0 * XSQUEEZE, dk.l.1), d.foot_l),
                    ("r", P::new(dk.r.0 * XSQUEEZE, dk.r.1), d.foot_r),
                ] {
                    assert!(
                        foot.dist(target) < 0.12,
                        "key {i} {which} leg {name} unreachable: target {target:?} foot {foot:?} (leg {leg})"
                    );
                }
            }
        }
    }

    #[test]
    fn both_dancers_move() {
        let s = BalletScene::new(7);
        let mut f_min = (f32::MAX, f32::MAX);
        let mut f_max = (f32::MIN, f32::MIN);
        let mut m_min = (f32::MAX, f32::MAX);
        let mut m_max = (f32::MIN, f32::MIN);
        let mut m_hand_lo = f32::MAX;
        let mut m_hand_hi = f32::MIN;
        let mut m_foot_lo = f32::MAX;
        let mut m_foot_hi = f32::MIN;
        for i in 0..720 {
            let t = i as f32 * (LOOP / 720.0);
            let pose = s.frame_at(t).1;
            for (d, lo, hi) in [
                (&pose.f, &mut f_min, &mut f_max),
                (&pose.m, &mut m_min, &mut m_max),
            ] {
                for p in [d.hand_out, d.hand_in, d.foot_l, d.foot_r, d.head] {
                    lo.0 = lo.0.min(p.x);
                    lo.1 = lo.1.min(p.y);
                    hi.0 = hi.0.max(p.x);
                    hi.1 = hi.1.max(p.y);
                }
            }
            m_hand_lo = m_hand_lo.min(pose.m.hand_out.y);
            m_hand_hi = m_hand_hi.max(pose.m.hand_out.y);
            m_foot_lo = m_foot_lo.min(pose.m.foot_l.y);
            m_foot_hi = m_foot_hi.max(pose.m.foot_l.y);
        }
        let f_span = ((f_max.0 - f_min.0).powi(2) + (f_max.1 - f_min.1).powi(2)).sqrt();
        let m_span = ((m_max.0 - m_min.0).powi(2) + (m_max.1 - m_min.1).powi(2)).sqrt();
        assert!(f_span > 4.0, "female barely moves: {f_span}");
        assert!(m_span > 4.0, "male barely moves: {m_span}");
        assert!(m_hand_hi - m_hand_lo > 4.0, "male arm is static");
        assert!(m_foot_hi - m_foot_lo > 3.0, "male legs are static");
    }

    #[test]
    fn motion_is_smooth() {
        let s = BalletScene::new(7);
        let mut prev = s.frame_at(0.0).1;
        // Sample at the real playback step so the test sees the motion the
        // viewer actually sees.
        for i in 1..900 {
            let t = i as f32 * (1.0 / 30.0) * TEMPO;
            let cur = s.frame_at(t).1;
            for (a, b) in [
                (prev.f.hand_out, cur.f.hand_out),
                (prev.f.foot_l, cur.f.foot_l),
                (prev.m.hand_in, cur.m.hand_in),
                (prev.m.foot_r, cur.m.foot_r),
                (prev.m.head, cur.m.head),
            ] {
                assert!(a.dist(b) < 0.9, "jump at t={t}: {a:?} -> {b:?}");
            }
            prev = cur;
        }
    }

    #[test]
    fn hands_meet_at_the_clasp() {
        let s = BalletScene::new(7);
        let mut held_frames = 0;
        for i in 0..480 {
            let pose = s.frame_at(i as f32 * 0.1).1;
            if pose.held > 0.9 {
                held_frames += 1;
                assert!(
                    pose.f.hand_in.dist(pose.m.hand_in) < 0.3,
                    "hand in hand but {} apart",
                    pose.f.hand_in.dist(pose.m.hand_in)
                );
            }
        }
        assert!(
            held_frames > 120,
            "couple should hold hands most of the loop"
        );
    }

    #[test]
    fn key_loop_is_continuous() {
        // The choreography wraps; the secondary layer is a function of the
        // running clock, so only the key table has a seam to check.
        let a = keys_at(LOOP - 0.001);
        let b = keys_at(LOOP + 0.001);
        for (p, q) in [
            (a.f.out, b.f.out),
            (a.m.out, b.m.out),
            (a.f.l, b.f.l),
            (a.m.out, b.m.out),
            (a.f.inn, b.f.inn),
        ] {
            assert!(
                (p.0 - q.0).abs() < 0.1 && (p.1 - q.1).abs() < 0.1,
                "loop seam jumps: {p:?} {q:?}"
            );
        }
    }
}
