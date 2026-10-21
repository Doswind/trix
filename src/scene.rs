//! Effect registry and the shared scene interface.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::scenes::{
    BalletScene, BeamsScene, BlackHoleScene, BuddhaScene, DandelionScene, DemonEyeScene,
    DragonScene, DshScene, EcgScene, FireScene, FireworksScene, FishScene, GalaxyScene,
    LandscapeScene, LaunchScene, LifeScene, LightsScene, MapScene, MarioScene, MatrixScene,
    MonaLisaScene, PlasmaScene, RainScene, ShanghaiScene, SolarScene, StormScene, TiananmenScene,
    TomJerryScene, TornadoScene, TrafficScene, VibeScene, VolcanoScene,
};
use crate::theme::{self, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectId {
    Rain,
    Fire,
    Dragon,
    Map,
    Matrix,
    BlackHole,
    Beams,
    Fireworks,
    Solar,
    Galaxy,
    Buddha,
    TomJerry,
    Mario,
    Ecg,
    Dandelion,
    FishTank,
    Landscape,
    Shanghai,
    Tiananmen,
    Traffic,
    Tornado,
    Storm,
    MonaLisa,
    Ballet,
    DemonEye,
    Life,
    Dsh,
    Vibe,
    Lights,
    Volcano,
    Launch,
    Plasma,
}

impl EffectId {
    pub const ALL: [EffectId; 32] = [
        EffectId::Rain,
        EffectId::Fire,
        EffectId::Dragon,
        EffectId::Map,
        EffectId::Matrix,
        EffectId::BlackHole,
        EffectId::Beams,
        EffectId::Fireworks,
        EffectId::Solar,
        EffectId::Galaxy,
        EffectId::Buddha,
        EffectId::TomJerry,
        EffectId::Mario,
        EffectId::Ecg,
        EffectId::Dandelion,
        EffectId::FishTank,
        EffectId::Landscape,
        EffectId::Shanghai,
        EffectId::Tiananmen,
        EffectId::Traffic,
        EffectId::Tornado,
        EffectId::Storm,
        EffectId::MonaLisa,
        EffectId::Ballet,
        EffectId::DemonEye,
        EffectId::Life,
        EffectId::Dsh,
        EffectId::Vibe,
        EffectId::Lights,
        EffectId::Volcano,
        EffectId::Launch,
        EffectId::Plasma,
    ];

    pub fn key(self) -> &'static str {
        match self {
            EffectId::Rain => "rain",
            EffectId::Fire => "fire",
            EffectId::Dragon => "dragon",
            EffectId::Map => "map",
            EffectId::Matrix => "matrix",
            EffectId::BlackHole => "blackhole",
            EffectId::Beams => "beams",
            EffectId::Fireworks => "fireworks",
            EffectId::Solar => "solar",
            EffectId::Galaxy => "galaxy",
            EffectId::Buddha => "buddha",
            EffectId::TomJerry => "tomjerry",
            EffectId::Mario => "mario",
            EffectId::Ecg => "ecg",
            EffectId::Dandelion => "dandelion",
            EffectId::FishTank => "fish",
            EffectId::Landscape => "landscape",
            EffectId::Shanghai => "shanghai",
            EffectId::Tiananmen => "tiananmen",
            EffectId::Traffic => "traffic",
            EffectId::Tornado => "tornado",
            EffectId::Storm => "storm",
            EffectId::MonaLisa => "monalisa",
            EffectId::Ballet => "ballet",
            EffectId::DemonEye => "demoneye",
            EffectId::Life => "life",
            EffectId::Dsh => "dsh",
            EffectId::Vibe => "vibe",
            EffectId::Lights => "lights",
            EffectId::Volcano => "volcano",
            EffectId::Launch => "launch",
            EffectId::Plasma => "plasma",
        }
    }

    pub fn title_en(self) -> &'static str {
        match self {
            EffectId::Rain => "Code Rain",
            EffectId::Fire => "Blazing Fire",
            EffectId::Dragon => "Coiling Dragon",
            EffectId::Map => "World Map",
            EffectId::Matrix => "The Matrix",
            EffectId::BlackHole => "Black Hole",
            EffectId::Beams => "Light Beams",
            EffectId::Fireworks => "Fireworks",
            EffectId::Solar => "Solar System",
            EffectId::Galaxy => "Starry Galaxy",
            EffectId::Buddha => "Buddha's Halo",
            EffectId::TomJerry => "Tom & Jerry",
            EffectId::Mario => "Super Mario",
            EffectId::Ecg => "Heartbeat Monitor",
            EffectId::Dandelion => "Dandelion",
            EffectId::FishTank => "Aquarium",
            EffectId::Landscape => "Colorful Landscape",
            EffectId::Shanghai => "Oriental Pearl",
            EffectId::Tiananmen => "Tiananmen Radiance",
            EffectId::Traffic => "Traffic Flow",
            EffectId::Tornado => "Tornado",
            EffectId::Storm => "Thunderstorm",
            EffectId::MonaLisa => "Mona Lisa",
            EffectId::Ballet => "Ice Ballet",
            EffectId::DemonEye => "Demon Eye",
            EffectId::Life => "Threads of Life",
            EffectId::Dsh => "DeepSeek Harness",
            EffectId::Vibe => "Vibe Coding",
            EffectId::Lights => "Aurora",
            EffectId::Volcano => "Volcano",
            EffectId::Launch => "Rocket Launch",
            EffectId::Plasma => "Plasma",
        }
    }

    pub fn title_zh(self) -> &'static str {
        match self {
            EffectId::Rain => "代码雨",
            EffectId::Fire => "燃烧火焰",
            EffectId::Dragon => "巨龙盘旋",
            EffectId::Map => "世界地图",
            EffectId::Matrix => "黑客帝国",
            EffectId::BlackHole => "黑洞",
            EffectId::Beams => "光束",
            EffectId::Fireworks => "烟花",
            EffectId::Solar => "太阳系",
            EffectId::Galaxy => "满天星河",
            EffectId::Buddha => "佛光普照",
            EffectId::TomJerry => "猫和老鼠",
            EffectId::Mario => "超级马里奥",
            EffectId::Ecg => "心率跳动",
            EffectId::Dandelion => "蒲公英",
            EffectId::FishTank => "水族馆",
            EffectId::Landscape => "五光十色",
            EffectId::Shanghai => "东方明珠",
            EffectId::Tiananmen => "光芒万丈",
            EffectId::Traffic => "车水马龙",
            EffectId::Tornado => "龙卷风",
            EffectId::Storm => "电闪雷鸣",
            EffectId::MonaLisa => "蒙娜丽莎",
            EffectId::Ballet => "冰上芭蕾",
            EffectId::DemonEye => "恶魔之眼",
            EffectId::Life => "生命的轨迹",
            EffectId::Dsh => "深度求索",
            EffectId::Vibe => "氛围编程",
            EffectId::Lights => "极光",
            EffectId::Volcano => "火山爆发",
            EffectId::Launch => "火箭发射",
            EffectId::Plasma => "等离子",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace(['_', '-', ' '], "").as_str() {
            "rain" | "code" | "coderain" | "codefall" => Some(EffectId::Rain),
            "fire" | "flame" | "firestorm" => Some(EffectId::Fire),
            "dragon" | "long" | "coil" | "orbit" => Some(EffectId::Dragon),
            "map" | "world" | "earth" | "globe" | "worldmap" => Some(EffectId::Map),
            "matrix" | "neo" | "digitalrain" | "hack" | "nasa" | "intrusion" | "login" => {
                Some(EffectId::Matrix)
            }
            "blackhole" | "bh" | "sagittarius" | "singularity" | "eventhorizon" => {
                Some(EffectId::BlackHole)
            }
            "beams" | "beam" | "lightbeams" | "lasers" | "laser" => Some(EffectId::Beams),
            "fireworks" | "firework" | "pyro" | "firecracker" => Some(EffectId::Fireworks),
            "solar" | "solarsystem" | "planets" | "orrery" => Some(EffectId::Solar),
            "galaxy" | "milkyway" | "stars" | "starfield" | "milky" => Some(EffectId::Galaxy),
            "buddha" | "fo" | "lotus" | "halo" | "amitabha" => Some(EffectId::Buddha),
            "tomjerry" | "tomandjerry" | "tom" | "jerry" | "chase" => Some(EffectId::TomJerry),
            "mario" | "supermario" | "plumber" | "mushroom" | "mushroomrun" => {
                Some(EffectId::Mario)
            }
            "ecg" | "ekg" | "heart" | "heartbeat" | "heartrate" | "pulse" => Some(EffectId::Ecg),
            "dandelion" | "dandy" | "seeds" | "wind" | "dandelions" => Some(EffectId::Dandelion),
            "fish" | "fishtank" | "aquarium" | "tank" => Some(EffectId::FishTank),
            "landscape" | "shanshui" | "mountain" | "lake" | "painting" => {
                Some(EffectId::Landscape)
            }
            "shanghai" | "bund" | "orientalpearl" | "pearl" => Some(EffectId::Shanghai),
            "tiananmen" | "gate" | "radiance" | "light" => Some(EffectId::Tiananmen),
            "traffic" | "street" | "city" | "cars" => Some(EffectId::Traffic),
            "tornado" | "twister" | "cyclone" => Some(EffectId::Tornado),
            "storm" | "thunder" | "lightning" | "rainstorm" => Some(EffectId::Storm),
            "monalisa" | "mona" | "lisa" | "gioconda" | "davinci" | "leonardo" | "louvre" => {
                Some(EffectId::MonaLisa)
            }
            "ballet" | "dance" | "dancers" | "duet" | "pasdedeux" | "ballerina" | "waltz"
            | "ice" | "iceballet" | "iceskate" | "skating" | "rink" => Some(EffectId::Ballet),
            "demoneye" | "eye" | "demon" | "evileye" | "watcher" | "oculus" | "gaze" => {
                Some(EffectId::DemonEye)
            }
            "life" | "lifetrack" | "lifeline" | "lifecycle" | "journey" | "trajectory"
            | "legacy" => Some(EffectId::Life),
            "dsh" | "deepseek" | "deepseekharness" | "harness" | "ds" | "banner" => {
                Some(EffectId::Dsh)
            }
            "vibe" | "vibecoding" | "claude" | "claudecode" | "coding" | "agent" => {
                Some(EffectId::Vibe)
            }
            "lights" | "aurora" | "borealis" | "northernlights" => Some(EffectId::Lights),
            "volcano" | "eruption" | "lava" => Some(EffectId::Volcano),
            "launch" | "rocket" | "liftoff" | "spacex" => Some(EffectId::Launch),
            "plasma" | "demoscene" | "sines" => Some(EffectId::Plasma),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&e| e == self).unwrap_or(0)
    }

    pub fn from_index(i: usize) -> Self {
        Self::ALL[i % Self::ALL.len()]
    }

    pub fn next(self) -> Self {
        Self::from_index(self.index() + 1)
    }

    pub fn prev(self) -> Self {
        Self::from_index(self.index() + Self::ALL.len() - 1)
    }

    pub fn default_theme(self) -> &'static Theme {
        theme::find(match self {
            EffectId::Rain => "neon",
            EffectId::Fire => "ember",
            EffectId::Dragon => "aurora",
            EffectId::Map => "ice",
            EffectId::Matrix => "matrix",
            EffectId::BlackHole => "plasma",
            EffectId::Beams => "neon",
            EffectId::Fireworks => "plasma",
            EffectId::Solar => "ice",
            EffectId::Galaxy => "ice",
            EffectId::Buddha => "gold",
            EffectId::TomJerry => "neon",
            EffectId::Mario => "gold",
            EffectId::Ecg => "matrix",
            EffectId::Dandelion => "gold",
            EffectId::FishTank => "aurora",
            EffectId::Landscape => "aurora",
            EffectId::Shanghai => "neon",
            EffectId::Tiananmen => "gold",
            EffectId::Traffic => "neon",
            EffectId::Tornado => "ice",
            EffectId::Storm => "ice",
            EffectId::MonaLisa => "gold",
            EffectId::Ballet => "aurora",
            EffectId::DemonEye => "ember",
            EffectId::Life => "aurora",
            EffectId::Dsh => "ice",
            EffectId::Vibe => "gold",
            EffectId::Lights => "aurora",
            EffectId::Volcano => "ember",
            EffectId::Launch => "ember",
            EffectId::Plasma => "plasma",
        })
        .unwrap_or(&theme::THEMES[0])
    }

    /// Effects that support both glyph sets start in ASCII.  Unicode is an
    /// explicit opt-in (`--unicode` / `--charset unicode`) so the default
    /// works in every terminal and every font.
    pub fn default_charset(self) -> Charset {
        Charset::Ascii
    }

    /// How much shared HUD this effect wants.
    ///
    /// `Ambient` keeps a deliberately text-free artwork (no header, no scene
    /// labels) but still gets the persistent key bar and playlist row.
    pub fn hud_mode(self) -> HudMode {
        match self {
            EffectId::Rain | EffectId::Fire | EffectId::MonaLisa => HudMode::Ambient,
            _ => HudMode::Full,
        }
    }
}

/// Per-run tuning knobs exposed on the command line.
#[derive(Clone, Copy, Debug)]
pub struct Tune {
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
    pub trail: f32,
    pub ghost: bool,
    pub plain: bool,
    pub gallery: bool,
    /// launch: rocket size multiplier.
    pub rocket_scale: f32,
    /// launch: camera shake strength (0 turns it off).
    pub shake: f32,
}

impl Default for Tune {
    fn default() -> Self {
        Tune {
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
            rocket_scale: 1.0,
            shake: 1.0,
        }
    }
}

pub struct Frame<'a> {
    pub t: f64,
    pub dt: f32,
    pub w: usize,
    pub h: usize,
    pub fps: u32,
    pub theme: &'a Theme,
    pub charset: Charset,
    pub seed: u64,
    pub speed: f32,
    pub tune: Tune,
}

pub trait Scene {
    fn resize(&mut self, w: usize, h: usize);
    fn update(&mut self, frame: &Frame);
    fn draw(&mut self, canvas: &mut Canvas, frame: &Frame);

    /// Optional live telemetry for the shared header.  Keep it short and
    /// ASCII; the header already adds speed, theme and charset.
    fn hud_status(&self, _frame: &Frame) -> Option<String> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudMode {
    Full,
    Ambient,
}

pub fn build(id: EffectId, seed: u64) -> Box<dyn Scene> {
    match id {
        EffectId::Rain => Box::new(RainScene::new(seed)),
        EffectId::Fire => Box::new(FireScene::new(seed)),
        EffectId::Dragon => Box::new(DragonScene::new(seed)),
        EffectId::Map => Box::new(MapScene::new(seed)),
        EffectId::Matrix => Box::new(MatrixScene::new(seed)),
        EffectId::BlackHole => Box::new(BlackHoleScene::new(seed)),
        EffectId::Beams => Box::new(BeamsScene::new(seed)),
        EffectId::Fireworks => Box::new(FireworksScene::new(seed)),
        EffectId::Solar => Box::new(SolarScene::new(seed)),
        EffectId::Galaxy => Box::new(GalaxyScene::new(seed)),
        EffectId::Buddha => Box::new(BuddhaScene::new(seed)),
        EffectId::TomJerry => Box::new(TomJerryScene::new(seed)),
        EffectId::Mario => Box::new(MarioScene::new(seed)),
        EffectId::Ecg => Box::new(EcgScene::new(seed)),
        EffectId::Dandelion => Box::new(DandelionScene::new(seed)),
        EffectId::FishTank => Box::new(FishScene::new(seed)),
        EffectId::Landscape => Box::new(LandscapeScene::new(seed)),
        EffectId::Shanghai => Box::new(ShanghaiScene::new(seed)),
        EffectId::Tiananmen => Box::new(TiananmenScene::new(seed)),
        EffectId::Traffic => Box::new(TrafficScene::new(seed)),
        EffectId::Tornado => Box::new(TornadoScene::new(seed)),
        EffectId::Storm => Box::new(StormScene::new(seed)),
        EffectId::MonaLisa => Box::new(MonaLisaScene::new(seed)),
        EffectId::Ballet => Box::new(BalletScene::new(seed)),
        EffectId::DemonEye => Box::new(DemonEyeScene::new(seed)),
        EffectId::Life => Box::new(LifeScene::new(seed)),
        EffectId::Dsh => Box::new(DshScene::new(seed)),
        EffectId::Vibe => Box::new(VibeScene::new(seed)),
        EffectId::Lights => Box::new(LightsScene::new(seed)),
        EffectId::Volcano => Box::new(VolcanoScene::new(seed)),
        EffectId::Launch => Box::new(LaunchScene::new(seed)),
        EffectId::Plasma => Box::new(PlasmaScene::new(seed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::ColorMode;

    #[test]
    fn aliases_resolve() {
        assert_eq!(EffectId::parse("code-rain"), Some(EffectId::Rain));
        assert_eq!(EffectId::parse("MAtrix"), Some(EffectId::Matrix));
        assert_eq!(EffectId::parse("world"), Some(EffectId::Map));
        assert_eq!(EffectId::parse("supermario"), Some(EffectId::Mario));
        assert_eq!(EffectId::parse("heartbeat"), Some(EffectId::Ecg));
        assert_eq!(EffectId::parse("dandy"), Some(EffectId::Dandelion));
        assert_eq!(EffectId::parse("mona-lisa"), Some(EffectId::MonaLisa));
        assert_eq!(EffectId::parse("pas de deux"), Some(EffectId::Ballet));
        assert_eq!(EffectId::parse("evil_eye"), Some(EffectId::DemonEye));
        assert_eq!(EffectId::parse("life-cycle"), Some(EffectId::Life));
        assert_eq!(EffectId::parse("deepseek-harness"), Some(EffectId::Dsh));
        assert_eq!(EffectId::parse("dsh"), Some(EffectId::Dsh));
        assert_eq!(EffectId::parse("vibe coding"), Some(EffectId::Vibe));
        assert_eq!(EffectId::parse("nope"), None);
    }

    #[test]
    fn every_effect_defaults_to_ascii() {
        for id in EffectId::ALL {
            assert_eq!(id.default_charset(), Charset::Ascii, "{:?}", id);
        }
    }

    #[test]
    fn only_the_text_free_effects_are_ambient() {
        let ambient: Vec<_> = EffectId::ALL
            .iter()
            .filter(|id| id.hud_mode() == HudMode::Ambient)
            .map(|id| id.key())
            .collect();
        assert_eq!(ambient, ["rain", "fire", "monalisa"]);
    }

    #[test]
    fn every_scene_draws_something() {
        for id in EffectId::ALL {
            let mut scene = build(id, 7);
            let (w, h) = (64, 24);
            scene.resize(w, h);
            let theme = id.default_theme();
            let mut canvas = Canvas::new(w, h);
            for step in 0..30 {
                let frame = Frame {
                    t: step as f64 / 30.0,
                    dt: 1.0 / 30.0,
                    w,
                    h,
                    fps: 30,
                    theme,
                    charset: id.default_charset(),
                    seed: 7,
                    speed: 1.0,
                    tune: Tune::default(),
                };
                scene.update(&frame);
                scene.draw(&mut canvas, &frame);
            }
            assert!(canvas.ink() > 20, "{:?} drew {} cells", id, canvas.ink());
            for cell in &canvas.cells {
                assert!(
                    cell.ch.is_ascii(),
                    "{:?} leaked non-ASCII glyph {:?} in its default charset",
                    id,
                    cell.ch
                );
            }
            let ansi = canvas.render_ansi(ColorMode::True);
            assert!(ansi.len() > 100);
        }
    }

    #[test]
    fn scenes_keep_text_out_of_reserved_hud_rows() {
        for id in EffectId::ALL {
            for &(w, h) in &[(40, 12), (80, 24), (120, 36)] {
                let mut scene = build(id, 7);
                scene.resize(w, h);
                let frame = Frame {
                    t: 5.0,
                    dt: 1.0 / 30.0,
                    w,
                    h,
                    fps: 30,
                    theme: id.default_theme(),
                    charset: id.default_charset(),
                    seed: 7,
                    speed: 1.0,
                    tune: Tune::default(),
                };
                let mut canvas = Canvas::new(w, h);
                canvas.set_chrome_guard(true);
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    scene.draw(&mut canvas, &frame);
                }));
                canvas.set_chrome_guard(false);
                assert!(
                    result.is_ok(),
                    "{id:?} wrote scene text into reserved HUD row 0 or h-1 at {w}x{h}"
                );
            }
        }
    }

    #[test]
    fn hud_status_is_short_and_ascii() {
        for id in EffectId::ALL {
            let mut scene = build(id, 7);
            let (w, h) = (120, 36);
            scene.resize(w, h);
            let frame = Frame {
                t: 5.0,
                dt: 1.0 / 30.0,
                w,
                h,
                fps: 30,
                theme: id.default_theme(),
                charset: id.default_charset(),
                seed: 7,
                speed: 1.0,
                tune: Tune::default(),
            };
            if let Some(status) = scene.hud_status(&frame) {
                assert!(status.is_ascii(), "{id:?} status is not ASCII: {status:?}");
                assert!(
                    status.chars().count() <= 60,
                    "{id:?} status is too long: {status:?}"
                );
            }
        }
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        for id in EffectId::ALL {
            let mut scene = build(id, 3);
            let (w, h) = (20, 8);
            scene.resize(w, h);
            let theme = id.default_theme();
            let mut canvas = Canvas::new(w, h);
            let frame = Frame {
                t: 2.5,
                dt: 1.0 / 30.0,
                w,
                h,
                fps: 30,
                theme,
                charset: id.default_charset(),
                seed: 3,
                speed: 1.0,
                tune: Tune::default(),
            };
            scene.update(&frame);
            scene.draw(&mut canvas, &frame);
        }
    }
}
