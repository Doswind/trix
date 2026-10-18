//! 东方明珠 — Shanghai Bund at night.
//!
//! The Oriental Pearl tower, Pudong skyline, neon Bund buildings and the
//! Huangpu river.  Two procedurally varied boats work the water, and every
//! few seconds one or two shooting stars cross the sky.
//! ASCII by default; `--unicode` uses box drawing and blocks.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};

const NIGHT: Rgb = Rgb::hex(0x070b1f);
const DEEP: Rgb = Rgb::hex(0x111827);
const CYAN: Rgb = Rgb::hex(0x22d3ee);
const PINK: Rgb = Rgb::hex(0xf472b6);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const WHITE: Rgb = Rgb::hex(0xe0f2fe);
const WATER: Rgb = Rgb::hex(0x083344);
const ICE: Rgb = Rgb::hex(0xcfe8ff);
const STEEL: Rgb = Rgb::hex(0x7c8b9c);

/// Hull liveries: rust, teal, steel and lacquer red.
const BOAT_TINTS: [Rgb; 4] = [
    Rgb::hex(0xc2803a),
    Rgb::hex(0x2aa8bd),
    Rgb::hex(0x9fb0c2),
    Rgb::hex(0xc65a72),
];

/// Hull / deck building blocks of a boat silhouette.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Part {
    Deck,
    RailL,
    RailR,
    /// Solid strake between the rail ends.
    Side,
    Hull,
    BowL,
    BowR,
    Porthole,
    Cabin,
    Roof,
    Window,
    Mast,
    Sail,
    SailEdge,
    Funnel,
    NavLight,
    Flag,
    Lantern,
}

impl Part {
    fn glyph(self, unicode: bool) -> char {
        match self {
            Part::Deck => {
                if unicode {
                    '▔'
                } else {
                    '_'
                }
            }
            Part::RailL => {
                if unicode {
                    '╱'
                } else {
                    '/'
                }
            }
            Part::RailR => {
                if unicode {
                    '╲'
                } else {
                    '\\'
                }
            }
            Part::Side => {
                if unicode {
                    '▒'
                } else {
                    '-'
                }
            }
            Part::Hull => {
                if unicode {
                    '▄'
                } else {
                    '_'
                }
            }
            Part::BowL => {
                if unicode {
                    '╲'
                } else {
                    '\\'
                }
            }
            Part::BowR => {
                if unicode {
                    '╱'
                } else {
                    '/'
                }
            }
            Part::Porthole => {
                if unicode {
                    '○'
                } else {
                    'o'
                }
            }
            Part::Cabin => {
                if unicode {
                    '█'
                } else {
                    '#'
                }
            }
            Part::Roof => {
                if unicode {
                    '▀'
                } else {
                    '_'
                }
            }
            Part::Window => {
                if unicode {
                    '▣'
                } else {
                    'o'
                }
            }
            Part::Mast => {
                if unicode {
                    '│'
                } else {
                    '|'
                }
            }
            Part::Sail => {
                if unicode {
                    '▒'
                } else {
                    ':'
                }
            }
            Part::SailEdge => {
                if unicode {
                    '╲'
                } else {
                    '\\'
                }
            }
            Part::Funnel => {
                if unicode {
                    '▓'
                } else {
                    '='
                }
            }
            Part::NavLight => {
                if unicode {
                    '✦'
                } else {
                    '*'
                }
            }
            Part::Flag => {
                if unicode {
                    '▸'
                } else {
                    '>'
                }
            }
            Part::Lantern => {
                if unicode {
                    '●'
                } else {
                    '*'
                }
            }
        }
    }

    /// Colour for one boat part.  `flick` is the shared 0..1 lamp flicker.
    fn color(self, tint: Rgb, flick: f32) -> Rgb {
        match self {
            Part::Deck | Part::Hull => tint.scale(0.70),
            Part::Side => tint.scale(0.55),
            Part::RailL | Part::RailR | Part::BowL | Part::BowR => {
                tint.lerp(WHITE, 0.45).scale(0.92)
            }
            Part::Porthole => GOLD.scale(0.5),
            Part::Cabin => tint.scale(0.85),
            Part::Roof => tint.lerp(WHITE, 0.32).scale(0.88),
            Part::Window => GOLD.scale(0.5 + 0.5 * flick),
            Part::Mast => WHITE.scale(0.45),
            Part::Sail => WHITE.lerp(tint, 0.5).scale(0.4),
            Part::SailEdge => WHITE.scale(0.6),
            Part::Funnel => STEEL.scale(0.62),
            Part::NavLight => GOLD.scale(flick),
            Part::Flag => PINK.scale(0.85),
            Part::Lantern => PINK.scale(0.6 + 0.4 * flick),
        }
    }
}

/// The three hand-built silhouettes; every boat picks one at rebuild time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BoatKind {
    /// Junk / sampan with a battened sail.
    Junk,
    /// Motor barge: wheelhouse, funnel and smoke.
    Barge,
    /// Two-deck ferry with a bridge over the main cabin.
    Ferry,
}

impl BoatKind {
    /// A different silhouette from this one, chosen by `flip`.
    fn other(self, flip: bool) -> BoatKind {
        match (self, flip) {
            (BoatKind::Junk, true) => BoatKind::Barge,
            (BoatKind::Junk, false) => BoatKind::Ferry,
            (BoatKind::Barge, true) => BoatKind::Junk,
            (BoatKind::Barge, false) => BoatKind::Ferry,
            (BoatKind::Ferry, true) => BoatKind::Junk,
            (BoatKind::Ferry, false) => BoatKind::Barge,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BoatCell {
    dx: isize,
    dy: isize,
    part: Part,
}

struct Boat {
    kind: BoatKind,
    /// Hull centre in columns; the hull is drawn from `x - half ..= x + half`.
    x: f32,
    /// Drift in cells per second (either direction, both slow).
    vx: f32,
    /// Screen row of the widest hull line (boat-local `dy == 0`).
    rail: isize,
    half: isize,
    cells: Vec<BoatCell>,
    tint: Rgb,
    phase: f32,
    /// Boat-local position of the funnel mouth, for the smoke plume.
    funnel: Option<(isize, isize)>,
}

/// A shooting star: a straight, fading streak across the night sky.
struct Meteor {
    x0: f32,
    y0: f32,
    vx: f32,
    vy: f32,
    age: f32,
    life: f32,
    /// Trail length in cells.
    trail: f32,
    warm: bool,
}

impl Meteor {
    fn pos(&self) -> (f32, f32) {
        (self.x0 + self.vx * self.age, self.y0 + self.vy * self.age)
    }

    /// 0..1 envelope: quick fade-in, long fade-out.
    fn fade(&self) -> f32 {
        let p = (self.age / self.life.max(0.001)).clamp(0.0, 1.0);
        let up = (p / 0.12).clamp(0.0, 1.0);
        let down = 1.0 - ((p - 0.5) / 0.5).clamp(0.0, 1.0);
        up * down
    }
}

pub struct ShanghaiScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    buildings: Vec<(f32, f32, f32, Rgb)>,
    stars: Vec<(f32, f32, f32)>,
    boats: Vec<Boat>,
    meteors: Vec<Meteor>,
    next_meteor: f32,
}

impl ShanghaiScene {
    pub fn new(seed: u64) -> Self {
        ShanghaiScene {
            seed,
            rng: Rng::new(seed ^ 0x5348_414e_4748),
            w: 0,
            h: 0,
            buildings: Vec::new(),
            stars: Vec::new(),
            boats: Vec::new(),
            meteors: Vec::new(),
            next_meteor: 1.0,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.buildings.clear();
        let palette = [CYAN, PINK, GOLD, Rgb::hex(0x60a5fa), Rgb::hex(0xa78bfa)];
        let base = self.h as f32 * 0.72;
        let mut x = 0.0;
        while x < self.w as f32 {
            let bw = self.rng.range(3.0, 7.0);
            let bh = self.rng.range(self.h as f32 * 0.12, self.h as f32 * 0.34);
            self.buildings
                .push((x, base - bh, bw, palette[self.rng.below(palette.len())]));
            x += bw + self.rng.range(0.4, 1.8);
        }
        self.stars.clear();
        for _ in 0..(w * h / 90).max(30) {
            self.stars.push((
                self.rng.range(0.0, w as f32),
                self.rng.range(0.0, h as f32 * 0.55),
                self.rng.range(0.0, TAU),
            ));
        }
        self.build_boats();
        self.meteors.clear();
        self.next_meteor = 1.0 + self.rng.range(0.0, 2.0);
    }

    /// Lay out the two boats: a small one far up the river, a big one close to
    /// the camera.  They always differ in size, silhouette and position.
    fn build_boats(&mut self) {
        self.boats.clear();
        let h = self.h as isize;
        let base = (self.h as f32 * 0.72) as isize;
        let water_h = (h - base).max(1);
        // The near boat rides in the foreground, low on the water; the far one
        // sits high up the river.  Both keep their mast rows inside the band.
        let near_rail = (h - 4).max(base);
        let far_rail = (base + (water_h - 4).clamp(2, 5))
            .min(near_rail - 1)
            .max(base);
        // Foreground masts may poke a little above the water line, in front of
        // the Bund skyline; distant hulls stay lower.
        let sky_allow = (base / 8).clamp(0, 3);
        let room_far = (far_rail - base + 1).clamp(3, 5);
        let room_near = (near_rail - base + sky_allow).clamp(3, 6);

        let unit = (self.w as isize / 22).clamp(3, 6);
        let far_half = (unit + self.rng.below(2) as isize).clamp(3, 7);
        let near_half = (far_half + 2 + self.rng.below(3) as isize)
            .min((self.w as isize / 10).clamp(4, 12))
            .max(far_half + 2);

        let near_x = self.w as f32 * self.rng.range(0.10, 0.34);
        let gap = (far_half + near_half + 3) as f32;
        let far_x = (self.w as f32 * self.rng.range(0.55, 0.90))
            .max(near_x + gap)
            .min((self.w as f32 - far_half as f32 - 1.0).max(near_x + gap));

        // Both silhouettes are random, but never the same on the same river.
        let far_kind = match self.rng.below(3) {
            0 => BoatKind::Junk,
            1 => BoatKind::Barge,
            _ => BoatKind::Ferry,
        };
        let near_kind = far_kind.other(self.rng.chance(0.5));

        // Contrasting liveries: the second boat always picks a different hull.
        let far_tint = self.rng.below(BOAT_TINTS.len());
        let near_tint = (far_tint + 1 + self.rng.below(BOAT_TINTS.len() - 1)) % BOAT_TINTS.len();

        let boat = self.build_boat(
            far_kind, far_half, far_x, far_rail, room_far, true, far_tint,
        );
        self.boats.push(boat);
        // A clearly bigger hull for the near boat.
        let boat = self.build_boat(
            near_kind, near_half, near_x, near_rail, room_near, false, near_tint,
        );
        self.boats.push(boat);
    }

    #[allow(clippy::too_many_arguments)]
    fn build_boat(
        &mut self,
        kind: BoatKind,
        half: isize,
        x: f32,
        rail: isize,
        room: isize,
        far: bool,
        tint: usize,
    ) -> Boat {
        let half = half.clamp(3, 14);
        let room = room.clamp(2, 8);

        let mut cells = Vec::with_capacity(96);
        // Deck plank line, then the flared rail ends and the hull bottom.
        for dx in -(half - 1)..=(half - 1) {
            cells.push(BoatCell {
                dx,
                dy: -1,
                part: Part::Deck,
            });
        }
        cells.push(BoatCell {
            dx: -half,
            dy: 0,
            part: Part::RailL,
        });
        cells.push(BoatCell {
            dx: half,
            dy: 0,
            part: Part::RailR,
        });
        // Solid strake between the rail ends, with a row of portholes.
        for dx in -(half - 1)..=(half - 1) {
            let part = if half >= 6 && dx.rem_euclid(3) == 0 && dx.abs() <= half - 2 {
                Part::Porthole
            } else {
                Part::Side
            };
            cells.push(BoatCell { dx, dy: 0, part });
        }
        cells.push(BoatCell {
            dx: -(half - 1),
            dy: 1,
            part: Part::BowL,
        });
        cells.push(BoatCell {
            dx: half - 1,
            dy: 1,
            part: Part::BowR,
        });
        for dx in -(half - 2)..=(half - 2) {
            cells.push(BoatCell {
                dx,
                dy: 1,
                part: Part::Hull,
            });
        }
        let mut funnel = None;
        // Everything above the deck needs vertical budget; a bare hull always
        // fits, so tiny terminals simply lose the superstructure.
        if room >= 3 {
            match kind {
                BoatKind::Junk => {
                    // Mast with a battened sail, plus a lantern at the bow.
                    let mast = (room - 2).min(half - 1).max(1);
                    for k in 0..mast {
                        cells.push(BoatCell {
                            dx: 0,
                            dy: -(2 + k),
                            part: Part::Mast,
                        });
                    }
                    let top = -(2 + mast);
                    cells.push(BoatCell {
                        dx: 0,
                        dy: top,
                        part: Part::NavLight,
                    });
                    // Pennant flying beside the truck, level with the light.
                    cells.push(BoatCell {
                        dx: 1,
                        dy: top,
                        part: Part::Flag,
                    });
                    for i in 0..mast {
                        let dy = top + 1 + i;
                        let edge = 1 + i;
                        for dx in 1..edge {
                            cells.push(BoatCell {
                                dx,
                                dy,
                                part: Part::Sail,
                            });
                        }
                        cells.push(BoatCell {
                            dx: edge,
                            dy,
                            part: Part::SailEdge,
                        });
                    }
                    if half >= 5 {
                        cells.push(BoatCell {
                            dx: -(half - 1),
                            dy: -2,
                            part: Part::Lantern,
                        });
                    }
                }
                BoatKind::Barge => {
                    let hw = (half - 2).clamp(2, 4);
                    let body = if room >= 5 { 2 } else { 1 };
                    let top = -(2 + body);
                    cabin(&mut cells, hw, top, body);
                    let air = room - (2 + body);
                    if air >= 1 {
                        let fx = (hw - 1).max(1);
                        for k in 1..=2.min(air) {
                            cells.push(BoatCell {
                                dx: fx,
                                dy: top - k,
                                part: Part::Funnel,
                            });
                        }
                        funnel = Some((fx, top - 2.min(air)));
                        cells.push(BoatCell {
                            dx: -hw,
                            dy: top - 1,
                            part: Part::Mast,
                        });
                        if air >= 2 {
                            cells.push(BoatCell {
                                dx: -hw,
                                dy: top - 2,
                                part: Part::NavLight,
                            });
                        }
                    }
                }
                BoatKind::Ferry => {
                    let hw = (half - 1).clamp(2, 5);
                    let body = if room >= 6 { 2 } else { 1 };
                    let top = -(2 + body);
                    cabin(&mut cells, hw, top, body);
                    let bw = (hw - 1).max(1);
                    let air = room - (3 + body);
                    if air >= 0 {
                        // Narrow bridge sitting on the main cabin roof.
                        cabin(&mut cells, bw, top - 1, 1);
                        funnel = Some((bw, top - 1));
                    }
                    if air >= 1 {
                        cells.push(BoatCell {
                            dx: bw,
                            dy: top - 2,
                            part: Part::Funnel,
                        });
                        funnel = Some((bw, top - 2));
                        cells.push(BoatCell {
                            dx: -bw,
                            dy: top - 2,
                            part: Part::Mast,
                        });
                        if air >= 2 {
                            cells.push(BoatCell {
                                dx: -bw,
                                dy: top - 3,
                                part: Part::NavLight,
                            });
                        }
                    }
                }
            }
        }

        let tint = BOAT_TINTS[tint % BOAT_TINTS.len()];
        let vx = if far {
            self.rng.range(0.35, 1.1)
        } else {
            -self.rng.range(0.35, 1.1)
        };
        Boat {
            kind,
            x,
            vx,
            rail,
            half,
            cells,
            tint,
            phase: self.rng.range(0.0, TAU),
            funnel,
        }
    }

    fn tower(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let cx = (self.w as f32 * 0.62) as isize;
        let base = (self.h as f32 * 0.72) as isize;
        let top = (self.h as f32 * 0.10) as isize;
        let vert = if unicode { '│' } else { '|' };
        let dia = if unicode { '╱' } else { '/' };
        let di2 = if unicode { '╲' } else { '\\' };
        // three columns / lattice shaft
        for y in top..base {
            let w = if y > base - (self.h as f32 * 0.18) as isize {
                2
            } else {
                1
            };
            for dx in -w..=w {
                let ch = if dx == 0 {
                    vert
                } else if (y / 2) % 2 == 0 {
                    dia
                } else {
                    di2
                };
                canvas.set(cx + dx, y, ch, WHITE.scale(0.42));
            }
            if y % 4 == 0 {
                canvas.set(cx - 2, y, if unicode { '·' } else { '.' }, GOLD.scale(0.55));
                canvas.set(cx + 2, y, if unicode { '·' } else { '.' }, PINK.scale(0.55));
            }
        }
        // antenna
        canvas.set(cx, top - 1, if unicode { '▴' } else { '^' }, GOLD);
        canvas.set(
            cx,
            top - 2,
            if unicode { '·' } else { '.' },
            GOLD.scale(0.8),
        );
        // three spheres: lower, middle, upper
        let spheres = [
            (base - (self.h as f32 * 0.18) as isize, 4.2, PINK),
            (base - (self.h as f32 * 0.33) as isize, 3.4, CYAN),
            (base - (self.h as f32 * 0.46) as isize, 2.3, GOLD),
        ];
        for (cy, r, col) in spheres {
            let rr = r as isize;
            for dy in -rr..=rr {
                for dx in -rr..=rr {
                    let d = dx * dx + dy * dy;
                    if d <= rr * rr {
                        let edge = d > (rr * rr * 3) / 4;
                        let ch = if unicode {
                            if edge {
                                '░'
                            } else {
                                '█'
                            }
                        } else if edge {
                            '.'
                        } else if dx == 0 || dy == 0 {
                            'O'
                        } else {
                            '#'
                        };
                        let shimmer = 0.82 + 0.18 * (t * 2.2 + cy as f32 * 0.2).sin();
                        canvas.set(
                            cx + dx,
                            cy + dy,
                            ch,
                            col.scale(if edge { 0.55 } else { 0.95 } * shimmer),
                        );
                    }
                }
            }
            for k in 0..6 {
                let a = k as f32 / 6.0 * std::f32::consts::TAU + t * 0.6;
                canvas.set(
                    cx + (a.cos() * (r + 1.2)) as isize,
                    cy + (a.sin() * (r + 1.2)) as isize,
                    if unicode { '·' } else { '*' },
                    GOLD.scale(0.55),
                );
            }
        }
        // splayed base legs and pedestal
        for dx in -9..=9 {
            canvas.set(
                cx + dx,
                base - 1,
                if unicode { '▄' } else { '_' },
                WHITE.scale(0.38),
            );
        }
        for k in 0..8 {
            canvas.set(cx - 8 + k, base - 2 - k / 2, dia, WHITE.scale(0.35));
            canvas.set(cx + 8 - k, base - 2 - k / 2, di2, WHITE.scale(0.35));
        }
        for y in (base - 5)..base {
            canvas.set(
                cx - 5,
                y,
                if unicode { '║' } else { '|' },
                WHITE.scale(0.40),
            );
            canvas.set(
                cx + 5,
                y,
                if unicode { '║' } else { '|' },
                WHITE.scale(0.40),
            );
        }
    }

    fn skyline(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        for (bx, by, bw, color) in &self.buildings {
            let x = *bx as isize;
            let y = *by as isize;
            let w = *bw as isize;
            let bottom = (self.h as f32 * 0.72) as isize;
            for yy in y..bottom {
                for xx in x..x + w {
                    let window =
                        (xx - x) > 0 && (xx - x) < w - 1 && yy > y + 1 && (xx + yy) % 3 != 0;
                    let on =
                        hash01((xx / 2) as i64, (yy / 2) as i64, self.seed.wrapping_add(9)) > 0.52;
                    let ch = if window && on {
                        if unicode {
                            '▦'
                        } else {
                            '#'
                        }
                    } else if unicode {
                        '▒'
                    } else {
                        ':'
                    };
                    let c = if window && on {
                        color.lerp(GOLD, 0.25).scale(0.9)
                    } else {
                        color.scale(0.20)
                    };
                    canvas.set(xx, yy, ch, c);
                }
            }
            // roof antenna
            if (*bx as i64) % 3 == 0 {
                canvas.set(
                    x + w / 2,
                    y - 1,
                    if unicode { '│' } else { '|' },
                    color.scale(0.7),
                );
                canvas.set(
                    x + w / 2,
                    y - 2,
                    if unicode { '·' } else { '.' },
                    PINK.scale(0.8),
                );
            }
        }
        // a few moving window glints
        for k in 0..8 {
            let bx = (self.w as f32 * (0.05 + k as f32 * 0.12)) as isize;
            let yy = (self.h as f32 * 0.42 + ((t + k as f32) % 3.0) * 2.0) as isize;
            canvas.set_if_blank(bx, yy, if unicode { '✦' } else { '*' }, GOLD.scale(0.45));
        }
    }

    fn river(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let base = (self.h as f32 * 0.72) as isize;
        let h = self.h as isize;
        for y in base..h {
            let v = (y - base) as f32 / (h - base).max(1) as f32;
            let c = WATER.lerp(NIGHT, 0.15 + v * 0.6);
            for x in 0..self.w as isize {
                let wave = ((x as f32 * 0.12 + t * 1.8 + y as f32 * 0.5).sin()) > 0.80;
                if wave {
                    let tint = if x % 17 == 0 {
                        GOLD
                    } else if x % 11 == 0 {
                        PINK
                    } else {
                        CYAN
                    };
                    canvas.set(x, y, if unicode { '≈' } else { '~' }, tint.scale(0.42));
                } else if (x + y * 3) % 5 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.6));
                }
            }
        }
        // neon reflections under the tower and major buildings
        for (i, col) in [PINK, CYAN, GOLD].iter().enumerate() {
            let x0 = (self.w as f32 * (0.62 + i as f32 * 0.07)) as isize;
            for y in base..h {
                let w = ((y - base) as f32 * 0.2) as isize + 1;
                for dx in -w..=w {
                    if ((dx + y + i as isize * 3) as f32 + t * 2.0).sin() > 0.0 {
                        canvas.set_if_blank(
                            x0 + dx,
                            y,
                            if unicode { '≈' } else { '~' },
                            col.scale(0.22),
                        );
                    }
                }
            }
        }
    }

    fn draw_boats(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let h = self.h as isize;
        let wave = if unicode { '≈' } else { '~' };
        let dot = if unicode { '·' } else { '.' };
        for boat in &self.boats {
            // Small boats ride the swell harder than the heavy ferry.
            let (rate, amp) = match boat.kind {
                BoatKind::Junk => (0.9, 1.0),
                BoatKind::Barge => (0.75, 0.9),
                BoatKind::Ferry => (0.55, 0.6),
            };
            let bob = ((t * rate + boat.phase).sin() * amp).max(0.0).round() as isize;
            let x0 = boat.x.round() as isize;
            let y0 = boat.rail - bob;
            let flick = 0.55 + 0.45 * (t * 2.3 + boat.phase).sin().abs();

            for c in &boat.cells {
                canvas.set(
                    x0 + c.dx,
                    y0 + c.dy,
                    c.part.glyph(unicode),
                    c.part.color(boat.tint, flick),
                );
            }

            // Wake trailing the stern, and the hull's rippled reflection.
            let dir = if boat.vx >= 0.0 { -1 } else { 1 };
            let bottom = y0 + 1;
            for k in 1..=3 {
                let x = x0 + dir * (boat.half + k);
                let y = bottom + k % 2;
                if y < h {
                    canvas.set(x, y, wave, boat.tint.lerp(WHITE, 0.25).scale(0.3));
                }
            }
            for row in 1..=2 {
                let y = bottom + row;
                if y >= h {
                    break;
                }
                let damp = 1.0 - row as f32 * 0.3;
                for dx in -(boat.half - 1)..=(boat.half - 1) {
                    let ph = (dx as f32 * 0.6 + row as f32 * 1.9 + t * 2.4).sin();
                    if ph > 0.1 {
                        canvas.set(
                            x0 + dx,
                            y,
                            wave,
                            boat.tint.scale(0.42 * damp * (0.35 + 0.65 * ph)),
                        );
                    }
                }
            }

            // Funnel smoke: puffs rise and drift while fading out.
            if let Some((fx, fy)) = boat.funnel {
                for k in 0..4 {
                    let u = (t * 0.28 + k as f32 * 0.25 + boat.phase) % 1.0;
                    let sx = x0 + fx - 1 - (u * 2.6) as isize;
                    let sy = y0 + fy - 1 - (u * (h as f32 * 0.09).max(2.0)) as isize;
                    let alpha = (1.0 - u) * 0.42;
                    if alpha > 0.04 {
                        canvas.blend(sx, sy, dot, STEEL, alpha);
                    }
                }
            }
        }
    }

    fn draw_meteors(&self, canvas: &mut Canvas, unicode: bool) {
        let base = (self.h as f32 * 0.72) as isize;
        for m in &self.meteors {
            let fade = m.fade();
            if fade <= 0.01 {
                continue;
            }
            let (hx, hy) = m.pos();
            if hy > base as f32 {
                continue; // already below the skyline
            }
            let speed = (m.vx * m.vx + m.vy * m.vy).sqrt().max(0.001);
            let (ux, uy) = (-m.vx / speed, -m.vy / speed);
            // White-hot head, then a streak that cools towards the tail.
            let hot = WHITE;
            let warm = if m.warm { GOLD } else { ICE };
            let cool = if m.warm { Rgb::hex(0xb45309) } else { CYAN };
            let n = m.trail.max(2.0) as isize;
            for k in 0..=n {
                let x = (hx + ux * k as f32).round() as isize;
                let y = (hy + uy * k as f32).round() as isize;
                let q = 1.0 - k as f32 / (n as f32 + 1.0);
                let a = (fade * q.powf(1.25)).clamp(0.0, 1.0);
                if a < 0.05 {
                    continue;
                }
                let col = if k == 0 {
                    hot.scale(fade)
                } else {
                    cool.lerp(warm, q).scale(a)
                };
                let ch = if k == 0 {
                    if unicode {
                        '✷'
                    } else {
                        '*'
                    }
                } else if k <= 2 {
                    '*'
                } else if k <= (n / 3).max(3) {
                    if unicode {
                        '∙'
                    } else {
                        '+'
                    }
                } else {
                    dot_for(unicode)
                };
                canvas.set(x, y, ch, col);
            }
            canvas.add_light(
                hx.round() as isize,
                hy.round() as isize,
                warm,
                0.8 * fade,
                '.',
            );
        }
    }
}

fn dot_for(unicode: bool) -> char {
    if unicode {
        '·'
    } else {
        '.'
    }
}

/// Add a boxy superstructure: roof row `top`, `body` rows of walls below it,
/// with a row of lit windows just above the deck line.
fn cabin(cells: &mut Vec<BoatCell>, hw: isize, top: isize, body: isize) {
    for dx in -hw..=hw {
        cells.push(BoatCell {
            dx,
            dy: top,
            part: Part::Roof,
        });
    }
    for dy in (top + 1)..=(top + body) {
        for dx in -hw..=hw {
            let part = if dx.abs() == hw {
                Part::Cabin
            } else if dy == top + body && (dx + hw).rem_euclid(2) == 0 {
                Part::Window
            } else {
                Part::Cabin
            };
            cells.push(BoatCell { dx, dy, part });
        }
    }
}

impl Scene for ShanghaiScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        // Slow drift, wrapping around when a boat leaves the frame.
        for boat in self.boats.iter_mut() {
            boat.x += boat.vx * dt;
            let half = boat.half as f32;
            if boat.vx >= 0.0 && boat.x - half > self.w as f32 + 2.0 {
                boat.x = -half - 2.0;
            } else if boat.vx < 0.0 && boat.x + half < -2.0 {
                boat.x = self.w as f32 + half + 2.0;
            }
        }
        // One or two shooting stars every 3–5 seconds, sometimes none at all.
        if f.t as f32 >= self.next_meteor {
            if !self.rng.chance(0.18) {
                let n = if self.rng.chance(0.55) { 2 } else { 1 };
                for _ in 0..n {
                    self.spawn_meteor();
                }
            }
            self.next_meteor = f.t as f32 + self.rng.range(3.0, 5.0);
        }
        for m in self.meteors.iter_mut() {
            m.age += dt;
        }
        self.meteors.retain(|m| m.age < m.life);
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / self.h.max(1) as f32;
            let c = NIGHT.lerp(DEEP, v * 0.8);
            for x in 0..self.w as isize {
                if (x + y * 3) % 8 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, c.scale(1.8));
                }
            }
        }
        for (sx, sy, ph) in &self.stars {
            let tw = 0.5 + 0.5 * (t * 1.4 + ph).sin().abs();
            canvas.set(
                *sx as isize,
                *sy as isize,
                if unicode { '·' } else { '.' },
                WHITE.scale(tw * 0.6),
            );
        }
        // Shooting stars go under the moon / skyline / HUD.
        self.draw_meteors(canvas, unicode);
        // moon
        let mx = (self.w as f32 * 0.18) as isize;
        let my = (self.h as f32 * 0.16) as isize;
        for dy in -2..=2 {
            for dx in -3..=3 {
                if dx * dx + dy * dy * 2 <= 9 {
                    canvas.set(
                        mx + dx,
                        my + dy,
                        if unicode { '◉' } else { 'O' },
                        WHITE.scale(0.72),
                    );
                }
            }
        }
        self.skyline(canvas, unicode, t);
        self.tower(canvas, unicode, t);
        self.river(canvas, unicode, t);
        self.draw_boats(canvas, unicode, t);
    }
}

impl ShanghaiScene {
    fn spawn_meteor(&mut self) {
        let sky = self.h as f32 * 0.45;
        let right = self.rng.chance(0.5);
        let dir = if right { 1.0 } else { -1.0 };
        let x0 = if right {
            self.rng.range(self.w as f32 * -0.15, self.w as f32 * 0.72)
        } else {
            self.rng.range(self.w as f32 * 0.28, self.w as f32 * 1.15)
        };
        let y0 = self.rng.range(0.0, sky.max(3.0));
        let dy = self.rng.range(self.h as f32 * 0.20, self.h as f32 * 0.38);
        let dx = dy * self.rng.range(1.5, 2.8) * dir;
        let life = self.rng.range(0.9, 1.6);
        self.meteors.push(Meteor {
            x0,
            y0,
            vx: dx / life,
            vy: dy / life,
            age: 0.0,
            life,
            trail: self.rng.range(5.0, 10.0),
            warm: self.rng.chance(0.45),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(w: usize, h: usize, seed: u64) -> ShanghaiScene {
        let mut s = ShanghaiScene::new(seed);
        s.resize(w, h);
        s
    }

    fn frame(t: f64, w: usize, h: usize, charset: Charset) -> Frame<'static> {
        Frame {
            t,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    #[test]
    fn skyline_exists() {
        let s = scene(120, 36, 7);
        assert!(s.buildings.len() > 10);
    }

    #[test]
    fn two_boats_differ_in_shape_size_and_place() {
        for seed in 0..40 {
            let s = scene(120, 36, seed);
            assert_eq!(s.boats.len(), 2, "seed {seed}");
            let (far, near) = (&s.boats[0], &s.boats[1]);
            assert_ne!(far.kind, near.kind, "seed {seed}");
            assert!(near.half > far.half, "seed {seed}");
            assert!(
                (far.x - near.x).abs() > (far.half + near.half) as f32,
                "seed {seed}"
            );
            assert_ne!(far.rail, near.rail, "seed {seed}");
            let shape = |b: &Boat| {
                let mut v: Vec<(isize, isize, u8)> =
                    b.cells.iter().map(|c| (c.dx, c.dy, c.part as u8)).collect();
                v.sort_unstable();
                v
            };
            assert_ne!(shape(far), shape(near), "seed {seed}");
        }
    }

    #[test]
    fn boats_are_recognisable_hulls() {
        let s = scene(120, 36, 5);
        for b in &s.boats {
            let has = |p: Part| b.cells.iter().any(|c| c.part == p);
            assert!(has(Part::Deck), "no deck line");
            assert!(has(Part::Hull), "no hull planking");
            assert!(has(Part::RailL) && has(Part::RailR), "no rail ends");
            assert!(has(Part::BowL) && has(Part::BowR), "no bow/stern taper");
            // mast, sail or wheelhouse above the deck line
            let tall = b.cells.iter().any(|c| {
                c.dy <= -2
                    && matches!(
                        c.part,
                        Part::Mast
                            | Part::Funnel
                            | Part::Cabin
                            | Part::Sail
                            | Part::SailEdge
                            | Part::Roof
                    )
            });
            assert!(tall, "no superstructure");
        }
    }

    #[test]
    fn boats_stay_on_the_water() {
        for (w, h) in [(20, 8), (40, 12), (60, 20), (80, 24), (140, 42)] {
            let s = scene(w, h, 3);
            let base = (h as f32 * 0.72) as isize;
            for b in &s.boats {
                let max_dy = b.cells.iter().map(|c| c.dy).max().unwrap_or(0);
                let min_dy = b.cells.iter().map(|c| c.dy).min().unwrap_or(0);
                assert!(b.rail >= base, "{w}x{h} boat floats above the river");
                assert!(
                    b.rail + max_dy < h as isize,
                    "{w}x{h} hull clipped (rail {} depth {})",
                    b.rail,
                    max_dy
                );
                assert!(b.rail + min_dy >= 0, "{w}x{h} superstructure clipped");
            }
        }
    }

    #[test]
    fn boats_drift_and_wrap() {
        let mut s = scene(80, 24, 9);
        let start: Vec<f32> = s.boats.iter().map(|b| b.x).collect();
        let f = frame(0.0, 80, 24, Charset::Ascii);
        for _ in 0..600 {
            s.update(&f);
        }
        for (b, x0) in s.boats.iter().zip(start) {
            assert!((b.x - x0).abs() > 0.5, "boat did not move");
            assert!(b.x > -60.0 && b.x < 160.0, "boat lost at {}", b.x);
        }
    }

    #[test]
    fn meteors_come_in_bursts_of_one_or_two() {
        let mut s = scene(100, 34, 11);
        let mut bursts = 0;
        let mut doubles = 0;
        let mut prev = 0usize;
        let mut alive_max = 0usize;
        let steps = 30 * 40;
        for step in 0..steps {
            let f = frame(step as f64 / 30.0, 100, 34, Charset::Ascii);
            s.update(&f);
            alive_max = alive_max.max(s.meteors.len());
            if s.meteors.len() > prev {
                bursts += 1;
                if s.meteors.len() - prev == 2 {
                    doubles += 1;
                }
            }
            prev = s.meteors.len();
        }
        assert!(bursts >= 5, "only {bursts} bursts in 40s");
        assert!(doubles >= 1, "never two at once");
        assert!(alive_max <= 2, "too many meteors: {alive_max}");
        assert!(prev == 0 || prev <= 2);
    }

    #[test]
    fn meteors_stay_in_the_upper_sky() {
        let mut s = scene(100, 34, 21);
        let mut seen = false;
        for step in 0..(30 * 8) {
            let f = frame(step as f64 / 30.0, 100, 34, Charset::Ascii);
            s.update(&f);
            for m in &s.meteors {
                let (_, y) = m.pos();
                seen |= y < 34.0 * 0.72;
                assert!(y > -6.0);
            }
        }
        assert!(seen);
    }

    #[test]
    fn meteor_trail_is_drawn_in_the_sky() {
        let (w, h) = (100usize, 34usize);
        let mut s = scene(w, h, 13);
        let mut canvas = Canvas::new(w, h);
        let base = (h as f32 * 0.72) as isize;
        let moon = ((w as f32 * 0.18) as isize, (h as f32 * 0.16) as isize);
        for step in 0..900 {
            let f = frame(step as f64 / 30.0, w, h, Charset::Ascii);
            s.update(&f);
            // Pick a mature meteor whose head is not hidden by the moon.
            let pick = s
                .meteors
                .iter()
                .filter(|m| m.age > m.life * 0.3)
                .find_map(|m| {
                    let (x, y) = m.pos();
                    let (hx, hy) = (x.round() as isize, y.round() as isize);
                    if hy >= base - 2 || (hx - moon.0).abs() < 6 && (hy - moon.1).abs() < 4 {
                        return None;
                    }
                    Some((x, y, hx, hy, m.vx, m.vy, m.trail))
                });
            if let Some((x, y, hx, hy, vx, vy, trail)) = pick {
                canvas.clear();
                s.draw(&mut canvas, &f);
                let cell = canvas.get(hx, hy).unwrap();
                assert_eq!(cell.ch, '*', "no meteor head at {hx},{hy}");
                let speed = (vx * vx + vy * vy).sqrt().max(0.001);
                let mut lit = 1;
                for k in 1..=(trail as isize) {
                    let px = (x - vx / speed * k as f32).round() as isize;
                    let py = (y - vy / speed * k as f32).round() as isize;
                    if py < base - 2 && canvas.get(px, py).map(|c| c.ch) != Some(' ') {
                        lit += 1;
                    }
                }
                assert!(lit >= 3, "meteor streak too short: {lit}");
                return;
            }
        }
        panic!("no meteor was ever drawn");
    }

    #[test]
    fn draws_boats_and_meteors_in_both_charsets() {
        for charset in [Charset::Ascii, Charset::Unicode] {
            let mut s = scene(100, 34, 13);
            let mut canvas = Canvas::new(100, 34);
            // Push past the first burst so a meteor is visible.
            for step in 0..150 {
                let f = frame(step as f64 / 30.0, 100, 34, charset);
                s.update(&f);
                s.draw(&mut canvas, &f);
            }
            assert!(canvas.ink() > 500);
            if charset == Charset::Ascii {
                for cell in &canvas.cells {
                    assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
                }
            }
        }
    }
}
