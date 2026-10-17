//! Effect registry and the shared scene interface.

use crate::canvas::Canvas;
use crate::cli::Charset;
use crate::scenes::{DragonScene, FireScene, MapScene, MatrixScene, RainScene};
use crate::theme::{self, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectId {
    Rain,
    Fire,
    Dragon,
    Map,
    Matrix,
}

impl EffectId {
    pub const ALL: [EffectId; 5] = [
        EffectId::Rain,
        EffectId::Fire,
        EffectId::Dragon,
        EffectId::Map,
        EffectId::Matrix,
    ];

    pub fn key(self) -> &'static str {
        match self {
            EffectId::Rain => "rain",
            EffectId::Fire => "fire",
            EffectId::Dragon => "dragon",
            EffectId::Map => "map",
            EffectId::Matrix => "matrix",
        }
    }

    pub fn title_en(self) -> &'static str {
        match self {
            EffectId::Rain => "Code rain",
            EffectId::Fire => "Burning flame",
            EffectId::Dragon => "Celestial dragon",
            EffectId::Map => "World map",
            EffectId::Matrix => "NASA intrusion",
        }
    }

    pub fn title_zh(self) -> &'static str {
        match self {
            EffectId::Rain => "代码雨",
            EffectId::Fire => "燃烧火焰",
            EffectId::Dragon => "巨龙盘旋",
            EffectId::Map => "世界地图",
            EffectId::Matrix => "黑客帝国",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
            "rain" | "code" | "coderain" | "codefall" => Some(EffectId::Rain),
            "fire" | "flame" | "firestorm" => Some(EffectId::Fire),
            "dragon" | "long" | "coil" | "orbit" => Some(EffectId::Dragon),
            "map" | "world" | "earth" | "globe" | "worldmap" => Some(EffectId::Map),
            "matrix" | "neo" | "digitalrain" | "hack" | "nasa" | "intrusion" => {
                Some(EffectId::Matrix)
            }
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
        })
        .unwrap_or(&theme::THEMES[0])
    }

    pub fn default_charset(self) -> Charset {
        Charset::Ascii
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
            };
            scene.update(&frame);
            scene.draw(&mut canvas, &frame);
        }
    }
}
