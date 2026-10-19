//! Command-line parsing.  Hand-rolled so that `--help` and start-up stay
//! instant and the binary needs no argument-parsing dependency.

use std::path::PathBuf;

use crate::color::ColorMode;
use crate::theme;
use crate::transition::TransitionMode;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaylistMode {
    Cycle,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Charset {
    Kana,
    Ascii,
    Unicode,
}

impl Charset {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "kana" | "katakana" | "matrix" => Some(Charset::Kana),
            "ascii" | "latin" => Some(Charset::Ascii),
            "unicode" | "utf8" | "utf-8" => Some(Charset::Unicode),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub effect: Option<String>,
    pub playlist: PlaylistMode,
    pub switch_secs: f32,
    /// How to blend from one effect into the next.
    pub transition: TransitionMode,
    /// Length of that blend in seconds.
    pub transition_secs: f32,
    pub fps: u32,
    pub speed: f32,
    pub theme: Option<String>,
    pub charset: Option<Charset>,
    pub color: Option<ColorMode>,
    pub duration: Option<f64>,
    pub density: f32,
    pub spin: f32,
    pub twinkle: f32,
    pub meteors: u32,
    pub beam_speed: f32,
    pub jump: bool,
    pub beam_jump: bool,
    pub seed_count: u32,
    pub seed_speed: f32,
    pub street_vertical: bool,
    pub tornado_count: u32,
    pub tornado_speed: f32,
    pub life_cycle: f32,
    pub eye_still: bool,
    /// dsh: afterimage strength (0 turns it off).
    pub trail: f32,
    /// monalisa: draw the flat pure-ASCII "ghost" portrait.
    pub ghost: bool,
    /// dragon: plain starfield, no cloud sea / mountains.
    pub plain: bool,
    /// monalisa: soft gallery glow behind the portrait.
    pub gallery: bool,
    pub seed: Option<u64>,
    pub size: Option<(usize, usize)>,
    pub no_alt: bool,
    /// Hide the persistent top/bottom HUD in interactive mode.
    pub no_hud: bool,
    pub snapshot: Option<PathBuf>,
    pub frames: Option<u64>,
    pub bench: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            effect: None,
            playlist: PlaylistMode::Cycle,
            switch_secs: 12.0,
            transition: TransitionMode::Fade,
            transition_secs: 0.6,
            fps: 30,
            speed: 1.0,
            theme: None,
            charset: None,
            color: None,
            duration: None,
            density: 1.0,
            spin: 0.5,
            twinkle: 1.0,
            meteors: 3,
            beam_speed: 1.0,
            jump: false,
            beam_jump: false,
            seed_count: 90,
            seed_speed: 1.0,
            street_vertical: false,
            tornado_count: 3,
            tornado_speed: 1.0,
            life_cycle: 120.0,
            eye_still: false,
            trail: 1.0,
            ghost: false,
            plain: false,
            gallery: false,
            seed: None,
            size: None,
            no_alt: false,
            no_hud: false,
            snapshot: None,
            frames: None,
            bench: false,
        }
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Parsed {
    Run(Options),
    List(Options),
    Help(Options),
    Completions(String),
    Version,
}

pub fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let mut opts = Options::default();
    let mut positionals: Vec<String> = Vec::new();
    let mut want_help = false;
    let mut want_list = false;
    let mut want_version = false;
    let mut want_completions: Option<String> = None;
    let mut i = 0;
    let mut only_positionals = false;

    while i < args.len() {
        let arg = &args[i];
        if only_positionals {
            positionals.push(arg.clone());
            i += 1;
            continue;
        }
        if arg == "--" {
            only_positionals = true;
            i += 1;
            continue;
        }
        if let Some(rest) = arg.strip_prefix("--") {
            let (name, inline) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v)),
                None => (rest, None),
            };
            match name {
                "help" => want_help = true,
                "version" => want_version = true,
                "list" => want_list = true,
                "completions" => {
                    want_completions = Some(take_value(args, &mut i, inline, "--completions")?);
                }
                "bench" => opts.bench = true,
                "no-alt" | "no-altscreen" => opts.no_alt = true,
                "no-hud" => opts.no_hud = true,
                "no-color" => opts.color = Some(ColorMode::Mono),
                "ascii" => opts.charset = Some(Charset::Ascii),
                "kana" => opts.charset = Some(Charset::Kana),
                "unicode" | "utf8" => opts.charset = Some(Charset::Unicode),
                "jump" => opts.jump = true,
                "no-jump" => opts.jump = false,
                "effect" => {
                    opts.effect = Some(take_value(args, &mut i, inline, "--effect")?);
                }
                "density" => {
                    let v = take_value(args, &mut i, inline, "--density")?;
                    opts.density = parse_num(&v, "--density")?;
                }
                "spin" | "orbit-speed" | "rotation" => {
                    let v = take_value(args, &mut i, inline, "--spin")?;
                    opts.spin = parse_num(&v, "--spin")?;
                }
                "twinkle" => {
                    let v = take_value(args, &mut i, inline, "--twinkle")?;
                    opts.twinkle = parse_num(&v, "--twinkle")?;
                }
                "meteors" => {
                    let v = take_value(args, &mut i, inline, "--meteors")?;
                    opts.meteors = parse_num(&v, "--meteors")?;
                }
                "seed-count" | "seeds" => {
                    let v = take_value(args, &mut i, inline, "--seed-count")?;
                    opts.seed_count = parse_num(&v, "--seed-count")?;
                }
                "seed-speed" => {
                    let v = take_value(args, &mut i, inline, "--seed-speed")?;
                    opts.seed_speed = parse_num(&v, "--seed-speed")?;
                }
                "street" => {
                    let v = take_value(args, &mut i, inline, "--street")?;
                    opts.street_vertical = match v.to_ascii_lowercase().as_str() {
                        "horizontal" | "h" | "horiz" => false,
                        "vertical" | "v" | "vert" => true,
                        _ => return Err("--street must be 'horizontal' or 'vertical'".into()),
                    };
                }
                "vertical" => opts.street_vertical = true,
                "tornado-count" => {
                    let v = take_value(args, &mut i, inline, "--tornado-count")?;
                    opts.tornado_count = parse_num(&v, "--tornado-count")?;
                }
                "tornado-speed" => {
                    let v = take_value(args, &mut i, inline, "--tornado-speed")?;
                    opts.tornado_speed = parse_num(&v, "--tornado-speed")?;
                }
                "life-cycle" | "life-span" | "life-duration" => {
                    let v = take_value(args, &mut i, inline, "--life-cycle")?;
                    let n = parse_num(&v, "--life-cycle")?;
                    if !(5.0..=600.0).contains(&n) {
                        return Err("--life-cycle must be between 5 and 600 seconds".into());
                    }
                    opts.life_cycle = n;
                }
                "eye-motion" | "eye-move" => {
                    let v = take_value(args, &mut i, inline, "--eye-motion")?;
                    opts.eye_still = match v.to_ascii_lowercase().as_str() {
                        "dart" | "jump" | "move" | "moving" | "on" => false,
                        "still" | "fixed" | "static" | "none" | "off" => true,
                        _ => return Err("--eye-motion must be 'dart' or 'still'".into()),
                    };
                }
                "eye-still" | "no-eye-dart" | "eye-fixed" => opts.eye_still = true,
                "eye-dart" => opts.eye_still = false,
                "trail" => {
                    let v = take_value(args, &mut i, inline, "--trail")?;
                    let n = parse_num(&v, "--trail")?;
                    if !(0.0..=3.0).contains(&n) {
                        return Err("--trail must be between 0 and 3".into());
                    }
                    opts.trail = n;
                }
                "ghost" | "ghost-mode" | "ascii-art" | "mono-art" => opts.ghost = true,
                "no-ghost" => opts.ghost = false,
                "plain" | "no-bg" | "no-background" => opts.plain = true,
                "bg" | "background" => opts.plain = false,
                "gallery" => opts.gallery = true,
                "no-gallery" => opts.gallery = false,
                "beam-motion" => {
                    let v = take_value(args, &mut i, inline, "--beam-motion")?;
                    opts.beam_jump = match v.to_ascii_lowercase().as_str() {
                        "straight" | "line" | "direct" => false,
                        "jump" | "bounce" | "wave" | "updown" => true,
                        _ => return Err("--beam-motion must be 'straight' or 'jump'".into()),
                    };
                }
                "beam-jump" => opts.beam_jump = true,
                "beam-speed" => {
                    let v = take_value(args, &mut i, inline, "--beam-speed")?;
                    opts.beam_speed = parse_num(&v, "--beam-speed")?;
                }
                "playlist" | "mode" | "shuffle" => {
                    let v = take_value(args, &mut i, inline, "--playlist")?;
                    opts.playlist = match v.to_ascii_lowercase().as_str() {
                        "cycle" | "loop" | "sequence" => PlaylistMode::Cycle,
                        "random" | "shuffle" | "rand" => PlaylistMode::Random,
                        _ => return Err("--playlist must be 'cycle' or 'random'".into()),
                    };
                }
                "switch" | "interval" => {
                    let v = take_value(args, &mut i, inline, "--switch")?;
                    opts.switch_secs = parse_num(&v, "--switch")?;
                }
                "transition" | "transition-mode" | "blend" => {
                    let v = take_value(args, &mut i, inline, "--transition")?;
                    opts.transition = TransitionMode::parse(&v).ok_or_else(|| {
                        format!("invalid --transition '{v}' (expected fade, dissolve or none)")
                    })?;
                }
                "transition-secs" | "transition-time" | "fade-secs" => {
                    let v = take_value(args, &mut i, inline, "--transition-secs")?;
                    opts.transition_secs = parse_num(&v, "--transition-secs")?;
                }
                "no-fade" | "no-transition" => opts.transition = TransitionMode::None,
                "fade" => opts.transition = TransitionMode::Fade,
                "dissolve" | "crossfade" => opts.transition = TransitionMode::Dissolve,
                "fps" => {
                    let v = take_value(args, &mut i, inline, "--fps")?;
                    opts.fps = parse_num(&v, "--fps")?;
                }
                "speed" | "chase-speed" => {
                    let v = take_value(args, &mut i, inline, "--speed")?;
                    opts.speed = parse_num(&v, "--speed")?;
                }
                "theme" => {
                    opts.theme = Some(take_value(args, &mut i, inline, "--theme")?);
                }
                "charset" => {
                    let v = take_value(args, &mut i, inline, "--charset")?;
                    opts.charset = Some(Charset::parse(&v).ok_or_else(|| {
                        format!("invalid --charset '{v}' (expected kana, ascii or unicode)")
                    })?);
                }
                "color" => {
                    let v = take_value(args, &mut i, inline, "--color")?;
                    opts.color = parse_color(&v)?;
                }
                "duration" => {
                    let v = take_value(args, &mut i, inline, "--duration")?;
                    opts.duration = Some(parse_num(&v, "--duration")?);
                }
                "seed" => {
                    let v = take_value(args, &mut i, inline, "--seed")?;
                    opts.seed = Some(
                        v.parse()
                            .map_err(|_| format!("invalid --seed '{v}' (expected u64)"))?,
                    );
                }
                "size" => {
                    let v = take_value(args, &mut i, inline, "--size")?;
                    opts.size = Some(parse_size(&v)?);
                }
                "frames" => {
                    let v = take_value(args, &mut i, inline, "--frames")?;
                    opts.frames = Some(parse_num(&v, "--frames")?);
                }
                "snapshot" => {
                    opts.snapshot = Some(PathBuf::from(take_value(
                        args,
                        &mut i,
                        inline,
                        "--snapshot",
                    )?));
                }
                _ => return Err(unknown_option_message(&format!("--{name}"))),
            }
            i += 1;
            continue;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            let body = &arg[1..];
            let flag = &body[..1];
            let attached = if body.len() > 1 {
                Some(&body[1..])
            } else {
                None
            };
            match flag {
                "h" => want_help = true,
                "V" => want_version = true,
                "l" => want_list = true,
                "e" => {
                    opts.effect = Some(take_value(args, &mut i, attached, "-e")?);
                }
                "f" => {
                    let v = take_value(args, &mut i, attached, "-f")?;
                    opts.fps = parse_num(&v, "-f")?;
                }
                "s" => {
                    let v = take_value(args, &mut i, attached, "-s")?;
                    opts.speed = parse_num(&v, "-s")?;
                }
                "t" => {
                    opts.theme = Some(take_value(args, &mut i, attached, "-t")?);
                }
                _ => return Err(unknown_option_message(&format!("-{flag}"))),
            }
            i += 1;
            continue;
        }
        positionals.push(arg.clone());
        i += 1;
    }

    // `trix help [effect]` and `trix list` are friendly aliases for the
    // flag forms.  They are not effect names, so removing them here is safe.
    if let Some(first) = positionals.first() {
        if first.eq_ignore_ascii_case("help") {
            want_help = true;
            positionals.remove(0);
        } else if first.eq_ignore_ascii_case("list") {
            want_list = true;
            positionals.remove(0);
        }
    }

    if positionals.len() > 1 {
        return Err(format!(
            "unexpected extra argument '{}' (one effect at a time)",
            positionals[1]
        ));
    }
    if let Some(pos) = positionals.pop() {
        if opts.effect.is_some() {
            return Err("effect specified both positionally and with --effect".into());
        }
        opts.effect = Some(pos);
    }

    if want_help {
        return Ok(Parsed::Help(opts));
    }
    if want_version {
        return Ok(Parsed::Version);
    }
    if want_list {
        return Ok(Parsed::List(opts));
    }
    if let Some(shell) = want_completions {
        if !matches!(shell.as_str(), "bash" | "zsh" | "fish") {
            return Err(format!(
                "--completions must be 'bash', 'zsh' or 'fish' (got '{shell}')"
            ));
        }
        return Ok(Parsed::Completions(shell));
    }

    validate(&mut opts)?;
    Ok(Parsed::Run(opts))
}

/// Long flags that the help screens promise to document.  Tests use this to
/// keep `--help all`, the README and the parser from drifting apart.
pub const PRIMARY_FLAGS: &[&str] = &[
    "--effect",
    "--playlist",
    "--switch",
    "--transition",
    "--transition-secs",
    "--no-fade",
    "--fps",
    "--speed",
    "--chase-speed",
    "--theme",
    "--charset",
    "--unicode",
    "--ascii",
    "--color",
    "--no-color",
    "--no-hud",
    "--no-alt",
    "--density",
    "--spin",
    "--twinkle",
    "--meteors",
    "--beam-speed",
    "--beam-motion",
    "--beam-jump",
    "--seed-count",
    "--seed-speed",
    "--street",
    "--tornado-count",
    "--tornado-speed",
    "--life-cycle",
    "--eye-motion",
    "--eye-still",
    "--ghost",
    "--gallery",
    "--plain",
    "--jump",
    "--no-jump",
    "--snapshot",
    "--frames",
    "--size",
    "--bench",
    "--seed",
    "--duration",
    "--list",
    "--help",
    "--version",
    "--completions",
];

fn normalize_name(text: &str) -> String {
    text.to_ascii_lowercase().replace(['_', '-', ' '], "")
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (cur[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Closest effect key for "did you mean" hints.
pub fn suggest_effect(input: &str) -> Option<&'static str> {
    let needle = normalize_name(input);
    let mut best: Option<(&'static str, usize)> = None;
    for id in crate::scene::EffectId::ALL {
        let key = id.key();
        let distance = levenshtein(&needle, key);
        let limit = if key.len() <= 4 { 2 } else { 3 };
        if distance <= limit && best.map(|(_, d)| distance < d).unwrap_or(true) {
            best = Some((key, distance));
        }
    }
    best.map(|(key, _)| key)
}

/// Closest long option for "did you mean" hints.
pub fn suggest_option(input: &str) -> Option<&'static str> {
    let needle = normalize_name(input.trim_start_matches('-'));
    let mut best: Option<(&'static str, usize)> = None;
    for flag in PRIMARY_FLAGS {
        let name = flag.trim_start_matches('-');
        let distance = levenshtein(&needle, name);
        if distance <= 2 && best.map(|(_, d)| distance < d).unwrap_or(true) {
            best = Some((flag, distance));
        }
    }
    best.map(|(flag, _)| flag)
}

pub fn unknown_effect_message(name: &str) -> String {
    match suggest_effect(name) {
        Some(hit) => format!("unknown effect '{name}' (did you mean '{hit}'?)"),
        None => format!("unknown effect '{name}' (try 'trix --list')"),
    }
}

pub fn unknown_option_message(name: &str) -> String {
    match suggest_option(name) {
        Some(hit) => format!("unknown option '{name}' (did you mean '{hit}'?)"),
        None => format!("unknown option '{name}' (try 'trix --help all')"),
    }
}

fn take_value(
    args: &[String],
    i: &mut usize,
    inline: Option<&str>,
    opt: &str,
) -> Result<String, String> {
    if let Some(v) = inline {
        if v.is_empty() {
            return Err(format!("{opt} requires a value"));
        }
        return Ok(v.to_string());
    }
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("{opt} requires a value"))
}

fn parse_num<T: std::str::FromStr>(v: &str, opt: &str) -> Result<T, String> {
    v.parse()
        .map_err(|_| format!("invalid value '{v}' for {opt}"))
}

fn parse_size(v: &str) -> Result<(usize, usize), String> {
    let (w, h) = v
        .split_once('x')
        .or_else(|| v.split_once('X'))
        .ok_or_else(|| format!("invalid --size '{v}' (expected WxH, e.g. 120x36)"))?;
    let w: usize = w
        .parse()
        .map_err(|_| format!("invalid width in --size '{v}'"))?;
    let h: usize = h
        .parse()
        .map_err(|_| format!("invalid height in --size '{v}'"))?;
    if !(10..=1000).contains(&w) || !(5..=1000).contains(&h) {
        return Err(format!("--size out of range: {v} (10..1000 x 5..1000)"));
    }
    Ok((w, h))
}

fn parse_color(v: &str) -> Result<Option<ColorMode>, String> {
    if v.eq_ignore_ascii_case("auto") {
        return Ok(None);
    }
    ColorMode::parse(v)
        .map(Some)
        .ok_or_else(|| format!("invalid --color '{v}' (auto, truecolor, 256, 16, mono)"))
}

fn validate(opts: &mut Options) -> Result<(), String> {
    if !(1..=240).contains(&opts.fps) {
        return Err("--fps must be between 1 and 240".into());
    }
    if !opts.speed.is_finite() || !(0.05..=8.0).contains(&opts.speed) {
        return Err("--speed must be between 0.05 and 8.0".into());
    }
    if let Some(d) = opts.duration {
        if !d.is_finite() || d <= 0.0 {
            return Err("--duration must be a positive number of seconds".into());
        }
    }
    if let Some(t) = &opts.theme {
        if theme::find(t).is_none() {
            return Err(format!(
                "unknown theme '{t}' (available: {})",
                theme::theme_keys()
            ));
        }
    }
    if let Some(f) = opts.frames {
        if f == 0 || f > 1_000_000 {
            return Err("--frames must be between 1 and 1000000".into());
        }
        if opts.snapshot.is_none() && !opts.bench {
            return Err("--frames only makes sense with --snapshot or --bench".into());
        }
    }
    if opts.snapshot.is_some() && opts.bench {
        return Err("--snapshot and --bench cannot be combined".into());
    }
    if opts.size.is_some() && opts.snapshot.is_none() && !opts.bench {
        return Err("--size only makes sense with --snapshot or --bench".into());
    }
    if opts.duration.is_some() && opts.snapshot.is_some() {
        return Err("--duration does not apply to --snapshot".into());
    }
    if !opts.density.is_finite() || !(0.05..=4.0).contains(&opts.density) {
        return Err("--density must be between 0.05 and 4.0".into());
    }
    if !opts.spin.is_finite() || !(0.05..=4.0).contains(&opts.spin) {
        return Err("--spin must be between 0.05 and 4.0".into());
    }
    if !opts.twinkle.is_finite() || !(0.05..=8.0).contains(&opts.twinkle) {
        return Err("--twinkle must be between 0.05 and 8.0".into());
    }
    if opts.meteors > 30 {
        return Err("--meteors must be between 0 and 30".into());
    }
    if !opts.beam_speed.is_finite() || !(0.05..=4.0).contains(&opts.beam_speed) {
        return Err("--beam-speed must be between 0.05 and 4.0".into());
    }
    if !(1..=400).contains(&opts.seed_count) {
        return Err("--seed-count must be between 1 and 400".into());
    }
    if !opts.seed_speed.is_finite() || !(0.05..=4.0).contains(&opts.seed_speed) {
        return Err("--seed-speed must be between 0.05 and 4.0".into());
    }
    if !opts.switch_secs.is_finite() || !(1.0..=600.0).contains(&opts.switch_secs) {
        return Err("--switch must be between 1 and 600 seconds".into());
    }
    if !opts.transition_secs.is_finite() || !(0.0..=5.0).contains(&opts.transition_secs) {
        return Err("--transition-secs must be between 0 and 5 seconds".into());
    }
    if !(1..=8).contains(&opts.tornado_count) {
        return Err("--tornado-count must be between 1 and 8".into());
    }
    if !opts.tornado_speed.is_finite() || !(0.05..=4.0).contains(&opts.tornado_speed) {
        return Err("--tornado-speed must be between 0.05 and 4.0".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(|x| x.to_string()).collect()
    }

    #[test]
    fn parses_effect_and_options() {
        let Parsed::Run(o) = parse_args(&args("fire --fps 60 -s 1.5 --theme ember")).unwrap()
        else {
            panic!()
        };
        assert_eq!(o.effect.as_deref(), Some("fire"));
        assert_eq!(o.fps, 60);
        assert!((o.speed - 1.5).abs() < f32::EPSILON);
        assert_eq!(o.theme.as_deref(), Some("ember"));
    }

    #[test]
    fn parses_tuning_options() {
        let Parsed::Run(o) = parse_args(&args(
            "galaxy --density 2 --twinkle 3.5 --meteors 8 --spin 0.25 --beam-speed 1.5 --beam-motion jump --seed-count 180 --seed-speed 1.4 --jump --unicode",
        ))
        .unwrap()
        else {
            panic!()
        };
        assert!((o.density - 2.0).abs() < f32::EPSILON);
        assert!((o.twinkle - 3.5).abs() < f32::EPSILON);
        assert_eq!(o.meteors, 8);
        assert!((o.spin - 0.25).abs() < f32::EPSILON);
        assert!((o.beam_speed - 1.5).abs() < f32::EPSILON);
        assert!(o.beam_jump);
        assert_eq!(o.seed_count, 180);
        assert!((o.seed_speed - 1.4).abs() < f32::EPSILON);
        assert!(o.jump);
        assert_eq!(o.charset, Some(Charset::Unicode));
    }

    #[test]
    fn parses_plain_and_gallery_flags() {
        let Parsed::Run(o) = parse_args(&args("dragon --plain --gallery")).unwrap() else {
            panic!()
        };
        assert!(o.plain);
        assert!(o.gallery);
        let Parsed::Run(o) = parse_args(&args("dragon --plain --no-bg")).unwrap() else {
            panic!()
        };
        assert!(o.plain);
        let Parsed::Run(o) = parse_args(&args("monalisa --gallery --no-gallery")).unwrap() else {
            panic!()
        };
        assert!(!o.gallery);
    }

    #[test]
    fn parses_ghost_flag() {
        let Parsed::Run(o) = parse_args(&args("monalisa --ghost")).unwrap() else {
            panic!()
        };
        assert!(o.ghost);
        let Parsed::Run(o) = parse_args(&args("monalisa --ghost --no-ghost")).unwrap() else {
            panic!()
        };
        assert!(!o.ghost);
    }

    #[test]
    fn parses_no_hud() {
        let Parsed::Run(o) = parse_args(&args("rain --no-hud")).unwrap() else {
            panic!()
        };
        assert!(o.no_hud);
        let Parsed::Run(o) = parse_args(&args("rain")).unwrap() else {
            panic!()
        };
        assert!(!o.no_hud);
    }

    #[test]
    fn parses_playlist_and_street() {
        let Parsed::Run(o) = parse_args(&args(
            "--playlist random --switch 8 --street vertical --tornado-count 5 --tornado-speed 1.8 --life-cycle 90",
        ))
        .unwrap()
        else {
            panic!()
        };
        assert_eq!(o.playlist, PlaylistMode::Random);
        assert!((o.switch_secs - 8.0).abs() < f32::EPSILON);
        assert!(o.street_vertical);
        assert_eq!(o.tornado_count, 5);
        assert!((o.life_cycle - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_eye_motion() {
        let Parsed::Run(o) = parse_args(&args("demoneye")).unwrap() else {
            panic!()
        };
        assert!(!o.eye_still, "the eyeball darts by default");
        let Parsed::Run(o) = parse_args(&args("demoneye --eye-motion still")).unwrap() else {
            panic!()
        };
        assert!(o.eye_still);
        let Parsed::Run(o) = parse_args(&args("demoneye --eye-still")).unwrap() else {
            panic!()
        };
        assert!(o.eye_still);
        let Parsed::Run(o) = parse_args(&args("demoneye --eye-motion dart")).unwrap() else {
            panic!()
        };
        assert!(!o.eye_still);
        let Parsed::Run(o) = parse_args(&args("demoneye --eye-still --eye-dart")).unwrap() else {
            panic!()
        };
        assert!(!o.eye_still);
        assert!(parse_args(&args("demoneye --eye-motion sideways")).is_err());
    }

    #[test]
    fn parses_transition_options() {
        let Parsed::Run(o) =
            parse_args(&args("--transition dissolve --transition-secs 1.2")).unwrap()
        else {
            panic!()
        };
        assert_eq!(o.transition, TransitionMode::Dissolve);
        assert!((o.transition_secs - 1.2).abs() < f32::EPSILON);

        let Parsed::Run(o) = parse_args(&args("--no-fade")).unwrap() else {
            panic!()
        };
        assert_eq!(o.transition, TransitionMode::None);

        let Parsed::Run(o) = parse_args(&args("rain --crossfade")).unwrap() else {
            panic!()
        };
        assert_eq!(o.transition, TransitionMode::Dissolve);

        let Parsed::Run(o) = parse_args(&args("rain")).unwrap() else {
            panic!()
        };
        assert_eq!(o.transition, TransitionMode::Fade, "fade is the default");
        assert!((o.transition_secs - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn rejects_bad_transition_values() {
        assert!(parse_args(&args("--transition sideways")).is_err());
        assert!(parse_args(&args("--transition-secs -1")).is_err());
        assert!(parse_args(&args("--transition-secs 9")).is_err());
    }

    #[test]
    fn rejects_bad_playlist_values() {
        assert!(parse_args(&args("--playlist sideways")).is_err());
        assert!(parse_args(&args("--switch 0")).is_err());
        assert!(parse_args(&args("traffic --street diagonal")).is_err());
        assert!(parse_args(&args("tornado --tornado-count 9")).is_err());
        assert!(parse_args(&args("life --life-cycle 4")).is_err());
        assert!(parse_args(&args("life --life-cycle 601")).is_err());
    }

    #[test]
    fn rejects_bad_tuning() {
        assert!(parse_args(&args("fire --density 9")).is_err());
        assert!(parse_args(&args("fire --meteors 99")).is_err());
        assert!(parse_args(&args("fire --beam-speed 0")).is_err());
        assert!(parse_args(&args("fire --seed-count 0")).is_err());
        assert!(parse_args(&args("fire --seed-speed 9")).is_err());
        assert!(parse_args(&args("beams --beam-motion sideways")).is_err());
    }

    #[test]
    fn short_attached_values() {
        let Parsed::Run(o) = parse_args(&args("rain -f60 -tneon")).unwrap() else {
            panic!()
        };
        assert_eq!(o.fps, 60);
        assert_eq!(o.theme.as_deref(), Some("neon"));
    }

    #[test]
    fn handles_equals_and_sizes() {
        let Parsed::Run(o) =
            parse_args(&args("map --snapshot=out.ans --size=100x30 --frames=7")).unwrap()
        else {
            panic!()
        };
        assert_eq!(o.size, Some((100, 30)));
        assert_eq!(o.frames, Some(7));
        assert_eq!(o.snapshot.unwrap().to_str(), Some("out.ans"));
    }

    #[test]
    fn rejects_bad_values() {
        assert!(parse_args(&args("rain --fps 0")).is_err());
        assert!(parse_args(&args("rain --fps 999")).is_err());
        assert!(parse_args(&args("rain --theme nope")).is_err());
        assert!(parse_args(&args("rain --size 2x2")).is_err());
        assert!(parse_args(&args("rain extra")).is_err());
        assert!(parse_args(&args("rain --wat")).is_err());
        assert!(parse_args(&args("rain --frames 10")).is_err());
    }

    #[test]
    fn commands() {
        assert!(matches!(
            parse_args(&args("--help")).unwrap(),
            Parsed::Help(_)
        ));
        assert!(matches!(parse_args(&args("-V")).unwrap(), Parsed::Version));
        assert!(matches!(
            parse_args(&args("--list")).unwrap(),
            Parsed::List(_)
        ));
    }

    #[test]
    fn help_accepts_an_effect_and_style_flags() {
        let Parsed::Help(o) = parse_args(&args("--help rain --no-color")).unwrap() else {
            panic!()
        };
        assert_eq!(o.effect.as_deref(), Some("rain"));
        assert_eq!(o.color, Some(ColorMode::Mono));

        let Parsed::Help(o) = parse_args(&args("rain --help")).unwrap() else {
            panic!()
        };
        assert_eq!(o.effect.as_deref(), Some("rain"));

        let Parsed::Help(o) = parse_args(&args("help all")).unwrap() else {
            panic!()
        };
        assert_eq!(o.effect.as_deref(), Some("all"));

        let Parsed::List(o) = parse_args(&args("list")).unwrap() else {
            panic!()
        };
        assert_eq!(o.effect, None);
    }

    #[test]
    fn suggestions_are_useful() {
        assert_eq!(suggest_effect("rainn"), Some("rain"));
        assert_eq!(suggest_option("--fsp"), Some("--fps"));
        assert!(unknown_effect_message("rainn").contains("rain"));
        assert!(unknown_option_message("--fsp").contains("--fps"));
    }

    #[test]
    fn parses_completions() {
        assert!(matches!(
            parse_args(&args("--completions bash")).unwrap(),
            Parsed::Completions(ref shell) if shell == "bash"
        ));
        assert!(parse_args(&args("--completions nope")).is_err());
    }

    #[test]
    fn duplicate_effect_rejected() {
        assert!(parse_args(&args("rain --effect fire")).is_err());
    }
}
