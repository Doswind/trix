//! 世界地图 — a holographic Equal Earth atlas with network arcs.
//!
//! Land/sea comes from an embedded 720x360 Natural Earth mask.  The static
//! map raster is built once per terminal size; each frame only animates the
//! scan beam, city pulses and great-circle links.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{value_noise, Rng};
use crate::scene::{Frame, Scene};
use crate::theme::Theme;
use crate::world;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ocean,
    Grid,
    Land,
    Coast,
}

#[derive(Clone, Copy)]
struct MapCell {
    ch: char,
    kind: Kind,
    terrain: f32,
}

struct Star {
    x: usize,
    y: usize,
    phase: f32,
    bright: f32,
    big: bool,
}

#[derive(Clone, Copy)]
struct City {
    code: &'static str,
    lon: f32,
    lat: f32,
}

const CITIES: &[City] = &[
    City {
        code: "NYC",
        lon: -74.0,
        lat: 40.7,
    },
    City {
        code: "LAX",
        lon: -118.2,
        lat: 34.0,
    },
    City {
        code: "LON",
        lon: -0.1,
        lat: 51.5,
    },
    City {
        code: "PAR",
        lon: 2.35,
        lat: 48.86,
    },
    City {
        code: "MOW",
        lon: 37.6,
        lat: 55.75,
    },
    City {
        code: "PEK",
        lon: 116.4,
        lat: 39.9,
    },
    City {
        code: "SHA",
        lon: 121.5,
        lat: 31.2,
    },
    City {
        code: "TYO",
        lon: 139.7,
        lat: 35.7,
    },
    City {
        code: "SIN",
        lon: 103.8,
        lat: 1.35,
    },
    City {
        code: "BOM",
        lon: 72.8,
        lat: 19.1,
    },
    City {
        code: "DXB",
        lon: 55.3,
        lat: 25.2,
    },
    City {
        code: "CAI",
        lon: 31.2,
        lat: 30.0,
    },
    City {
        code: "LOS",
        lon: 3.4,
        lat: 6.5,
    },
    City {
        code: "SAO",
        lon: -46.6,
        lat: -23.5,
    },
    City {
        code: "BUE",
        lon: -58.4,
        lat: -34.6,
    },
    City {
        code: "SYD",
        lon: 151.2,
        lat: -33.9,
    },
    City {
        code: "JNB",
        lon: 28.0,
        lat: -26.2,
    },
    City {
        code: "YYZ",
        lon: -79.4,
        lat: 43.7,
    },
];

const LINKS: &[(usize, usize)] = &[
    (0, 2),   // NYC-LON
    (0, 13),  // NYC-SAO
    (1, 7),   // LAX-TYO
    (1, 15),  // LAX-SYD
    (2, 4),   // LON-MOW
    (2, 12),  // LON-LOS
    (2, 11),  // LON-CAI
    (4, 5),   // MOW-PEK
    (5, 8),   // PEK-SIN
    (6, 7),   // SHA-TYO
    (8, 15),  // SIN-SYD
    (9, 10),  // BOM-DXB
    (10, 2),  // DXB-LON
    (11, 12), // CAI-LOS
    (13, 14), // SAO-BUE
    (16, 15), // JNB-SYD
];

struct Link {
    segments: Vec<Vec<(f32, f32, f32)>>,
    offset: f32,
    period: f32,
}

struct MapView {
    x0: usize,
    y0: usize,
    mw: usize,
    mh: usize,
    cells: Vec<MapCell>,
    city_x: Vec<usize>,
    city_y: Vec<usize>,
    label_used: Vec<bool>,
    links: Vec<Link>,
    border: bool,
}

impl MapView {
    fn at(&self, x: usize, y: usize) -> MapCell {
        self.cells[y * self.mw + x]
    }
}

pub struct MapScene {
    seed: u64,
    rng: Rng,
    w: usize,
    h: usize,
    view: Option<MapView>,
    stars: Vec<Star>,
    clock: String,
    clock_secs: u64,
}

impl MapScene {
    pub fn new(seed: u64) -> Self {
        MapScene {
            seed,
            rng: Rng::new(seed ^ 0xea27_4ce5),
            w: 0,
            h: 0,
            view: None,
            stars: Vec::new(),
            clock: String::from("--:--:--"),
            clock_secs: u64::MAX,
        }
    }

    fn rebuild(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);

        // Reserve room for title + status.  Geographic 2:1 needs four
        // columns per row on a terminal (cells are roughly 1:2).
        let margin_x = if self.w >= 50 { 3 } else { 1 };
        let avail_w = self.w.saturating_sub(margin_x * 2).max(12);
        let avail_h = self.h.saturating_sub(4).max(6);
        let mut mw = avail_w;
        let mut mh = (mw / 4).max(4);
        if mh > avail_h {
            mh = avail_h;
            mw = (mh * 4).min(avail_w);
        }
        mw = mw.max(12);
        mh = mh.max(4);
        let x0 = (self.w - mw) / 2;
        let y0 = 2 + (avail_h.saturating_sub(mh)) / 2;

        let mut cells = Vec::with_capacity(mw * mh);
        let grid_x = (mw / 12).max(2);
        let grid_y = (mh / 6).max(1);
        for y in 0..mh {
            let lat0 = 90.0 - (y as f32 / mh as f32) * 180.0;
            let lat1 = 90.0 - ((y + 1) as f32 / mh as f32) * 180.0;
            for x in 0..mw {
                let lon0 = -180.0 + (x as f32 / mw as f32) * 360.0;
                let lon1 = -180.0 + ((x + 1) as f32 / mw as f32) * 360.0;
                let cov = world::coverage(lon0, lon1, lat0, lat1, 4);
                let terrain = (value_noise(
                    (lon0 + lon1) * 0.028 + 17.0,
                    (lat0 + lat1) * 0.045 - 9.0,
                    self.seed,
                ) * 0.72
                    + value_noise(
                        (lon0 + lon1) * 0.085 - 4.0,
                        (lat0 + lat1) * 0.12 + 6.0,
                        self.seed ^ 0x51,
                    ) * 0.28)
                    .clamp(0.0, 1.0);
                let (kind, ch) = if cov >= 0.5 {
                    let ch = if terrain > 0.82 {
                        '@'
                    } else if terrain > 0.35 {
                        '#'
                    } else {
                        '*'
                    };
                    (Kind::Land, ch)
                } else if cov >= 0.12 {
                    (Kind::Coast, '+')
                } else if x % grid_x == 0 || y % grid_y == 0 {
                    (Kind::Grid, '.')
                } else {
                    (Kind::Ocean, ' ')
                };
                cells.push(MapCell { ch, kind, terrain });
            }
        }

        let city_x = CITIES
            .iter()
            .map(|c| (((c.lon + 180.0) / 360.0 * mw as f32) as usize).min(mw - 1))
            .collect();
        let city_y = CITIES
            .iter()
            .map(|c| (((90.0 - c.lat) / 180.0 * mh as f32) as usize).min(mh - 1))
            .collect();

        let mut links = Vec::new();
        for (idx, &(a, b)) in LINKS.iter().enumerate() {
            let segments = great_circle_segments(CITIES[a], CITIES[b], mw, mh, 48);
            if segments.is_empty() {
                continue;
            }
            links.push(Link {
                segments,
                offset: self.rng.f32(),
                period: self.rng.range(7.0, 15.0) + idx as f32 * 0.17,
            });
        }

        // Star field (map cells will overdraw the stars they cover).
        self.stars.clear();
        let count = (self.w * self.h / 90).max(12);
        for _ in 0..count {
            self.stars.push(Star {
                x: self.rng.below(self.w),
                y: self.rng.below(self.h),
                phase: self.rng.range(0.0, std::f32::consts::TAU),
                bright: self.rng.range(0.15, 0.8),
                big: self.rng.chance(0.12),
            });
        }

        self.view = Some(MapView {
            x0,
            y0,
            mw,
            mh,
            cells,
            city_x,
            city_y,
            label_used: vec![false; mw * mh],
            links,
            border: mw >= 24 && mh >= 8,
        });
        self.clock_secs = u64::MAX;
    }
}

fn to_vec3(lon_deg: f32, lat_deg: f32) -> [f32; 3] {
    let lon = lon_deg.to_radians();
    let lat = lat_deg.to_radians();
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

fn slerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    let omega = dot.acos();
    let so = omega.sin();
    let (wa, wb) = if so.abs() < 1e-5 {
        (1.0 - t, t)
    } else {
        (((1.0 - t) * omega).sin() / so, (t * omega).sin() / so)
    };
    [
        a[0] * wa + b[0] * wb,
        a[1] * wa + b[1] * wb,
        a[2] * wa + b[2] * wb,
    ]
}

fn great_circle_segments(
    a: City,
    b: City,
    mw: usize,
    mh: usize,
    steps: usize,
) -> Vec<Vec<(f32, f32, f32)>> {
    let pa = to_vec3(a.lon, a.lat);
    let pb = to_vec3(b.lon, b.lat);
    let dot = (pa[0] * pb[0] + pa[1] * pb[1] + pa[2] * pb[2]).clamp(-1.0, 1.0);
    if dot > 0.999_99 {
        return Vec::new();
    }
    let mut segments: Vec<Vec<(f32, f32, f32)>> = Vec::new();
    let mut current: Vec<(f32, f32, f32)> = Vec::new();
    let mut prev_x: Option<f32> = None;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let v = slerp(pa, pb, t);
        let lon = v[1].atan2(v[0]).to_degrees();
        let lat = v[2].clamp(-1.0, 1.0).asin().to_degrees();
        let x = ((lon + 180.0) / 360.0 * mw as f32).rem_euclid(mw as f32);
        let y = ((90.0 - lat) / 180.0 * mh as f32).clamp(0.0, mh as f32 - 0.001);
        if let Some(px) = prev_x {
            if (x - px).abs() > mw as f32 * 0.5 {
                if current.len() > 1 {
                    segments.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
        }
        current.push((x, y, t));
        prev_x = Some(x);
    }
    if current.len() > 1 {
        segments.push(current);
    }
    segments
}

fn utc_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn format_hms(secs: u64) -> String {
    let s = secs % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

fn terrain_color(theme: &Theme, terrain: f32, kind: Kind) -> Rgb {
    match kind {
        Kind::Land => theme
            .c(1)
            .lerp(theme.c(2), 0.3 + terrain * 0.7)
            .lerp(theme.c(3), (terrain - 0.75).max(0.0) * 1.1),
        Kind::Coast => theme.c(2).lerp(theme.c(3), 0.45 + terrain * 0.25),
        Kind::Grid => theme.c(1).scale(0.55),
        Kind::Ocean => theme.c(0),
    }
}

impl Scene for MapScene {
    fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            self.rebuild(w, h);
        }
    }

    fn update(&mut self, _f: &Frame) {
        // The map is mostly analytic; only the clock needs a syscall.
        let secs = utc_secs();
        if secs != self.clock_secs {
            self.clock_secs = secs;
            self.clock = format_hms(secs);
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.view.is_none() {
            self.rebuild(self.w, self.h);
        }
        let Some(view) = self.view.as_mut() else {
            return;
        };
        let theme = f.theme;
        let t = f.t as f32;

        // Stars behind everything.
        for s in &self.stars {
            let tw = 0.4 + 0.6 * (t * 1.3 + s.phase).sin().abs();
            let col = theme.c(2).scale(s.bright * tw * 0.35);
            let ch = if s.big { '+' } else { '.' };
            canvas.set_if_blank(s.x as isize, s.y as isize, ch, col);
        }

        let scan_frac = (f.t as f32 * 0.045).fract();
        let scan_x = scan_frac * (view.mw as f32 + 12.0) - 6.0;
        let flicker = 0.93 + 0.07 * (t * 11.0).sin();

        // The atlas raster.
        for y in 0..view.mh {
            for x in 0..view.mw {
                let cell = view.at(x, y);
                let dx = x as f32 - scan_x;
                let beam = (-(dx * dx) / 9.0).exp();
                let mut color = terrain_color(theme, cell.terrain, cell.kind);
                let mut ch = cell.ch;
                let mut scale = match cell.kind {
                    Kind::Land => 0.48 + cell.terrain * 0.28,
                    Kind::Coast => 0.82,
                    Kind::Grid => 0.55,
                    Kind::Ocean => 0.0,
                };
                scale *= flicker;
                let lat = 90.0 - (y as f32 / view.mh as f32) * 180.0;
                scale *= 1.0 - ((lat.abs() - 65.0).max(0.0) / 25.0).min(1.0) * 0.45;
                if cell.kind == Kind::Ocean && beam > 0.22 {
                    ch = '.';
                    scale = beam * 0.35;
                }
                scale += beam * 0.9;
                if cell.kind == Kind::Coast {
                    color = color.lerp(theme.c(4), beam * 0.35);
                }
                color = color.scale(scale).add(theme.hot(), beam * 0.28 * scale);
                if ch != ' ' && color.luminance() > 0.015 {
                    canvas.set((view.x0 + x) as isize, (view.y0 + y) as isize, ch, color);
                }
            }
        }

        // Scan cursor line over blank ocean cells.
        let sx = view.x0 as isize + scan_x as isize;
        if scan_x >= 0.0 && (scan_x as usize) < view.mw {
            for y in (0..view.mh).step_by(2) {
                let py = view.y0 + y;
                if canvas
                    .get(sx, py as isize)
                    .map(|c| c.ch == ' ')
                    .unwrap_or(false)
                {
                    canvas.set(sx, py as isize, '|', theme.c(3).scale(0.45));
                }
            }
        }

        // Network arcs.
        for (li, link) in view.links.iter().enumerate() {
            let phase = ((t / link.period) + link.offset).fract();
            let base = theme.c(2).hue_rotate((li % 5) as f32 * 0.035).scale(0.6);
            for seg in &link.segments {
                for (i, &(x, y, s)) in seg.iter().enumerate() {
                    if i % 2 != 0 {
                        continue;
                    }
                    let d = (phase - s).rem_euclid(1.0);
                    let (ch, col) = if d < 0.03 {
                        ('@', theme.hot())
                    } else if d < 0.12 {
                        ('*', theme.c(3).scale(0.85 - d * 4.0))
                    } else {
                        ('.', base)
                    };
                    canvas.set(
                        (view.x0 as f32 + x) as isize,
                        (view.y0 as f32 + y) as isize,
                        ch,
                        col,
                    );
                    if d < 0.03 {
                        canvas.add_light(
                            (view.x0 as f32 + x) as isize,
                            (view.y0 as f32 + y) as isize,
                            theme.c(3),
                            0.4,
                            '.',
                        );
                    }
                }
            }
        }

        // City pulses, markers and labels.
        for used in view.label_used.iter_mut() {
            *used = false;
        }
        for (i, city) in CITIES.iter().enumerate() {
            let cx = view.city_x[i] as f32;
            let cy = view.city_y[i] as f32;
            let phase = (t * 0.45 + i as f32 * 0.137).fract();
            let radius = 1.0 + phase * 3.5;
            let ring_col = theme.c(3).scale((1.0 - phase) * 0.55);
            let ax = view.x0 as isize + cx as isize;
            let ay = view.y0 as isize + cy as isize;
            for (dx, dy) in [
                (radius, 0.0),
                (-radius, 0.0),
                (0.0, radius * 0.5),
                (0.0, -radius * 0.5),
            ] {
                canvas.blend(ax + dx as isize, ay + dy as isize, '.', ring_col, 0.6);
            }
            let hot = phase < 0.18;
            canvas.set(
                ax,
                ay,
                if hot { '@' } else { '*' },
                if hot { theme.hot() } else { theme.c(3) },
            );

            if view.mw >= 70 && view.mh >= 16 {
                let label_w = city.code.len();
                let lx = if (view.city_x[i] + 1 + label_w) < view.mw {
                    view.city_x[i] + 1
                } else {
                    view.city_x[i].saturating_sub(label_w + 1)
                };
                let ly = view.city_y[i];
                let mut free = true;
                // only place labels that do not collide with an earlier one
                for k in 0..label_w {
                    let xx = lx + k;
                    if xx >= view.mw || view.label_used[ly * view.mw + xx] {
                        free = false;
                        break;
                    }
                }
                if free {
                    for (k, ch) in city.code.chars().enumerate() {
                        let xx = lx + k;
                        if xx < view.mw {
                            view.label_used[ly * view.mw + xx] = true;
                            if xx + 1 < view.mw {
                                view.label_used[ly * view.mw + xx + 1] = true;
                            }
                            canvas.set(
                                (view.x0 + xx) as isize,
                                (view.y0 + ly) as isize,
                                ch,
                                theme.c(3).scale(0.92),
                            );
                        }
                    }
                }
            }
        }

        // Frame, title, status.
        if view.border {
            canvas.rect(
                view.x0 as isize - 1,
                view.y0 as isize - 1,
                view.mw as isize + 2,
                view.mh as isize + 2,
                theme.c(1).scale(0.85),
            );
        }

        if view.y0 >= 2 {
            let title = " GLOBAL NETWORK ";
            canvas.put_str_gradient(
                view.x0 as isize,
                0,
                title,
                &[theme.c(2), theme.c(3), theme.hot()],
            );
            let sub = "// EARTH HOLO-ATLAS";
            let sub_x = view.x0 as isize + title.len() as isize + 1;
            if view.mw >= 70 && sub_x + sub.len() as isize <= view.x0 as isize + view.mw as isize {
                canvas.put_str(sub_x, 0, sub, theme.c(1).scale(1.4));
            }
            let time = format!("UTC {}", self.clock);
            let tx = view.x0 as isize + view.mw as isize - time.len() as isize;
            if view.mw >= 48 && tx > view.x0 as isize + 18 {
                canvas.put_str(tx, 0, &time, theme.c(2));
            }
            let line_y = view.y0 as isize - 1;
            for x in (view.x0 as isize..(view.x0 + view.mw) as isize).step_by(3) {
                canvas.add_light(x, line_y, theme.c(1), 0.4, '-');
            }
        }

        let status_y = self.h as isize - 1;
        let left = format!(
            " NODES {:02}   LINKS {:02}   SCAN {:03}% ",
            CITIES.len(),
            view.links.len(),
            (scan_frac * 100.0) as u32
        );
        canvas.put_str(view.x0 as isize, status_y, &left, theme.c(2).scale(0.62));
        let right = "ORBITAL UPLINK: NOMINAL ";
        let rx = view.x0 as isize + view.mw as isize - right.len() as isize;
        if rx > view.x0 as isize + 25 {
            canvas.put_str(rx, status_y, right, theme.c(2).scale(0.85));
        }
    }
}
