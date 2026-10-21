//! volcano — a night eruption that builds a permanent ash canopy.
//!
//! The sky starts clear over a compact, truncated cone with a crater lava lake
//! and two curved lava rivers.  The eruption then feeds a thick, churning
//! umbrella roughly half the screen wide: lava bombs and rock blocks rain out
//! of its underside and burst where they land, the central canopy keeps rolling
//! as fresh ash is supplied, and volcanic lightning crawls through it.  This is
//! a one-way build-up, not a loop: once the umbrella exists it never shrinks.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise, Rng};
use crate::scene::{Frame, Scene};

/// White-hot core → yellow → orange → red → deep red → cold crust.
const LAVA: [Rgb; 6] = [
    Rgb::hex(0xfff8e7),
    Rgb::hex(0xffd166),
    Rgb::hex(0xff8a3d),
    Rgb::hex(0xe2490f),
    Rgb::hex(0x8c1d05),
    Rgb::hex(0x2a0a03),
];

const ROCK_DARK: Rgb = Rgb::hex(0x14100f);
const ROCK_MID: Rgb = Rgb::hex(0x241c1c);
const ROCK_LIT: Rgb = Rgb::hex(0x8a6c62);
const ASH_DARK: Rgb = Rgb::hex(0x16141c);
const ASH_MID: Rgb = Rgb::hex(0x3a3545);
const ASH_LIGHT: Rgb = Rgb::hex(0x9a93a8);
const STAR: Rgb = Rgb::hex(0xcfd6ff);
const BOLT: Rgb = Rgb::hex(0xeef4ff);
const SKY_TOP: Rgb = Rgb::hex(0x060a18);
const SKY_LOW: Rgb = Rgb::hex(0x1a0f18);
const FAR_RIDGE: Rgb = Rgb::hex(0x151221);
const FAR_RIDGE_LIT: Rgb = Rgb::hex(0x201a2c);
const FOREGROUND: Rgb = Rgb::hex(0x080607);
const PINE: Rgb = Rgb::hex(0x24452f);

fn lava_color(heat: f32) -> Rgb {
    let h = heat.clamp(0.0, 1.0);
    if h > 0.82 {
        LAVA[1].lerp(LAVA[0], (h - 0.82) / 0.18)
    } else if h > 0.58 {
        LAVA[2].lerp(LAVA[1], (h - 0.58) / 0.24)
    } else if h > 0.34 {
        LAVA[3].lerp(LAVA[2], (h - 0.34) / 0.24)
    } else if h > 0.14 {
        LAVA[4].lerp(LAVA[3], (h - 0.14) / 0.20)
    } else {
        LAVA[5].lerp(LAVA[4], h / 0.14)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Fumarole,
    Strombolian,
    Plinian,
}

impl Phase {
    fn label(self) -> &'static str {
        match self {
            Phase::Fumarole => "FUMAROLE",
            Phase::Strombolian => "STROMBOLIAN",
            Phase::Plinian => "PLINIAN",
        }
    }

    fn vei(self) -> u32 {
        match self {
            Phase::Fumarole => 1,
            Phase::Strombolian => 3,
            Phase::Plinian => 5,
        }
    }
}

/// The first five and a half seconds are the ignition; after that the eruption
/// settles into a sustained Plinian phase that never ends.
fn phase_at(t: f32) -> (Phase, f32) {
    if t < 2.0 {
        (Phase::Fumarole, 0.16)
    } else if t < 5.5 {
        let mut e = 0.52;
        for p in [2.35f32, 3.45, 4.65] {
            e += 0.62 * (-((t - p) / 0.17).powi(2)).exp();
        }
        (Phase::Strombolian, e.min(1.45))
    } else {
        (Phase::Plinian, 0.92 + 0.08 * (t * 0.9).sin())
    }
}

/// Static world geometry, rebuilt only on resize.
struct Terrain {
    /// Skyline of the cone; `-1` means "no cone in this column".
    top: Vec<f32>,
    /// Skyline of the near foreground ridge.
    fg_top: Vec<f32>,
    /// Skyline of the far ridge behind the cone.
    far_top: Vec<f32>,
    /// Left/right edge of the cone for every row (used to keep lava on the face).
    left_at: Vec<f32>,
    right_at: Vec<f32>,
    cx: f32,
    rim_y: f32,
    base_y: f32,
    plateau: f32,
    bowl: f32,
    lake_y: f32,
    front_y: f32,
}

impl Terrain {
    fn empty() -> Self {
        Terrain {
            top: Vec::new(),
            fg_top: Vec::new(),
            far_top: Vec::new(),
            left_at: Vec::new(),
            right_at: Vec::new(),
            cx: 0.0,
            rim_y: 0.0,
            base_y: 0.0,
            plateau: 0.0,
            bowl: 0.0,
            lake_y: 0.0,
            front_y: 0.0,
        }
    }

    fn build(&mut self, w: usize, h: usize, seed: u64) {
        self.top = vec![-1.0; w];
        self.fg_top = vec![h as f32; w];
        self.far_top = vec![h as f32; w];
        self.left_at = vec![0.0; h];
        self.right_at = vec![w as f32 - 1.0; h];

        self.cx = (w as f32 * 0.40).round();
        self.rim_y = (h as f32 * 0.40).round().max(4.0);
        self.base_y = (h as f32 * 0.82).round().min(h as f32 - 4.0);
        if self.base_y < self.rim_y + 5.0 {
            self.base_y = self.rim_y + 5.0;
        }
        let bowl_max = (w as f32 * 0.06).max(2.5);
        self.bowl = (w as f32 * 0.024)
            .clamp(2.5, bowl_max)
            .min((self.base_y * 0.20).max(2.5));
        self.plateau = self.bowl + 2.0;
        self.lake_y = self.rim_y + 2.0;
        self.front_y = self.lake_y + 2.0;

        let hl = (w as f32 * 0.19).max(9.0).min((self.cx - 1.0).max(1.0));
        let hr = (w as f32 * 0.21)
            .max(10.0)
            .min((w as f32 - self.cx - 1.0).max(1.0));

        for x in 0..w {
            let dx = x as f32 - self.cx;
            let ad = dx.abs();
            let top = if ad <= self.plateau {
                if ad <= self.bowl {
                    self.rim_y + 2.0
                } else if ad <= self.bowl + 1.0 {
                    self.rim_y
                } else if ad <= self.bowl + 2.0 {
                    // raised lips frame the crater and catch the glow
                    self.rim_y - 2.0
                } else {
                    self.rim_y - 2.0
                }
            } else {
                let half = if dx < 0.0 { hl } else { hr };
                if ad > half {
                    -1.0
                } else {
                    let u = ((ad - self.plateau) / (half - self.plateau).max(1.0)).clamp(0.0, 1.0);
                    // Left flank is a little steeper; the right one runs longer.
                    let power = if dx < 0.0 { 0.48 } else { 0.44 };
                    let crest = self.rim_y - 2.0;
                    let mut y = crest + (self.base_y - crest) * u.powf(power);
                    // A shoulder keeps the silhouette from reading as a triangle.
                    y -= 1.2 * (-((u - 0.58) / 0.11).powi(2)).exp();
                    y += (value_noise(x as f32 * 0.16, 2.7, seed) - 0.5) * 1.0;
                    y.max(crest + 0.5)
                }
            };
            self.top[x] = top;
        }

        for x in 0..w {
            self.far_top[x] =
                (self.base_y - 2.0 - value_noise(x as f32 * 0.05, 4.0, seed ^ 0x22) * 4.0)
                    .clamp(1.0, self.base_y);
            self.fg_top[x] =
                (self.base_y + 1.5 + value_noise(x as f32 * 0.07, 8.3, seed ^ 0x51) * 2.0)
                    .min(h as f32 - 2.0)
                    .max(self.base_y + 1.0);
        }

        for y in 0..h {
            let mut l = w as f32;
            let mut r: f32 = -1.0;
            for x in 0..w {
                let t = self.top[x];
                if t >= 0.0 && t <= y as f32 {
                    l = l.min(x as f32);
                    r = r.max(x as f32);
                }
            }
            if r < 0.0 {
                l = self.cx;
                r = self.cx;
            }
            self.left_at[y] = l;
            self.right_at[y] = r;
        }
    }
}

struct Star {
    x: f32,
    y: f32,
    phase: f32,
    bright: f32,
}

/// A lava river sampled as a polyline that stays on the near face of the cone.
struct Flow {
    path: Vec<(f32, f32)>,
    /// 0..1 — how much of the path has been reached by the flow front.
    front: f32,
    speed: f32,
    phase: f32,
}

struct Spark {
    x: f32,
    y: f32,
    px: f32,
    py: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    max_life: f32,
    heat: f32,
    heavy: bool,
}

struct Puff {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    r: f32,
}

struct Ash {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
}

/// A heavy rock thrown out of the crater; it explodes where it lands.
struct Block {
    x: f32,
    y: f32,
    px: f32,
    py: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    size: u8,
    heat: f32,
    hot: bool,
}

/// Expanding debris ring left by a landing block.
struct Burst {
    x: f32,
    y: f32,
    age: f32,
    life: f32,
    power: f32,
}

pub struct VolcanoScene {
    w: usize,
    h: usize,
    seed: u64,
    rng: Rng,
    terrain: Terrain,
    stars: Vec<Star>,
    flows: Vec<Flow>,
    sparks: Vec<Spark>,
    puffs: Vec<Puff>,
    ash: Vec<Ash>,
    blocks: Vec<Block>,
    bursts: Vec<Burst>,
    landed: Vec<(f32, f32, u8)>,
    density: Vec<f32>,
    spark_acc: f32,
    puff_acc: f32,
    ash_acc: f32,
    block_acc: f32,
    rain_acc: f32,
    bolt: Vec<Vec<(i16, i16)>>,
    flash: f32,
    next_bolt: f32,
    phase: Phase,
    energy: f32,
    /// How far the umbrella has grown; rises after the eruption starts.
    cloud: f32,
    plume_top: f32,
}

impl VolcanoScene {
    pub fn new(seed: u64) -> Self {
        VolcanoScene {
            w: 0,
            h: 0,
            seed,
            rng: Rng::new(seed ^ 0x564f_4c43_414e),
            terrain: Terrain::empty(),
            stars: Vec::new(),
            flows: Vec::new(),
            sparks: Vec::new(),
            puffs: Vec::new(),
            ash: Vec::new(),
            blocks: Vec::new(),
            bursts: Vec::new(),
            landed: Vec::new(),
            density: Vec::new(),
            spark_acc: 0.0,
            puff_acc: 0.0,
            ash_acc: 0.0,
            block_acc: 0.0,
            rain_acc: 0.0,
            bolt: Vec::new(),
            flash: 0.0,
            next_bolt: 4.0,
            phase: Phase::Fumarole,
            energy: 0.16,
            cloud: 0.0,
            plume_top: 0.0,
        }
    }

    fn rebuild(&mut self) {
        let (w, h) = (self.w, self.h);
        if w == 0 || h == 0 {
            return;
        }
        self.terrain.build(w, h, self.seed);
        self.stars.clear();
        let n = (w * h / 90).clamp(24, 110);
        for i in 0..n {
            let x = hash01(i as i64, 1, self.seed) * w as f32;
            let y = 1.0 + hash01(i as i64, 2, self.seed) * (h as f32 * 0.58);
            self.stars.push(Star {
                x,
                y,
                phase: hash01(i as i64, 3, self.seed) * std::f32::consts::TAU,
                bright: hash01(i as i64, 4, self.seed),
            });
        }
        self.build_flows();
        self.sparks.clear();
        self.puffs.clear();
        self.ash.clear();
        self.blocks.clear();
        self.bursts.clear();
        self.landed.clear();
        self.bolt.clear();
        self.density = vec![0.0; w * h];
        self.spark_acc = 0.0;
        self.puff_acc = 0.0;
        self.ash_acc = 0.0;
        self.block_acc = 0.0;
        self.rain_acc = 0.0;
        self.phase = Phase::Fumarole;
        self.energy = 0.16;
        self.cloud = 0.0;
        self.plume_top = self.terrain.rim_y;
    }

    fn build_flows(&mut self) {
        let t = &self.terrain;
        let specs = [
            (
                t.cx + t.bowl,
                t.cx + self.w as f32 * 0.10,
                t.base_y + 2.0,
                0.0f32,
            ),
            (
                t.cx - t.bowl,
                t.cx - self.w as f32 * 0.12,
                t.base_y - 2.0,
                1.9,
            ),
        ];
        let y0 = t.front_y;
        let mut flows = Vec::new();
        for (x0, x1, y1, phase) in specs {
            if y1 <= y0 + 1.0 {
                continue;
            }
            let steps = ((y1 - y0) * 2.0).max(4.0) as usize;
            let mut path = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let f = i as f32 / steps as f32;
                let y = y0 + (y1 - y0) * f;
                let bend = (f * 5.0 + phase).sin() * (2.0 + 3.0 * f)
                    + (f * 11.0 + phase * 1.7).sin() * 1.2 * f;
                let mut x = x0 + (x1 - x0) * f.powf(0.85) + bend;
                let yi = (y.round() as usize).min(self.h.saturating_sub(1));
                let l = t.left_at.get(yi).copied().unwrap_or(0.0) + 2.0;
                let r = t.right_at.get(yi).copied().unwrap_or(self.w as f32 - 1.0) - 2.0;
                if r > l {
                    x = x.clamp(l, r);
                }
                path.push((x, y));
            }
            flows.push(Flow {
                path,
                front: 0.0,
                speed: 0.05 + self.rng.range(0.0, 0.02),
                phase,
            });
        }
        self.flows = flows;
    }

    fn spawn_spark(&mut self, energy: f32) {
        let t = &self.terrain;
        let heavy = self.rng.chance(0.03 + 0.05 * energy);
        let life = if heavy {
            self.rng.range(1.7, 2.9)
        } else {
            self.rng.range(0.55, 1.35)
        };
        let x = t.cx + self.rng.range(-t.bowl * 0.35, t.bowl * 0.35);
        let vy = -self.rng.range(4.5, 15.5) * (0.4 + 0.6 * energy) * if heavy { 1.5 } else { 1.0 };
        self.sparks.push(Spark {
            x,
            y: t.lake_y - 0.5,
            px: x,
            py: t.lake_y - 0.5,
            vx: self.rng.range(-3.5, 3.5) * (0.3 + 0.7 * energy),
            vy,
            age: 0.0,
            life,
            max_life: life,
            heat: 1.0,
            heavy,
        });
    }

    fn spawn_puff(&mut self, energy: f32) {
        let t = &self.terrain;
        let spread = t.bowl * 0.22 + self.cloud * 0.05;
        let x = t.cx + self.rng.range(-spread, spread);
        let life = self.rng.range(5.0, 9.5);
        self.puffs.push(Puff {
            x,
            y: t.lake_y - 0.6,
            vx: self.rng.range(-0.4, 0.4),
            vy: -self.rng.range(5.0, 12.0) * (0.35 + 0.65 * energy),
            age: 0.0,
            life,
            r: self.rng.range(1.0, 1.8) + self.cloud * 0.20,
        });
    }

    fn spawn_block(&mut self, energy: f32) {
        let t = &self.terrain;
        let size = if self.rng.chance(0.55) { 2 } else { 1 };
        let life = self.rng.range(2.2, 4.2);
        let x = t.cx + self.rng.range(-t.bowl * 0.7, t.bowl * 0.7);
        let vy = -self.rng.range(10.0, 24.0) * (0.55 + 0.45 * energy.min(1.0));
        let vx = self.rng.range(-14.0, 14.0) + 1.5;
        self.blocks.push(Block {
            x,
            y: t.lake_y - 0.8,
            px: x,
            py: t.lake_y - 0.8,
            vx,
            vy,
            age: 0.0,
            life,
            size,
            heat: 1.0,
            hot: self.rng.chance(0.50),
        });
    }

    fn spawn_cloud_block(&mut self) {
        let t = &self.terrain;
        let cap_half = self.w as f32 * (0.05 + 0.135 * self.cloud);
        let cap_h = self.h as f32 * (0.16 + 0.09 * self.cloud);
        let x = t.cx + self.rng.range(-cap_half * 0.48, cap_half * 0.48);
        let y = self.rng.range(2.0, (cap_h * 0.85).max(3.0));
        let vx = (x - t.cx) * 0.12 + self.rng.range(-3.5, 3.5);
        let vy = self.rng.range(-2.0, 3.0);
        let life = 5.0;
        let size = if self.rng.chance(0.30) { 2 } else { 1 };
        let hot = self.rng.chance(0.65);
        self.blocks.push(Block {
            x,
            y,
            px: x,
            py: y,
            vx,
            vy,
            age: 0.0,
            life,
            size,
            heat: 1.0,
            hot,
        });
    }

    fn spawn_ash(&mut self) {
        let t = &self.terrain;
        let life = self.rng.range(3.0, 7.0);
        self.ash.push(Ash {
            x: t.cx
                + self
                    .rng
                    .range(-(self.w as f32) * 0.14, self.w as f32 * 0.14),
            y: self.rng.range(2.0, (t.rim_y * 0.6).max(3.0)),
            vx: self.rng.range(0.3, 1.8),
            vy: self.rng.range(0.2, 0.9),
            life,
            max_life: life,
        });
    }

    fn strike(&mut self) {
        let t = &self.terrain;
        let (cx, lake) = (t.cx, t.lake_y);
        let h = self.h as f32;
        let start_x = cx
            + self
                .rng
                .range(-(self.w as f32) * 0.04, self.w as f32 * 0.09);
        let start_y = self.rng.range(2.0, (h * 0.20).max(3.0));
        let end_x = cx + self.rng.range(-t.bowl, t.bowl);
        let end_y = lake - 1.0;
        let steps = ((end_y - start_y).abs() as usize).max(5);
        let mut x = start_x;
        let mut main = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let f = i as f32 / steps as f32;
            let y = start_y + (end_y - start_y) * f;
            if i % 2 == 0 {
                x += self.rng.range(-1.4, 1.4);
            }
            if i % 5 == 0 {
                x += self.rng.range(-2.6, 2.6);
            }
            x += (end_x - x) * 0.08;
            main.push((x.round() as i16, y.round() as i16));
        }
        let mut paths = vec![main];
        for at in [steps / 3, steps * 2 / 3] {
            if at == 0 || at >= steps {
                continue;
            }
            let (bx, by) = paths[0][at];
            let dir = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
            let mut branch = Vec::with_capacity(6);
            let (mut bx, mut by) = (bx as f32, by as f32);
            for _ in 0..5 {
                bx += dir * self.rng.range(1.0, 2.8);
                by += self.rng.range(0.6, 1.6);
                branch.push((bx.round() as i16, by.round() as i16));
            }
            paths.push(branch);
        }
        self.bolt = paths;
        self.flash = 1.0;
    }

    fn update_puffs(&mut self, dt: f32, t: f32) {
        let w = self.w as f32;
        let h = self.h as f32;
        let (cx, lake) = (self.terrain.cx, self.terrain.lake_y);
        let span = (lake - 1.0).max(2.0);
        let seed = self.seed;
        for p in &mut self.puffs {
            p.age += dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            let alt = ((lake - p.y) / span).clamp(0.0, 1.0);
            // A straight vertical jet: buoyancy lifts it, a centring spring
            // keeps it on the vent axis, and the umbrella does the spreading.
            p.vy -= 4.8 * dt * (1.0 - alt * 0.72);
            p.vy *= 1.0 - 0.10 * dt;
            p.vx += (cx - p.x) * 2.4 * dt;
            p.vx *= 1.0 - 1.8 * dt;
            p.vx += (value_noise(p.x * 0.12, p.y * 0.20 + t * 0.40, seed) - 0.5) * 1.4 * dt;
            p.vy +=
                (value_noise(p.x * 0.11 + 7.0, p.y * 0.17 - t * 0.30, seed ^ 7) - 0.5) * 0.8 * dt;
            p.r += (0.45 + 0.20 * self.cloud) * dt;
            // Puffs that reach the cap are retired; the umbrella above takes over.
            if p.y < 2.0 + self.cloud * 5.8 {
                p.life = p.age;
            }
        }
        self.puffs.retain(|p| {
            p.age < p.life && p.y > 1.0 && p.y < h - 1.0 && p.x > -w * 0.2 && p.x < w * 1.2
        });
    }

    fn update_sparks(&mut self, dt: f32) {
        let w = self.w;
        let h = self.h as f32;
        for s in &mut self.sparks {
            s.px = s.x;
            s.py = s.y;
            s.age += dt;
            s.vy += 30.0 * dt;
            s.vx += 0.9 * dt;
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.heat = (s.heat - dt * 0.55).max(0.0);
            s.life -= dt;
        }
        let top = &self.terrain.top;
        self.sparks.retain(|s| {
            if s.life <= 0.0 || s.y > h || s.y < 0.0 {
                return false;
            }
            if s.vy > 0.0 && s.age > 0.25 {
                let xi = s.x.round() as isize;
                if xi >= 0 && (xi as usize) < w {
                    let ty = top[xi as usize];
                    if ty >= 0.0 && s.y >= ty {
                        return false;
                    }
                }
            }
            true
        });
    }

    fn update_blocks(&mut self, dt: f32) {
        let w = self.w;
        let h = self.h as f32;
        for b in &mut self.blocks {
            b.px = b.x;
            b.py = b.y;
            b.age += dt;
            b.vy += 34.0 * dt;
            b.vx += 1.4 * dt;
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.heat = (b.heat - dt * 0.45).max(0.0);
            b.life -= dt;
        }
        let top = &self.terrain.top;
        let fg = &self.terrain.fg_top;
        let landed = &mut self.landed;
        landed.clear();
        let mut blocks = std::mem::take(&mut self.blocks);
        blocks.retain(|b| {
            if b.life <= 0.0 || b.x < 0.0 || b.x >= w as f32 || b.y < 1.0 || b.y > h {
                return false;
            }
            let xi = b.x.round() as usize;
            let ground = if top[xi] >= 0.0 { top[xi] } else { fg[xi] };
            if b.vy > 0.0 && b.y >= ground {
                landed.push((b.x, ground, b.size));
                return false;
            }
            true
        });
        self.blocks = blocks;
        for (x, y, size) in self.landed.drain(..) {
            let life = 0.80;
            self.bursts.push(Burst {
                x,
                y,
                age: 0.0,
                life,
                power: if size >= 2 { 1.35 } else { 0.9 },
            });
        }
        for burst in &mut self.bursts {
            burst.age += dt;
        }
        self.bursts.retain(|b| b.age < b.life);
    }

    fn splat_plume(&mut self, t: f32) {
        let w = self.w;
        let h = self.h;
        self.density.fill(0.0);
        for p in &self.puffs {
            let life = (1.0 - p.age / p.life).clamp(0.0, 1.0);
            let wgt = (0.45 + 0.85 * life) * 1.10;
            let r = p.r.max(0.6);
            let ri = r.ceil() as isize;
            let (x0, y0) = (p.x as isize, p.y as isize);
            for dy in -ri..=ri {
                for dx in -ri..=ri {
                    let d2 = (dx * dx + dy * dy) as f32 / (r * r);
                    if d2 >= 1.0 {
                        continue;
                    }
                    let xi = x0 + dx;
                    let yi = y0 + dy;
                    if xi < 0 || yi < 1 || xi >= w as isize || yi >= h as isize - 1 {
                        continue;
                    }
                    self.density[yi as usize * w + xi as usize] += (1.0 - d2) * wgt;
                }
            }
        }

        // A thick, half-screen-wide umbrella. The middle is dense and rolls
        // with convection; the flanks are deliberately thinner.  Fresh ash is
        // continuously supplied, so the canopy never shrinks.
        if self.cloud > 0.04 {
            let cap_h =
                ((h as f32 * (0.16 + 0.090 * self.cloud)) as usize).clamp(5, h.saturating_sub(3));
            let cx = self.terrain.cx;
            let max_half = (w as f32 * (0.050 + 0.135 * self.cloud)).max(5.0);
            let mid_y = 1.0 + cap_h as f32 * 0.45;
            for y in 1..=cap_h {
                let v = (y as f32 - 1.0) / (cap_h as f32 - 1.0).max(1.0);
                let dome = (std::f32::consts::PI * (0.10 + 0.80 * v)).sin();
                let half = max_half * (0.42 + 0.58 * dome);
                let x0 = (cx - half).floor() as isize;
                let x1 = (cx + half).ceil() as isize;
                for x in x0..=x1 {
                    if x < 0 || x >= w as isize {
                        continue;
                    }
                    let rel = ((x as f32 - cx).abs() / half).clamp(0.0, 1.0);
                    let edge = 1.0 - rel.powf(2.2);
                    let n = value_noise(
                        x as f32 * 0.11 + t * 0.03,
                        y as f32 * 0.55,
                        self.seed ^ 0x9a,
                    ) * 0.65
                        + value_noise(x as f32 * 0.28, y as f32 * 1.20, self.seed ^ 0xa1) * 0.35;
                    // Slow churn cells travel through the canopy.
                    let churn = (((x as f32 - cx) * 0.17 + t * 1.5).sin() * 0.22
                        + (y as f32 * 0.55 - t * 1.0).sin() * 0.18)
                        * (1.0 - rel * 0.5);
                    let side = 1.0 - 0.55 * rel.powf(1.5);
                    let d = (n - (0.38 - 0.07 * self.cloud)) * 3.4 * edge * side + churn;
                    if d > 0.0 {
                        self.density[y * w + x as usize] += d.min(1.2);
                    }
                    // A solid inner core keeps the thick canopy from looking porous.
                    let core = 1.0 - (rel / 0.55).min(1.0);
                    if core > 0.0 {
                        self.density[y * w + x as usize] += core * 0.5;
                    }
                }
                // A ragged, lighter fringe lets the sides read as thinner.
                if (0.28..0.70).contains(&v) {
                    let outer = half * 1.12;
                    let x0 = (cx - outer).floor() as isize;
                    let x1 = (cx + outer).ceil() as isize;
                    for x in x0..=x1 {
                        if x < 0 || x >= w as isize {
                            continue;
                        }
                        let dist = (x as f32 - cx).abs();
                        if dist <= half {
                            continue;
                        }
                        let n = value_noise(
                            x as f32 * 0.20 + t * 0.05,
                            y as f32 * 0.80,
                            self.seed ^ 0x5e,
                        );
                        if n < 0.58 {
                            continue;
                        }
                        self.density[y * w + x as usize] +=
                            ((n - 0.58) * 2.0).min(0.40) * (1.0 - (dist - half) / (outer - half));
                    }
                }
            }

            // A lower deck gives the canopy real thickness and a dark underside.
            let deck_top = (cap_h as f32 * 0.55) as usize;
            let deck_bottom = ((cap_h as f32 * 1.15) as usize).min(h.saturating_sub(2));
            for y in deck_top..=deck_bottom {
                let v = (y - deck_top) as f32 / (deck_bottom - deck_top).max(1) as f32;
                let half = max_half * (0.90 - 0.16 * v);
                let x0 = (cx - half).floor() as isize;
                let x1 = (cx + half).ceil() as isize;
                for x in x0..=x1 {
                    if x < 0 || x >= w as isize {
                        continue;
                    }
                    let rel = ((x as f32 - cx).abs() / half).clamp(0.0, 1.0);
                    let n = value_noise(
                        x as f32 * 0.13 - t * 0.04,
                        y as f32 * 0.42 + 7.0,
                        self.seed ^ 0xc7,
                    );
                    let d = (n - 0.40) * 1.8 * (1.0 - rel.powf(2.0));
                    if d > 0.0 {
                        self.density[y * w + x as usize] += d.min(0.9);
                    }
                }
            }

            // Three orbiting convection cells make the middle visibly roll.
            for k in 0..3 {
                let a = t * 0.9 + k as f32 * std::f32::consts::TAU / 3.0;
                let bx = cx + a.cos() * max_half * 0.32;
                let by = mid_y + a.sin() * cap_h as f32 * 0.18;
                let r = (cap_h as f32 * 0.24).max(1.5);
                for dy in -(r as isize)..=(r as isize) {
                    for dx in -(r as isize)..=(r as isize) {
                        let d2 = (dx * dx + dy * dy) as f32 / (r * r);
                        if d2 >= 1.0 {
                            continue;
                        }
                        let xi = bx as isize + dx;
                        let yi = by as isize + dy;
                        if xi < 0 || xi >= w as isize || yi < 1 || yi >= h as isize - 1 {
                            continue;
                        }
                        self.density[yi as usize * w + xi as usize] += (1.0 - d2) * 0.65;
                    }
                }
            }
        }

        // Track the top of the column for the HUD.
        self.plume_top = self.terrain.rim_y;
        for y in 1..self.terrain.rim_y.max(2.0) as usize {
            if self.density[y * w..y * w + w].iter().any(|d| *d > 0.22) {
                self.plume_top = y as f32;
                break;
            }
        }
    }

    fn draw_sky(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let w = self.w;
        let theme = f.theme;
        let (cx, lake) = (self.terrain.cx, self.terrain.lake_y);
        let h = self.h as f32;
        for y in 1..=max_y {
            let v = ((y as f32 - 1.0) / (max_y as f32 - 1.0).max(1.0)).clamp(0.0, 1.0);
            let sky = SKY_TOP.lerp(SKY_LOW, v.powf(2.1));
            for x in 0..w {
                let dx = (x as f32 - cx) / (w as f32 * 0.15).max(1.0);
                let dy = (y as f32 - lake) / (h * 0.10).max(1.0);
                let halo = (-(dx * dx + dy * dy)).exp();
                let col = sky.add(theme.c(2), halo * 0.13);
                canvas.paint(x as isize, y, ' ', Rgb::BLACK, Some(col));
            }
        }
    }

    fn draw_stars(&self, canvas: &mut Canvas, f: &Frame, t: f32, max_y: isize) {
        let unicode = f.charset == Charset::Unicode;
        for s in &self.stars {
            let x = s.x.round() as isize;
            let y = s.y.round() as isize;
            if x < 0 || x >= self.w as isize || y < 1 || y >= max_y - 2 {
                continue;
            }
            let tw = 0.55 + 0.45 * (t * 1.2 + s.phase).sin();
            let ch = if s.bright > 0.88 {
                '*'
            } else if s.bright > 0.42 {
                '.'
            } else if unicode {
                '·'
            } else {
                '.'
            };
            canvas.set(x, y, ch, STAR.scale(0.16 + 0.52 * s.bright * tw));
        }
    }

    fn draw_far_ridge(&self, canvas: &mut Canvas, max_y: isize) {
        for x in 0..self.w {
            let top = self.terrain.far_top[x].round() as isize;
            let top = top.clamp(1, max_y);
            let limit = (self.terrain.base_y.round() as isize).min(max_y);
            for y in top..=limit {
                let ch = if y == top { '#' } else { ':' };
                let col = if y <= top + 1 {
                    FAR_RIDGE_LIT
                } else {
                    FAR_RIDGE
                };
                canvas.set(x as isize, y, ch, col.scale(0.55));
            }
        }
    }

    fn draw_plume(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let w = self.w;
        let unicode = f.charset == Charset::Unicode;
        let lake = self.terrain.lake_y;
        let rim_y = self.terrain.rim_y;

        for y in 1..=max_y {
            let yy = y as f32;
            if yy >= lake {
                // Keep the lava lake readable under the column.
                continue;
            }
            let dy = (lake - yy).max(0.0);
            let lit = (-dy / 6.5).exp();
            for x in 0..w {
                let d = self.density[y as usize * w + x];
                if d < 0.04 {
                    continue;
                }
                let rel = ((x as f32 - self.terrain.cx) / (w as f32 * 0.26)).abs();
                let hole = yy < rim_y * 0.55
                    && d < 0.85
                    && value_noise(x as f32 * 0.37, yy * 0.90, self.seed ^ 0x77) < 0.10;
                let ch = if d > 0.55 && rel < 0.62 && !hole {
                    if unicode {
                        '▓'
                    } else {
                        '%'
                    }
                } else if d > 0.16 {
                    if unicode {
                        '▒'
                    } else {
                        ':'
                    }
                } else if unicode {
                    '░'
                } else {
                    '.'
                };
                let top_light = ((rim_y - yy) / rim_y.max(1.0)).clamp(0.0, 1.0);
                let hi = ASH_MID
                    .lerp(ASH_LIGHT, (0.30 + 0.55 * top_light).min(1.0))
                    .lerp(f.theme.c(2), 0.04);
                let ash = ASH_DARK.lerp(hi, (0.18 + d * 0.52).min(1.0));
                let mut col = ash.lerp(LAVA[2], 0.90 * lit);
                if d > 0.9 && yy > rim_y - 3.0 {
                    col = col.lerp(LAVA[1], 0.18);
                }
                // Darker underside and thinner flanks give the canopy depth.
                if yy > rim_y * 0.55 {
                    col = col.scale(0.82);
                }
                col = col.scale(1.0 - 0.20 * rel.min(1.0));
                if self.flash > 0.15 {
                    col = col.add(BOLT, 0.14 * self.flash);
                }
                canvas.set(x as isize, y, ch, col);
            }
        }
    }

    fn draw_bolt(&self, canvas: &mut Canvas, max_y: isize) {
        if self.bolt.is_empty() || self.flash < 0.05 {
            return;
        }
        for path in &self.bolt {
            for (i, &(x, y)) in path.iter().enumerate() {
                let (xi, yi) = (x as isize, y as isize);
                if yi < 1 || yi > max_y || xi < 0 || xi >= self.w as isize {
                    continue;
                }
                let top = self.terrain.top[xi as usize];
                if top >= 0.0 && y as f32 >= top {
                    continue;
                }
                let ch = if i == 0 {
                    '|'
                } else {
                    let (px, py) = path[i - 1];
                    match (x - px, y - py) {
                        (dx, dy) if dy == 0 || dx.abs() > dy.abs() => '-',
                        (dx, _) if dx > 0 => '\\',
                        _ => '/',
                    }
                };
                canvas.set(xi, yi, ch, BOLT);
                canvas.set(xi + 1, yi, '*', BOLT.scale(0.55));
                canvas.add_light(xi, yi, BOLT, 0.50, '*');
                canvas.add_light(xi - 1, yi, Rgb::hex(0x93c5fd), 0.22, '.');
                canvas.add_light(xi + 1, yi, Rgb::hex(0x93c5fd), 0.22, '.');
                canvas.add_light(xi, yi - 1, Rgb::hex(0x93c5fd), 0.14, '.');
            }
        }
    }

    fn draw_cone(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let w = self.w;
        let unicode = f.charset == Charset::Unicode;
        let (cx, base, lake) = (self.terrain.cx, self.terrain.base_y, self.terrain.lake_y);
        let hf = self.h as f32;
        for x in 0..w {
            let top = self.terrain.top[x];
            if top < 0.0 {
                continue;
            }
            let top_i = top.round().max(1.0) as isize;
            let dx = x as f32 - cx;
            for y in top_i..=max_y {
                let yf = y as f32;
                if yf > base + 1.0 {
                    break;
                }
                let depth = (yf - top).max(0.0);
                let heat = (-((dx / (w as f32 * 0.13).max(1.0)).powi(2)
                    + ((yf - lake) / (hf * 0.30).max(1.0)).powi(2)))
                .exp();
                let mut col = ROCK_DARK.lerp(ROCK_MID, (depth / 18.0).min(1.0) * 0.55);
                col = col.add(LAVA[2], 0.95 * heat);
                let mut ch = if unicode { '█' } else { '#' };
                if y == top_i {
                    col = ROCK_LIT.scale(0.95).lerp(LAVA[2], 0.80 * heat);
                    if unicode {
                        ch = '▓';
                    }
                } else if y == top_i + 1 {
                    if unicode {
                        ch = '▓';
                    } else {
                        ch = '%';
                    }
                    col = ROCK_LIT.scale(0.42).lerp(LAVA[2], 0.40 * heat);
                } else if depth >= 3.0
                    && value_noise(x as f32 * 0.19, y as f32 * 0.47, self.seed ^ 0x33) > 0.74
                {
                    if unicode {
                        ch = '▓';
                    } else {
                        ch = '%';
                    }
                    col = col.add(LAVA[3], 0.045);
                }
                if yf >= base - 2.0 {
                    ch = if unicode { '░' } else { ':' };
                    col = col.lerp(SKY_LOW, 0.28);
                }
                canvas.set(x as isize, y, ch, col);
            }
        }
    }

    fn draw_crater_back(&self, canvas: &mut Canvas, f: &Frame, t: f32, max_y: isize) {
        let unicode = f.charset == Charset::Unicode;
        let (cx, lake) = (self.terrain.cx, self.terrain.lake_y);
        let bl = (cx - self.terrain.bowl).round() as isize;
        let br = (cx + self.terrain.bowl).round() as isize;
        if br <= bl {
            return;
        }

        // Colour the stepped notch walls; the sky stays open above the lake,
        // so the crater reads as a real U-shaped dip in the skyline.
        for x in (bl - 2)..=(br + 2) {
            let dx = (x as f32 - cx).abs();
            if dx > self.terrain.bowl + 2.0 {
                continue;
            }
            let top = self.terrain.top[x as usize];
            if top < 0.0 {
                continue;
            }
            let top_i = top.round().max(1.0) as isize;
            for y in top_i..(lake as isize) {
                if y < 1 || y > max_y {
                    continue;
                }
                let f = ((y as f32 - top) / (lake - top).max(1.0)).clamp(0.0, 1.0);
                let col = ROCK_DARK.add(LAVA[3], 0.20 + 0.55 * f);
                canvas.set(x, y, if unicode { '█' } else { '#' }, col);
            }
        }

        // Two rows of lava lake: a white-hot back edge and a cooler near edge.
        for x in bl..=br {
            let centre = 1.0 - ((x as f32 - cx).abs() / (self.terrain.bowl + 0.5)).min(1.0);
            let flick = 0.86 + 0.14 * (t * 4.0 + x as f32 * 0.8).sin();
            for (row, near) in [(lake as isize, false), (lake as isize + 1, true)] {
                if row < 1 || row > max_y {
                    continue;
                }
                let ch = if !near && centre > 0.55 {
                    '@'
                } else if near {
                    '~'
                } else {
                    '*'
                };
                let col = if near {
                    LAVA[2].lerp(LAVA[1], centre * 0.45).scale(flick * 0.8)
                } else {
                    LAVA[1].lerp(LAVA[0], centre * 0.85).scale(flick)
                };
                canvas.set(x, row, ch, col);
            }
            canvas.add_light(x, lake as isize, LAVA[2], 0.35, '.');
            for k in 1..=4 {
                let y = lake as isize - k;
                if y > 0 {
                    canvas.add_light(x, y, LAVA[2], 0.12 * (1.0 - k as f32 / 5.0), '.');
                }
            }
        }

        // The glow spills over the lips and onto the upper cone.
        for dy in -8..=6 {
            for dx in -11..=11 {
                let d2 = (dx * dx) as f32 / 121.0 + (dy * dy) as f32 / 64.0;
                if d2 <= 1.0 {
                    canvas.add_light(
                        cx as isize + dx,
                        lake as isize + dy,
                        LAVA[2],
                        0.22 * (1.0 - d2),
                        '.',
                    );
                }
            }
        }
    }

    fn draw_crater_front(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let unicode = f.charset == Charset::Unicode;
        let (cx, front) = (self.terrain.cx, self.terrain.front_y);
        let bl = (cx - self.terrain.bowl).round() as isize;
        let br = (cx + self.terrain.bowl).round() as isize;
        if front as isize > max_y {
            return;
        }
        // A dark near rim closes the bowl; the two notches let lava spill out.
        for x in (bl - 1)..=(br + 1) {
            if x < 0 || x >= self.w as isize {
                continue;
            }
            if x == br || x == bl {
                canvas.set(x, front as isize, '~', LAVA[2].scale(0.9));
                continue;
            }
            canvas.set(
                x,
                front as isize,
                if unicode { '█' } else { '#' },
                ROCK_DARK,
            );
        }
    }

    fn draw_flows(&self, canvas: &mut Canvas, t: f32, max_y: isize) {
        for flow in &self.flows {
            let len = flow.path.len();
            if len < 2 {
                continue;
            }
            let n = ((flow.front.clamp(0.0, 1.0) * len as f32) as usize).min(len);
            for i in 0..n {
                let (fx, fy) = flow.path[i];
                let xi = fx.round() as isize;
                let yi = fy.round() as isize;
                if yi < 1 || yi > max_y || xi < 0 || xi >= self.w as isize {
                    continue;
                }
                let u = i as f32 / len as f32;
                let pulse_pos = (t * 7.0 + flow.phase * 11.0) % len as f32;
                let pulse = (-((i as f32 - pulse_pos) / 3.5).powi(2)).exp();
                let heat = ((0.85 - 0.55 * u) * 0.70 + pulse * 0.70).clamp(0.0, 1.0);
                let col = lava_color(heat * 0.9 + 0.08);
                let ch = if heat > 0.62 {
                    '~'
                } else if heat > 0.35 {
                    '='
                } else {
                    '-'
                };
                canvas.set(xi, yi, ch, col);
                let bank = value_noise(i as f32 * 0.23, flow.phase, self.seed ^ 0x2f) > 0.46;
                if bank && xi + 1 < self.w as isize {
                    let edge_top = self.terrain.top[(xi + 1) as usize];
                    if edge_top >= 0.0 && edge_top <= yi as f32 {
                        canvas.set(xi + 1, yi, ':', col.scale(0.50));
                    }
                }
                if heat > 0.48 {
                    canvas.add_light(xi, yi - 1, LAVA[2], 0.10, '.');
                }
                if i % 8 == 4 {
                    canvas.set(xi + 1, yi, ':', LAVA[4]);
                }
            }
        }
    }

    fn draw_sparks(&self, canvas: &mut Canvas, max_y: isize) {
        for s in &self.sparks {
            if s.y < 1.0 || s.y > max_y as f32 {
                continue;
            }
            let life = (s.life / s.max_life).clamp(0.0, 1.0);
            let heat = (s.heat * life).clamp(0.0, 1.0);
            let col = lava_color(heat);
            let ch = if heat > 0.88 && s.age < 0.20 {
                '@'
            } else if heat > 0.58 {
                '*'
            } else if heat > 0.30 {
                '+'
            } else {
                '.'
            };
            if s.heavy {
                canvas.line(
                    s.px as isize,
                    s.py as isize,
                    s.x as isize,
                    s.y as isize,
                    '.',
                    col.scale(0.45),
                );
            }
            canvas.set(s.x as isize, s.y as isize, ch, col);
            if heat > 0.7 {
                canvas.add_light(s.x as isize, s.y as isize, LAVA[1], 0.25, '.');
            }
        }
    }

    fn draw_blocks(&self, canvas: &mut Canvas, f: &Frame, max_y: isize) {
        let unicode = f.charset == Charset::Unicode;
        for b in &self.blocks {
            if b.y < 1.0 || b.y > max_y as f32 {
                continue;
            }
            let x = b.x.round() as isize;
            let y = b.y.round() as isize;
            canvas.set(
                b.px.round() as isize,
                b.py.round() as isize,
                '.',
                LAVA[4].scale(0.45),
            );
            if b.hot {
                let core = if b.size >= 2 { '@' } else { '*' };
                let edge = if b.size >= 2 { '*' } else { '+' };
                let col = LAVA[1];
                let dim = LAVA[2];
                if b.size >= 2 {
                    for dy in 0..2isize {
                        for dx in 0..2isize {
                            canvas.set(
                                x + dx,
                                y + dy,
                                if (dx + dy) % 2 == 0 { core } else { edge },
                                if dx == 0 { col } else { dim },
                            );
                        }
                    }
                } else {
                    canvas.set(x, y, core, col);
                    canvas.set(x + 1, y, edge, dim);
                }
                for k in 1..=2 {
                    canvas.set(
                        (b.px as isize) - k,
                        (b.py as isize) - k,
                        '.',
                        LAVA[3].scale(0.65),
                    );
                }
                canvas.add_light(x, y, LAVA[1], 0.38, '*');
            } else {
                let core = if unicode { '▓' } else { 'O' };
                let edge = if unicode { '█' } else { '#' };
                let hot = LAVA[3];
                let dark = LAVA[4];
                if b.size >= 2 {
                    for dy in 0..2isize {
                        for dx in 0..2isize {
                            let ch = if (dx + dy) % 2 == 0 { core } else { edge };
                            canvas.set(x + dx, y + dy, ch, if dx == 0 { hot } else { dark });
                        }
                    }
                } else {
                    canvas.set(x, y, core, hot);
                    canvas.set(x + 1, y, edge, dark);
                }
                for k in 1..=2 {
                    canvas.set(
                        (b.px as isize) - k,
                        (b.py as isize) - k,
                        ':',
                        ASH_MID.scale(0.45),
                    );
                }
                if b.heat > 0.5 {
                    canvas.add_light(x, y, LAVA[2], 0.30, '*');
                }
            }
        }
    }

    fn draw_bursts(&self, canvas: &mut Canvas, max_y: isize) {
        for b in &self.bursts {
            if b.y < 1.0 || b.y > max_y as f32 {
                continue;
            }
            let f = (b.age / b.life).clamp(0.0, 1.0);
            let r = 1.0 + f * 8.5 * b.power;
            if f < 0.30 {
                for dy in -1isize..=1 {
                    for dx in -1isize..=1 {
                        if dx.abs() + dy.abs() <= 1 {
                            let ch = if dx == 0 && dy == 0 { '@' } else { '*' };
                            canvas.set(
                                b.x as isize + dx,
                                b.y as isize + dy,
                                ch,
                                if dx == 0 && dy == 0 { LAVA[0] } else { LAVA[1] },
                            );
                        }
                    }
                }
            }
            let n = 16;
            for k in 0..n {
                let a = k as f32 / n as f32 * std::f32::consts::TAU + b.x * 0.7;
                let x = b.x + a.cos() * r;
                let y = b.y + a.sin() * r * 0.55;
                if y < 1.0 || y > max_y as f32 {
                    continue;
                }
                let ch = if f < 0.35 {
                    '*'
                } else if f < 0.65 {
                    '+'
                } else {
                    '.'
                };
                canvas.set(x as isize, y as isize, ch, LAVA[1].lerp(LAVA[3], f));
            }
            canvas.add_light(b.x as isize, b.y as isize, LAVA[1], (1.0 - f) * 0.9, '@');
            if f < 0.35 {
                canvas.set(b.x as isize, b.y as isize, '@', LAVA[0]);
                canvas.set(b.x as isize - 1, b.y as isize, '*', LAVA[1]);
                canvas.set(b.x as isize + 1, b.y as isize, '*', LAVA[1]);
                canvas.set(b.x as isize, b.y as isize - 1, '*', LAVA[1]);
            }
            for k in 0..3 {
                canvas.set(
                    b.x as isize + k - 1,
                    (b.y - r * 0.45 - k as f32) as isize,
                    ':',
                    ASH_MID.scale(0.65 * (1.0 - f)),
                );
            }
        }
    }

    fn draw_ash(&self, canvas: &mut Canvas, max_y: isize) {
        for a in &self.ash {
            if a.y < 1.0 || a.y > max_y as f32 {
                continue;
            }
            let life = (a.life / a.max_life).clamp(0.0, 1.0);
            let ch = if life > 0.5 { ':' } else { '.' };
            canvas.set(
                a.x as isize,
                a.y as isize,
                ch,
                ASH_MID.scale(0.35 + 0.30 * life),
            );
        }
    }

    fn draw_foreground(&self, canvas: &mut Canvas, t: f32, max_y: isize) {
        let w = self.w;
        for x in 0..w {
            let top = self.terrain.fg_top[x].round() as isize;
            let top = top.clamp(1, max_y);
            for y in top..=max_y {
                let ch = if y < max_y - 1 { '#' } else { ':' };
                canvas.set(x as isize, y, ch, FOREGROUND);
            }
        }

        // A handful of pines on the left gives the cone a human scale.
        if w >= 52 && self.h >= 20 {
            for (frac, height) in [(0.060, 5usize), (0.098, 4), (0.140, 6), (0.180, 4)] {
                let x = (w as f32 * frac).round() as isize;
                if x < 2 || x >= w as isize - 2 {
                    continue;
                }
                let base = self.terrain.fg_top[x as usize].round() as isize;
                for k in 0..height.saturating_sub(1) {
                    let y = base - height as isize + 1 + k as isize;
                    if y < 2 || y > max_y {
                        continue;
                    }
                    let half = (k / 2) as isize;
                    for dx in -half..=half {
                        canvas.set(x + dx, y, '^', PINE);
                    }
                }
                for k in 1..=2 {
                    let y = base - k + 1;
                    if y >= 2 && y <= max_y {
                        canvas.set(x, y, '|', PINE.scale(0.55));
                    }
                }
            }
        }

        // The main river spills off the cone and crosses the dark apron.
        let rx = self.terrain.cx + w as f32 * 0.17;
        let steps = ((max_y as f32 - self.terrain.base_y).max(2.0) * 2.0) as usize;
        for i in 0..=steps {
            let f = i as f32 / steps as f32;
            let x = rx + f * w as f32 * 0.10 + (f * 5.0 + t * 1.5).sin() * 1.5;
            let y = self.terrain.base_y + 1.0 + f * (max_y as f32 - self.terrain.base_y - 1.0);
            if x < 0.0 || x >= w as f32 || y < 1.0 || y > max_y as f32 {
                continue;
            }
            let col = LAVA[3].lerp(LAVA[2], 0.35 + 0.35 * (f * 4.0 + t).sin().abs());
            let ch = if i % 3 == 0 { '=' } else { '~' };
            canvas.set(x as isize, y as isize, ch, col);
            canvas.set(x as isize + 1, y as isize, '~', col.scale(0.55));
            canvas.add_light(x as isize, y as isize - 1, LAVA[2], 0.14, '.');
        }
    }
}

impl Scene for VolcanoScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.rebuild();
    }

    fn update(&mut self, f: &Frame) {
        if self.w == 0 || self.h < 4 {
            return;
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let t = f.t as f32;

        let (phase, energy) = phase_at(t);
        self.phase = phase;
        self.energy = energy;
        // One-way growth: the canopy is continuously replenished and never
        // shrinks once the eruption has begun.
        let cloud_target = match phase {
            Phase::Fumarole => 0.0,
            Phase::Strombolian => 0.10 + 0.18 * (energy - 0.5).max(0.0),
            Phase::Plinian => 1.55,
        };
        if cloud_target > self.cloud {
            self.cloud += (cloud_target - self.cloud) * (dt * 1.0).min(1.0);
        }

        self.spark_acc += dt * (1.0 + energy * 24.0);
        while self.spark_acc >= 1.0 {
            self.spark_acc -= 1.0;
            if self.sparks.len() < 180 {
                self.spawn_spark(energy);
            }
        }
        let puff_rate = if phase == Phase::Fumarole {
            5.0
        } else {
            9.0 + energy * 62.0
        };
        self.puff_acc += dt * puff_rate;
        while self.puff_acc >= 1.0 {
            self.puff_acc -= 1.0;
            if self.puffs.len() < 420 {
                self.spawn_puff(energy);
            }
        }
        self.ash_acc += dt * (1.5 + energy * 5.0);
        while self.ash_acc >= 1.0 {
            self.ash_acc -= 1.0;
            if self.ash.len() < 60 {
                self.spawn_ash();
            }
        }

        // Lava rivers advance with the eruption; they never retreat.
        let target = match phase {
            Phase::Fumarole => 0.12,
            Phase::Strombolian => 0.30,
            Phase::Plinian => 1.0,
        };
        for flow in &mut self.flows {
            let want = target;
            flow.front += (want - flow.front) * dt * flow.speed.max(0.05) * 3.0;
            flow.front = flow.front.clamp(0.0, 1.0);
        }

        self.update_sparks(dt);
        self.update_puffs(dt, t);

        self.block_acc += dt * (0.5 + (energy - 0.45).max(0.0) * 1.6);
        while self.block_acc >= 1.0 {
            self.block_acc -= 1.0;
            if energy > 0.45 && self.blocks.len() < 34 {
                self.spawn_block(energy);
            }
        }
        // Lava and rock rain out of the underside of the umbrella.
        if self.cloud > 0.35 && phase != Phase::Fumarole {
            self.rain_acc += dt * (1.2 + 6.0 * self.cloud);
            while self.rain_acc >= 1.0 {
                self.rain_acc -= 1.0;
                if self.blocks.len() < 34 {
                    self.spawn_cloud_block();
                }
            }
        }
        self.update_blocks(dt);

        for a in &mut self.ash {
            a.x += a.vx * dt;
            a.y += a.vy * dt;
            a.life -= dt;
        }
        self.ash
            .retain(|a| a.life > 0.0 && a.y < self.terrain.base_y && a.x < self.w as f32 + 2.0);

        self.flash = (self.flash - dt * 3.0).max(0.0);
        let can_bolt = phase == Phase::Plinian || (phase == Phase::Strombolian && energy > 1.15);
        if can_bolt && t > self.next_bolt {
            self.strike();
            self.next_bolt = t + if phase == Phase::Plinian {
                self.rng.range(0.8, 2.0)
            } else {
                self.rng.range(1.4, 3.2)
            };
        }

        self.splat_plume(t);
    }

    fn hud_status(&self, _f: &Frame) -> Option<String> {
        let km =
            ((self.terrain.rim_y - self.plume_top).max(0.0) / self.h as f32 * 26.0).round() as u32;
        let state = if self.flash > 0.15 {
            "STRIKE"
        } else {
            self.phase.label()
        };
        Some(format!(
            "VEI {} | ASH {:02}KM | {}",
            self.phase.vei(),
            km.min(30),
            state
        ))
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h < 4 {
            return;
        }
        let t = f.t as f32;
        let max_y = self.h as isize - 2;

        self.draw_sky(canvas, f, max_y);
        self.draw_stars(canvas, f, t, max_y);
        self.draw_far_ridge(canvas, max_y);
        self.draw_cone(canvas, f, max_y);
        self.draw_crater_back(canvas, f, t, max_y);
        self.draw_plume(canvas, f, max_y);
        self.draw_bolt(canvas, max_y);
        self.draw_crater_front(canvas, f, max_y);
        self.draw_flows(canvas, t, max_y);
        self.draw_sparks(canvas, max_y);
        self.draw_ash(canvas, max_y);
        self.draw_foreground(canvas, t, max_y);
        self.draw_blocks(canvas, f, max_y);
        self.draw_bursts(canvas, max_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(t: f32) -> Frame<'static> {
        Frame {
            t: t as f64,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: crate::theme::find("ember").unwrap(),
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    fn run(scene: &mut VolcanoScene, t: f32) {
        let mut canvas = Canvas::new(scene.w, scene.h);
        let mut clock: f32 = 0.0;
        let mut f = frame(0.0);
        f.w = scene.w;
        f.h = scene.h;
        while clock < t {
            f.t = clock as f64;
            scene.update(&f);
            clock += 1.0 / 30.0;
        }
        f.t = clock as f64;
        scene.draw(&mut canvas, &f);
    }

    #[test]
    fn terrain_is_a_truncated_asymmetric_cone() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let t = &scene.terrain;
        let cx = t.cx as usize;
        assert!(t.top[cx] > t.rim_y, "crater must dip below the rim");
        assert!(
            t.top[(t.cx - t.bowl - 1.0) as usize] < t.rim_y,
            "lips must rise above the rim"
        );
        assert!(t.top[0] < 0.0, "the cone must not span the whole width");
        assert!(t.base_y > t.rim_y + 6.0);
    }

    #[test]
    fn plume_stays_connected_from_vent_to_umbrella() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let mut f = frame(0.0);
        let mut t = 0.0;
        // Run into the Plinian phase.
        while t < 7.0 {
            f.t = t;
            scene.update(&f);
            t += 1.0 / 30.0;
        }
        let w = scene.w;
        let rim = scene.terrain.rim_y as usize;
        for y in 2..rim {
            let row = &scene.density[y * w..y * w + w];
            assert!(
                row.iter().any(|d| *d > 0.10),
                "plume column is broken at row {y}"
            );
        }
    }

    #[test]
    fn eruption_draws_crater_lava_and_ash() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        run(&mut scene, 7.0);
        let mut canvas = Canvas::new(120, 36);
        let f = frame(7.0);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('#'), "no rock");
        assert!(text.contains('@') || text.contains('*'), "no lava");
        assert!(text.contains('%') || text.contains(':'), "no ash");
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }

    #[test]
    fn clear_start_then_steady_thick_canopy() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let mut f = frame(0.0);
        scene.update(&f);
        assert!(scene.cloud < 0.02, "the sky must start clear");

        let mut t = 0.0;
        while t < 12.0 {
            f.t = t;
            scene.update(&f);
            t += 1.0 / 30.0;
        }
        let cloud_at_12 = scene.cloud;
        assert!(cloud_at_12 > 1.35, "canopy did not grow: {cloud_at_12}");
        assert_eq!(scene.phase, Phase::Plinian);

        let w = scene.w;
        let mut min_x = w;
        let mut max_x = 0;
        for y in 2..8 {
            for x in 0..w {
                if scene.density[y * w + x] > 0.25 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
        }
        assert!(
            max_x.saturating_sub(min_x) >= w * 45 / 100,
            "canopy is not half-screen wide: {}",
            max_x.saturating_sub(min_x)
        );

        while t < 20.0 {
            f.t = t;
            scene.update(&f);
            t += 1.0 / 30.0;
        }
        assert!(
            scene.cloud + 1e-4 >= cloud_at_12,
            "canopy shrank from {cloud_at_12} to {}",
            scene.cloud
        );
    }

    #[test]
    fn rain_falls_from_the_canopy_middle() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let mut f = frame(0.0);
        let mut t = 0.0;
        while t < 12.0 {
            f.t = t;
            scene.update(&f);
            t += 1.0 / 30.0;
        }
        let rim = scene.terrain.rim_y;
        let mid = scene.terrain.cx;
        assert!(
            scene.blocks.iter().any(|b| b.y < rim),
            "no ejecta spawned inside the canopy"
        );
        assert!(
            scene
                .blocks
                .iter()
                .any(|b| (b.x - mid).abs() < scene.w as f32 * 0.30),
            "ejecta did not come from the middle"
        );
        assert!(scene.blocks.iter().any(|b| b.hot), "no lava bombs");
    }

    #[test]
    fn lightning_draws_a_bolt_above_the_cone() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let mut f = frame(8.0);
        let mut canvas = Canvas::new(120, 36);
        scene.update(&f);
        scene.strike();
        f.t = 8.0;
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(
            text.contains('|') || text.contains('\\') || text.contains('/'),
            "no lightning bolt"
        );
    }

    #[test]
    fn two_lava_rivers_are_curved() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        assert_eq!(scene.flows.len(), 2, "expected exactly two rivers");
        for flow in &scene.flows {
            assert!(flow.path.len() > 8);
            let len = flow.path.len();
            let (x0, _) = flow.path[0];
            let (x1, _) = flow.path[len - 1];
            let (xm, _) = flow.path[len / 2];
            let linear = (x0 + x1) * 0.5;
            assert!(
                (xm - linear).abs() > 0.8,
                "river is too straight: {xm} vs {linear}"
            );
        }
    }

    #[test]
    fn blocks_explode_where_they_land() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        scene.spawn_block(1.0);
        let b = scene.blocks.last_mut().unwrap();
        b.x = scene.terrain.cx;
        let cx = scene.terrain.cx as usize;
        b.y = scene.terrain.top[cx] - 1.0;
        b.vx = 0.0;
        b.vy = 8.0;
        for _ in 0..10 {
            scene.update_blocks(1.0 / 30.0);
        }
        assert!(!scene.bursts.is_empty(), "landing block did not burst");
    }

    #[test]
    fn unicode_uses_block_glyphs() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(120, 36);
        let mut f = frame(0.0);
        f.charset = Charset::Unicode;
        let mut canvas = Canvas::new(120, 36);
        let mut t = 0.0f32;
        while t < 7.0 {
            f.t = t as f64;
            scene.update(&f);
            t += 1.0 / 30.0;
        }
        f.t = t as f64;
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('█') || text.contains('▓'), "no block glyphs");
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        let mut scene = VolcanoScene::new(7);
        scene.resize(40, 12);
        run(&mut scene, 3.0);
    }
}
