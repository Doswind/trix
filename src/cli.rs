//! Command-line parsing.  Hand-rolled so that `--help` and start-up stay
//! instant and the binary needs no argument-parsing dependency.

use std::path::PathBuf;

use crate::color::ColorMode;
use crate::theme;

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
    pub petal_count: u32,
    pub petal_speed: f32,
    pub tornado_count: u32,
    pub tornado_speed: f32,
    pub seed: Option<u64>,
    pub size: Option<(usize, usize)>,
    pub no_alt: bool,
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
            petal_count: 80,
            petal_speed: 1.0,
            tornado_count: 3,
            tornado_speed: 1.0,
            seed: None,
            size: None,
            no_alt: false,
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
    List,
    Help,
    Version,
}

pub fn parse_args(args: &[String]) -> Result<Parsed, String> {
    let mut opts = Options::default();
    let mut positionals: Vec<String> = Vec::new();
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
                "help" => return Ok(Parsed::Help),
                "version" => return Ok(Parsed::Version),
                "list" => return Ok(Parsed::List),
                "bench" => opts.bench = true,
                "no-alt" | "no-altscreen" => opts.no_alt = true,
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
                "petal-count" | "petals" => {
                    let v = take_value(args, &mut i, inline, "--petal-count")?;
                    opts.petal_count = parse_num(&v, "--petal-count")?;
                }
                "petal-speed" | "fall-speed" => {
                    let v = take_value(args, &mut i, inline, "--petal-speed")?;
                    opts.petal_speed = parse_num(&v, "--petal-speed")?;
                }
                "tornado-count" => {
                    let v = take_value(args, &mut i, inline, "--tornado-count")?;
                    opts.tornado_count = parse_num(&v, "--tornado-count")?;
                }
                "tornado-speed" => {
                    let v = take_value(args, &mut i, inline, "--tornado-speed")?;
                    opts.tornado_speed = parse_num(&v, "--tornado-speed")?;
                }
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
                        format!("invalid --charset '{v}' (expected kana or ascii)")
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
                _ => return Err(format!("unknown option '--{name}'")),
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
                "h" => return Ok(Parsed::Help),
                "V" => return Ok(Parsed::Version),
                "l" => return Ok(Parsed::List),
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
                _ => return Err(format!("unknown option '-{flag}'")),
            }
            i += 1;
            continue;
        }
        positionals.push(arg.clone());
        i += 1;
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

    validate(&mut opts)?;
    Ok(Parsed::Run(opts))
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
    if !(1..=300).contains(&opts.petal_count) {
        return Err("--petal-count must be between 1 and 300".into());
    }
    if !opts.petal_speed.is_finite() || !(0.05..=4.0).contains(&opts.petal_speed) {
        return Err("--petal-speed must be between 0.05 and 4.0".into());
    }
    if !(1..=8).contains(&opts.tornado_count) {
        return Err("--tornado-count must be between 1 and 8".into());
    }
    if !opts.tornado_speed.is_finite() || !(0.05..=4.0).contains(&opts.tornado_speed) {
        return Err("--tornado-speed must be between 0.05 and 4.0".into());
    }
    Ok(())
}

pub fn help_text() -> String {
    format!(
        r#"
  t r i x   ::  terminal effects engine  v{VERSION}

  Low-footprint, sci-fi ASCII effects for your terminal.

USAGE
  trix [effect] [options]
  trix --list
  trix <effect> --snapshot out.ans --size 120x36 --frames 180

EFFECTS
  rain      代码雨      blue/green code rain with falling source tokens
  fire      燃烧火焰    bottom fire bed 2x wider, separate slender tips
  dragon    巨龙盘旋    Chinese celestial dragon flying across the sky
  map       世界地图    holographic Earth with live network arcs
  matrix    黑客帝国    NASA backend login screen with a large logo
  blackhole 黑洞        horizon + slower, wider outer star orbits
  beams     光束        one-way beams; straight or jumping motion
  fireworks 烟花        layered fireworks over a neon skyline
  solar     太阳系      Sun + planets; radial light wave sweeps outward
  galaxy    满天星河    full-screen Milky Way starfield
  buddha    佛光普照    glowing Buddha, halo and lotus mandala
  tomjerry  猫和老鼠    Tom chasing Jerry through a space station
  mario     超级马里奥  mushroom-bumping platformer (Unicode / --ascii)
  ecg       心率跳动    glowing ECG monitor (Unicode / --ascii)
  dandelion 蒲公英      clustered stems, staggered blooms, drifting seeds
  fish      鱼缸        colorful aquarium fish in every direction
  landscape 五光十色    mountains, lake, boat, geese and jungle
  shanghai  东方明珠    Shanghai Bund and Oriental Pearl (ASCII default)
  tiananmen 光芒万丈    Tiananmen gate radiating light
  traffic   车水马龙    city street with cars; --street horizontal|vertical
  fairy     仙女散花    flying fairy scattering petals
  tornado   龙卷风      twisted funnels, 2x wider tops, heavy base debris
  storm     电闪雷鸣    rain, clouds and full-screen lightning flashes

OPTIONS
  -e, --effect <name>     Effect to run; omit to loop all effects
      --playlist <mode>   cycle | random (default: cycle)  [no effect]
      --switch <secs>     seconds per effect in playlist   [default: 12]
  -f, --fps <1-240>       Target frame rate                    [default: 30]
  -s, --speed <0.05-8>    Animation speed multiplier           [default: 1.0]
      --chase-speed <f>   Alias of --speed (Tom & Jerry chase)
  -t, --theme <name>      Color theme, see below               [effect default]
      --charset <mode>    kana | ascii | unicode              [see effect defaults]
      --unicode           Force Unicode single-cell glyphs
      --ascii             Force ASCII fallback
      --color <mode>      auto | truecolor | 256 | 16 | mono   [default: auto]
      --density <f>       beams/fireworks/galaxy density      [default: 1.0]
      --spin <f>          black hole rotation speed            [default: 0.5]
      --twinkle <f>       galaxy twinkle rate                  [default: 1.0]
      --meteors <n>       black hole / galaxy meteors          [default: 3]
      --beam-speed <f>    beams horizontal speed               [default: 1.0]
      --beam-motion <m>   straight | jump (default: straight)
      --beam-jump         alias for --beam-motion jump
      --seed-count <n>    dandelion drifting seeds            [default: 90]
      --seed-speed <f>    dandelion seed drift speed          [default: 1.0]
      --street <dir>      traffic street: horizontal|vertical [default: horizontal]
      --petal-count <n>   fairy falling petals                [default: 80]
      --petal-speed <f>   fairy petal fall speed              [default: 1.0]
      --tornado-count <n> tornado count                        [default: 3]
      --tornado-speed <f> tornado travel speed                 [default: 1.0]
      --jump              Tom & Jerry jumps (default: off)
      --no-jump           disable Tom & Jerry jumps
      --seed <u64>        Deterministic random seed
      --duration <secs>   Exit automatically after N seconds
      --no-alt            Do not use the alternate screen
  -l, --list              List effects and themes
  -h, --help              Show this help
  -V, --version           Print version

CAPTURE / BENCHMARK
      --snapshot <path>   Render after --frames steps to an ANSI file ('-' = stdout)
      --frames <n>        Steps to simulate before snapshot / benchmark
      --size <WxH>        Offscreen canvas size for snapshot/bench
      --bench             Offscreen render benchmark (includes ANSI encoding)

THEMES
  {themes}

KEYS (interactive)
  q / Esc   quit            space   pause
  n / p     next/prev      1-9,0,[ ]  first 12 effects; m/e/d more
  m/e/d     Mario/ECG/Dandelion
  f/l/b/j   Fish/Landscape/Bund/Gate
  c/x/z/k   Traffic/Fairy/Tornado/Storm
  t         next theme     + / -   speed
  r         restart        i       status line
  h         help overlay

EXAMPLES
  trix
  trix --playlist random --switch 8
  trix rain
  trix fire --fps 60 --theme ember
  trix dragon -f 45 -t gold
  trix map --theme ice
  trix matrix --ascii
  trix map --snapshot earth.ans --size 120x36 --frames 150
  trix mario --ascii
  trix dandelion --seed-count 180 --seed-speed 1.4
  trix traffic --street vertical
  trix fairy --petal-count 160 --petal-speed 1.5
  trix tornado --tornado-count 5 --tornado-speed 1.6
"#,
        VERSION = VERSION,
        themes = theme::THEMES
            .iter()
            .map(|t| format!("{} ({})", t.key, t.label))
            .collect::<Vec<_>>()
            .join("\n  "),
    )
}

pub fn list_text() -> String {
    let mut s = String::new();
    s.push_str("EFFECTS\n");
    for e in crate::scene::EffectId::ALL {
        s.push_str(&format!(
            "  {:<10} {:<28} {}\n",
            e.key(),
            e.title_en(),
            e.title_zh()
        ));
    }
    s.push_str("\nTHEMES\n");
    for t in theme::THEMES {
        s.push_str(&format!("  {:<10} {}\n", t.key, t.label));
    }
    s.push_str("\nRun `trix --help` for options.\n");
    s
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
    fn parses_playlist_and_street() {
        let Parsed::Run(o) = parse_args(&args(
            "--playlist random --switch 8 --street vertical --petal-count 150 --petal-speed 1.5 --tornado-count 5 --tornado-speed 1.8",
        ))
        .unwrap()
        else {
            panic!()
        };
        assert_eq!(o.playlist, PlaylistMode::Random);
        assert!((o.switch_secs - 8.0).abs() < f32::EPSILON);
        assert!(o.street_vertical);
        assert_eq!(o.petal_count, 150);
        assert_eq!(o.tornado_count, 5);
    }

    #[test]
    fn rejects_bad_playlist_values() {
        assert!(parse_args(&args("--playlist sideways")).is_err());
        assert!(parse_args(&args("--switch 0")).is_err());
        assert!(parse_args(&args("traffic --street diagonal")).is_err());
        assert!(parse_args(&args("fairy --petal-count 0")).is_err());
        assert!(parse_args(&args("tornado --tornado-count 9")).is_err());
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
        assert!(matches!(parse_args(&args("--help")).unwrap(), Parsed::Help));
        assert!(matches!(parse_args(&args("-V")).unwrap(), Parsed::Version));
        assert!(matches!(parse_args(&args("--list")).unwrap(), Parsed::List));
    }

    #[test]
    fn duplicate_effect_rejected() {
        assert!(parse_args(&args("rain --effect fire")).is_err());
    }
}
