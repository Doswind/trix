//! Application entry: CLI dispatch, interactive loop, snapshot & benchmark.

use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::canvas::{Canvas, PresentOptions};
use crate::cli::{self, Options, Parsed, PlaylistMode};
use crate::color::ColorMode;
use crate::help;
use crate::hud;
use crate::rng::Rng;
use crate::scene::{self, EffectId, Frame, Scene, Tune};
use crate::term::{self, Key, Terminal};
use crate::theme::{self, Theme};
use crate::transition::{Transition, TransitionMode};

const SNAPSHOT_SEED: u64 = 0x7e51_a017;

/// Encoder profile for traffic that actually goes to a terminal.
fn present_options(opts: &Options) -> PresentOptions {
    PresentOptions {
        color_slack: opts.perf.color_slack(),
        sync: !opts.no_sync,
        line_flush: opts.line_flush,
        ..PresentOptions::default()
    }
}

/// Adaptive frame-rate governor.
///
/// The requested `--fps` is an upper bound.  When writing a frame takes most
/// of the frame budget, the terminal (or the SSH link in front of it) is the
/// bottleneck; producing more frames only builds a queue of stale pictures.
/// The governor lowers the rate in steps, and recovers slowly once writes are
/// comfortably fast again.
struct Pacer {
    max_fps: u32,
    min_fps: u32,
    fps: u32,
    slow: u32,
    fast: u32,
}

impl Pacer {
    fn new(max_fps: u32, min_fps: u32) -> Self {
        Pacer {
            max_fps,
            min_fps: min_fps.min(max_fps).max(1),
            fps: max_fps,
            slow: 0,
            fast: 0,
        }
    }

    fn frame_dur(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.fps as f64)
    }

    /// Feed one frame's write time.  Returns true when the target changed.
    ///
    /// The thresholds are intentionally strict: a write that already uses a
    /// third of the frame budget leaves almost no room for a transition or a
    /// panel repaint, and the SSH/TCP buffers in front of a terminal can hide
    /// that pressure until the user notices seconds of input lag.
    fn observe(&mut self, present: Duration) -> bool {
        let target = self.frame_dur();
        if present > target.mul_f32(0.30) {
            self.slow = self.slow.saturating_add(1);
            self.fast = 0;
        } else if present < target.mul_f32(0.15) {
            self.fast = self.fast.saturating_add(1);
            self.slow = 0;
        } else {
            self.slow = 0;
            self.fast = 0;
        }

        if self.slow >= 3 && self.fps > self.min_fps {
            let next = ((self.fps as f32 * 0.8).floor() as u32).max(self.min_fps);
            self.fps = next.min(self.fps - 1).max(self.min_fps);
            self.slow = 0;
            return true;
        }
        if self.fast >= 90 && self.fps < self.max_fps {
            self.fps = (self.fps + 2).min(self.max_fps);
            self.fast = 0;
            return true;
        }
        false
    }
}

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse_args(&args) {
        Ok(Parsed::Help(opts)) => help::run_help(&opts),
        Ok(Parsed::Version) => help::run_version(),
        Ok(Parsed::List(opts)) => help::run_list(&opts),
        Ok(Parsed::Completions(shell)) => help::run_completions(&shell),
        Ok(Parsed::Run(opts)) => run_options(opts),
        Err(e) => {
            let style = help::HelpStyle::detect(&Options::default());
            eprint!("{}", help::error(&e, &style));
            2
        }
    }
}

fn run_options(opts: Options) -> i32 {
    if let Some(path) = opts.snapshot.clone() {
        return snapshot(&opts, &path);
    }
    if opts.bench {
        return benchmark(&opts);
    }
    interactive(opts)
}

fn resolve_effect(opts: &Options, deterministic: bool) -> Result<(EffectId, u64), String> {
    let name = opts
        .effect
        .as_deref()
        .ok_or_else(|| "no effect selected. Try 'trix rain' or 'trix --list'".to_string())?;
    let id = EffectId::parse(name).ok_or_else(|| cli::unknown_effect_message(name))?;
    let seed = match opts.seed {
        Some(s) => s,
        None if deterministic => SNAPSHOT_SEED,
        None => Rng::from_entropy().next_u64(),
    };
    Ok((id, seed))
}

fn theme_for(opts: &Options, id: EffectId) -> &'static Theme {
    opts.theme
        .as_deref()
        .and_then(theme::find)
        .unwrap_or_else(|| id.default_theme())
}

fn charset_for(opts: &Options, id: EffectId) -> cli::Charset {
    opts.charset.unwrap_or_else(|| id.default_charset())
}

fn tune_for(opts: &Options) -> Tune {
    Tune {
        density: opts.density,
        spin: opts.spin,
        twinkle: opts.twinkle,
        meteors: opts.meteors,
        beam_speed: opts.beam_speed,
        jump: opts.jump,
        beam_jump: opts.beam_jump,
        seed_count: opts.seed_count,
        seed_speed: opts.seed_speed,
        street_vertical: opts.street_vertical,
        tornado_count: opts.tornado_count,
        tornado_speed: opts.tornado_speed,
        life_cycle: opts.life_cycle,
        eye_still: opts.eye_still,
        trail: opts.trail,
        ghost: opts.ghost,
        plain: opts.plain,
        gallery: opts.gallery,
        rocket_scale: opts.rocket_scale,
        shake: opts.shake,
    }
}

fn snapshot(opts: &Options, path: &Path) -> i32 {
    let (id, seed) = match resolve_effect(opts, true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("trix: {e}");
            return 2;
        }
    };
    let theme = theme_for(opts, id);
    let charset = charset_for(opts, id);
    let tune = tune_for(opts);
    let mode = opts.color.unwrap_or(ColorMode::True);
    let (w, h) = opts.size.unwrap_or((120, 36));
    let frames = opts.frames.unwrap_or(180);
    let fps = opts.fps;
    let dt = (1.0 / fps as f64) * opts.speed as f64;

    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let mut canvas = Canvas::new(w, h);
    let mut last_t = 0.0;
    for i in 0..frames {
        let t = i as f64 * dt;
        last_t = t;
        let frame = Frame {
            t,
            dt: dt as f32,
            w,
            h,
            fps,
            theme,
            charset,
            seed,
            speed: opts.speed,
            tune,
        };
        scene.update(&frame);
    }
    let frame = Frame {
        t: last_t,
        dt: dt as f32,
        w,
        h,
        fps,
        theme,
        charset,
        seed,
        speed: opts.speed,
        tune,
    };
    canvas.clear();
    scene.draw(&mut canvas, &frame);
    let ansi = canvas.render_ansi(mode);

    if path == Path::new("-") {
        let mut out = std::io::stdout().lock();
        if out.write_all(&ansi).and_then(|_| out.flush()).is_err() {
            return 1;
        }
    } else {
        match std::fs::write(path, &ansi) {
            Ok(()) => {
                eprintln!(
                    "trix: wrote {} ({}x{}, {} frames, {:.1} KiB)",
                    path.display(),
                    w,
                    h,
                    frames,
                    ansi.len() as f64 / 1024.0
                );
            }
            Err(e) => {
                eprintln!("trix: cannot write '{}': {e}", path.display());
                return 1;
            }
        }
    }
    0
}

fn benchmark(opts: &Options) -> i32 {
    let (id, seed) = match resolve_effect(opts, true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("trix: {e}");
            return 2;
        }
    };
    let theme = theme_for(opts, id);
    let charset = charset_for(opts, id);
    let tune = tune_for(opts);
    let mode = opts.color.unwrap_or(ColorMode::True);
    let (w, h) = opts.size.unwrap_or((160, 48));
    let frames = opts.frames.unwrap_or(240);
    let fps = opts.fps;
    let dt = (1.0 / fps as f64) * opts.speed as f64;

    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let mut canvas = Canvas::new(w, h);
    let mut sink: Vec<u8> = Vec::with_capacity(w * h * 8 + 1024);
    let mut bytes = 0usize;
    let mut sizes = Vec::with_capacity(frames as usize);
    let mut times = Vec::with_capacity(frames as usize);
    let mut t = 0.0;
    let popts = present_options(opts);

    // Warm up allocator, caches and the scene's buffers.
    for i in 0..15 {
        let frame = Frame {
            t: i as f64 * dt,
            dt: dt as f32,
            w,
            h,
            fps,
            theme,
            charset,
            seed,
            speed: opts.speed,
            tune,
        };
        scene.update(&frame);
        canvas.clear();
        scene.draw(&mut canvas, &frame);
        sink.clear();
        let _ = canvas.present_with(&mut sink, mode, popts);
        t = frame.t;
    }

    let mut total = 0.0f64;
    for i in 0..frames {
        let frame = Frame {
            t: t + (i + 1) as f64 * dt,
            dt: dt as f32,
            w,
            h,
            fps,
            theme,
            charset,
            seed,
            speed: opts.speed,
            tune,
        };
        let start = Instant::now();
        scene.update(&frame);
        canvas.clear();
        scene.draw(&mut canvas, &frame);
        sink.clear();
        let n = canvas.present_with(&mut sink, mode, popts).unwrap_or(0);
        let el = start.elapsed().as_secs_f64();
        total += el;
        times.push(el);
        sizes.push(n);
        bytes += n;
    }
    let avg = total / frames as f64;
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sizes.sort_unstable();
    let p95 = times[((frames as f64 * 0.95) as usize).min(times.len() - 1)];
    let size_p95 = sizes[((frames as f64 * 0.95) as usize).min(sizes.len() - 1)];
    let size_max = *sizes.last().unwrap_or(&0);
    let avg_bytes = bytes as f64 / frames as f64;
    println!("trix /// offscreen render benchmark");
    println!("  effect      {:<10} theme {}", id.key(), theme.key);
    println!("  canvas      {w}x{h}  chars {}", w * h);
    println!("  frames      {frames} @ {fps} fps target");
    println!("  seed        {seed}");
    println!(
        "  render      avg {:.3} ms  p95 {:.3} ms  ({:.0} fps headroom)",
        avg * 1000.0,
        p95 * 1000.0,
        if avg > 0.0 { 1.0 / avg } else { 0.0 }
    );
    println!(
        "  ansi        avg {:.1} KiB/frame  p95 {:.1}  max {:.1}  ({:.2} MiB/s at {fps} fps)",
        avg_bytes / 1024.0,
        size_p95 as f64 / 1024.0,
        size_max as f64 / 1024.0,
        avg_bytes * fps as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  profile     {} (colour slack {}, sync {})",
        opts.perf.key(),
        opts.perf.color_slack(),
        if opts.no_sync { "off" } else { "on" }
    );
    println!("  note        timed section = scene update + draw + diff encode");
    0
}

fn interactive(opts: Options) -> i32 {
    let playlist = opts.effect.is_none();
    let (mut id, mut seed) = if playlist {
        let start = if opts.playlist == PlaylistMode::Random {
            Rng::from_entropy().below(EffectId::ALL.len())
        } else {
            0
        };
        (EffectId::from_index(start), Rng::from_entropy().next_u64())
    } else {
        match resolve_effect(&opts, false) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("trix: {e}");
                return 2;
            }
        }
    };
    let mut explicit_theme = opts.theme.is_some();
    let mut current_theme = theme_for(&opts, id);
    let mut charset = charset_for(&opts, id);
    let explicit_charset = opts.charset.is_some();
    let mode = opts.color.unwrap_or_else(ColorMode::detect);

    let mut term = match Terminal::enter(!opts.no_alt) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("trix: {e}");
            return 1;
        }
    };
    let mut w = term.width.max(1);
    let mut h = term.height.max(1);
    let mut canvas = Canvas::new(w, h);
    // Two off-screen buffers let the outgoing and incoming effects render at
    // the same time while a transition blends them.
    let mut back_a = Canvas::new(w, h);
    let mut back_b = Canvas::new(w, h);
    let trans_mode = opts.transition;
    let trans_secs = if playlist {
        // Leave at least half of each playlist slot untouched by the blend.
        opts.transition_secs.min(opts.switch_secs * 0.5)
    } else {
        opts.transition_secs
    };
    // A two-scene blend is the largest burst the renderer can produce; on the
    // low-latency profile keep it short instead of doubling the traffic for
    // 0.6 s.
    let trans_secs = if opts.perf == cli::PerfMode::Low {
        trans_secs.min(0.2)
    } else {
        trans_secs
    };
    let mut transition: Option<Transition> = None;
    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let tune = tune_for(&opts);

    let started = Instant::now();
    let mut pacer = Pacer::new(
        opts.perf.max_fps(opts.fps),
        opts.perf.min_fps(opts.perf.max_fps(opts.fps)),
    );
    let popts = present_options(&opts);
    let mut last_tick = Instant::now();
    let mut t = 0.0f64;
    let mut paused = false;
    let mut speed = opts.speed;
    let mut panel = hud::Panel::None;
    let mut flash = 0.0f32;
    let mut fps_smooth = opts.fps as f32;
    let mut frame_ms = 0.0f32;
    let mut write_ms = 0.0f32;
    let mut auto_clock = 0.0f32;
    let mut ansi_window = 0usize;
    let mut ansi_kib_s = 0.0f32;
    let mut ansi_clock = Instant::now();

    loop {
        let frame_start = Instant::now();
        let now = frame_start;
        let real_dt = (now - last_tick).as_secs_f32().clamp(0.0, 0.1);
        last_tick = now;
        if real_dt > 0.0005 {
            let inst = 1.0 / real_dt;
            fps_smooth = fps_smooth * 0.9 + inst.min(1000.0) * 0.1;
        }

        if term.refresh_size() {
            w = term.width.max(1);
            h = term.height.max(1);
            canvas.resize(w, h);
            back_a.resize(w, h);
            back_b.resize(w, h);
            scene.resize(w, h);
            if let Some(tr) = transition.as_mut() {
                tr.from.resize(w, h);
            }
            term.clear();
            canvas.invalidate();
        }

        for key in term.read_keys() {
            // Shift and Caps Lock should never break a shortcut: terminals
            // report `Q` / `N` / `H` exactly like `q` / `n` / `h`.
            let key = normalize_key(key);
            match key {
                Key::Char('q') | Key::Char('\u{3}') => return 0,
                Key::Esc => {
                    if panel == hud::Panel::None {
                        return 0;
                    }
                    panel = hud::Panel::None;
                }
                Key::Char(' ') => paused = !paused,
                Key::Char('h') | Key::Char('?') => {
                    panel = if panel == hud::Panel::Help {
                        hud::Panel::None
                    } else {
                        hud::Panel::Help
                    };
                }
                Key::Char('i') => {
                    panel = if panel == hud::Panel::Info {
                        hud::Panel::None
                    } else {
                        hud::Panel::Info
                    };
                }
                Key::Char('t') => {
                    current_theme = theme::next_theme(current_theme);
                    explicit_theme = true;
                }
                Key::Char('+') | Key::Char('=') => {
                    speed = (speed * 1.15).clamp(0.1, 8.0);
                }
                Key::Char('-') | Key::Char('_') => {
                    speed = (speed / 1.15).clamp(0.1, 8.0);
                }
                Key::Char('n') | Key::Right | Key::Down => {
                    let target = id.next();
                    switch_effect(
                        &mut id,
                        &mut seed,
                        target,
                        &mut scene,
                        &mut t,
                        &mut flash,
                        &mut explicit_theme,
                        &mut current_theme,
                        &mut charset,
                        explicit_charset,
                        w,
                        h,
                        &mut transition,
                        trans_mode,
                        trans_secs,
                    );
                    auto_clock = 0.0;
                }
                Key::Char('p') | Key::Left | Key::Up => {
                    let target = id.prev();
                    switch_effect(
                        &mut id,
                        &mut seed,
                        target,
                        &mut scene,
                        &mut t,
                        &mut flash,
                        &mut explicit_theme,
                        &mut current_theme,
                        &mut charset,
                        explicit_charset,
                        w,
                        h,
                        &mut transition,
                        trans_mode,
                        trans_secs,
                    );
                    auto_clock = 0.0;
                }
                Key::Char('r') => {
                    seed = Rng::from_entropy().next_u64();
                    scene = scene::build(id, seed);
                    scene.resize(w, h);
                    t = 0.0;
                    flash = 0.3;
                    auto_clock = 0.0;
                    // `r` is a restart, not an effect change: cut straight back.
                    transition = None;
                }
                Key::Char(c) if EffectId::from_shortcut(c).is_some() => {
                    let target = EffectId::from_shortcut(c).expect("shortcut guard");
                    switch_effect(
                        &mut id,
                        &mut seed,
                        target,
                        &mut scene,
                        &mut t,
                        &mut flash,
                        &mut explicit_theme,
                        &mut current_theme,
                        &mut charset,
                        explicit_charset,
                        w,
                        h,
                        &mut transition,
                        trans_mode,
                        trans_secs,
                    );
                    auto_clock = 0.0;
                }
                _ => {}
            }
            if term::quit_requested() {
                return 0;
            }
        }
        if term::quit_requested() {
            return 0;
        }

        if playlist && !paused {
            auto_clock += real_dt;
            if auto_clock >= opts.switch_secs {
                auto_clock = 0.0;
                let target = if opts.playlist == PlaylistMode::Random {
                    let mut rng = Rng::from_entropy();
                    let mut target = id;
                    for _ in 0..8 {
                        let cand = EffectId::from_index(rng.below(EffectId::ALL.len()));
                        if cand != id {
                            target = cand;
                            break;
                        }
                    }
                    target
                } else {
                    id.next()
                };
                switch_effect(
                    &mut id,
                    &mut seed,
                    target,
                    &mut scene,
                    &mut t,
                    &mut flash,
                    &mut explicit_theme,
                    &mut current_theme,
                    &mut charset,
                    explicit_charset,
                    w,
                    h,
                    &mut transition,
                    trans_mode,
                    trans_secs,
                );
            }
        }

        let anim_dt = if paused { 0.0 } else { real_dt * speed };
        if let Some(tr) = transition.as_mut() {
            // The blend follows wall-clock time so it stays legible at any
            // --speed; the scenes themselves follow pause/speed.
            tr.tick(real_dt, anim_dt);
        }
        if !paused {
            t += real_dt as f64 * speed as f64;
        }
        let frame = Frame {
            t,
            dt: anim_dt,
            w,
            h,
            fps: opts.fps,
            theme: current_theme,
            charset,
            seed,
            speed,
            tune,
        };

        // The outgoing scene keeps its own clock and look while it fades.
        let from_frame = transition.as_ref().map(|tr| Frame {
            t: tr.from_t,
            dt: anim_dt,
            w,
            h,
            fps: opts.fps,
            theme: tr.from_theme,
            charset: tr.from_charset,
            seed: tr.from_seed,
            speed,
            tune,
        });

        let render_start = Instant::now();
        if !paused {
            scene.update(&frame);
            if let (Some(tr), Some(ff)) = (transition.as_mut(), from_frame.as_ref()) {
                tr.from.update(ff);
            }
        }
        canvas.clear();
        if let (Some(tr), Some(ff)) = (transition.as_mut(), from_frame.as_ref()) {
            back_a.clear();
            tr.from.draw(&mut back_a, ff);
            back_b.clear();
            scene.draw(&mut back_b, &frame);
            tr.compose(&mut canvas, &back_a, &back_b);
            if tr.finished() {
                transition = None;
            }
        } else {
            scene.draw(&mut canvas, &frame);
        }

        let scene_status = scene.hud_status(&frame);
        if !opts.no_hud {
            if id.hud_mode() == scene::HudMode::Full {
                hud::draw_header(&mut canvas, &frame, id, scene_status.as_deref());
            }
            hud::draw_keybar(&mut canvas, &frame, paused);
            if playlist {
                hud::draw_playlist(&mut canvas, &frame, id, opts.switch_secs, auto_clock);
            }
        }
        if flash > 0.0 {
            draw_flash(&mut canvas, &frame, flash);
            flash = (flash - real_dt).max(0.0);
        }
        if paused {
            hud::draw_pause(&mut canvas, &frame);
        }
        match panel {
            hud::Panel::None => {}
            hud::Panel::Help => {
                hud::draw_help(&mut canvas, &frame, id, speed, paused);
            }
            hud::Panel::Info => {
                let params = info_params(
                    id,
                    &tune,
                    &opts,
                    playlist,
                    current_theme,
                    speed,
                    charset,
                    mode,
                );
                let playlist_info = if playlist {
                    Some(hud::PlaylistInfo {
                        index: id.index(),
                        total: EffectId::ALL.len(),
                        switch_secs: opts.switch_secs,
                        remaining: (opts.switch_secs - auto_clock).max(0.0) as u32,
                        transition: opts.transition.key(),
                    })
                } else {
                    None
                };
                let info = hud::Info {
                    effect: id,
                    fps: fps_smooth,
                    target_fps: pacer.fps as f32,
                    ansi_kib_s,
                    frame_ms,
                    write_ms,
                    mode,
                    status: scene_status.as_deref(),
                    playlist: playlist_info,
                    params: &params,
                };
                hud::draw_info(&mut canvas, &frame, &info);
            }
        }

        let present_start = Instant::now();
        let written = match term.present(&mut canvas, mode, popts) {
            Ok(n) => n,
            Err(_) => return 0,
        };
        let present_time = present_start.elapsed();
        write_ms = present_time.as_secs_f32() * 1000.0;
        frame_ms = render_start.elapsed().as_secs_f32() * 1000.0;
        ansi_window += written;
        if ansi_clock.elapsed() >= Duration::from_millis(500) {
            let secs = ansi_clock.elapsed().as_secs_f32().max(0.001);
            ansi_kib_s = ansi_window as f32 / 1024.0 / secs;
            ansi_window = 0;
            ansi_clock = Instant::now();
        }

        if let Some(limit) = opts.duration {
            if started.elapsed().as_secs_f64() >= limit {
                return 0;
            }
        }

        // A write that cannot keep up with the frame budget means the SSH
        // channel, the pseudo-terminal or the emulator is saturated.  Lower
        // the frame rate instead of queueing seconds of stale frames; the
        // animation clock is wall time, so the effect still plays at the
        // requested speed, just with fewer frames.
        if opts.perf.adaptive() && !paused {
            pacer.observe(present_time);
        }

        let work = frame_start.elapsed();
        let frame_dur = pacer.frame_dur();
        if work < frame_dur {
            std::thread::sleep(frame_dur - work);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn switch_effect(
    id: &mut EffectId,
    seed: &mut u64,
    target: EffectId,
    sc: &mut Box<dyn Scene>,
    t: &mut f64,
    flash: &mut f32,
    explicit_theme: &mut bool,
    current_theme: &mut &'static Theme,
    charset: &mut cli::Charset,
    explicit_charset: bool,
    w: usize,
    h: usize,
    transition: &mut Option<Transition>,
    trans_mode: TransitionMode,
    trans_secs: f32,
) {
    if target == *id {
        return;
    }
    let old_theme = *current_theme;
    let old_charset = *charset;
    let old_seed = *seed;
    let old_t = *t;

    let next_seed = Rng::from_entropy().next_u64();
    let mut next = scene::build(target, next_seed);
    next.resize(w, h);

    *id = target;
    *seed = next_seed;
    if !*explicit_theme {
        *current_theme = target.default_theme();
    }
    if !explicit_charset {
        *charset = target.default_charset();
    }

    if trans_mode != TransitionMode::None && trans_secs > 0.001 {
        // Hand the old scene to the transition instead of dropping it, so it
        // keeps animating while it fades out.
        let from = std::mem::replace(sc, next);
        *transition = Some(Transition::new(
            trans_mode,
            trans_secs,
            from,
            old_t,
            old_seed,
            old_theme,
            old_charset,
        ));
    } else {
        *sc = next;
        *transition = None;
        *flash = 0.3;
    }
    *t = 0.0;
}

fn normalize_key(key: Key) -> Key {
    match key {
        Key::Char(c) => Key::Char(c.to_ascii_lowercase()),
        other => other,
    }
}

fn charset_label(charset: cli::Charset) -> &'static str {
    match charset {
        cli::Charset::Kana => "kana",
        cli::Charset::Ascii => "ascii",
        cli::Charset::Unicode => "unicode",
    }
}

fn draw_flash(canvas: &mut Canvas, frame: &Frame, flash: f32) {
    let k = (flash / 0.3).clamp(0.0, 1.0);
    let y = frame.h as isize / 2;
    for dy in -1..=1 {
        for x in (0..frame.w).step_by(2) {
            let ch = if (x / 2 + dy as usize) % 3 == 0 {
                '#'
            } else {
                '='
            };
            let color = frame
                .theme
                .hot()
                .lerp(frame.theme.c(2), (x as f32 / frame.w as f32) * 0.7)
                .scale(k);
            canvas.set(x as isize, y + dy, ch, color);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn info_params(
    id: EffectId,
    tune: &Tune,
    opts: &Options,
    playlist: bool,
    current_theme: &Theme,
    speed: f32,
    charset: cli::Charset,
    mode: ColorMode,
) -> Vec<(String, String)> {
    let mut params = vec![
        ("--theme".to_string(), current_theme.key.to_string()),
        ("--charset".to_string(), charset_label(charset).to_string()),
        ("--fps".to_string(), opts.fps.to_string()),
        ("--perf".to_string(), opts.perf.key().to_string()),
        ("--speed".to_string(), format!("{speed:.2}")),
        ("--color".to_string(), mode.label().to_string()),
    ];
    if playlist {
        params.push((
            "--playlist".to_string(),
            match opts.playlist {
                PlaylistMode::Cycle => "cycle".to_string(),
                PlaylistMode::Random => "random".to_string(),
            },
        ));
        params.push(("--switch".to_string(), format!("{:.1}", opts.switch_secs)));
        params.push((
            "--transition".to_string(),
            opts.transition.key().to_string(),
        ));
        params.push((
            "--transition-secs".to_string(),
            format!("{:.1}", opts.transition_secs),
        ));
    }
    if let Some(d) = opts.duration {
        params.push(("--duration".to_string(), format!("{d}s")));
    }

    for t in id.tuning() {
        let value = match t.flag {
            "--density" => format!("{:.2}", tune.density),
            "--spin" => format!("{:.2}", tune.spin),
            "--twinkle" => format!("{:.2}", tune.twinkle),
            "--meteors" => tune.meteors.to_string(),
            "--beam-speed" => format!("{:.2}", tune.beam_speed),
            "--beam-motion" => {
                if tune.beam_jump {
                    "jump".to_string()
                } else {
                    "straight".to_string()
                }
            }
            "--seed-count" => tune.seed_count.to_string(),
            "--seed-speed" => format!("{:.2}", tune.seed_speed),
            "--street" => {
                if tune.street_vertical {
                    "vertical".to_string()
                } else {
                    "horizontal".to_string()
                }
            }
            "--tornado-count" => tune.tornado_count.to_string(),
            "--tornado-speed" => format!("{:.2}", tune.tornado_speed),
            "--life-cycle" => format!("{:.1}s", tune.life_cycle),
            "--eye-motion" => {
                if tune.eye_still {
                    "still".to_string()
                } else {
                    "dart".to_string()
                }
            }
            "--trail" => format!("{:.2}", tune.trail),
            "--ghost" => {
                if tune.ghost {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            }
            "--gallery" => {
                if tune.gallery {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            }
            "--plain" => {
                if tune.plain {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            }
            "--jump" => {
                if tune.jump {
                    "on".to_string()
                } else {
                    "off".to_string()
                }
            }
            "--rocket-scale" => format!("{:.2}", tune.rocket_scale),
            "--shake" => format!("{:.2}", tune.shake),
            _ => continue,
        };
        params.push((t.flag.to_string(), value));
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Charset;

    #[allow(clippy::too_many_arguments)]
    fn switch(
        id: &mut EffectId,
        scene: &mut Box<dyn Scene>,
        t: &mut f64,
        flash: &mut f32,
        theme: &mut &'static Theme,
        charset: &mut Charset,
        transition: &mut Option<Transition>,
        target: EffectId,
        mode: TransitionMode,
    ) {
        switch_effect(
            id,
            &mut 7,
            target,
            scene,
            t,
            flash,
            &mut false,
            theme,
            charset,
            false,
            80,
            24,
            transition,
            mode,
            if mode == TransitionMode::None {
                0.0
            } else {
                0.6
            },
        );
    }

    #[test]
    fn switching_starts_a_transition_and_keeps_the_old_scene() {
        let mut id = EffectId::Rain;
        let mut scene = scene::build(id, 7);
        let mut t = 3.0;
        let mut flash = 0.0;
        let mut theme = id.default_theme();
        let mut charset = id.default_charset();
        let mut transition = None;

        switch(
            &mut id,
            &mut scene,
            &mut t,
            &mut flash,
            &mut theme,
            &mut charset,
            &mut transition,
            EffectId::Fire,
            TransitionMode::Fade,
        );

        assert_eq!(id, EffectId::Fire);
        assert_eq!(t, 0.0, "the incoming scene restarts its clock");
        assert_eq!(flash, 0.0, "no flash when blending");
        let tr = transition.expect("a transition should be running");
        assert_eq!(tr.from_t, 3.0, "the outgoing scene keeps its clock");
        assert!(std::ptr::eq(tr.from_theme, EffectId::Rain.default_theme()));
        assert_eq!(tr.from_charset, EffectId::Rain.default_charset());
        assert_eq!(tr.duration, 0.6);
    }

    #[test]
    fn switching_during_a_transition_starts_from_the_current_scene() {
        let mut id = EffectId::Rain;
        let mut scene = scene::build(id, 7);
        let mut t = 0.0;
        let mut flash = 0.0;
        let mut theme = id.default_theme();
        let mut charset = id.default_charset();
        let mut transition = None;

        switch(
            &mut id,
            &mut scene,
            &mut t,
            &mut flash,
            &mut theme,
            &mut charset,
            &mut transition,
            EffectId::Fire,
            TransitionMode::Dissolve,
        );
        t = 0.4;
        switch(
            &mut id,
            &mut scene,
            &mut t,
            &mut flash,
            &mut theme,
            &mut charset,
            &mut transition,
            EffectId::Matrix,
            TransitionMode::Dissolve,
        );

        assert_eq!(id, EffectId::Matrix);
        let tr = transition.expect("the second switch should blend too");
        assert!((tr.from_t - 0.4).abs() < 1e-9);
        assert!(std::ptr::eq(tr.from_theme, EffectId::Fire.default_theme()));
    }

    #[test]
    fn none_mode_keeps_the_hard_cut_and_flash() {
        let mut id = EffectId::Rain;
        let mut scene = scene::build(id, 7);
        let mut t = 2.0;
        let mut flash = 0.0;
        let mut theme = id.default_theme();
        let mut charset = id.default_charset();
        let mut transition = None;

        switch(
            &mut id,
            &mut scene,
            &mut t,
            &mut flash,
            &mut theme,
            &mut charset,
            &mut transition,
            EffectId::Fire,
            TransitionMode::None,
        );

        assert!(transition.is_none());
        assert_eq!(flash, 0.3);
        assert_eq!(t, 0.0);
    }

    #[test]
    fn same_effect_is_a_no_op() {
        let mut id = EffectId::Rain;
        let mut scene = scene::build(id, 7);
        let mut t = 2.0;
        let mut flash = 0.0;
        let mut theme = id.default_theme();
        let mut charset = id.default_charset();
        let mut transition = None;

        switch(
            &mut id,
            &mut scene,
            &mut t,
            &mut flash,
            &mut theme,
            &mut charset,
            &mut transition,
            EffectId::Rain,
            TransitionMode::Fade,
        );

        assert!(transition.is_none());
        assert_eq!(t, 2.0);
        assert_eq!(flash, 0.0);
    }

    #[test]
    fn uppercase_keys_are_normalized() {
        assert_eq!(normalize_key(Key::Char('Q')), Key::Char('q'));
        assert_eq!(normalize_key(Key::Char('N')), Key::Char('n'));
        assert_eq!(normalize_key(Key::Char('?')), Key::Char('?'));
        assert_eq!(normalize_key(Key::Up), Key::Up);
    }

    #[test]
    fn info_params_list_the_current_scene_knobs() {
        let tune = Tune {
            seed_count: 123,
            seed_speed: 1.6,
            ..Tune::default()
        };
        let opts = Options::default();
        let th = theme::find("gold").unwrap();
        let params = info_params(
            EffectId::Dandelion,
            &tune,
            &opts,
            false,
            th,
            1.25,
            Charset::Unicode,
            ColorMode::True,
        );
        let text = params
            .iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("--seed-count 123"), "{text}");
        assert!(text.contains("--seed-speed 1.60"), "{text}");
        assert!(text.contains("--charset unicode"), "{text}");
        assert!(text.contains("--speed 1.25"), "{text}");
    }

    #[test]
    fn info_params_cover_every_effect_specific_flag() {
        let tune = Tune::default();
        let opts = Options::default();
        let th = theme::find("neon").unwrap();
        for id in EffectId::ALL {
            let params = info_params(
                id,
                &tune,
                &opts,
                false,
                th,
                1.0,
                Charset::Ascii,
                ColorMode::True,
            );
            for t in id.tuning() {
                assert!(
                    params.iter().any(|(k, _)| k == t.flag),
                    "{id:?} missing {} in info panel",
                    t.flag
                );
            }
        }
    }

    #[test]
    fn info_params_show_playlist_settings() {
        let tune = Tune::default();
        let opts = Options::default();
        let th = theme::find("neon").unwrap();
        let params = info_params(
            EffectId::Rain,
            &tune,
            &opts,
            true,
            th,
            1.0,
            Charset::Ascii,
            ColorMode::True,
        );
        let text = params
            .iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("--playlist cycle"), "{text}");
        assert!(text.contains("--switch 12.0"), "{text}");
        assert!(text.contains("--transition fade"), "{text}");
    }

    #[test]
    fn pacer_drops_to_its_floor_under_pressure() {
        let mut p = Pacer::new(30, 12);
        assert_eq!(p.frame_dur(), Duration::from_secs_f64(1.0 / 30.0));
        let slow = Duration::from_millis(60);
        for _ in 0..12 {
            p.observe(slow);
        }
        assert_eq!(
            p.fps, 12,
            "three slow frames per step, 30 -> 24 -> 19 -> 15 -> 12"
        );
        for _ in 0..30 {
            p.observe(slow);
        }
        assert_eq!(
            p.fps, 12,
            "the governor must not go below the profile floor"
        );
    }

    #[test]
    fn pacer_throttles_before_the_pipe_saturates() {
        // 14 ms of write time at 30 fps is 42% of the budget: far too little
        // headroom for a transition or panel repaint, so the governor must
        // step down even though the frame still technically fits.
        let mut p = Pacer::new(30, 12);
        let present = Duration::from_millis(14);
        for _ in 0..3 {
            p.observe(present);
        }
        assert_eq!(p.fps, 24);
        for _ in 0..3 {
            p.observe(present);
        }
        assert_eq!(p.fps, 19);
        for _ in 0..60 {
            p.observe(present);
        }
        assert_eq!(p.fps, 19, "14 ms is comfortable in a 52 ms frame");
    }

    #[test]
    fn pacer_recovers_slowly_when_writes_are_cheap() {
        let mut p = Pacer::new(30, 12);
        p.fps = 12;
        let fast = Duration::from_millis(1);
        for _ in 0..89 {
            p.observe(fast);
        }
        assert_eq!(p.fps, 12);
        p.observe(fast);
        assert_eq!(p.fps, 14);
        for _ in 0..90 * 20 {
            p.observe(fast);
        }
        assert_eq!(p.fps, 30, "it must never exceed the requested rate");
    }

    #[test]
    fn high_profile_never_adapts() {
        let opts = Options {
            perf: cli::PerfMode::High,
            fps: 60,
            ..Options::default()
        };
        let mut p = Pacer::new(opts.fps, opts.perf.min_fps(opts.fps));
        assert_eq!(p.min_fps, 60);
        for _ in 0..100 {
            p.observe(Duration::from_millis(500));
        }
        assert_eq!(p.fps, 60);
    }
}
