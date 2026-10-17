//! 车水马龙 — a neon city street.
//!
//! Horizontal by default (buildings above/below, traffic through the middle)
//! and `--street vertical` for a canyon of towers with traffic flowing up and
//! down.  ASCII by default; `--unicode` uses block/box glyphs.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::color::Rgb;
use crate::rng::{hash01, Rng};
use crate::scene::{Frame, Scene};

const NIGHT: Rgb = Rgb::hex(0x070b16);
const BUILD: Rgb = Rgb::hex(0x1e293b);
const ROAD: Rgb = Rgb::hex(0x111827);
const LINE: Rgb = Rgb::hex(0xfacc15);
const CYAN: Rgb = Rgb::hex(0x22d3ee);
const PINK: Rgb = Rgb::hex(0xf472b6);
const GOLD: Rgb = Rgb::hex(0xfbbf24);
const WHITE: Rgb = Rgb::hex(0xe0f2fe);
const CAR_COLORS: [Rgb; 6] = [
    Rgb::hex(0xef4444),
    Rgb::hex(0x3b82f6),
    Rgb::hex(0xfacc15),
    Rgb::hex(0x22c55e),
    Rgb::hex(0xf472b6),
    Rgb::hex(0xe0f2fe),
];

#[derive(Clone)]
struct Car {
    pos: f32,
    lane: f32,
    speed: f32,
    dir: f32,
    len: usize,
    color: Rgb,
}

pub struct TrafficScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    cars: Vec<Car>,
}

impl TrafficScene {
    pub fn new(seed: u64) -> Self {
        TrafficScene {
            seed,
            rng: Rng::new(seed ^ 0x5452_4146),
            w: 0,
            h: 0,
            cars: Vec::new(),
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        self.cars.clear();
        let n = ((w * h) / 150).clamp(12, 30);
        for i in 0..n {
            let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
            self.cars.push(Car {
                pos: self
                    .rng
                    .range(0.0, (if self.w > self.h { self.w } else { self.h }) as f32),
                lane: self.rng.f32(),
                speed: self.rng.range(6.0, 18.0),
                dir,
                len: self.rng.below(3) + 3,
                color: CAR_COLORS[self.rng.below(CAR_COLORS.len())],
            });
        }
    }

    fn vertical(&self, f: &Frame) -> bool {
        f.tune.street_vertical
    }

    fn draw_buildings_h(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let w = self.w as isize;
        let h = self.h as isize;
        let road_top = (self.h as f32 * 0.42) as isize;
        let road_bot = (self.h as f32 * 0.80) as isize;
        // top skyline
        let mut x = 0isize;
        while x < w {
            let bw = 3 + (hash01(x as i64, 1, self.seed) * 5.0) as isize;
            let bh = (self.h as f32 * 0.08 + hash01(x as i64, 2, self.seed) * self.h as f32 * 0.30)
                as isize;
            for y in (road_top - bh)..road_top {
                for xx in x..(x + bw).min(w) {
                    let window = (xx - x) > 0 && (xx - x) < bw - 1 && (xx + y) % 3 != 0;
                    let on = hash01(xx as i64, y as i64, self.seed + 3) > 0.48;
                    let ch = if window && on {
                        if unicode {
                            '▦'
                        } else {
                            '#'
                        }
                    } else if unicode {
                        '▓'
                    } else {
                        ':'
                    };
                    let c = if window && on {
                        [CYAN, PINK, GOLD][((xx + y) as usize) % 3].scale(0.82)
                    } else {
                        BUILD.scale(1.45)
                    };
                    canvas.set(xx, y, ch, c);
                }
            }
            x += bw + 1;
        }
        // lower buildings below the road
        let mut xx = 0isize;
        while xx < w {
            let bw = 4 + (hash01(xx as i64, 7, self.seed) * 5.0) as isize;
            let bh = (self.h as f32 * 0.05 + hash01(xx as i64, 8, self.seed) * self.h as f32 * 0.10)
                as isize;
            for y in road_bot..(road_bot + bh).min(h) {
                for x in xx..(xx + bw).min(w) {
                    let ch = if (x + y) % 2 == 0 {
                        if unicode {
                            '▪'
                        } else {
                            '#'
                        }
                    } else if unicode {
                        '▒'
                    } else {
                        ':'
                    };
                    canvas.set(x, y, ch, BUILD.scale(0.95));
                }
            }
            xx += bw + 1;
        }
        // sidewalks + road
        for y in road_top..road_bot {
            for x in 0..w {
                let ch = if y == road_top || y == road_bot - 1 {
                    if unicode {
                        '═'
                    } else {
                        '='
                    }
                } else if (x + (t * 18.0) as isize) % 9 == 0 {
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
                let c = if y == road_top || y == road_bot - 1 {
                    LINE.scale(0.42)
                } else {
                    ROAD
                };
                canvas.set(x, y, ch, c);
            }
        }
        // neon signs
        for k in 0..5 {
            let sx = (self.w as f32 * (0.08 + k as f32 * 0.20)) as isize;
            let sy = road_top - 4 - (k % 2) * 3;
            for i in 0..4 {
                canvas.set(
                    sx + i,
                    sy,
                    if unicode { '▰' } else { '=' },
                    [CYAN, PINK, GOLD][(k % 3) as usize].scale(0.7),
                );
            }
        }
    }

    fn draw_buildings_v(&self, canvas: &mut Canvas, unicode: bool, t: f32) {
        let w = self.w as isize;
        let h = self.h as isize;
        let road_l = (self.w as f32 * 0.38) as isize;
        let road_r = (self.w as f32 * 0.62) as isize;
        for side in 0..2 {
            let (x0, x1) = if side == 0 { (0, road_l) } else { (road_r, w) };
            let mut y = 0isize;
            while y < h {
                let bh = 4 + (hash01(y as i64, side as i64, self.seed) * 8.0) as isize;
                for yy in y..(y + bh).min(h) {
                    for x in x0..x1 {
                        let window = x > x0 + 1 && x < x1 - 2 && (x + yy) % 3 != 0;
                        let on = hash01(x as i64, yy as i64, self.seed + 5) > 0.50;
                        let ch = if window && on {
                            if unicode {
                                '▦'
                            } else {
                                '='
                            }
                        } else if unicode {
                            '▓'
                        } else {
                            '#'
                        };
                        let c = if window && on {
                            [PINK, CYAN, GOLD][((x + yy) as usize) % 3].scale(0.82)
                        } else {
                            BUILD.scale(1.05)
                        };
                        canvas.set(x, yy, ch, c);
                    }
                }
                y += bh + 1;
            }
        }
        for y in 0..h {
            for x in road_l..road_r {
                let ch = if x == road_l || x == road_r - 1 {
                    if unicode {
                        '║'
                    } else {
                        '|'
                    }
                } else if (y + (t * 16.0) as isize) % 8 == 0 {
                    if unicode {
                        '┄'
                    } else {
                        '-'
                    }
                } else if unicode {
                    '·'
                } else {
                    '.'
                };
                let c = if x == road_l || x == road_r - 1 {
                    LINE.scale(0.42)
                } else {
                    ROAD
                };
                canvas.set(x, y, ch, c);
            }
        }
    }

    fn draw_cars_h(&self, canvas: &mut Canvas, unicode: bool, f: &Frame) {
        let road_top = (self.h as f32 * 0.42) as isize;
        let road_bot = (self.h as f32 * 0.80) as isize;
        let lanes = (road_bot - road_top - 2).max(1);
        for car in &self.cars {
            let lane = road_top + 1 + (car.lane * lanes as f32) as isize % lanes;
            let x = car.pos.round() as isize;
            let dir = if car.dir > 0.0 { 1 } else { -1 };
            let head = if dir > 0 { '>' } else { '<' };
            let body = if unicode { '▰' } else { '=' };
            for k in 0..car.len as isize {
                canvas.set(x - dir * k, lane, body, car.color.scale(0.9));
            }
            canvas.set(x, lane, head, WHITE);
            canvas.set(
                x - dir,
                lane,
                if unicode { '▫' } else { 'o' },
                WHITE.scale(0.7),
            );
            if unicode {
                canvas.set(x - dir * 2, lane + 1, '▴', car.color.scale(0.6));
            }
            // headlight glow
            canvas.add_light(x + dir, lane, WHITE, 0.28, '.');
            canvas.add_light(x - dir * (car.len as isize - 1), lane, car.color, 0.20, '.');
        }
        let _ = f;
    }

    fn draw_cars_v(&self, canvas: &mut Canvas, unicode: bool, _f: &Frame) {
        let road_l = (self.w as f32 * 0.38) as isize;
        let road_r = (self.w as f32 * 0.62) as isize;
        let lanes = (road_r - road_l - 2).max(1);
        for car in &self.cars {
            let lane = road_l + 1 + (car.lane * lanes as f32) as isize % lanes;
            let y = car.pos.round() as isize;
            let dir = if car.dir > 0.0 { 1 } else { -1 };
            let head = if dir > 0 { 'v' } else { '^' };
            for k in 0..car.len as isize {
                canvas.set(
                    lane,
                    y - dir * k,
                    if unicode { '█' } else { '#' },
                    car.color.scale(0.85),
                );
                if unicode {
                    canvas.set(lane + 1, y - dir * k, '▰', car.color.scale(0.7));
                }
            }
            canvas.set(lane, y, head, WHITE);
            canvas.add_light(lane, y + dir, WHITE, 0.25, '.');
        }
    }
}

impl Scene for TrafficScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, f: &Frame) {
        let dt = f.dt.clamp(0.0, 0.1);
        let vertical = self.vertical(f);
        let span = if vertical {
            self.h as f32
        } else {
            self.w as f32
        };
        for car in &mut self.cars {
            car.pos += car.speed * car.dir * dt;
            if car.pos > span + 8.0 {
                car.pos = -8.0;
            } else if car.pos < -8.0 {
                car.pos = span + 8.0;
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        let t = f.t as f32;
        for y in 0..self.h as isize {
            let v = y as f32 / self.h.max(1) as f32;
            for x in 0..self.w as isize {
                if (x + y * 3) % 9 == 0 {
                    canvas.set(
                        x,
                        y,
                        if unicode { '·' } else { '.' },
                        NIGHT.lerp(CYAN, v * 0.08).scale(1.5),
                    );
                }
            }
        }
        if self.vertical(f) {
            self.draw_buildings_v(canvas, unicode, t);
            self.draw_cars_v(canvas, unicode, f);
        } else {
            self.draw_buildings_h(canvas, unicode, t);
            self.draw_cars_h(canvas, unicode, f);
        }
        let w = self.w as isize;
        let h = self.h as isize;
        let title = if unicode {
            "╡ 车水马龙 // NEON STREET ╞"
        } else {
            " TRAFFIC // NEON STREET "
        };
        canvas.put_str(2, 0, title, PINK.scale(0.9));
        let mode = if self.vertical(f) {
            "VERTICAL"
        } else {
            "HORIZONTAL"
        };
        let status = format!("{}  {}", mode, if unicode { "UNICODE" } else { "ASCII" });
        canvas.put_str(
            (w - status.len() as isize - 2).max(2),
            0,
            &status,
            GOLD.scale(0.65),
        );
        canvas.put_str(2, h - 1, "--street horizontal | vertical", CYAN.scale(0.45));
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Tune;
    #[test]
    fn has_cars() {
        let mut s = TrafficScene::new(7);
        s.resize(120, 36);
        assert!(s.cars.len() > 8);
        let t = Tune {
            street_vertical: true,
            ..Tune::default()
        };
        assert!(t.street_vertical);
    }
}
