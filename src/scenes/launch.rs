//! launch — a cinematic rocket launch, from the pad to orbit.
//!
//! A twenty-four second loop: blue-hour pad, ignition, ascent, max-Q, MECO and
//! booster separation, a second-stage climb and an Earth-limb sunrise.  The
//! rocket is drawn at terminal scale (roughly 45% of the screen height) with
//! layered exhaust, world-space smoke, a parallax sky and a small camera shake.
//! Everything is glyphs; the default is pure ASCII.

#![allow(clippy::needless_range_loop)]

use std::f32::consts::TAU;

use crate::canvas::Canvas;
use crate::color::Rgb;
use crate::rng::{hash01, value_noise, Rng};
use crate::scene::{Frame, Scene};

const CYCLE: f32 = 24.0;
const T_IGNITION: f32 = 4.0;
const T_LIFTOFF: f32 = 4.6;
const T_MAX_Q: f32 = 9.0;
const T_MECO: f32 = 12.0;
const T_STAGE2: f32 = 13.0;
const T_ORBIT: f32 = 16.0;
const T_SUNRISE: f32 = 18.5;
const T_FADE_OUT: f32 = 22.7;

const BODY: Rgb = Rgb::hex(0xd7dce4);
const BODY_DARK: Rgb = Rgb::hex(0x8f97a6);
const EDGE: Rgb = Rgb::hex(0xf3f6fa);
const ENGINE: Rgb = Rgb::hex(0x4b5563);
const WINDOW: Rgb = Rgb::hex(0x93c5fd);
const STEEL: Rgb = Rgb::hex(0x64748b);
const SMOKE: Rgb = Rgb::hex(0x64748b);
const STAR: Rgb = Rgb::hex(0xbfdbfe);
const CITY: Rgb = Rgb::hex(0xffcf7a);
const ATMOS: [Rgb; 3] = [Rgb::hex(0x7dd3fc), Rgb::hex(0x38bdf8), Rgb::hex(0x0ea5e9)];
const SURFACE: [Rgb; 3] = [Rgb::hex(0x0b2033), Rgb::hex(0x102a42), Rgb::hex(0x163a55)];
const FLAME: [Rgb; 5] = [
    Rgb::hex(0xffffff),
    Rgb::hex(0xfff3c4),
    Rgb::hex(0xffb020),
    Rgb::hex(0xff5a0a),
    Rgb::hex(0x8c1600),
];
const VACUUM: [Rgb; 4] = [
    Rgb::hex(0xffffff),
    Rgb::hex(0xdbeafe),
    Rgb::hex(0x93c5fd),
    Rgb::hex(0x3b82f6),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Flame,
    Smoke,
    Spark,
    Steam,
}

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    /// World altitude above the pad, in rows.
    alt: f32,
    vx: f32,
    valt: f32,
    life: f32,
    max_life: f32,
    size: f32,
    kind: Kind,
}

#[derive(Clone, Copy)]
struct Star {
    x: f32,
    y: f32,
    phase: f32,
    bright: f32,
    layer: f32,
    band: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TumbleKind {
    Booster,
    Core,
}

#[derive(Clone, Copy)]
struct Tumble {
    x: f32,
    alt: f32,
    vx: f32,
    valt: f32,
    spin: f32,
    life: f32,
    kind: TumbleKind,
}

#[derive(Clone, Copy)]
struct Payload {
    x: f32,
    y: f32,
    age: f32,
}

pub struct LaunchScene {
    w: usize,
    h: usize,
    rng: Rng,
    stars: Vec<Star>,
    particles: Vec<Particle>,
    tumble: Vec<Tumble>,
    payload: Option<Payload>,
    rocket: Vec<Vec<char>>,
    upper: Vec<Vec<char>>,
    rocket_rows: usize,
    upper_rows: usize,
    rocket_w: usize,
    scale: f32,
    spawn_flame: f32,
    spawn_smoke: f32,
    spawn_spark: f32,
    spawn_steam: f32,
    last_phase: f32,
    sep_spawned: bool,
}

impl LaunchScene {
    pub fn new(seed: u64) -> Self {
        LaunchScene {
            w: 0,
            h: 0,
            rng: Rng::new(seed ^ 0x4c41_554e_4348),
            stars: Vec::new(),
            particles: Vec::new(),
            tumble: Vec::new(),
            payload: None,
            rocket: Vec::new(),
            upper: Vec::new(),
            rocket_rows: 0,
            upper_rows: 0,
            rocket_w: 0,
            scale: 1.0,
            spawn_flame: 0.0,
            spawn_smoke: 0.0,
            spawn_spark: 0.0,
            spawn_steam: 0.0,
            last_phase: 0.0,
            sep_spawned: false,
        }
    }

    /// Normalised ascent curve.  The returned altitude is in screen rows above
    /// the pad; it accelerates through max-Q, then settles into orbit.
    fn altitude(p: f32, h: f32) -> f32 {
        if p < T_LIFTOFF {
            return 0.0;
        }
        let h = h.max(1.0);
        if p < T_MAX_Q {
            h * 0.30 * smoothstep((p - T_LIFTOFF) / (T_MAX_Q - T_LIFTOFF))
        } else if p < T_MECO {
            h * (0.30 + 1.25 * smoothstep((p - T_MAX_Q) / (T_MECO - T_MAX_Q)))
        } else if p < T_ORBIT {
            h * (1.55 + 1.65 * smoothstep((p - T_MECO) / (T_ORBIT - T_MECO)))
        } else {
            h * (3.20 + 0.40 * smoothstep((p - T_ORBIT) / 8.0))
        }
    }

    fn pad_ground(h: usize) -> f32 {
        (h as f32 - 3.0).max(2.0)
    }

    fn focus_y(h: usize) -> f32 {
        h as f32 * 0.60
    }

    fn camera(alt: f32, h: usize) -> f32 {
        let shift = (Self::pad_ground(h) - 2.0 - Self::focus_y(h)).max(0.0);
        (alt - shift).max(0.0)
    }

    fn throttle(p: f32) -> f32 {
        if p < T_IGNITION {
            0.0
        } else if p < T_LIFTOFF {
            0.35 + 0.65 * smoothstep((p - T_IGNITION) / (T_LIFTOFF - T_IGNITION))
        } else if p < T_MECO {
            1.0
        } else if p < T_STAGE2 {
            0.25 + 0.25 * smoothstep((p - T_MECO) / (T_STAGE2 - T_MECO))
        } else if p < T_ORBIT {
            0.45
        } else {
            0.30
        }
    }

    fn vacuum(p: f32) -> bool {
        p >= T_MECO + 0.25
    }

    fn rebuild_rocket(&mut self, scale: f32) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        let base = (self.h as f32 * 0.44).round().clamp(9.0, 19.0);
        let rows = (base * scale.clamp(0.55, 1.6)).round().clamp(8.0, 21.0) as usize;
        let w = if rows <= 10 {
            7
        } else if rows <= 14 {
            9
        } else {
            11
        };
        self.rocket = build_rocket(rows, w);
        self.rocket_rows = rows;
        self.upper_rows = ((rows as f32) * 0.55).round().clamp(7.0, 11.0) as usize;
        let upper_w = if w >= 11 { 7 } else { 5 };
        self.upper = build_upper_stage(self.upper_rows, upper_w);
        self.rocket_w = w;
        self.scale = scale;
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn(&mut self, kind: Kind, x: f32, alt: f32, vx: f32, valt: f32, life: f32, size: f32) {
        if self.particles.len() >= self.budget() {
            return;
        }
        self.particles.push(Particle {
            x,
            alt,
            vx,
            valt,
            life,
            max_life: life,
            size,
            kind,
        });
    }

    fn budget(&self) -> usize {
        ((self.w * self.h) / 6).clamp(180, 820)
    }

    fn spawn_engine_particles(&mut self, p: f32, alt: f32) {
        let w = self.w as f32;
        let cx = w * 0.5;
        let half = self.rocket_w as f32 * 0.5;
        let engines = [cx - half + 0.5, cx, cx + half - 0.5];
        let throttled = Self::throttle(p);
        let vac = Self::vacuum(p);
        let world = alt + 2.0;

        // Flame sparks from each engine.
        for &ex in &engines {
            let flick = self.rng.range(0.55, 1.35);
            let spread = if vac { 0.8 } else { 2.2 };
            let px = ex + self.rng.range(-0.8, 0.8);
            let pa = world + self.rng.range(-0.4, 0.4);
            let pvx = self.rng.range(-spread, spread);
            let pvalt = -self.rng.range(9.0, 19.0) * throttled * flick;
            let plife = self.rng.range(0.28, 0.75);
            let psize = self.rng.range(0.6, 1.4);
            self.spawn(Kind::Flame, px, pa, pvx, pvalt, plife, psize);
            if !vac && self.rng.chance(0.18) {
                let pvx = self.rng.range(-7.0, 7.0);
                let pvalt = -self.rng.range(4.0, 14.0);
                let plife = self.rng.range(0.5, 1.3);
                self.spawn(Kind::Spark, ex, world, pvx, pvalt, plife, 1.0);
            }
        }

        // The long exhaust column that gets left behind as the camera climbs.
        if p < T_MECO + 0.8 {
            let px = cx + self.rng.range(-2.5, 2.5);
            let pa = world - self.rng.range(0.0, 4.0);
            let pvx = self.rng.range(-3.0, 3.0);
            let pvalt = -self.rng.range(2.5, 7.0);
            let plife = self.rng.range(2.2, 5.0);
            let psize = self.rng.range(1.4, 3.4);
            self.spawn(Kind::Smoke, px, pa, pvx, pvalt, plife, psize);
        }

        // Hold-down exhaust deflects along the pad instead of falling.
        if (T_IGNITION..T_LIFTOFF + 0.4).contains(&p) {
            let spread = (p - T_IGNITION).clamp(0.0, 1.0);
            for s in [-1.0, 1.0] {
                let px = cx + s * self.rng.range(3.0, 9.0);
                let pvx = s * self.rng.range(3.5, 11.0) * (0.4 + spread);
                let pvalt = self.rng.range(1.0, 4.5);
                let plife = self.rng.range(1.4, 3.6);
                let psize = self.rng.range(1.5, 3.2);
                self.spawn(Kind::Steam, px, 1.0, pvx, pvalt, plife, psize);
            }
        }
    }

    fn reset(&mut self) {
        self.particles.clear();
        self.tumble.clear();
        self.payload = None;
        self.spawn_flame = 0.0;
        self.spawn_smoke = 0.0;
        self.spawn_spark = 0.0;
        self.spawn_steam = 0.0;
        self.sep_spawned = false;
    }
}

impl Scene for LaunchScene {
    fn resize(&mut self, w: usize, h: usize) {
        self.w = w.max(1);
        self.h = h.max(1);
        let mut rng = Rng::new(0x5354_4152_4c41);
        self.stars.clear();
        let span = (self.h as f32 * 0.88).max(2.0);
        for i in 0..(self.w * self.h / 34).clamp(70, 210) {
            let band = i % 5 == 0;
            let (x, y) = if band {
                let x = rng.range(0.0, self.w as f32);
                let y = (x * 0.42 + rng.range(-4.0, 4.0)).rem_euclid(span);
                (x, y)
            } else {
                (rng.range(0.0, self.w as f32), rng.range(0.0, span))
            };
            self.stars.push(Star {
                x,
                y,
                phase: rng.range(0.0, TAU),
                bright: rng.range(0.25, 1.0),
                layer: rng.range(0.15, 1.0),
                band,
            });
        }
        self.reset();
        self.last_phase = 0.0;
        self.scale = 1.0;
        self.rebuild_rocket(1.0);
    }

    fn update(&mut self, f: &Frame) {
        if self.w == 0 || self.h == 0 {
            return;
        }
        if (self.scale - f.tune.rocket_scale).abs() > 0.001 {
            self.rebuild_rocket(f.tune.rocket_scale);
        }
        let dt = f.dt.clamp(0.0, 0.1);
        let p = (f.t as f32) % CYCLE;
        if p < self.last_phase {
            self.reset();
        }
        self.last_phase = p;

        let h = self.h as f32;
        let alt = Self::altitude(p, h);

        // Ground steam while the plume is still hitting the trench.
        if (T_IGNITION + 0.2..7.5).contains(&p) {
            self.spawn_steam += dt * 24.0;
            while self.spawn_steam >= 1.0 {
                self.spawn_steam -= 1.0;
                let s = if self.rng.chance(0.5) { -1.0 } else { 1.0 };
                let cx = self.w as f32 * 0.5;
                let px = cx + s * self.rng.range(4.0, 14.0);
                let pa = self.rng.range(0.0, 2.0);
                let pvx = s * self.rng.range(2.0, 8.0);
                let pvalt = self.rng.range(1.5, 5.0);
                let plife = self.rng.range(1.6, 4.2);
                let psize = self.rng.range(1.4, 3.6);
                self.spawn(Kind::Steam, px, pa, pvx, pvalt, plife, psize);
            }
        }

        if (T_IGNITION..T_MECO + 1.0).contains(&p) {
            let rate = if p < T_LIFTOFF + 1.0 { 120.0 } else { 62.0 };
            self.spawn_flame += dt * rate;
            while self.spawn_flame >= 1.0 {
                self.spawn_flame -= 1.0;
                self.spawn_engine_particles(p, alt);
            }
        }
        if (T_LIFTOFF..T_MECO + 1.5).contains(&p) {
            self.spawn_smoke += dt * 22.0;
            while self.spawn_smoke >= 1.0 {
                self.spawn_smoke -= 1.0;
                let cx = self.w as f32 * 0.5;
                let world = alt + 2.0 - self.rng.range(2.0, 9.0);
                let px = cx + self.rng.range(-3.0, 3.0);
                let pvx = self.rng.range(-2.2, 2.2);
                let pvalt = -self.rng.range(1.0, 4.0);
                let plife = self.rng.range(2.5, 5.5);
                let psize = self.rng.range(1.8, 4.0);
                self.spawn(Kind::Smoke, px, world, pvx, pvalt, plife, psize);
            }
        }
        if (T_IGNITION..7.2).contains(&p) {
            self.spawn_spark += dt * 28.0;
            while self.spawn_spark >= 1.0 {
                self.spawn_spark -= 1.0;
                let s = if self.rng.chance(0.5) { -1.0 } else { 1.0 };
                let px = self.w as f32 * 0.5 + s * self.rng.range(1.0, 10.0);
                let pa = self.rng.range(0.0, 2.0);
                let pvx = s * self.rng.range(1.0, 7.0);
                let pvalt = self.rng.range(2.0, 9.0);
                let plife = self.rng.range(0.6, 1.8);
                self.spawn(Kind::Spark, px, pa, pvx, pvalt, plife, 1.0);
            }
        }

        if p >= T_MECO && !self.sep_spawned {
            self.sep_spawned = true;
            let cx = self.w as f32 * 0.5;
            // The boosters peel off from the sides; the core stage falls last.
            for s in [-1.0, 1.0] {
                self.tumble.push(Tumble {
                    x: cx + s * (self.rocket_w as f32 * 0.5 + 0.5),
                    alt: alt + 2.0 + 5.0,
                    vx: s * self.rng.range(5.0, 10.0),
                    valt: -self.rng.range(1.0, 4.0),
                    spin: self.rng.range(0.0, TAU),
                    life: 5.0,
                    kind: TumbleKind::Booster,
                });
            }
            self.tumble.push(Tumble {
                x: cx,
                alt: alt + 2.0 + 7.0,
                vx: self.rng.range(-1.2, 1.2),
                valt: -self.rng.range(1.5, 3.5),
                spin: self.rng.range(0.0, TAU),
                life: 6.0,
                kind: TumbleKind::Core,
            });
            for _ in 0..34 {
                let s = if self.rng.chance(0.5) { -1.0 } else { 1.0 };
                let px = cx + s * self.rng.range(1.0, 6.0);
                let pvx = s * self.rng.range(3.0, 13.0);
                let pvalt = self.rng.range(-6.0, 5.0);
                let plife = self.rng.range(0.6, 1.8);
                self.spawn(Kind::Spark, px, alt + 2.5, pvx, pvalt, plife, 1.0);
            }
        }

        if (20.0..21.5).contains(&p) && self.payload.is_none() {
            self.payload = Some(Payload {
                x: self.w as f32 * 0.5,
                y: Self::pad_ground(self.h) - 2.0 + Self::camera(alt, self.h)
                    - alt
                    - self.rocket_rows as f32
                    + 1.0,
                age: 0.0,
            });
        }
        if let Some(pay) = self.payload.as_mut() {
            pay.age += dt;
            pay.x += 2.1 * dt;
            pay.y -= 1.15 * dt;
            if pay.age > 3.0 {
                self.payload = None;
            }
        }

        for part in &mut self.particles {
            match part.kind {
                Kind::Smoke => {
                    part.valt += 1.4 * dt;
                    part.vx *= 1.0 - 0.25 * dt;
                }
                Kind::Steam => {
                    part.valt += 0.9 * dt;
                    part.vx *= 1.0 - 0.5 * dt;
                }
                Kind::Spark => part.valt -= 8.0 * dt,
                Kind::Flame => part.valt += 1.5 * dt,
            }
            part.x += part.vx * dt;
            part.alt += part.valt * dt;
            part.life -= dt;
            if part.alt < 0.0 {
                part.alt = 0.0;
                part.valt = part.valt.abs() * 0.25 + 0.2;
                part.vx *= 1.8;
            }
        }
        self.particles.retain(|p| {
            p.life > 0.0 && p.alt < h * 8.0 && p.x > -12.0 && p.x < self.w as f32 + 12.0
        });

        for b in &mut self.tumble {
            b.x += b.vx * dt;
            b.alt += b.valt * dt;
            b.valt -= 3.0 * dt;
            b.spin += 5.0 * dt;
            b.life -= dt;
        }
        self.tumble.retain(|b| b.life > 0.0);
    }

    fn hud_status(&self, f: &Frame) -> Option<String> {
        let p = (f.t as f32) % CYCLE;
        let alt = Self::altitude(p, f.h as f32);
        let km = ((alt / (f.h as f32 * 3.6)).clamp(0.0, 1.0).powf(1.25) * 420.0) as u32;
        if p < T_IGNITION {
            Some(format!(
                "T-{:02} | PAD | GO",
                (T_IGNITION - p).ceil() as u32
            ))
        } else if p < T_MECO {
            let phase = if p < T_LIFTOFF {
                "IGNITION"
            } else if p < T_MAX_Q {
                "LIFTOFF"
            } else {
                "MAX-Q"
            };
            Some(format!(
                "T+{:02} | ALT {:03}KM | {phase}",
                (p - T_IGNITION) as u32,
                km
            ))
        } else if p < T_STAGE2 + 0.8 {
            Some(format!(
                "T+{:02} | ALT {:03}KM | MECO/SEP",
                (p - T_IGNITION) as u32,
                km
            ))
        } else if p < T_ORBIT {
            Some(format!(
                "T+{:02} | ALT {:03}KM | STAGE 2",
                (p - T_IGNITION) as u32,
                km
            ))
        } else {
            Some(format!(
                "T+{:02} | ALT {:03}KM | ORBIT",
                (p - T_IGNITION) as u32,
                km
            ))
        }
    }

    fn draw(&mut self, canvas: &mut Canvas, f: &Frame) {
        if self.w == 0 || self.h == 0 || self.rocket_rows == 0 {
            return;
        }
        let t = f.t as f32;
        let p = t % CYCLE;
        let max_y = self.h as isize - 3;
        if max_y < 3 {
            return;
        }
        let theme = f.theme;
        let w = self.w as f32;
        let h = self.h as f32;
        let cx = (w * 0.5) as isize;
        let alt = Self::altitude(p, h);
        let cam = Self::camera(alt, self.h);
        let ground_f = Self::pad_ground(self.h) + cam;
        let base_f = Self::pad_ground(self.h) - 2.0 + cam - alt;

        let shake = shake_offset(t, p, f.tune.shake);
        let base = base_f.round() as isize + shake.1;
        let ground = ground_f.round() as isize + shake.1;

        let separated = p >= T_MECO + 0.12;
        let stack_base = if separated {
            base - (self.rocket_rows as isize - self.upper_rows as isize)
        } else {
            base
        };

        self.draw_sky(canvas, p, cam, t, theme, max_y);
        self.draw_stars(canvas, p, cam, t, max_y);
        self.draw_earth(canvas, p, t, cx, max_y, theme);
        self.draw_pad(canvas, p, t, cx, ground, max_y, theme, shake.0);
        self.draw_clouds(canvas, p, t, cx, ground_f, base_f, max_y, theme, shake.0);
        self.draw_smoke(canvas, ground_f, max_y, theme, p);
        self.draw_ignition(canvas, p, t, cx, base, ground, max_y, theme);
        self.draw_plume(
            canvas,
            p,
            t,
            stack_base,
            cx + shake.0,
            max_y,
            f.tune.rocket_scale,
            separated,
        );
        if separated {
            self.draw_stack(
                canvas,
                &self.upper,
                stack_base,
                cx,
                p,
                max_y,
                theme,
                shake.0,
            );
        } else {
            self.draw_stack(canvas, &self.rocket, base, cx, p, max_y, theme, shake.0);
        }

        for part in &self.particles {
            if part.kind == Kind::Flame || part.kind == Kind::Spark {
                self.draw_particle(canvas, part, ground_f, max_y, p);
            }
        }
        for b in &self.tumble {
            self.draw_tumble(canvas, b, ground_f, max_y, theme);
        }
        if let Some(pay) = self.payload {
            draw_payload(canvas, pay, max_y, theme);
        }

        self.draw_flash(canvas, p, max_y, cx, base);

        let fade = fade_value(p);
        if fade < 0.999 {
            for y in 1..=max_y {
                let row = y as usize * self.w;
                for x in 0..self.w {
                    let cell = &mut canvas.cells[row + x];
                    cell.fg = cell.fg.scale(fade);
                    if let Some(bg) = cell.bg {
                        cell.bg = Some(bg.scale(fade));
                    }
                }
            }
        }
    }
}

impl LaunchScene {
    fn draw_sky(
        &self,
        canvas: &mut Canvas,
        p: f32,
        cam: f32,
        t: f32,
        theme: &crate::theme::Theme,
        max_y: isize,
    ) {
        let h = self.h as f32;
        let space = smoothstep((p - 7.0) / 8.0);
        let top = Rgb::hex(0x02040a);
        let mid = Rgb::hex(0x071526);
        let low = Rgb::hex(0x13263a);
        let warm = theme.c(3).scale(0.22);
        for y in 1..=max_y {
            let v = (y as f32 / h).clamp(0.0, 1.0);
            let mut col = top
                .lerp(mid, (v / 0.62).min(1.0))
                .lerp(low, ((v - 0.58) / 0.42).clamp(0.0, 1.0));
            // Blue hour glow at the horizon, fading as the camera climbs.
            let horizon = v.powf(3.0) * (1.0 - space) * 0.55;
            col = col.lerp(warm, horizon);
            // Slow haze bands keep the gradient from looking like a flat fill.
            let haze = 0.035 * (v * 26.0 + t * 0.08 + cam * 0.01).sin() * (1.0 - space);
            col = col.add(Rgb::hex(0x2a3b52), haze.max(0.0));
            let bg = Some(col);
            let row = y as usize * self.w;
            for x in 0..self.w {
                canvas.cells[row + x] = crate::canvas::Cell {
                    ch: ' ',
                    fg: Rgb::BLACK,
                    bg,
                };
            }
        }
    }

    fn draw_stars(&self, canvas: &mut Canvas, p: f32, cam: f32, t: f32, max_y: isize) {
        let span = (self.h as f32 * 0.88).max(2.0);
        let visible = 0.22 + 0.78 * smoothstep((p - 6.0) / 9.0);
        for s in &self.stars {
            let y = ((s.y + cam * s.layer * 0.035).rem_euclid(span) + 1.0).round() as isize;
            if y < 1 || y > max_y {
                continue;
            }
            let tw = 0.45 + 0.55 * (t * 1.2 + s.phase).sin().abs();
            let a = (0.16 + 0.62 * s.bright * tw * visible).clamp(0.0, 1.0);
            let x = s.x.round() as isize;
            if s.band {
                canvas.add_light(x, y, Rgb::hex(0x64748b), 0.10 * visible, '.');
                continue;
            }
            let ch = if s.bright > 0.86 && tw > 0.72 {
                '+'
            } else {
                '.'
            };
            canvas.set(x, y, ch, STAR.scale(a));
            if s.bright > 0.80 && visible > 0.4 {
                canvas.add_light(x, y, STAR, 0.16 * visible, '.');
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_pad(
        &self,
        canvas: &mut Canvas,
        p: f32,
        t: f32,
        cx: isize,
        ground: isize,
        max_y: isize,
        theme: &crate::theme::Theme,
        shake_x: isize,
    ) {
        if ground < 1 || ground > max_y + 4 {
            return;
        }
        let h = self.h as f32;
        let w = self.w as f32;
        let metal = STEEL.scale(0.55);
        let dark = theme.shadow().add(Rgb::hex(0x334155), 0.05);

        if ground <= max_y {
            for x in 0..self.w as isize {
                let shade = 0.35 + 0.15 * ((x as f32 * 0.17).sin() * 0.5 + 0.5);
                canvas.set(x, ground, '#', theme.c(2).scale(shade));
            }
            // Flame trench and platform glow.
            for dx in -12isize..=12 {
                let x = cx + dx + shake_x;
                if x > 0 && x < self.w as isize - 1 {
                    canvas.set(x, ground - 1, '=', metal);
                }
            }
            for s in [-13isize, 13] {
                let x = cx + s + shake_x;
                if x > 0 && x < self.w as isize - 1 {
                    canvas.set(x, ground - 1, '+', metal);
                    canvas.set(x, ground - 2, '|', metal);
                }
            }
            for x in (cx - 18).max(1)..=(cx + 18).min(self.w as isize - 1) {
                if (x - cx).abs() > 14 && (x + ground) % 2 == 0 {
                    canvas.set(x, ground - 1, ':', theme.c(1).scale(0.35));
                }
            }
        }

        // Service tower on the right, with retracting swing arms.
        let tower_x = (cx + 13).min(self.w as isize - 6).max(cx + 9);
        let tower_h = (h * 0.42).round().clamp(7.0, 15.0) as isize;
        let top = (ground - tower_h).max(2);
        if ground > 4 {
            for y in top..ground {
                let x = tower_x + shake_x;
                canvas.set(x, y, '|', metal.scale(0.7));
                canvas.set(x + 3, y, '|', metal.scale(0.7));
                if (ground - y) % 2 == 0 {
                    for xx in x + 1..x + 3 {
                        canvas.set(xx, y, '-', metal.scale(0.45));
                    }
                } else if (ground - y) % 4 == 1 {
                    canvas.set(x + 1, y, '/', metal.scale(0.5));
                    canvas.set(x + 2, y, '\\', metal.scale(0.5));
                }
                if (ground - y) % 6 == 0 {
                    canvas.set(x + 1, y, '=', theme.c(2).scale(0.28));
                    canvas.set(x + 2, y, '=', theme.c(2).scale(0.28));
                }
            }
            let blink = if (t * 1.8).sin() > -0.2 { '*' } else { '.' };
            canvas.set(tower_x + shake_x, top - 1, blink, Rgb::hex(0xff4d4d));
            canvas.set(tower_x + 3 + shake_x, top - 1, '+', metal);
            // Swing arms retract during the terminal count.
            let retract = smoothstep((p - 2.8) / 1.2);
            let arm_len = (10.0 * (1.0 - retract)).round() as isize;
            for (i, dy) in [3isize, 7, 11].iter().enumerate() {
                let y = ground - dy;
                if y <= top + 1 {
                    continue;
                }
                for k in 1..=arm_len {
                    let x = tower_x - k + shake_x;
                    if x <= cx + 3 {
                        break;
                    }
                    let ch = if k == arm_len && arm_len > 0 {
                        '+'
                    } else {
                        '='
                    };
                    canvas.set(x, y, ch, metal.scale(0.55 + 0.06 * i as f32));
                }
            }
        }

        // Lightning masts and floodlights frame the pad.
        for s in [-1.0, 1.0] {
            let mx = cx + (s * w * 0.31) as isize + shake_x;
            if mx > 1 && mx < self.w as isize - 1 {
                let mh = (h * 0.26).round().clamp(5.0, 11.0) as isize;
                for y in (ground - mh).max(2)..ground {
                    if y <= max_y {
                        canvas.set(mx, y, '|', metal.scale(0.45));
                    }
                }
                let blink = if (t * 2.1 + s).sin() > 0.0 { 'o' } else { '.' };
                canvas.set(mx, (ground - mh).max(2) - 1, blink, Rgb::hex(0xff6b6b));
            }
            let lx = cx + (s * w * 0.22) as isize + shake_x;
            if lx > 1 && lx < self.w as isize - 1 && ground - 2 <= max_y {
                canvas.set(lx, ground - 2, '*', Rgb::hex(0xfff1b8));
                canvas.add_light(lx, ground - 3, theme.c(3), 0.20, '.');
            }
        }

        // Pad crew and a tiny truck sell the scale at wider sizes.
        if self.w >= 90 && ground - 1 <= max_y {
            for (dx, ch, col) in [
                (-27isize, 'A', Rgb::hex(0xe5e7eb)),
                (-18, 'A', Rgb::hex(0xd1d5db)),
                (22, '=', Rgb::hex(0xfbbf24)),
            ] {
                let x = cx + dx + shake_x;
                if x > 1 && x < self.w as isize - 1 {
                    canvas.set(x, ground - 1, ch, col.scale(0.65));
                }
            }
        }

        // Cryogenic venting before ignition.
        if p < T_IGNITION + 0.3 && ground > 6 {
            for i in 0..4 {
                let x = cx - 5 + (i % 2) * 2 + shake_x;
                let y = ground - 3 - i * 2 - ((t * 1.6 + i as f32) % 2.0) as isize;
                if y > 2 && y <= max_y {
                    canvas.add_light(x, y, Rgb::hex(0xdbeafe), 0.22, ':');
                }
            }
        }
        let _ = dark;
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_clouds(
        &self,
        canvas: &mut Canvas,
        p: f32,
        t: f32,
        cx: isize,
        ground: f32,
        base: f32,
        max_y: isize,
        theme: &crate::theme::Theme,
        shake_x: isize,
    ) {
        if !(6.0..=15.0).contains(&p) {
            return;
        }
        let h = self.h as f32;
        for layer in 0..2 {
            let cloud_alt = h * (0.48 + layer as f32 * 0.07);
            let y = ground - cloud_alt;
            if y < 1.0 || y > max_y as f32 + 2.0 {
                continue;
            }
            let closeness = (1.0 - ((y - base).abs() / 12.0)).clamp(0.0, 1.0);
            let near = theme.c(3).lerp(FLAME[2], 0.5);
            for x in 0..self.w as isize {
                if (x - cx).abs() < 5 && (y - base).abs() < 10.0 {
                    continue;
                }
                let fx = x as f32;
                let n = value_noise(
                    fx * 0.085 + layer as f32 * 5.0 + t * 0.06,
                    layer as f32 * 7.0,
                    0x51,
                );
                let wave = (fx * 0.06 + t * 0.25 + layer as f32).sin();
                let yy = (y + wave).round() as isize;
                if n > 0.82 && yy >= 1 && yy <= max_y {
                    let ch = if n > 0.92 { '~' } else { '-' };
                    let mut col = Rgb::hex(0x2b3d52).scale(0.26 + 0.16 * n);
                    if closeness > 0.0 {
                        col = col.lerp(near, closeness * 0.45);
                    }
                    canvas.set(x + shake_x, yy, ch, col);
                    if n > 0.62 {
                        canvas.add_light(x + shake_x, yy + 1, col, 0.12, ':');
                    }
                }
                if layer == 1 && n > 0.66 && yy + 2 <= max_y {
                    canvas.add_light(x + shake_x, yy + 2, theme.c(2), 0.06, ':');
                }
            }
        }
    }

    fn draw_smoke(
        &self,
        canvas: &mut Canvas,
        ground: f32,
        max_y: isize,
        theme: &crate::theme::Theme,
        p: f32,
    ) {
        for part in &self.particles {
            if part.kind == Kind::Smoke || part.kind == Kind::Steam {
                self.draw_particle(canvas, part, ground, max_y, p);
                let _ = theme;
            }
        }
    }

    fn draw_particle(
        &self,
        canvas: &mut Canvas,
        part: &Particle,
        ground: f32,
        max_y: isize,
        p: f32,
    ) {
        let y = (ground - part.alt).round() as isize;
        if y < 1 || y > max_y {
            return;
        }
        let x = part.x.round() as isize;
        let fade = (part.life / part.max_life.max(0.01)).clamp(0.0, 1.0);
        match part.kind {
            Kind::Smoke => {
                let ch = if part.size > 2.6 && fade > 0.45 {
                    '%'
                } else if fade > 0.3 {
                    ':'
                } else {
                    '.'
                };
                let mut col = SMOKE.scale(0.16 + 0.26 * fade);
                let warm = (1.0 - (y as f32 - (ground - 10.0)).abs() / 12.0).clamp(0.0, 1.0);
                col = col.lerp(FLAME[3], 0.20 * warm * if p < 12.0 { 1.0 } else { 0.2 });
                let r = (part.size * 0.45).round().clamp(0.0, 2.0) as isize;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() + dy.abs() > r + 1 {
                            continue;
                        }
                        let yy = y + dy;
                        if yy < 1 || yy > max_y {
                            continue;
                        }
                        canvas.set(
                            x + dx,
                            yy,
                            ch,
                            col.scale(0.70 + 0.15 * (r as f32 - dy.abs() as f32)),
                        );
                    }
                }
            }
            Kind::Steam => {
                let ch = if fade > 0.55 { '%' } else { ':' };
                let col = Rgb::hex(0xdbeafe).scale(0.14 + 0.28 * fade);
                let r = (part.size * 0.4).round().clamp(0.0, 2.0) as isize;
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx.abs() + dy.abs() > r + 1 {
                            continue;
                        }
                        let yy = y + dy;
                        if yy < 1 || yy > max_y {
                            continue;
                        }
                        canvas.add_light(x + dx, yy, col, 0.07, ch);
                    }
                }
            }
            Kind::Flame => {
                let idx = ((1.0 - fade) * 4.0).round() as usize;
                let col = FLAME[idx.min(4)].scale(0.55 + 0.45 * fade);
                canvas.set(x, y, if fade > 0.55 { '*' } else { '.' }, col);
                canvas.add_light(x, y + 1, FLAME[2], 0.08 * fade, '.');
            }
            Kind::Spark => {
                let col = FLAME[1].scale(0.45 + 0.5 * fade);
                canvas.set(x, y, if fade > 0.4 { '*' } else { '.' }, col);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_ignition(
        &self,
        canvas: &mut Canvas,
        p: f32,
        t: f32,
        cx: isize,
        base: isize,
        ground: isize,
        max_y: isize,
        theme: &crate::theme::Theme,
    ) {
        if !(T_IGNITION..6.2).contains(&p) {
            return;
        }
        let age = p - T_IGNITION;
        let fade = (1.0 - age / 2.1).clamp(0.0, 1.0);
        let cy = (ground - 1).max(2);
        let radius = 2.0 + age * 2.2;
        let rx = (radius * 2.0).round() as isize;
        let ry = radius.round() as isize;
        let ceiling = (base - 1).max(2);
        for dy in -ry..=ry {
            for dx in -rx..=rx {
                if cy + dy < ceiling || cy + dy > max_y {
                    continue;
                }
                let d = ((dx as f32 / rx.max(1) as f32).powi(2)
                    + (dy as f32 / ry.max(1) as f32).powi(2))
                .sqrt();
                if d >= 1.0 {
                    continue;
                }
                if d > 0.72
                    && hash01(dx as i64, dy as i64 + (age * 10.0) as i64, 0x464c_4153) > 0.55
                {
                    continue;
                }
                let ch = if d < 0.26 {
                    '*'
                } else if d < 0.55 {
                    '#'
                } else if d < 0.82 {
                    '='
                } else {
                    '.'
                };
                let idx = if d < 0.22 {
                    0
                } else if d < 0.48 {
                    1
                } else if d < 0.74 {
                    2
                } else {
                    3
                };
                let col = FLAME[idx].scale((0.80 + 0.20 * fade) * (1.0 - d * 0.25));
                canvas.set(cx + dx, cy + dy, ch, col);
                canvas.add_light(cx + dx, cy + dy, FLAME[1], 0.28 * fade, ch);
            }
        }
        // A bright flame column links the engine to the pad fireball.
        let top = base.max(2);
        if top < cy {
            for y in top..=cy {
                let depth = (y - top) as f32 / (cy - top + 1) as f32;
                let width = (1.0 + depth * 2.0).round() as isize;
                for dx in -width..=width {
                    let d = dx.abs() as f32 / (width as f32 + 0.5);
                    let ch = if d < 0.45 { '*' } else { '#' };
                    let col = if d < 0.45 { FLAME[1] } else { FLAME[2] }
                        .scale((0.55 + 0.45 * fade) * (1.0 - depth * 0.35));
                    canvas.set(cx + dx, y, ch, col);
                }
            }
        }

        // Ground jets out of the flame trench.
        let reach = (5.0 + age * 11.0) as isize;
        for side in [-1isize, 1] {
            for k in 1..=reach {
                let x = cx + side * k;
                let y = ground - 1;
                let f = (1.0 - k as f32 / (reach as f32 + 1.0)) * fade;
                let ch = if k < 3 {
                    '*'
                } else if k < reach / 2 {
                    '='
                } else {
                    '.'
                };
                canvas.add_light(x, y, FLAME[1], 0.55 * f, ch);
                canvas.add_light(x, y - 1, FLAME[2], 0.30 * f, '.');
                if k % 3 == 0 {
                    canvas.add_light(x, y - 2, FLAME[3], 0.18 * f, ':');
                }
            }
        }
        // Water deluge steam climbing on both sides of the pad.
        for side in [-1isize, 1] {
            for i in 0..5isize {
                let x = cx + side * (5 + i * 3);
                let wobble = (t * 1.7 + i as f32 * 1.3 + side as f32).sin() * 1.2;
                for j in 0..7isize {
                    let y = ground - 2 - j;
                    if y < 2 {
                        break;
                    }
                    let f = (1.0 - j as f32 / 8.0) * fade;
                    let xx = x + (wobble * j as f32 * 0.16).round() as isize;
                    let ch = if j < 3 { '%' } else { ':' };
                    canvas.add_light(xx, y, Rgb::hex(0xdbeafe), 0.30 * f, ch);
                }
            }
        }
        canvas.set(cx, cy, '*', Rgb::hex(0xfffbe8));
        canvas.add_light(cx, cy, theme.hot(), 0.95 * fade, '*');
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_plume(
        &self,
        canvas: &mut Canvas,
        p: f32,
        t: f32,
        base: isize,
        cx: isize,
        max_y: isize,
        scale: f32,
        separated: bool,
    ) {
        if p < T_IGNITION {
            return;
        }
        let throttled = Self::throttle(p);
        let vac = Self::vacuum(p);
        let half = (self.rocket_w / 2) as isize;
        let engines: Vec<isize> = if separated {
            vec![cx]
        } else {
            vec![cx - half + 1, cx, cx + half - 1]
        };
        for (e, &ex) in engines.iter().enumerate() {
            let strong = if engines.len() == 1 || e == 1 {
                1.0
            } else {
                0.62
            };
            let len = if separated {
                (3.0 + 6.0 * throttled).round().max(3.0) as isize
            } else {
                ((9.0 + 18.0 * throttled) * strong * scale.clamp(0.6, 1.5))
                    .round()
                    .max(3.0) as isize
            };
            for k in 0..len {
                let y = base + 1 + k;
                if y < 1 || y > max_y {
                    break;
                }
                let depth = k as f32 / len as f32;
                let noise = value_noise(t * 7.0 + e as f32 * 3.1, k as f32 * 0.5, 0x464c_414d);
                let max_width = if separated {
                    0.45 + depth * 1.1
                } else if vac {
                    0.8 + depth * 1.8
                } else {
                    0.9 + depth * 2.8
                };
                let width = (max_width * (0.75 + 0.5 * noise) * scale.clamp(0.7, 1.4))
                    .round()
                    .max(0.0) as isize;
                // A solid white-hot core gives the exhaust a readable spine.
                let core = if separated {
                    VACUUM[1]
                } else if depth < 0.45 {
                    FLAME[0]
                } else {
                    FLAME[1]
                };
                for dx in -width..=width {
                    let dist = dx.abs() as f32 / (width as f32 + 0.7);
                    let ch = if separated {
                        if dist < 0.5 {
                            '*'
                        } else {
                            '.'
                        }
                    } else if dist < 0.30 && depth < 0.4 {
                        '*'
                    } else if dist < 0.66 {
                        '#'
                    } else if depth < 0.9 {
                        '='
                    } else {
                        '.'
                    };
                    let pal: &[Rgb] = if vac { &VACUUM } else { &FLAME };
                    let idx = ((depth * (pal.len() - 1) as f32) + dist * 1.2).round() as usize;
                    let mut col = pal[idx.min(pal.len() - 1)].scale(0.55 + 0.45 * (1.0 - depth));
                    if !vac && k % 3 == 0 && depth < 0.62 {
                        col = col.lerp(FLAME[0], 0.65);
                    }
                    canvas.set(ex + dx, y, ch, col);
                }
                canvas.set(ex, y, '*', core.scale(0.85 + 0.15 * (1.0 - depth)));
                // A soft halo keeps the exhaust from looking like a sticker.
                if width > 0 {
                    canvas.add_light(
                        ex - width - 1,
                        y,
                        if vac { VACUUM[2] } else { FLAME[2] },
                        0.10,
                        '.',
                    );
                    canvas.add_light(
                        ex + width + 1,
                        y,
                        if vac { VACUUM[2] } else { FLAME[2] },
                        0.10,
                        '.',
                    );
                }
                if !vac && k % 3 == 0 && depth < 0.62 {
                    canvas.set(ex, y, '*', FLAME[0]);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_stack(
        &self,
        canvas: &mut Canvas,
        art: &[Vec<char>],
        base: isize,
        cx: isize,
        p: f32,
        max_y: isize,
        theme: &crate::theme::Theme,
        shake_x: isize,
    ) {
        let Some(first) = art.first() else {
            return;
        };
        let rows = art.len();
        let art_w = first.len();
        let top = base - rows as isize + 1;
        let left = cx - (art_w as isize / 2) + shake_x;
        let throttled = Self::throttle(p);
        for (r, row) in art.iter().enumerate() {
            let y = top + r as isize;
            if y < 1 || y > max_y {
                continue;
            }
            for (c, &ch) in row.iter().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let mut col = match ch {
                    '^' => EDGE,
                    '/' | '\\' => EDGE.scale(0.90),
                    '|' => EDGE.scale(0.78),
                    '#' => BODY,
                    '%' => BODY_DARK,
                    '=' => theme.c(3),
                    'o' => WINDOW,
                    '+' => theme.c(2).lerp(EDGE, 0.35),
                    'v' | 'V' => ENGINE,
                    _ => BODY,
                };
                if (ch == '#' || ch == '%' || ch == '|') && r as f32 > rows as f32 * 0.66 {
                    col = col.lerp(FLAME[2], 0.10 * throttled);
                }
                canvas.set(left + c as isize, y, ch, col);
            }
        }
        // Engine glow bleeding up the skirt.
        for dx in -2isize..=2 {
            canvas.add_light(
                cx + shake_x + dx,
                base,
                if Self::vacuum(p) { VACUUM[2] } else { FLAME[2] },
                0.22 * Self::throttle(p),
                ' ',
            );
        }
    }

    fn draw_tumble(
        &self,
        canvas: &mut Canvas,
        b: &Tumble,
        ground: f32,
        max_y: isize,
        theme: &crate::theme::Theme,
    ) {
        let y = (ground - b.alt).round() as isize;
        if y < 1 || y > max_y + 8 {
            return;
        }
        let x = b.x.round() as isize;
        let flip = b.spin.sin() >= 0.0;
        let (sprite, half): (&[&str], isize) = match b.kind {
            TumbleKind::Booster => (
                if flip {
                    &["/\\", "|#", "|#", "|#", "|#", "^^"]
                } else {
                    &["/\\", "#|", "#|", "#|", "#|", "^^"]
                },
                1,
            ),
            TumbleKind::Core => (
                &[
                    " /###\\ ", "|#####|", "|##=##|", "|#####|", "|#####|", "\\^^^^^/", " ^^ ^^ ",
                ],
                3,
            ),
        };
        for (r, line) in sprite.iter().enumerate() {
            let yy = y + r as isize;
            if yy < 1 || yy > max_y {
                continue;
            }
            for (c, ch) in line.chars().enumerate() {
                let ch = if flip {
                    line.chars().nth(line.chars().count() - 1 - c).unwrap_or(ch)
                } else {
                    ch
                };
                let col = match ch {
                    '#' => BODY_DARK,
                    '|' => EDGE.scale(0.7),
                    '/' | '\\' => EDGE.scale(0.8),
                    '^' => theme.c(3),
                    '=' => theme.c(3).scale(0.7),
                    _ => STEEL,
                };
                canvas.set(x + c as isize - half, yy, ch, col);
            }
        }
        canvas.add_light(x, y + sprite.len() as isize, FLAME[2], 0.18, '.');
    }

    fn draw_flash(&self, canvas: &mut Canvas, p: f32, max_y: isize, cx: isize, base: isize) {
        let mut flash = 0.0;
        let mut warm = true;
        if (T_IGNITION..T_IGNITION + 0.7).contains(&p) {
            flash = (-(p - T_IGNITION) / 0.16).exp();
        }
        if (T_MECO..T_MECO + 0.6).contains(&p) {
            let f = (-(p - T_MECO) / 0.13).exp();
            if f > flash {
                flash = f;
                warm = false;
            }
        }
        if flash > 0.015 {
            let col = if warm { FLAME[1] } else { VACUUM[1] }.scale(0.22 * flash);
            for y in 1..=max_y {
                for x in 0..self.w {
                    canvas.set_bg(x as isize, y, Some(col));
                }
            }
            // Expanding shock ring.
            let age = (p - if warm { T_IGNITION } else { T_MECO }).max(0.0);
            let radius = age * 26.0;
            if radius > 2.0 {
                let steps = (radius * 8.0) as usize + 24;
                for i in 0..steps {
                    let a = i as f32 / steps as f32 * TAU;
                    let x = cx + (a.cos() * radius) as isize;
                    let y = base + (a.sin() * radius * 0.5) as isize;
                    let fade = (1.0 - radius / (self.w as f32 * 0.6)).clamp(0.0, 1.0);
                    if fade <= 0.0 {
                        break;
                    }
                    let ch = if i % 3 == 0 { '*' } else { '.' };
                    canvas.add_light(x, y, FLAME[0], 0.35 * fade, ch);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_earth(
        &self,
        canvas: &mut Canvas,
        p: f32,
        t: f32,
        cx: isize,
        max_y: isize,
        theme: &crate::theme::Theme,
    ) {
        if p < 13.0 {
            return;
        }
        let h = self.h as f32;
        let w = self.w as f32;
        let ep = smoothstep((p - 13.0) / 4.5);
        let limb = h + 1.0 - ep * h * 0.32;
        let drop = h * 0.20 * (0.5 + 0.5 * ep);
        let sun = smoothstep((p - T_SUNRISE) / 3.0);
        let sun_x = cx + (w * 0.29) as isize;
        let sun_u = (sun_x - cx) as f32 / cx.max(1) as f32;
        let sun_y = (limb + sun_u * sun_u * drop).round() as isize;

        for x in 0..self.w as isize {
            let u = (x - cx) as f32 / cx.max(1) as f32;
            let y = limb + u * u * drop;
            let yi = y.round() as isize;
            if yi > max_y + 1 || yi < 1 {
                continue;
            }
            // Atmosphere: a few translucent bands above the limb.
            for k in 1..=3 {
                let yy = yi - k;
                if yy >= 1 && yy <= max_y {
                    let a = 0.15 - 0.035 * k as f32;
                    canvas.add_light(x, yy, ATMOS[(k - 1).min(2) as usize], a, '.');
                }
            }
            if yi <= max_y {
                let warm = sun > 0.0 && (x - sun_x).abs() < 18;
                let ch = if x % 9 == 0 { '#' } else { '=' };
                let col = if warm {
                    ATMOS[0].lerp(FLAME[1], 0.55 * sun)
                } else {
                    ATMOS[0]
                };
                canvas.set(x, yi, ch, col);
                canvas.add_light(x, yi - 1, ATMOS[0], 0.38, '.');
            }
            // Night side: dark ocean, low land, city lights.
            for yy in (yi + 1).max(1)..=max_y {
                let land = value_noise(x as f32 * 0.045, yy as f32 * 0.11 + 2.0, 0x0e47);
                let ocean = land < 0.53;
                let ch = if ocean {
                    if land < 0.24 {
                        ':'
                    } else {
                        '.'
                    }
                } else if land > 0.80 {
                    '#'
                } else {
                    ':'
                };
                let mut col = if ocean {
                    SURFACE[0].add(Rgb::hex(0x04182a), land * 0.4)
                } else {
                    Rgb::hex(0x081c18).add(SURFACE[1], land * 0.25)
                };
                if sun > 0.0 {
                    let near = (1.0 - ((x - sun_x).abs() as f32 / 34.0)).clamp(0.0, 1.0);
                    col = col.lerp(FLAME[2].scale(0.45), near * sun * 0.30);
                }
                let bg = Some(col.scale(0.42));
                canvas.paint(x, yy, ch, col.scale(1.15), bg);
                if !ocean {
                    let city = hash01(x as i64 / 2, yy as i64 / 2, 0x99a1);
                    if city > 0.972 {
                        let tw = 0.45 + 0.55 * (t * 2.1 + x as f32 * 0.7).sin().abs();
                        canvas.set(x, yy, '*', CITY.scale(0.45 + 0.55 * tw));
                    }
                }
            }
        }

        // Orbital sunrise: a dithered bloom over the limb, plus a bright core.
        if sun > 0.01 && sun_y <= max_y + 4 {
            for dy in -7isize..=7 {
                for dx in -17isize..=17 {
                    let d = ((dx as f32 / 17.0).powi(2) + (dy as f32 / 7.0).powi(2)).sqrt();
                    if d >= 1.0 {
                        continue;
                    }
                    let a = (1.0 - d).powf(1.7) * sun;
                    if d < 0.18 {
                        let col = FLAME[1].scale(0.45 + 0.45 * a);
                        canvas.set_bg(sun_x + dx, sun_y + dy, Some(col));
                    } else if hash01(dx as i64, dy as i64, 0x0053_554e) < 0.22 + 0.45 * (1.0 - d) {
                        let ch = if d < 0.45 { '*' } else { '.' };
                        canvas.add_light(sun_x + dx, sun_y + dy, FLAME[1], 0.24 * a, ch);
                    }
                }
            }
            for d in 0..=16isize {
                let f = (1.0 - d as f32 / 17.0) * sun;
                let ch = if d % 3 == 0 { '*' } else { '-' };
                canvas.add_light(sun_x - d, sun_y, FLAME[1], 0.45 * f, ch);
                canvas.add_light(sun_x + d, sun_y, FLAME[1], 0.45 * f, ch);
                if d < 10 {
                    canvas.add_light(sun_x - d, sun_y - d, FLAME[2], 0.18 * f, '\\');
                    canvas.add_light(sun_x + d, sun_y - d, FLAME[2], 0.18 * f, '/');
                }
            }
            for d in 0..=7isize {
                canvas.add_light(
                    sun_x,
                    sun_y - d,
                    FLAME[1],
                    0.28 * (1.0 - d as f32 / 8.0) * sun,
                    '|',
                );
            }
            canvas.set(sun_x, sun_y, '*', Rgb::hex(0xfffbe8));
            canvas.add_light(sun_x, sun_y, Rgb::WHITE, 0.9 * sun, '*');
            // Light the horizon around the sunrise.
            for x in (sun_x - 22).max(1)..=(sun_x + 22).min(self.w as isize - 1) {
                let f = (1.0 - (x - sun_x).abs() as f32 / 23.0).max(0.0) * sun;
                canvas.add_light(x, sun_y + 1, FLAME[2], 0.24 * f, '.');
                canvas.add_light(x, sun_y - 1, FLAME[1], 0.16 * f, '.');
            }
        }
        let _ = theme;
    }
}

fn build_rocket(rows: usize, w: usize) -> Vec<Vec<char>> {
    let mut g = vec![vec![' '; w]; rows];
    if rows < 6 || w < 5 {
        return g;
    }
    let core_w = match w {
        7 => 3,
        9 => 5,
        _ => 7,
    };
    let fair_w = (core_w + 2).min(w);
    let fair_half = fair_w / 2;
    let core_half = core_w / 2;
    let cx = w / 2;
    let engine_rows = if rows >= 13 { 2 } else { 1 };
    let body_end = rows.saturating_sub(engine_rows);
    let nose_rows = (fair_half + 1).min(body_end.saturating_sub(1));
    let booster_h = (((body_end.saturating_sub(nose_rows)) as f32) * 0.58).round() as usize;
    let booster_top =
        body_end.saturating_sub(booster_h.max(3).min(body_end.saturating_sub(nose_rows)));

    // Nose cone and payload fairing.
    for i in 0..nose_rows {
        let half = i.min(fair_half);
        if half == 0 {
            g[i][cx] = '^';
            continue;
        }
        let left = cx - half;
        let right = cx + half;
        g[i][left] = '/';
        g[i][right] = '\\';
        for (x, cell) in g[i].iter_mut().enumerate().take(right).skip(left + 1) {
            *cell = if x == right - 1 { '%' } else { '#' };
        }
    }

    // Upper stage: fairing shoulder, body, a window and accent bands.
    for y in nose_rows..booster_top {
        let left = cx - core_half;
        let right = cx + core_half;
        g[y][left] = '|';
        g[y][right] = '|';
        for x in left + 1..right {
            g[y][x] = if x == right - 1 { '%' } else { '#' };
        }
        if y == nose_rows + 1 && core_w >= 5 {
            g[y][cx - 1] = 'o';
            g[y][cx + 1] = 'o';
        }
        if (y - nose_rows) % 4 == 3 {
            for x in left + 1..right {
                g[y][x] = '=';
            }
        }
    }

    // Core stage and the two side boosters.
    for y in booster_top..body_end {
        let left = cx - core_half;
        let right = cx + core_half;
        g[y][left] = '|';
        g[y][right] = '|';
        for x in left + 1..right {
            g[y][x] = if x == right - 1 { '%' } else { '#' };
        }
        if (y - booster_top) % 4 == 2 {
            for x in left + 1..right {
                g[y][x] = '=';
            }
        }
        if y == booster_top {
            g[y][0] = '/';
            g[y][1] = '\\';
            g[y][w - 2] = '/';
            g[y][w - 1] = '\\';
        } else {
            g[y][0] = '|';
            g[y][1] = if y == booster_top + 1 { '+' } else { '#' };
            g[y][w - 2] = if y == booster_top + 1 { '+' } else { '#' };
            g[y][w - 1] = '|';
            if (y - booster_top) % 4 == 2 {
                g[y][1] = '=';
                g[y][w - 2] = '=';
            }
        }
    }

    // Engine skirt and nozzle bells.
    let skirt = body_end.saturating_sub(1);
    if skirt > nose_rows {
        let left = cx - core_half;
        let right = cx + core_half;
        g[skirt][left] = '\\';
        g[skirt][right] = '/';
        for x in left + 1..right {
            g[skirt][x] = '^';
        }
        g[skirt][0] = '\\';
        g[skirt][1] = '\\';
        g[skirt][w - 2] = '/';
        g[skirt][w - 1] = '/';
    }
    if body_end < rows {
        let y = body_end;
        for x in 0..w {
            g[y][x] = ' ';
        }
        g[y][0] = '^';
        g[y][1] = '^';
        g[y][cx] = 'v';
        g[y][w - 2] = '^';
        g[y][w - 1] = '^';
        if core_w >= 5 {
            g[y][cx - 2] = 'v';
            g[y][cx + 2] = 'v';
        }
    }
    g
}

fn build_upper_stage(rows: usize, w: usize) -> Vec<Vec<char>> {
    let mut g = vec![vec![' '; w]; rows];
    if rows < 5 || w < 3 {
        return g;
    }
    let cx = w / 2;
    let half = w / 2;
    let nose_rows = (half + 1).min(rows.saturating_sub(2));
    for i in 0..nose_rows {
        let h = i.min(half);
        if h == 0 {
            g[i][cx] = '^';
            continue;
        }
        let left = cx - h;
        let right = cx + h;
        g[i][left] = '/';
        g[i][right] = '\\';
        for x in left + 1..right {
            g[i][x] = if x == right - 1 { '%' } else { '#' };
        }
    }
    let body_end = rows.saturating_sub(1);
    for y in nose_rows..body_end {
        let left = cx - half;
        let right = cx + half;
        g[y][left] = '|';
        g[y][right] = '|';
        for x in left + 1..right {
            g[y][x] = if x == right - 1 { '%' } else { '#' };
        }
        if y == nose_rows + 1 && w >= 5 {
            g[y][cx] = 'o';
        }
        if (y - nose_rows) % 3 == 2 {
            for x in left + 1..right {
                g[y][x] = '=';
            }
        }
    }
    let skirt = body_end.saturating_sub(1);
    if skirt > nose_rows {
        g[skirt][cx - half] = '\\';
        g[skirt][cx + half] = '/';
        for x in cx - half + 1..cx + half {
            g[skirt][x] = '^';
        }
    }
    if body_end < rows {
        let y = body_end;
        g[y][cx] = 'v';
        if w >= 5 {
            g[y][cx - 1] = '^';
            g[y][cx + 1] = '^';
        }
    }
    g
}

fn smoothstep(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn shake_offset(t: f32, p: f32, strength: f32) -> (isize, isize) {
    let engine = if (T_IGNITION..T_MECO).contains(&p) {
        0.22 + 0.15 * LaunchScene::throttle(p)
    } else {
        0.0
    };
    let kick = if p >= T_IGNITION {
        0.9 * (-(p - T_IGNITION) / 0.20).exp()
    } else {
        0.0
    } + if p >= T_MECO {
        0.8 * (-(p - T_MECO) / 0.18).exp()
    } else {
        0.0
    };
    let amp = ((engine + kick) * strength).min(1.8);
    let x = ((t * 47.0).sin() + 0.6 * (t * 73.0).sin()) * amp;
    let y = ((t * 41.0).cos() + 0.5 * (t * 67.0).sin()) * amp * 0.55;
    (x.round() as isize, y.round() as isize)
}

fn fade_value(p: f32) -> f32 {
    let fade_in = smoothstep(p / 0.7);
    let fade_out = 1.0 - smoothstep((p - T_FADE_OUT) / 1.1);
    fade_in.min(fade_out)
}

fn draw_payload(canvas: &mut Canvas, pay: Payload, max_y: isize, theme: &crate::theme::Theme) {
    let x = pay.x.round() as isize;
    let y = pay.y.round() as isize;
    if y < 2 || y > max_y - 1 {
        return;
    }
    let fade = (1.0 - (pay.age / 3.0)).clamp(0.0, 1.0);
    let col = theme.c(3).scale(0.5 + 0.5 * fade);
    for dx in -2isize..=2 {
        canvas.set(x + dx, y, '=', col);
    }
    canvas.set(x, y, '#', Rgb::hex(0xe5e7eb));
    canvas.set(x, y - 1, '|', Rgb::hex(0xe5e7eb));
    canvas.set(x, y - 2, '+', Rgb::hex(0xfbbf24));
    canvas.add_light(x, y, theme.hot(), 0.22 * fade, '.');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(t: f64, w: usize, h: usize) -> Frame<'static> {
        Frame {
            t,
            dt: 1.0 / 30.0,
            w,
            h,
            fps: 30,
            theme: crate::theme::find("ember").unwrap(),
            charset: crate::cli::Charset::Ascii,
            seed: 7,
            speed: 1.0,
            tune: crate::scene::Tune::default(),
        }
    }

    #[test]
    fn launch_draws_pad_and_rocket() {
        let mut scene = LaunchScene::new(7);
        scene.resize(80, 24);
        let f = frame(3.5, 80, 24);
        scene.update(&f);
        let mut canvas = Canvas::new(80, 24);
        canvas.set_chrome_guard(true);
        scene.draw(&mut canvas, &f);
        let text: String = canvas.cells.iter().map(|c| c.ch).collect();
        assert!(text.contains('#'), "no pad or rocket");
        assert!(text.contains('^'), "no nose cone");
        assert!(canvas.ink() > 90, "scene looks empty: {}", canvas.ink());
        for cell in &canvas.cells {
            assert!(cell.ch.is_ascii(), "leaked {:?}", cell.ch);
        }
    }

    #[test]
    fn rocket_scales_with_the_terminal() {
        let mut small = LaunchScene::new(1);
        small.resize(80, 24);
        assert!(small.rocket_rows >= 10, "{}", small.rocket_rows);
        assert!(small.rocket_w >= 7);
        let mut large = LaunchScene::new(1);
        large.resize(160, 50);
        assert!(large.rocket_rows >= 17, "{}", large.rocket_rows);
        assert!(large.rocket_w >= 11);
    }

    #[test]
    fn phases_reach_separation_and_orbit() {
        let mut scene = LaunchScene::new(7);
        scene.resize(120, 40);
        let f = frame(3.0, 120, 40);
        scene.update(&f);
        assert!(scene.tumble.is_empty());
        let f = frame(12.4, 120, 40);
        scene.update(&f);
        assert!(!scene.tumble.is_empty(), "boosters did not separate");
        let f = frame(18.0, 120, 40);
        scene.update(&f);
        let f = frame(20.5, 120, 40);
        scene.update(&f);
        assert!(scene.payload.is_some(), "payload was not deployed");
    }

    #[test]
    fn particles_stay_within_budget() {
        let mut scene = LaunchScene::new(3);
        scene.resize(120, 40);
        let mut t = 0.0;
        while t < 20.0 {
            let f = frame(t, 120, 40);
            scene.update(&f);
            assert!(
                scene.particles.len() <= scene.budget(),
                "{}",
                scene.particles.len()
            );
            t += 1.0 / 30.0;
        }
    }

    #[test]
    fn loop_restarts_cleanly() {
        let mut scene = LaunchScene::new(5);
        scene.resize(100, 30);
        let f = frame(23.5, 100, 30);
        scene.update(&f);
        let f = frame(0.2, 100, 30);
        scene.update(&f);
        assert!(scene.tumble.is_empty());
        assert!(scene.particles.is_empty());
        assert!(scene.payload.is_none());
    }

    #[test]
    fn rocket_never_enters_hud_rows_at_any_phase() {
        let mut scene = LaunchScene::new(9);
        scene.resize(80, 24);
        for t in 0..24 {
            let f = frame(t as f64, 80, 24);
            scene.update(&f);
            let mut canvas = Canvas::new(80, 24);
            canvas.set_chrome_guard(true);
            scene.draw(&mut canvas, &f);
        }
    }

    #[test]
    fn rocket_is_a_scaled_multi_part_sprite() {
        let art = build_rocket(18, 11);
        assert_eq!(art.len(), 18);
        assert!(art.iter().any(|r| r.contains(&'|')));
        assert!(art.iter().any(|r| r.contains(&'=')));
        assert!(art.iter().any(|r| r.contains(&'o')));
        assert!(art.iter().any(|r| r.contains(&'v')));
    }
}
