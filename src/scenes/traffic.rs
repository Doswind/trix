//! 车水马龙 — a neon city street.
//!
//! Horizontal by default (buildings above/below, traffic through the middle)
//! and `--street vertical` for a canyon of towers with traffic flowing up and
//! down.  ASCII by default; `--unicode` uses block/box glyphs.
//!
//! Horizontal vehicles are three-row side profiles (roof / body / wheels) and
//! vertical ones are six-column top-down bodies, instead of single-row dashes:
//! the roadway is split into a handful of real lanes, opposing traffic is
//! separated by lane, and each lane keeps one speed so cars keep their spacing
//! instead of sliding through each other.

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
const TAIL: Rgb = Rgb::hex(0xff3b3b);
const CAR_COLORS: [Rgb; 6] = [
    Rgb::hex(0xef4444),
    Rgb::hex(0x3b82f6),
    Rgb::hex(0xfacc15),
    Rgb::hex(0x22c55e),
    Rgb::hex(0xf472b6),
    Rgb::hex(0xe0f2fe),
];

/// Horizontal street: top and bottom of the roadway, as a fraction of `h`.
const ROAD_TOP: f32 = 0.36;
const ROAD_BOT: f32 = 0.84;
/// Vertical street: left and right edge of the roadway, as a fraction of `w`.
const ROAD_LEFT: f32 = 0.34;
const ROAD_RIGHT: f32 = 0.66;

/// Rows per lane on a horizontal street: three-row car + one gap row.
const CAR_SPAN_H: isize = 3;
const LANE_GAP_H: isize = 1;
/// Columns per lane on a vertical street: top-down car + glow gap.
const CAR_SPAN_V: isize = 6;
const LANE_GAP_V: isize = 4;
/// Never crowd the roadway with more lanes than this.
const LANES_MAX: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CarKind {
    Sedan,
    Van,
    Bus,
}

impl CarKind {
    fn pick(rng: &mut Rng) -> Self {
        match rng.below(10) {
            0..=5 => CarKind::Sedan,
            6..=8 => CarKind::Van,
            _ => CarKind::Bus,
        }
    }

    /// Body length in cells, front bumper included.
    fn len(self, rng: &mut Rng) -> usize {
        match self {
            CarKind::Sedan => 8 + rng.below(3),
            CarKind::Van => 11 + rng.below(3),
            CarKind::Bus => 14 + rng.below(3),
        }
    }

    /// Terminal cells are twice as tall as they are wide, so a top-down car
    /// stays short on screen: 5–9 rows for a six-column body.
    fn len_v(self, rng: &mut Rng) -> usize {
        match self {
            CarKind::Sedan => 5 + rng.below(2),
            CarKind::Van => 7 + rng.below(1),
            CarKind::Bus => 8 + rng.below(2),
        }
    }
}

#[derive(Clone)]
struct Car {
    /// Head (front bumper) position along the street axis.
    pos: f32,
    lane: usize,
    dir: f32,
    speed: f32,
    len: usize,
    kind: CarKind,
    color: Rgb,
}

pub struct TrafficScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    vertical: bool,
    density: f32,
    lanes: usize,
    cars: Vec<Car>,
}

impl TrafficScene {
    pub fn new(seed: u64) -> Self {
        TrafficScene {
            seed,
            rng: Rng::new(seed ^ 0x5452_4146),
            w: 0,
            h: 0,
            vertical: false,
            density: 1.0,
            lanes: 0,
            cars: Vec::new(),
        }
    }

    /// Roadway extent on the cross axis (rows for horizontal, columns for
    /// vertical); returns `(first, last_exclusive)`.
    fn road_bounds(&self, vertical: bool) -> (isize, isize) {
        if vertical {
            let l = (self.w as f32 * ROAD_LEFT) as isize;
            let r = (self.w as f32 * ROAD_RIGHT) as isize;
            (l.max(1), r.max(l + 6))
        } else {
            let t = (self.h as f32 * ROAD_TOP) as isize;
            let b = (self.h as f32 * ROAD_BOT) as isize;
            (t.max(2), b.max(t + 6))
        }
    }

    fn lane_pitch(vertical: bool) -> isize {
        Self::car_span(vertical) + Self::lane_gap(vertical)
    }

    fn car_span(vertical: bool) -> isize {
        if vertical {
            CAR_SPAN_V
        } else {
            CAR_SPAN_H
        }
    }

    fn lane_gap(vertical: bool) -> isize {
        if vertical {
            LANE_GAP_V
        } else {
            LANE_GAP_H
        }
    }

    /// Lanes that fit in the roadway between the two curbs.
    fn lane_count(&self, vertical: bool) -> usize {
        let (lo, hi) = self.road_bounds(vertical);
        let interior = (hi - lo - 2).max(1);
        let span = Self::car_span(vertical);
        if interior <= span {
            return 1;
        }
        let lanes = (interior - span) / Self::lane_pitch(vertical) + 1;
        lanes.clamp(1, LANES_MAX as isize) as usize
    }

    /// Cross-axis coordinate of lane `lane`'s first car row/column.
    fn lane_start(&self, vertical: bool, lane: usize) -> isize {
        let (lo, hi) = self.road_bounds(vertical);
        let span = Self::car_span(vertical);
        let interior = (hi - lo - 2).max(1);
        if self.lanes <= 1 {
            return lo + 1 + (interior - span).max(0) / 2;
        }
        // Spread the lanes over the whole roadway instead of packing them at
        // the top and leaving a wide empty strip at the far curb.
        let step = ((interior - span) / (self.lanes as isize - 1)).max(1);
        let used = span + step * (self.lanes as isize - 1);
        let pad = (interior - used).max(0);
        lo + 1 + pad / 2 + lane as isize * step
    }

    fn rebuild(&mut self) {
        self.lanes = self.lane_count(self.vertical);
        self.cars.clear();
        let span = if self.vertical {
            self.h as f32
        } else {
            self.w as f32
        }
        .max(1.0);
        // `--density` scales how many cars share a lane (default 1.0 = 3/2).
        let per_lane = if self.vertical {
            (2.0 * self.density).round() as usize
        } else {
            (3.0 * self.density).round() as usize
        }
        .clamp(1, 8);
        for lane in 0..self.lanes {
            // Opposite directions live on opposite lanes, never in one lane.
            let dir = if lane % 2 == 0 { 1.0 } else { -1.0 };
            let speed = self.rng.range(9.0, 17.0);
            let offset = self.rng.range(0.0, span / per_lane as f32);
            for k in 0..per_lane {
                let kind = CarKind::pick(&mut self.rng);
                let len = if self.vertical {
                    kind.len_v(&mut self.rng)
                } else {
                    kind.len(&mut self.rng)
                };
                let color = CAR_COLORS[self.rng.below(CAR_COLORS.len())];
                let pos = (offset + span * k as f32 / per_lane as f32) % span;
                self.cars.push(Car {
                    pos,
                    lane,
                    dir,
                    speed,
                    len,
                    kind,
                    color,
                });
            }
        }
    }

    fn is_vertical(&self, f: &Frame) -> bool {
        f.tune.street_vertical
    }

    fn road_surface_h(&self, canvas: &mut Canvas, unicode: bool) {
        let w = self.w as isize;
        let (road_top, road_bot) = self.road_bounds(false);
        for y in road_top..road_bot {
            for x in 0..w {
                if y == road_top || y == road_bot - 1 {
                    let ch = if unicode { '═' } else { '=' };
                    canvas.set(x, y, ch, LINE.scale(0.50));
                    continue;
                }
                // Static asphalt: changing the dot colour every frame would
                // light up the whole roadway in the ANSI diff for nothing.
                let ch = if unicode { '·' } else { '.' };
                canvas.set(x, y, ch, ROAD.scale(1.15));
            }
        }
        // Lane dividers: dashed paint between the lanes, never over a car.
        for lane in 0..self.lanes.saturating_sub(1) {
            let gap = self.lane_start(false, lane + 1) - 1;
            if gap <= road_top || gap >= road_bot - 1 {
                continue;
            }
            for x in 0..w {
                if x % 16 < 9 {
                    let ch = if unicode { '┄' } else { '-' };
                    canvas.set(x, gap, ch, LINE.scale(0.34));
                }
            }
        }
    }

    fn draw_buildings_h(&self, canvas: &mut Canvas, unicode: bool) {
        let w = self.w as isize;
        let h = self.h as isize;
        let (road_top, road_bot) = self.road_bounds(false);
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
        self.road_surface_h(canvas, unicode);
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

    fn draw_buildings_v(&self, canvas: &mut Canvas, unicode: bool) {
        let w = self.w as isize;
        let h = self.h as isize;
        let (road_l, road_r) = self.road_bounds(true);
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
                } else if y % 11 == 0 {
                    if unicode {
                        '·'
                    } else {
                        '.'
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
        // lane dividers between the vertical lanes
        for lane in 0..self.lanes.saturating_sub(1) {
            let gap = self.lane_start(true, lane + 1) - 1;
            if gap <= road_l || gap >= road_r - 1 {
                continue;
            }
            for y in 0..h {
                if y % 16 < 9 {
                    let ch = if unicode { '┆' } else { '|' };
                    canvas.set(gap, y, ch, LINE.scale(0.30));
                }
            }
        }
    }

    fn draw_cars_h(&self, canvas: &mut Canvas, unicode: bool) {
        let body_ch = if unicode { '█' } else { '#' };
        let roof_ch = if unicode { '▄' } else { '_' };
        let wheel_ch = if unicode { '▀' } else { 'o' };
        let glow_ch = if unicode { '·' } else { '.' };
        for car in &self.cars {
            let roof_y = self.lane_start(false, car.lane);
            let body_y = roof_y + 1;
            let wheel_y = roof_y + 2;
            let dir = if car.dir > 0.0 { 1 } else { -1 };
            let head = car.pos.round() as isize;
            let len = car.len as isize;
            let rear = head - dir * (len - 1);
            for k in 0..len {
                let x = head - dir * k;
                canvas.set(x, body_y, body_ch, car.color);
                // Cabin: a short roof line over the middle of the body.  Kept
                // short so the roof row does not read as a detached bar.
                let cabin = match car.kind {
                    CarKind::Sedan => 4,
                    CarKind::Van => 6,
                    CarKind::Bus => 10,
                }
                .min(len - 4)
                .max(2);
                let first = (len - cabin) / 2;
                if k >= first && k < first + cabin {
                    canvas.set(x, roof_y, roof_ch, car.color.lerp(WHITE, 0.42));
                }
            }
            canvas.set(
                rear,
                body_y,
                if dir > 0 { '[' } else { ']' },
                car.color.scale(0.55),
            );
            canvas.set(head, body_y, if dir > 0 { '>' } else { '<' }, WHITE);
            for k in [2, len - 3] {
                if k > 0 && k < len - 1 {
                    canvas.set(head - dir * k, wheel_y, wheel_ch, car.color.scale(0.32));
                }
            }
            // Headlight cone in front, tail glow behind.
            for b in 1..=3 {
                canvas.add_light(
                    head + dir * b,
                    body_y,
                    WHITE,
                    0.30 - b as f32 * 0.07,
                    glow_ch,
                );
            }
            for b in 1..=2 {
                canvas.add_light(
                    rear - dir * b,
                    body_y,
                    TAIL,
                    0.26 - b as f32 * 0.08,
                    glow_ch,
                );
            }
        }
    }

    fn draw_cars_v(&self, canvas: &mut Canvas, unicode: bool) {
        let body_ch = if unicode { '█' } else { '#' };
        let roof_ch = if unicode { '▬' } else { '=' };
        let wheel_ch = if unicode { '▀' } else { 'o' };
        let glow_ch = if unicode { '·' } else { '.' };
        let span = CAR_SPAN_V;
        for car in &self.cars {
            let left = self.lane_start(true, car.lane);
            let dir = if car.dir > 0.0 { 1 } else { -1 };
            let head = car.pos.round() as isize;
            let len = car.len as isize;
            let rear = head - dir * (len - 1);
            let cabin = match car.kind {
                CarKind::Sedan => 1,
                CarKind::Van => 2,
                CarKind::Bus => 2,
            }
            .min(len - 3)
            .max(1);
            // Windscreen just behind the nose, rear window just before the boot.
            let cabin_first = 1;
            for k in 0..len {
                let y = head - dir * k;
                // Nose and boot are one column narrower: a rounded silhouette.
                let (c0, c1) = if k == 0 || k == len - 1 {
                    (1, span - 1)
                } else {
                    (0, span)
                };
                for c in c0..c1 {
                    canvas.set(left + c, y, body_ch, car.color);
                }
                let window =
                    (k >= cabin_first && k < cabin_first + cabin) || (len >= 8 && k == len - 2);
                if window && k > 0 && k < len - 1 {
                    for c in 1..span - 1 {
                        canvas.set(left + c, y, roof_ch, car.color.lerp(WHITE, 0.34));
                    }
                }
            }
            // Four wheels on the outer columns.
            for k in [2, len - 3] {
                if k > 1 && k < len - 2 {
                    let y = head - dir * k;
                    canvas.set(left, y, wheel_ch, car.color.scale(0.30));
                    canvas.set(left + span - 1, y, wheel_ch, car.color.scale(0.30));
                }
            }
            // Front row = white headlights, rear row = red tail lights.
            canvas.set(left + 1, head, wheel_ch, WHITE);
            canvas.set(left + span - 2, head, wheel_ch, WHITE);
            canvas.set(left + 1, rear, wheel_ch, TAIL.scale(0.85));
            canvas.set(left + span - 2, rear, wheel_ch, TAIL.scale(0.85));
            for b in 1..=2 {
                canvas.add_light(
                    left + span / 2,
                    head + dir * b,
                    WHITE,
                    0.26 - b as f32 * 0.08,
                    glow_ch,
                );
                canvas.add_light(
                    left + span / 2,
                    rear - dir * b,
                    TAIL,
                    0.20 - b as f32 * 0.06,
                    glow_ch,
                );
            }
        }
    }
}

impl Scene for TrafficScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.w = w.max(1);
            self.h = h.max(1);
            self.lanes = 0;
            self.cars.clear();
        }
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        Some(
            if self.is_vertical(f) {
                "VERTICAL"
            } else {
                "HORIZONTAL"
            }
            .to_string(),
        )
    }

    fn update(&mut self, f: &Frame) {
        let vertical = self.is_vertical(f);
        let density = f.tune.density.clamp(0.05, 4.0);
        if self.cars.is_empty()
            || self.vertical != vertical
            || (density - self.density).abs() > 0.01
        {
            self.vertical = vertical;
            self.density = density;
            self.rebuild();
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let span = if vertical {
            self.h as f32
        } else {
            self.w as f32
        };
        for car in &mut self.cars {
            let margin = car.len as f32 + 6.0;
            car.pos += car.speed * car.dir * dt;
            if car.pos > span + margin {
                car.pos = -margin;
            } else if car.pos < -margin {
                car.pos = span + margin;
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        let unicode = f.charset == Charset::Unicode;
        if self.cars.is_empty() {
            self.vertical = self.is_vertical(f);
            self.rebuild();
        }
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
        if self.is_vertical(f) {
            self.draw_buildings_v(canvas, unicode);
            self.draw_cars_v(canvas, unicode);
        } else {
            self.draw_buildings_h(canvas, unicode);
            self.draw_cars_h(canvas, unicode);
        }
        let _ = self.seed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Tune;

    fn frame(w: usize, h: usize, vertical: bool) -> Frame<'static> {
        Frame {
            t: 1.0,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme: &crate::theme::THEMES[0],
            charset: Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: Tune {
                street_vertical: vertical,
                ..Tune::default()
            },
        }
    }

    #[test]
    fn lays_out_a_few_real_lanes() {
        let mut s = TrafficScene::new(7);
        s.resize(200, 40);
        s.update(&frame(200, 40, false));
        assert_eq!(s.lanes, 4, "200x40 should give four lanes");
        assert_eq!(s.cars.len(), 12, "three cars per lane");
        assert!(s.cars.iter().all(|c| (8..=16).contains(&c.len)));
    }

    #[test]
    fn density_scales_the_traffic() {
        let mut s = TrafficScene::new(3);
        s.resize(160, 40);
        let mut f = frame(160, 40, false);
        s.update(&f);
        let normal = s.cars.len();
        f.tune.density = 2.0;
        s.update(&f);
        assert!(s.cars.len() > normal, "density 2 should add cars");
    }

    #[test]
    fn opposing_traffic_keeps_to_its_own_lane() {
        let mut s = TrafficScene::new(11);
        s.resize(200, 40);
        s.update(&frame(200, 40, false));
        for lane in 0..s.lanes {
            let dirs: Vec<f32> = s
                .cars
                .iter()
                .filter(|c| c.lane == lane)
                .map(|c| c.dir)
                .collect();
            assert!(!dirs.is_empty());
            assert!(
                dirs.iter().all(|d| *d == dirs[0]),
                "lane {lane} mixes directions"
            );
        }
        assert!(s.lanes > 1);
        assert_ne!(
            s.cars[0].dir,
            s.cars.iter().find(|c| c.lane == 1).unwrap().dir,
            "neighbouring lanes should flow opposite ways"
        );
    }

    #[test]
    fn cars_stay_inside_the_roadway() {
        let mut s = TrafficScene::new(3);
        s.resize(120, 36);
        s.update(&frame(120, 36, false));
        let (road_top, road_bot) = s.road_bounds(false);
        for car in &s.cars {
            let roof = s.lane_start(false, car.lane);
            assert!(roof > road_top, "car roof above the road");
            assert!(
                roof + TrafficScene::car_span(false) < road_bot,
                "car wheels below the road"
            );
        }
    }

    #[test]
    fn vertical_traffic_uses_wide_lanes() {
        let mut s = TrafficScene::new(5);
        s.resize(160, 40);
        s.update(&frame(160, 40, true));
        assert!(s.lanes >= 2);
        assert!(!s.cars.is_empty());
        let (road_l, road_r) = s.road_bounds(true);
        for car in &s.cars {
            let left = s.lane_start(true, car.lane);
            assert!(left >= road_l);
            assert!(left + TrafficScene::car_span(true) <= road_r);
        }
    }

    #[test]
    fn ascii_default_only() {
        let mut s = TrafficScene::new(7);
        s.resize(120, 36);
        let f = frame(120, 36, false);
        s.update(&f);
        let mut canvas = Canvas::new(120, 36);
        for _ in 0..10 {
            s.update(&f);
            s.draw(&mut canvas, &f);
        }
        assert!(canvas.ink() > 100);
        assert!(canvas.cells.iter().all(|c| c.ch.is_ascii()));
    }
}
