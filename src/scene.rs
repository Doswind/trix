//! Effect registry and the shared scene interface.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::scenes::{
    BeamsScene, BlackHoleScene, BuddhaScene, DandelionScene, DragonScene, EcgScene, FireScene,
    FireworksScene, GalaxyScene, MapScene, MarioScene, MatrixScene, RainScene, SolarScene,
    TomJerryScene,
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
}

impl EffectId {
    pub const ALL: [EffectId; 15] = [
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
        }
    }

    pub fn title_en(self) -> &'static str {
        match self {
            EffectId::Rain => "Code rain",
            EffectId::Fire => "Burning flame",
            EffectId::Dragon => "Celestial dragon",
            EffectId::Map => "World map",
            EffectId::Matrix => "NASA login",
            EffectId::BlackHole => "Black hole",
            EffectId::Beams => "Light beams",
            EffectId::Fireworks => "Fireworks",
            EffectId::Solar => "Solar system",
            EffectId::Galaxy => "Milky Way",
            EffectId::Buddha => "Radiant Buddha",
            EffectId::TomJerry => "Tom & Jerry",
            EffectId::Mario => "Super Mario",
            EffectId::Ecg => "Heart monitor",
            EffectId::Dandelion => "Dandelion field",
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
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
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
        })
        .unwrap_or(&theme::THEMES[0])
    }

    pub fn default_charset(self) -> Charset {
        match self {
            EffectId::Matrix | EffectId::Mario | EffectId::Ecg | EffectId::Dandelion => {
                Charset::Unicode
            }
            _ => Charset::Ascii,
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
    pub seed_count: u32,
    pub seed_speed: f32,
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
            seed_count: 90,
            seed_speed: 1.0,
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
        assert_eq!(EffectId::parse("nope"), None);
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
            let ansi = canvas.render_ansi(ColorMode::True);
            assert!(ansi.len() > 100);
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
