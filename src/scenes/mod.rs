//! The effect scenes.  Each scene owns its state and draws into a Canvas.

pub mod ballet;
pub mod beams;
pub mod blackhole;
pub mod buddha;
pub mod dandelion;
pub mod demoneye;
pub mod dragon;
pub mod ecg;
// `fairy` is intentionally left out of the build for now; its source stays in
// this directory so the effect can be finished and re-enabled later.
pub mod fire;
pub mod fireworks;
pub mod fish;
pub mod galaxy;
pub mod landscape;
pub mod life;
pub mod map;
pub mod mario;
mod mario_art;
pub mod matrix;
pub mod monalisa;
pub mod rain;
pub mod shanghai;
pub mod solar;
pub mod storm;
pub mod tiananmen;
pub mod tomjerry;
pub mod tornado;
pub mod traffic;

pub use ballet::BalletScene;
pub use beams::BeamsScene;
pub use blackhole::BlackHoleScene;
pub use buddha::BuddhaScene;
pub use dandelion::DandelionScene;
pub use demoneye::DemonEyeScene;
pub use dragon::DragonScene;
pub use ecg::EcgScene;
pub use fire::FireScene;
pub use fireworks::FireworksScene;
pub use fish::FishScene;
pub use galaxy::GalaxyScene;
pub use landscape::LandscapeScene;
pub use life::LifeScene;
pub use map::MapScene;
pub use mario::MarioScene;
pub use matrix::MatrixScene;
pub use monalisa::MonaLisaScene;
pub use rain::RainScene;
pub use shanghai::ShanghaiScene;
pub use solar::SolarScene;
pub use storm::StormScene;
pub use tiananmen::TiananmenScene;
pub use tomjerry::TomJerryScene;
pub use tornado::TornadoScene;
pub use traffic::TrafficScene;
