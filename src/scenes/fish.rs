//! 水族馆 — a full-screen aquarium with a few big sprite fish and a shoal of
//! small stick fish.
//!
//! The big fish are hand-drawn character sprites (tiny/small/medium/large) with
//! a forked tail, a bright eye and a mouth; the little ones are the classic
//! `<==o>` shapes.  Fish keep their heading on screen (they only change
//! direction while off-screen, so no head or mouth ever spins around in view),
//! rising bubbles of three sizes, randomly scattered seaweed patches, light
//! rays and sand fill the rest of the tank.

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::Rng;
use crate::scene::{Frame, Scene};

const DEEP: Rgb = Rgb::hex(0x031b2e);
const WATER: Rgb = Rgb::hex(0x075985);
const LIGHT: Rgb = Rgb::hex(0x38bdf8);
const SAND: Rgb = Rgb::hex(0x92400e);
const WEED: Rgb = Rgb::hex(0x16a34a);
const WEED_DARK: Rgb = Rgb::hex(0x14532d);
const BUBBLE: Rgb = Rgb::hex(0xbfefff);
const SHADE: Rgb = Rgb::hex(0x02121f);
const FISH_COLORS: [Rgb; 6] = [
    Rgb::hex(0xf97316),
    Rgb::hex(0xfacc15),
    Rgb::hex(0x22d3ee),
    Rgb::hex(0xf472b6),
    Rgb::hex(0xa3e635),
    Rgb::hex(0x818cf8),
];

// ---------------------------------------------------------------------------
// Big sprites (drawn nose-right; mirrored when the fish swims left).
//
// Legend
//   `#` body        `%` dark back      `+` bright belly
//   `,` `'` `;` soft edge              `/` `\` `_` `-` `~` `|` fins
//   `o` eye                            `>` mouth
// ---------------------------------------------------------------------------

const SPRITE_SMALL: &[&str] = &[
    " ##,     ,%%%,",
    " '###,   ### o ##;,",
    "   '####,##########>",
    "  ,###' '#########;'",
    " ##'     '++++++'",
];

const SPRITE_MEDIUM: &[&str] = &[
    "##,",
    "'###,",
    "  '###,     ,%%%%%,",
    "    '####,  #### o ##;,",
    "     '#####,################>",
    "     ,####'#################;'",
    "  ,###'     '+++++++'",
];

const SPRITE_LARGE: &[&str] = &[
    "##,",
    "'###,",
    "  '###,      ,%%%%%%%%,",
    "    '####,   ####### o ###;,",
    "     '#####,###################/#####>",
    "     ,#####'###################;'",
    "  ,###'      '################'",
    "##'          '++++++++++++'",
];

// Small stick fish, the classic `<==o>` shapes.
const STICK_1: &[&str] = &["<=o>"];
const STICK_2: &[&str] = &["<-=o>", " ^"];
const STICK_3: &[&str] = &["<==-o>", "  ^"];
const STICK_4: &[&str] = &["<=--=o>", "   ^"];
const STICK_5: &[&str] = &["<o=-=>", " ^"];
const STICK_6: &[&str] = &["<==o>", " ^"];
// Double-layer sticks: the same little fish drawn two rows thick (one eye,
// the lower row shades the belly).
const STICK_D1: &[&str] = &[" <=-=o>", "<%===%>"];
const STICK_D2: &[&str] = &["  <-=o>", "<%==--%>"];
const STICK_D3: &[&str] = &[" <=-=o>", "<%===--%>"];
const STICK_D4: &[&str] = &["  <==-o>", "<%===--%>"];
// Triple-layer sticks, the chunky ones.
const STICK_T1: &[&str] = &["  <=-=o>", " <==--=>", "<%====%>"];
const STICK_T2: &[&str] = &["   <-=o>", "  <==--=>", "<%===--%>"];

#[derive(Clone, Copy)]
struct Sprite {
    rows: &'static [&'static str],
    w: isize,
    /// Columns of tail fin at the back; only these wave (0 = rigid).
    tail: isize,
    depth_rank: u8,
    big: bool,
}

const SMALL: Sprite = Sprite {
    rows: SPRITE_SMALL,
    w: 20,
    tail: 6,
    depth_rank: 1,
    big: true,
};
const MEDIUM: Sprite = Sprite {
    rows: SPRITE_MEDIUM,
    w: 30,
    tail: 8,
    depth_rank: 2,
    big: true,
};
const LARGE: Sprite = Sprite {
    rows: SPRITE_LARGE,
    w: 38,
    tail: 10,
    depth_rank: 3,
    big: true,
};

const STICKS_S: [Sprite; 6] = [
    Sprite {
        rows: STICK_1,
        w: 4,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_2,
        w: 5,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_3,
        w: 6,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_4,
        w: 7,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_5,
        w: 6,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_6,
        w: 5,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
];

const STICKS_M: [Sprite; 4] = [
    Sprite {
        rows: STICK_D1,
        w: 7,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_D2,
        w: 8,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_D3,
        w: 9,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_D4,
        w: 9,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
];

const STICKS_L: [Sprite; 2] = [
    Sprite {
        rows: STICK_T1,
        w: 8,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
    Sprite {
        rows: STICK_T2,
        w: 9,
        tail: 0,
        depth_rank: 0,
        big: false,
    },
];

fn pick_stick(r: f32, idx: usize) -> Sprite {
    if r < 0.15 {
        STICKS_L[idx % STICKS_L.len()]
    } else if r < 0.50 {
        STICKS_M[idx % STICKS_M.len()]
    } else {
        STICKS_S[idx % STICKS_S.len()]
    }
}

struct Fish {
    x: f32,
    y: f32,
    /// Big fish are pinned to a lane; small ones drift at their own height.
    lane: Option<usize>,
    speed: f32,
    dir: f32,
    phase: f32,
    color: Rgb,
    sprite: Sprite,
    depth: u8,
}

struct Bubble {
    /// Straight-line x the bubble rises along.
    base_x: f32,
    x: f32,
    y: f32,
    speed: f32,
    /// Peak horizontal offset for the whole ascent (<= 1 cell).
    sway_amp: f32,
    /// Completed left-right swings over the ascent, 0..=3.
    sway_cycles: f32,
    glyph: char,
    /// Random brightness so no two bubbles look the same.
    shade: f32,
}

#[derive(Clone, Copy)]
struct Weed {
    x: f32,
    height: f32,
    phase: f32,
    shade: f32,
}

/// Mirror a glyph so left-facing sprites keep their shape.
fn mirror_char(ch: char) -> char {
    match ch {
        '/' => '\\',
        '\\' => '/',
        '>' => '<',
        '<' => '>',
        ',' => '.',
        '.' => ',',
        '\'' => '`',
        '`' => '\'',
        ';' => ':',
        ':' => ';',
        '(' => ')',
        ')' => '(',
        _ => ch,
    }
}

/// Glyph + colour for one sprite cell.
fn sprite_cell(ch: char, base: Rgb, unicode: bool) -> Option<(char, Rgb)> {
    let tint = |t: f32| base.lerp(Rgb::WHITE, t);
    Some(match ch {
        '#' => (if unicode { '█' } else { '#' }, base),
        '%' => (if unicode { '▓' } else { '%' }, base.lerp(SHADE, 0.42)),
        '+' => (if unicode { '▒' } else { '+' }, tint(0.38)),
        ',' | '\'' | ';' | '`' => (ch, tint(0.22)),
        '/' | '\\' | '_' | '-' | '~' | '|' => (ch, tint(0.26)),
        'o' => (if unicode { 'O' } else { 'o' }, Rgb::WHITE),
        '>' | '<' => (ch, tint(0.45)),
        '=' => (ch, base),
        '^' => ('^', tint(0.30)),
        '.' => ('.', tint(0.30)),
        _ => return None,
    })
}

/// Bubble glyph; the three shapes are picked independently of the shade.
fn bubble_glyph(idx: usize) -> char {
    match idx % 3 {
        0 => 'o',
        1 => 'O',
        _ => '8',
    }
}

pub struct FishScene {
    rng: Rng,
    w: usize,
    h: usize,
    fish: Vec<Fish>,
    bubbles: Vec<Bubble>,
    weeds: Vec<Weed>,
}

impl FishScene {
    pub fn new(seed: u64) -> Self {
        FishScene {
            rng: Rng::new(seed ^ 0x4649_5348),
            w: 0,
            h: 0,
            fish: Vec::new(),
            bubbles: Vec::new(),
            weeds: Vec::new(),
        }
    }

    /// Vertical band a lane sits in, keeping fish clear of the sand and the HUD.
    fn lane_y(&self, lane: usize, lanes: usize) -> f32 {
        let top = 4.0;
        let bottom = (self.h as f32 - 6.0).max(top + 4.0);
        top + (bottom - top) * (lane as f32 + 0.5) / lanes as f32
    }

    fn lanes(&self) -> usize {
        (self.h / 9).clamp(3, 7)
    }

    fn add_fish(&mut self, sprite: Sprite, x: f32, lane: Option<usize>, lanes: usize) {
        let color = FISH_COLORS[self.rng.below(FISH_COLORS.len())];
        let depth = if sprite.big {
            1 + self.rng.below(2) as u8
        } else {
            self.rng.below(3) as u8
        };
        let base_speed = if sprite.big {
            self.rng.range(2.2, 5.0)
        } else {
            self.rng.range(3.6, 9.0)
        };
        let y = match lane {
            Some(l) => self.lane_y(l, lanes),
            None => self.rng.range(3.0, (self.h as f32 - 5.0).max(4.0)),
        };
        self.fish.push(Fish {
            x,
            y,
            lane,
            speed: base_speed * depth_speed(depth),
            dir: if self.rng.chance(0.5) { 1.0 } else { -1.0 },
            phase: self.rng.range(0.0, TAU),
            color,
            sprite,
            depth,
        });
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.fish.clear();
        let lanes = self.lanes();

        // A handful of big fish...
        let big_n = if self.w >= 110 && self.h >= 30 {
            5
        } else if self.w >= 80 && self.h >= 24 {
            3
        } else {
            2
        };
        for i in 0..big_n {
            let sprite = if self.h < 24 {
                SMALL
            } else {
                match i {
                    0 | 1 => LARGE,
                    2 | 3 => MEDIUM,
                    _ => SMALL,
                }
            };
            let x = (i as f32 + self.rng.range(0.2, 0.8)) * self.w as f32 / big_n as f32;
            self.add_fish(sprite, x, Some(i % lanes), lanes);
        }

        // ...plus a shoal of small stick fish, spread across the water column.
        let stick_n = (self.w / 7).clamp(8, 30);
        for i in 0..stick_n {
            let roll = self.rng.f32();
            let idx = self.rng.below(30);
            let sprite = pick_stick(roll, idx);
            let x = (i as f32 + self.rng.range(0.1, 0.9)) * self.w as f32 / stick_n as f32;
            self.add_fish(sprite, x, None, lanes);
        }

        // Bubbles: `o` / `O` / `8`, each with its own shade, rising straight
        // up along a fixed column with at most three little left-right swings.
        self.bubbles.clear();
        let nb = (self.w / 6).clamp(10, 36);
        for _ in 0..nb {
            let y = if self.rng.chance(0.7) {
                self.rng
                    .range((self.h as f32 - 10.0).max(2.0), self.h as f32 - 3.0)
            } else {
                self.rng.range(1.0, self.h as f32 - 3.0)
            };
            let base_x = self.rng.range(1.0, (self.w as f32 - 1.0).max(2.0));
            self.bubbles.push(Bubble {
                base_x,
                x: base_x,
                y,
                speed: self.rng.range(0.45, 2.0),
                sway_amp: self.rng.range(0.0, 0.9),
                sway_cycles: self.rng.range(0.0, 3.0),
                glyph: bubble_glyph(self.rng.below(3)),
                shade: self.rng.range(0.42, 0.95),
            });
        }

        // Seaweed: random patches, random heights and spacing.
        self.weeds.clear();
        let patches = (self.w / 34).clamp(3, 8);
        for _ in 0..patches {
            let cx = self.rng.range(2.0, (self.w as f32 - 3.0).max(3.0));
            let stems = 1 + self.rng.below(4);
            let spread = self.rng.range(0.8, 2.2);
            for s in 0..stems {
                let off = (s as f32 - (stems as f32 - 1.0) * 0.5) * spread;
                let height = self.rng.range(3.0, 14.0);
                self.weeds.push(Weed {
                    x: cx + off,
                    height,
                    phase: self.rng.range(0.0, TAU),
                    shade: self.rng.range(0.0, 1.0),
                });
            }
        }
    }

    fn draw_water(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let w = self.w as isize;
        let h = self.h as isize;
        let ground = h - 3;
        // Water column: dim, sparse dust so the fish stay readable.
        for y in 0..h {
            let v = y as f32 / h.max(1) as f32;
            let col = DEEP.lerp(WATER, 0.22 + v * 0.42);
            for x in 0..w {
                if (x + y * 3) % 6 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col.scale(1.15));
                } else if (x * 2 + y * 5) % 7 != 0 {
                    canvas.set(x, y, if unicode { '░' } else { '.' }, col.scale(0.30));
                }
            }
        }
        // Light rays from the surface.
        for k in 0..5 {
            let x0 = (self.w as f32 * (0.14 + k as f32 * 0.18)) as isize
                + ((t * 0.35 + k as f32).sin() * 3.0) as isize;
            for y in 2..ground {
                let x = x0 + (y as f32 * 0.20) as isize;
                if x >= 0 && x < w {
                    canvas.add_light(x, y, LIGHT, 0.036, if unicode { '░' } else { ':' });
                }
            }
        }
        // Caustic sparkles.
        for i in 0..(self.w * self.h / 150).max(20) {
            let x = ((i * 37 + (t * 7.0) as usize) % self.w) as isize;
            let y = ((i * 53) % (self.h - 5).max(1)) as isize;
            let sparkle = if unicode { '·' } else { '.' };
            canvas.set_if_blank(x, y, sparkle, LIGHT.scale(0.16));
        }
        // Sand.
        for y in ground..h {
            for x in 0..w {
                let ch = if y == ground {
                    if unicode {
                        '▀'
                    } else {
                        '~'
                    }
                } else if (x + y) % 2 == 0 {
                    if unicode {
                        '▒'
                    } else {
                        ':'
                    }
                } else if unicode {
                    '█'
                } else {
                    '#'
                };
                canvas.set(x, y, ch, SAND.scale(0.34 + (y - ground) as f32 * 0.06));
            }
        }
        // Seaweed: each stored stem bends with the current.
        for weed in &self.weeds {
            let base = ground - 1;
            let height = weed.height.max(1.0);
            for k in 0..height as isize {
                let up = k as f32 / height;
                let sway = (t * 0.8 + weed.phase + k as f32 * 0.42).sin() * 2.4 * up;
                let x = weed.x.round() as isize + sway as isize;
                let y = base - k;
                let ch = if up > 0.55 {
                    if (k + weed.x as isize) % 2 == 0 {
                        '/'
                    } else {
                        '\\'
                    }
                } else if unicode {
                    '║'
                } else {
                    '|'
                };
                let col = WEED.lerp(WEED_DARK, weed.shade * 0.55 + up * 0.15);
                canvas.set(x, y, ch, col.scale(0.72 + 0.28 * up));
                if up > 0.28 && k % 3 == 0 {
                    let leaf = if (k / 3) % 2 == 0 { '/' } else { '\\' };
                    canvas.set(x + 1, y, leaf, WEED_DARK.scale(0.95));
                    canvas.set(x - 1, y, mirror_char(leaf), WEED_DARK.scale(0.85));
                }
            }
        }
    }

    fn draw_fish(&self, canvas: &mut Canvas, f: &Fish, unicode: bool, t: f32) {
        let sp = f.sprite;
        let w = sp.w;
        let h = sp.rows.len() as isize;
        let right = f.dir >= 0.0;
        let x0 = f.x.round() as isize;
        let y0 = f.y.round() as isize - h / 2;
        let dim = depth_scale(f.depth);
        for (r, row) in sp.rows.iter().enumerate() {
            for (c, ch) in row.chars().enumerate() {
                let c = c as isize;
                if c >= w {
                    continue;
                }
                // The tail is the sprite's left columns no matter which
                // way the fish faces: never let the wave reach the mouth.
                let tail_col = c;
                let sx = if right { x0 + c } else { x0 + w - 1 - c };
                let wave = if sp.tail > 0 && tail_col < sp.tail {
                    let k = 1.0 - tail_col as f32 / sp.tail as f32;
                    ((t * 8.0 + f.phase * 2.0).sin() * 0.9 * k).round() as isize
                } else {
                    0
                };
                let Some((glyph, col)) = sprite_cell(ch, f.color, unicode) else {
                    continue;
                };
                let glyph = if right { glyph } else { mirror_char(glyph) };
                canvas.set(sx, y0 + r as isize + wave, glyph, col.scale(dim));
            }
        }
        // A soft glow around the big fish sells the depth.
        if sp.depth_rank >= 3 {
            canvas.add_light(x0 + w / 2, f.y as isize, f.color, 0.10, '.');
        }
    }
}

fn depth_scale(depth: u8) -> f32 {
    match depth {
        0 => 0.80,
        1 => 1.0,
        _ => 1.15,
    }
}

fn depth_speed(depth: u8) -> f32 {
    match depth {
        0 => 0.85,
        1 => 1.0,
        _ => 1.18,
    }
}

impl Scene for FishScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let lanes = self.lanes();
        let centers: Vec<f32> = (0..lanes).map(|l| self.lane_y(l, lanes)).collect();
        let w = self.w as f32;
        let max_y = (self.h as f32 - 5.0).max(5.0);
        for fish in &mut self.fish {
            let move_speed = fish.speed * f.speed.clamp(0.1, 4.0);
            fish.x += move_speed * fish.dir * dt;
            fish.phase += dt * 1.5;
            let margin = fish.sprite.w as f32 + 4.0;
            let (turn, exit_left) = if fish.dir > 0.0 && fish.x > w + margin {
                (true, false)
            } else if fish.dir < 0.0 && fish.x < -margin {
                (true, true)
            } else {
                (false, false)
            };
            if turn {
                // Turn around only while off-screen: the head never flips in view.
                if self.rng.chance(0.35) {
                    fish.dir = -fish.dir;
                    fish.x = if exit_left { -margin } else { w + margin };
                } else {
                    fish.x = if exit_left { w + margin } else { -margin };
                }
                if fish.lane.is_some() {
                    let lane = self.rng.below(centers.len());
                    fish.lane = Some(lane);
                    fish.y = centers[lane];
                } else {
                    fish.y = self.rng.range(3.0, max_y);
                }
            }
            if let Some(lane) = fish.lane {
                fish.y = centers[lane];
            }
            let top = 2.0 + (fish.sprite.rows.len() as f32) * 0.5;
            fish.y = fish.y.clamp(top, max_y.max(top));
        }
        let span = (self.h as f32 - 5.0).max(1.0);
        for b in &mut self.bubbles {
            b.y -= b.speed * f.speed.clamp(0.1, 4.0) * dt;
            // The sway is a function of climbed height, so it completes at most
            // `sway_cycles` (<= 3) swings and never drifts sideways.
            let progress = ((self.h as f32 - 3.0 - b.y) / span).clamp(0.0, 1.0);
            b.x = (b.base_x + (TAU * b.sway_cycles * progress).sin() * b.sway_amp)
                .clamp(0.0, self.w as f32 - 1.0);
            if b.y < 1.0 {
                b.y = self.h as f32 - 3.0;
                b.base_x = self.rng.range(1.0, (self.w as f32 - 1.0).max(2.0));
                b.x = b.base_x;
                b.sway_amp = self.rng.range(0.0, 0.9);
                b.sway_cycles = self.rng.range(0.0, 3.0);
                b.glyph = bubble_glyph(self.rng.below(3));
                b.shade = self.rng.range(0.42, 0.95);
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_water(canvas, unicode, t);
        // Bubbles overwrite the dust trail so they rise as one continuous stream;
        // fish are drawn afterwards, so a bubble that passes behind a fish is hidden.
        for b in &self.bubbles {
            canvas.set(
                b.x.round() as isize,
                b.y.round() as isize,
                b.glyph,
                BUBBLE.scale(b.shade),
            );
        }
        // Shoal first, then the big fish back-to-front: the small fish may pass
        // behind a big one, but never scribble over its body.
        let mut order: Vec<&Fish> = self.fish.iter().collect();
        order.sort_by_key(|fish| (fish.sprite.big as u8, fish.depth));
        for fish in order {
            Self::draw_fish(self, canvas, fish, unicode, t);
        }
        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ AQUARIUM // DEEP BLUE ╞"
        } else {
            " AQUARIUM // DEEP BLUE "
        };
        canvas.put_str(2, 0, title, LIGHT.scale(0.9));
        let status = format!("{} FISH", self.fish.len());
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            BUBBLE.scale(0.75),
        );
        let hint = "TAP GLASS   //   n/p NEXT";
        canvas.put_str(2, h - 1, hint, LIGHT.scale(0.45));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Tune;

    fn frame(t: f64, unicode: bool) -> Frame<'static> {
        Frame {
            t,
            dt: 1.0 / 30.0,
            w: 120,
            h: 36,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: if unicode {
                Charset::Unicode
            } else {
                Charset::Ascii
            },
            seed: 7,
            speed: 1.0,
            tune: Tune::default(),
        }
    }

    fn all_sprites() -> Vec<Sprite> {
        let mut v = vec![SMALL, MEDIUM, LARGE];
        v.extend(STICKS_S.iter().copied());
        v.extend(STICKS_M.iter().copied());
        v.extend(STICKS_L.iter().copied());
        v
    }

    #[test]
    fn sprites_are_ascii_and_rectangular() {
        for sp in all_sprites() {
            assert!(!sp.rows.is_empty());
            for row in sp.rows {
                assert!(row.is_ascii(), "non-ascii {row:?}");
                assert!(
                    row.chars().count() as isize <= sp.w,
                    "row wider than sprite: {row:?}"
                );
            }
            if sp.tail > 0 {
                assert!(sp.tail < sp.w, "tail wider than sprite");
            }
        }
    }

    #[test]
    fn big_sprites_have_eyes_and_mouths() {
        for sp in [SMALL, MEDIUM, LARGE] {
            let joined = sp.rows.join("");
            assert!(joined.contains('o'), "no eye in {joined:?}");
            assert!(joined.contains('>'), "no mouth in {joined:?}");
        }
    }

    #[test]
    fn stick_fish_have_a_tail_a_nose_and_an_eye() {
        for sp in STICKS_S
            .iter()
            .chain(STICKS_M.iter())
            .chain(STICKS_L.iter())
            .copied()
        {
            let joined = sp.rows.join("");
            assert!(joined.contains('<'), "no tail in {joined:?}");
            assert!(joined.contains('>'), "no nose in {joined:?}");
            assert!(joined.contains('o'), "no eye in {joined:?}");
        }
    }

    #[test]
    fn shoal_has_single_and_double_layer_sticks() {
        let doubles = STICKS_M.iter().chain(STICKS_L.iter());
        assert!(doubles.into_iter().any(|s| s.rows.len() >= 2));
        assert!(STICKS_S.iter().any(|s| s.rows.len() == 1));
        // The weighted picker reaches every class.
        assert!(pick_stick(0.05, 0).rows.len() >= 3);
        assert_eq!(pick_stick(0.30, 0).rows.len(), 2);
        assert_eq!(pick_stick(0.90, 0).rows.len(), 1);
    }

    #[test]
    fn mouth_never_moves() {
        // The mouth must stay on one cell (no bobbing, no tail wave reaching
        // the head) for both headings.
        for dir in [1.0f32, -1.0] {
            let mut s = FishScene::new(7);
            s.resize(120, 36);
            s.fish.clear();
            s.fish.push(Fish {
                x: 40.0,
                y: 18.0,
                lane: None,
                speed: 0.0,
                dir,
                phase: 0.0,
                color: FISH_COLORS[0],
                sprite: LARGE,
                depth: 1,
            });
            let glyph = if dir > 0.0 { '>' } else { '<' };
            let mut spots = Vec::new();
            for step in 0..12 {
                let f = frame(step as f64 * 0.11, false);
                let mut canvas = Canvas::new(120, 36);
                s.draw(&mut canvas, &f);
                let mut found = None;
                for y in 0..36 {
                    for x in 0..120 {
                        if canvas.cells[y * 120 + x].ch == glyph {
                            found = Some((x, y));
                        }
                    }
                }
                spots.push(found.unwrap_or_else(|| panic!("no mouth at step {step}")));
            }
            assert!(
                spots.windows(2).all(|w| w[0] == w[1]),
                "mouth moved for dir {dir}: {spots:?}"
            );
        }
    }

    #[test]
    fn mirror_round_trips() {
        for ch in [
            '/', '\\', '>', '<', ',', '.', '\'', '`', ';', ':', 'o', '#', '=', '^',
        ] {
            assert_eq!(mirror_char(mirror_char(ch)), ch, "mirror {ch:?}");
        }
    }

    #[test]
    fn tank_has_five_big_fish_and_a_shoal() {
        let mut s = FishScene::new(7);
        s.resize(160, 44);
        let big = s.fish.iter().filter(|f| f.sprite.big).count();
        let small = s.fish.iter().filter(|f| !f.sprite.big).count();
        assert_eq!(big, 5, "expected 5 big fish, got {big}");
        assert!(small >= 15, "expected a shoal, got {small}");
    }

    #[test]
    fn bubbles_use_all_glyphs_and_random_shades() {
        let mut s = FishScene::new(7);
        s.resize(120, 36);
        let glyphs: std::collections::HashSet<char> = s.bubbles.iter().map(|b| b.glyph).collect();
        assert!(glyphs.contains(&'o'), "missing o");
        assert!(glyphs.contains(&'O'), "missing O");
        assert!(glyphs.contains(&'8'), "missing 8");
        let min = s.bubbles.iter().map(|b| b.shade).fold(f32::MAX, f32::min);
        let max = s.bubbles.iter().map(|b| b.shade).fold(0.0, f32::max);
        assert!(max - min > 0.3, "bubble shades too uniform: {min}..{max}");
    }

    #[test]
    fn bubbles_rise_straight_with_at_most_three_swings() {
        let mut s = FishScene::new(7);
        s.resize(120, 36);
        assert!(
            s.bubbles
                .iter()
                .all(|b| (0.0..=3.0).contains(&b.sway_cycles)),
            "a bubble sways more than three times"
        );
        for step in 0..900 {
            let f = frame(step as f64 / 30.0, false);
            s.update(&f);
            for b in &s.bubbles {
                assert!(
                    (b.x - b.base_x).abs() <= b.sway_amp + 0.01,
                    "bubble drifted sideways: x={} base={} amp={}",
                    b.x,
                    b.base_x,
                    b.sway_amp
                );
            }
        }
    }

    #[test]
    fn ascii_default_only_and_draws() {
        let mut s = FishScene::new(7);
        s.resize(120, 36);
        let mut canvas = Canvas::new(120, 36);
        for step in 0..12 {
            let f = frame(step as f64 / 30.0, false);
            s.update(&f);
            s.draw(&mut canvas, &f);
        }
        assert!(canvas.ink() > 150, "drew {} cells", canvas.ink());
        assert!(canvas.cells.iter().all(|c| c.ch.is_ascii()));
    }

    #[test]
    fn unicode_uses_blocks() {
        let mut s = FishScene::new(7);
        s.resize(120, 36);
        let mut canvas = Canvas::new(120, 36);
        let f = frame(1.0, true);
        s.update(&f);
        s.draw(&mut canvas, &f);
        assert!(
            canvas.cells.iter().any(|c| c.ch == '█'),
            "no block glyphs in unicode mode"
        );
    }

    #[test]
    fn fish_stay_inside_the_tank() {
        let mut s = FishScene::new(3);
        s.resize(90, 30);
        let mut canvas = Canvas::new(90, 30);
        for step in 0..120 {
            let f = frame(step as f64 / 30.0, false);
            s.update(&f);
            s.draw(&mut canvas, &f);
        }
        for fish in &s.fish {
            let half = fish.sprite.rows.len() as f32 * 0.5;
            assert!(fish.y - half >= 0.0, "fish above tank: {}", fish.y);
            assert!(fish.y + half <= 28.0, "fish below tank: {}", fish.y);
        }
    }

    #[test]
    fn weed_patches_vary_in_height() {
        let mut s = FishScene::new(7);
        s.resize(160, 44);
        assert!(!s.weeds.is_empty());
        let min = s.weeds.iter().map(|w| w.height).fold(f32::MAX, f32::min);
        let max = s.weeds.iter().map(|w| w.height).fold(0.0, f32::max);
        assert!(
            max - min > 4.0,
            "seaweed heights are too uniform: {min}..{max}"
        );
    }
}
