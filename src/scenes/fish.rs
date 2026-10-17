//! 鱼缸 — a full-screen aquarium with colorful fish.
//!
//! Big and small fish cruise in both directions, change heading at random,
//! and wag their tails.  Light rays, bubbles, seaweed and a sandy floor fill
//! the rest of the tank.

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
const FISH_COLORS: [Rgb; 6] = [
    Rgb::hex(0xf97316),
    Rgb::hex(0xfacc15),
    Rgb::hex(0x22d3ee),
    Rgb::hex(0xf472b6),
    Rgb::hex(0xa3e635),
    Rgb::hex(0x818cf8),
];

#[derive(Clone)]
struct Fish {
    x: f32,
    y: f32,
    speed: f32,
    len: f32,
    phase: f32,
    dir: f32,
    color: Rgb,
    turn: f32,
    huge: bool,
}

struct Bubble {
    x: f32,
    y: f32,
    speed: f32,
    phase: f32,
}

pub struct FishScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    fish: Vec<Fish>,
    bubbles: Vec<Bubble>,
}

impl FishScene {
    pub fn new(seed: u64) -> Self {
        FishScene {
            seed,
            rng: Rng::new(seed ^ 0x4649_5348),
            w: 0,
            h: 0,
            fish: Vec::new(),
            bubbles: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.fish.clear();
        let n = ((w * h) / 120).clamp(10, 34);
        for _ in 0..n {
            self.spawn(true);
        }
        self.bubbles.clear();
        for _ in 0..18 {
            self.bubbles.push(Bubble {
                x: self.rng.range(0.0, self.w as f32),
                y: self.rng.range(0.0, self.h as f32),
                speed: self.rng.range(1.2, 3.2),
                phase: self.rng.range(0.0, TAU),
            });
        }
    }

    fn spawn(&mut self, initial: bool) {
        let huge = self.rng.chance(0.16);
        let dir = if self.rng.chance(0.5) { 1.0 } else { -1.0 };
        let x = if initial {
            self.rng.range(0.0, self.w as f32)
        } else if dir > 0.0 {
            -6.0
        } else {
            self.w as f32 + 6.0
        };
        let color = FISH_COLORS[self.rng.below(FISH_COLORS.len())];
        self.fish.push(Fish {
            x,
            y: self.rng.range(3.0, (self.h as f32 - 5.0).max(4.0)),
            speed: self.rng.range(2.2, 7.5) * if huge { 0.7 } else { 1.0 },
            len: if huge {
                self.rng.range(7.0, 11.0)
            } else {
                self.rng.range(2.0, 5.0)
            },
            phase: self.rng.range(0.0, TAU),
            dir,
            color,
            turn: 0.0,
            huge,
        });
    }

    fn draw_water(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let w = self.w as isize;
        let h = self.h as isize;
        let ground = h - 3;
        for y in 0..h {
            let v = y as f32 / h.max(1) as f32;
            let col = DEEP.lerp(WATER, 0.25 + v * 0.45);
            for x in 0..w {
                if (x + y * 3) % 6 == 0 {
                    canvas.set(x, y, if unicode { '·' } else { '.' }, col.scale(1.6));
                } else if unicode {
                    canvas.set(x, y, '░', col.scale(0.55));
                } else {
                    canvas.set(x, y, '.', col.scale(0.45));
                }
            }
        }
        // light rays
        for k in 0..5 {
            let x0 = (self.w as f32 * (0.15 + k as f32 * 0.17)) as isize
                + ((t * 0.4 + k as f32).sin() * 3.0) as isize;
            for y in 2..ground {
                let x = x0 + (y as f32 * 0.22) as isize;
                if x >= 0 && x < w {
                    canvas.add_light(x, y, LIGHT, 0.035, if unicode { '░' } else { ':' });
                }
            }
        }
        // caustic sparkles
        for i in 0..(self.w * self.h / 90).max(24) {
            let x = ((i * 37 + (t * 7.0) as usize) % self.w) as isize;
            let y = ((i * 53) % (self.h - 5).max(1)) as isize;
            canvas.set_if_blank(x, y, if unicode { '·' } else { '.' }, LIGHT.scale(0.18));
        }
        // sand
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
                } else {
                    if unicode {
                        '█'
                    } else {
                        '#'
                    }
                };
                canvas.set(x, y, ch, SAND.scale(0.38 + (y - ground) as f32 * 0.06));
            }
        }
        // seaweed and coral
        for (i, cx) in [3isize, 9, 15, w / 2, w - 12, w - 6].iter().enumerate() {
            let base = ground - 1;
            let height = 5 + (i * 3) % 8;
            for k in 0..height {
                let x = cx + ((t * 0.8 + i as f32 + k as f32 * 0.3).sin() * 1.6) as isize;
                canvas.set(
                    x,
                    base - k as isize,
                    if unicode { '║' } else { '|' },
                    WEED.scale(0.65),
                );
                if k % 3 == 0 {
                    canvas.set(x - 1, base - k as isize, '/', WEED_DARK);
                    canvas.set(x + 1, base - k as isize, '\\', WEED_DARK);
                }
            }
        }
    }

    fn draw_fish(&self, canvas: &mut Canvas, f: &Fish, unicode: bool, t: f32) {
        let x = f.x.round() as isize;
        let bob = ((t * 2.0 + f.phase).sin() * 0.8).round() as isize;
        let y = f.y.round() as isize + bob;
        let dir = if f.dir >= 0.0 { 1 } else { -1 };
        let tail = if unicode { '◀' } else { '<' };
        let tail2 = if unicode { '▶' } else { '>' };
        let body = if unicode { '█' } else { '=' };
        let len = f.len.round() as isize;
        let color = f.color;
        let head = if dir > 0 { tail2 } else { tail };
        let tailch = if dir > 0 { tail } else { tail2 };
        // tail, slightly above/below to suggest a swishing pose
        let tw = ((t * 6.0 + f.phase).sin() * 1.4).round() as isize;
        canvas.set(x - dir * len, y + tw, tailch, color.scale(0.8));
        canvas.set(x - dir * len, y + tw + 1, tailch, color.scale(0.55));
        for k in 1..len {
            let bx = x - dir * k;
            let ch = if unicode {
                if f.huge {
                    '▓'
                } else {
                    '▰'
                }
            } else if f.huge {
                '='
            } else {
                '-'
            };
            canvas.set(bx, y, body, color);
            if f.huge && k % 2 == 0 {
                canvas.set(bx, y + 1, ch, color.scale(0.7));
            }
        }
        canvas.set(x, y, head, color.lerp(Rgb::hex(0xffffff), 0.25));
        // eye
        canvas.set(
            x - dir,
            y,
            if unicode { '·' } else { 'o' },
            Rgb::hex(0xffffff),
        );
        // pectoral fin
        canvas.set(
            x - dir * 2,
            y + 1,
            if unicode { '▴' } else { '^' },
            color.scale(0.6),
        );
        if f.huge {
            canvas.add_light(x, y, color, 0.25, '.');
        }
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
        for fish in &mut self.fish {
            fish.x += fish.speed * fish.dir * dt;
            fish.phase += dt * 1.5;
            fish.y += ((f.t as f32 * 1.3 + fish.phase).sin() * 1.2) * dt;
            fish.turn -= dt;
            if fish.turn <= 0.0 && self.rng.chance(0.35) {
                fish.dir = -fish.dir;
                fish.turn = self.rng.range(2.0, 7.0);
            }
            if fish.dir > 0.0 && fish.x > self.w as f32 + 8.0 {
                fish.x = -8.0;
                fish.y = self.rng.range(3.0, (self.h as f32 - 5.0).max(4.0));
            } else if fish.dir < 0.0 && fish.x < -8.0 {
                fish.x = self.w as f32 + 8.0;
                fish.y = self.rng.range(3.0, (self.h as f32 - 5.0).max(4.0));
            }
            fish.y = fish.y.clamp(3.0, (self.h as f32 - 4.0).max(4.0));
        }
        for b in &mut self.bubbles {
            b.y -= b.speed * dt;
            b.x += ((f.t as f32 * 2.0 + b.phase).sin() * 0.3) * dt;
            if b.y < 1.0 {
                b.y = self.h as f32 - 4.0;
                b.x = self.rng.range(0.0, self.w as f32);
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        self.draw_water(canvas, unicode, t);
        for fish in &self.fish {
            self.draw_fish(canvas, fish, unicode, t);
        }
        for b in &self.bubbles {
            canvas.set(
                b.x.round() as isize,
                b.y.round() as isize,
                if unicode { '○' } else { 'o' },
                BUBBLE.scale(0.55),
            );
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
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tank_has_fish() {
        let mut s = FishScene::new(7);
        s.resize(120, 36);
        assert!(s.fish.len() >= 10);
        assert!(s.fish.iter().any(|f| f.huge));
    }
}
