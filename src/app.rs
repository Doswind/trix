//! Application entry: CLI dispatch, interactive loop, snapshot & benchmark.

use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::canvas::Canvas;
use crate::cli::{self, Options, Parsed};
use crate::color::ColorMode;
use crate::rng::Rng;
use crate::scene::{self, EffectId, Frame, Scene, Tune};
use crate::term::{self, Key, Terminal};
use crate::theme::{self, Theme};

const SNAPSHOT_SEED: u64 = 0x7e51_a017;

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cli::parse_args(&args) {
        Ok(Parsed::Help) => {
            print!("{}", cli::help_text());
            0
        }
        Ok(Parsed::Version) => {
            println!("trix {}", cli::VERSION);
            0
        }
        Ok(Parsed::List) => {
            print!("{}", cli::list_text());
            0
        }
        Ok(Parsed::Run(opts)) => run_options(opts),
        Err(e) => {
            eprintln!("trix: {e}");
            eprintln!("Try 'trix --help' for usage.");
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
    let id = EffectId::parse(name).ok_or_else(|| {
        let names = EffectId::ALL
            .iter()
            .map(|e| e.key())
            .collect::<Vec<_>>()
            .join(", ");
        format!("unknown effect '{name}' (available: {names})")
    })?;
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
        seed_count: opts.seed_count,
        seed_speed: opts.seed_speed,
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
    let mut times = Vec::with_capacity(frames as usize);
    let mut t = 0.0;

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
        let _ = canvas.present(&mut sink, mode);
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
        let n = canvas.present(&mut sink, mode).unwrap_or(0);
        let el = start.elapsed().as_secs_f64();
        total += el;
        times.push(el);
        bytes += n;
    }
    let avg = total / frames as f64;
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p95 = times[((frames as f64 * 0.95) as usize).min(times.len() - 1)];
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
        "  ansi        avg {:.1} KiB/frame  ({:.2} MiB/s at {fps} fps)",
        avg_bytes / 1024.0,
        avg_bytes * fps as f64 / (1024.0 * 1024.0)
    );
    println!("  note        timed section = scene update + draw + diff encode");
    0
}

fn interactive(opts: Options) -> i32 {
    let (mut id, mut seed) = match resolve_effect(&opts, false) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("trix: {e}");
            return 2;
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
    let mut scene = scene::build(id, seed);
    scene.resize(w, h);
    let tune = tune_for(&opts);

    let started = Instant::now();
    let frame_dur = Duration::from_secs_f64(1.0 / opts.fps as f64);
    let mut next_frame = Instant::now();
    let mut last_tick = Instant::now();
    let mut t = 0.0f64;
    let mut paused = false;
    let mut speed = opts.speed;
    let mut show_help = false;
    let mut show_status = false;
    let mut flash = 0.0f32;
    let mut fps_smooth = opts.fps as f32;
    let mut frame_ms = 0.0f32;
    let hint_until = 6.0f32;

    loop {
        let now = Instant::now();
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
            scene.resize(w, h);
            term.clear();
            canvas.invalidate();
        }

        for key in term.read_keys() {
            match key {
                Key::Char('q') | Key::Char('\u{3}') => return 0,
                Key::Esc => {
                    if show_help {
                        show_help = false;
                    } else {
                        return 0;
                    }
                }
                Key::Char(' ') => paused = !paused,
                Key::Char('h') | Key::Char('?') => show_help = !show_help,
                Key::Char('i') => show_status = !show_status,
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
                    );
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
                    );
                }
                Key::Char('r') => {
                    seed = Rng::from_entropy().next_u64();
                    scene = scene::build(id, seed);
                    scene.resize(w, h);
                    t = 0.0;
                    flash = 0.3;
                }
                Key::Char(c @ ('1'..='9' | '0' | '[' | ']' | 'm' | 'e' | 'd')) => {
                    let idx = match c {
                        '0' => 9,
                        '[' => 10,
                        ']' => 11,
                        'm' => 12,
                        'e' => 13,
                        'd' => 14,
                        _ => c as usize - '1' as usize,
                    };
                    let target = EffectId::from_index(idx);
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
                    );
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

        if !paused {
            t += real_dt as f64 * speed as f64;
        }
        let anim_dt = if paused { 0.0 } else { real_dt * speed };
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

        let render_start = Instant::now();
        if !paused {
            scene.update(&frame);
        }
        canvas.clear();
        scene.draw(&mut canvas, &frame);

        let elapsed = started.elapsed().as_secs_f32();
        if !show_help && elapsed < hint_until {
            draw_hint(&mut canvas, &frame, elapsed, hint_until);
        }
        if show_help {
            draw_help(&mut canvas, &frame, id, current_theme, speed, paused);
        } else if paused {
            draw_pause(&mut canvas, &frame);
        }
        if show_status && !show_help {
            draw_status(
                &mut canvas,
                &frame,
                id,
                current_theme,
                speed,
                paused,
                fps_smooth,
                frame_ms,
                mode,
            );
        }
        if flash > 0.0 {
            draw_flash(&mut canvas, &frame, flash);
            flash = (flash - real_dt).max(0.0);
        }

        if let Err(e) = term.present(&mut canvas, mode) {
            let _ = e;
            return 0;
        }
        frame_ms = render_start.elapsed().as_secs_f32() * 1000.0;

        if let Some(limit) = opts.duration {
            if started.elapsed().as_secs_f64() >= limit {
                return 0;
            }
        }

        next_frame += frame_dur;
        let now = Instant::now();
        if next_frame > now {
            std::thread::sleep(next_frame - now);
        } else {
            next_frame = now;
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
) {
    if target == *id {
        return;
    }
    *id = target;
    *seed = Rng::from_entropy().next_u64();
    if !*explicit_theme {
        *current_theme = target.default_theme();
    }
    if !explicit_charset {
        *charset = target.default_charset();
    }
    *sc = scene::build(target, *seed);
    sc.resize(w, h);
    *t = 0.0;
    *flash = 0.3;
}

fn centered_x(w: usize, text: &str) -> isize {
    let tw = text.chars().count();
    ((w as isize - tw as isize) / 2).max(0)
}

fn draw_hint(canvas: &mut Canvas, frame: &Frame, elapsed: f32, hint_until: f32) {
    if frame.h < 4 || frame.w < 30 {
        return;
    }
    let text = if frame.w >= 76 {
        "q quit  |  space pause  |  n next  |  t theme  |  h help"
    } else {
        "q quit  |  n next  |  h help"
    };
    let x = centered_x(frame.w, text);
    let y = frame.h as isize - 1;
    canvas.clear_rect(0, y, frame.w as isize, 1);
    let fade = ((hint_until - elapsed) / 1.8).clamp(0.0, 1.0);
    let color = frame.theme.c(2).scale(0.35 + 0.55 * fade);
    canvas.put_str(x, y, text, color);
}

fn draw_pause(canvas: &mut Canvas, frame: &Frame) {
    if frame.w < 24 || frame.h < 6 {
        return;
    }
    let text = " PAUSED ";
    let bw = text.len() as isize + 4;
    let x = ((frame.w as isize - bw) / 2).max(0);
    let y = frame.h as isize / 2 - 1;
    canvas.clear_rect(x, y, bw, 3);
    canvas.rect(x, y, bw, 3, frame.theme.c(2).scale(0.8));
    canvas.put_str(x + 3, y + 1, text, frame.theme.hot());
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
fn draw_status(
    canvas: &mut Canvas,
    frame: &Frame,
    id: EffectId,
    theme: &Theme,
    speed: f32,
    paused: bool,
    fps: f32,
    frame_ms: f32,
    mode: ColorMode,
) {
    if frame.h < 3 {
        return;
    }
    let text = format!(
        " {:.0}fps {:.1}ms | {} | {} | {:.2}x | {}x{} | {} | seed {}{}",
        fps,
        frame_ms,
        id.key(),
        theme.key,
        speed,
        frame.w,
        frame.h,
        mode.label(),
        frame.seed,
        if paused { " | PAUSED" } else { "" }
    );
    let y = frame.h as isize - 1;
    canvas.clear_rect(0, y, frame.w as isize, 1);
    let shown: String = text.chars().take(frame.w.saturating_sub(1)).collect();
    canvas.put_str(0, y, &shown, theme.c(2).scale(0.75));
}

fn draw_help(
    canvas: &mut Canvas,
    frame: &Frame,
    id: EffectId,
    theme: &Theme,
    speed: f32,
    paused: bool,
) {
    let w = frame.w as isize;
    let h = frame.h as isize;
    if w < 34 || h < 10 {
        canvas.clear_rect(0, 0, w, 1);
        canvas.put_str(0, 0, " h/q close help  n/p next effect ", theme.c(2));
        return;
    }
    let bw = 62.min(w - 4).max(32);
    let bh = 20.min(h - 2).max(11);
    let x = (w - bw) / 2;
    let y = (h - bh) / 2;
    canvas.clear_rect(x, y, bw, bh);
    canvas.rect(x, y, bw, bh, theme.c(2).scale(0.65));
    let title = "TRIX // MISSION CONTROL";
    canvas.put_str_gradient(x + 2, y + 1, title, &[theme.c(2), theme.c(3), theme.hot()]);
    let rows: &[(&str, String)] = &[
        ("q / Esc", "quit".to_string()),
        (
            "space",
            if paused {
                "resume".into()
            } else {
                "pause".into()
            },
        ),
        ("n / p", "next / previous effect".into()),
        ("1-9,0,[ ]", "first 12 effects".into()),
        ("m / e / d", "Mario / ECG / Dandelion".into()),
        ("t", "cycle theme".into()),
        ("+ / -", format!("speed  {:.2}x", speed)),
        ("r", "restart current effect".into()),
        ("i", "status line".into()),
        ("h", "close this panel".into()),
    ];
    for (i, (key, desc)) in rows.iter().enumerate() {
        let line = y + 3 + i as isize;
        if line >= y + bh - 4 {
            break;
        }
        canvas.put_str(x + 2, line, key, theme.c(3));
        canvas.put_str(x + 12, line, desc, theme.c(2).scale(0.95));
    }
    // Current effect list on the right half when there's room.
    if bw >= 56 {
        canvas.put_str(x + 36, y + 2, "EFFECTS", theme.c(1).scale(1.5));
        for (i, e) in EffectId::ALL.iter().enumerate() {
            let ey = y + 3 + i as isize;
            if ey >= y + bh - 2 {
                break;
            }
            let marker = if *e == id { ">" } else { " " };
            let color = if *e == id {
                theme.hot()
            } else {
                theme.c(2).scale(0.8)
            };
            canvas.put_str(x + 36, ey, marker, color);
            canvas.put_str(x + 38, ey, e.key(), color);
        }
    }
    let footer = "space pauses, all effects animate in real time";
    if y + bh - 2 > y {
        canvas.put_str(x + 2, y + bh - 2, footer, theme.c(1).scale(1.4));
    }
}
