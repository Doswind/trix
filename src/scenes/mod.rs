//! The effect scenes.  Each scene owns its state and draws into a Canvas.

pub mod beams;
pub mod blackhole;
pub mod buddha;
pub mod dragon;
pub mod fire;
pub mod fireworks;
pub mod galaxy;
pub mod map;
pub mod matrix;
pub mod rain;
pub mod solar;
pub mod tomjerry;

pub use beams::BeamsScene;
pub use blackhole::BlackHoleScene;
pub use buddha::BuddhaScene;
pub use dragon::DragonScene;
pub use fire::FireScene;
pub use fireworks::FireworksScene;
pub use galaxy::GalaxyScene;
pub use map::MapScene;
pub use matrix::MatrixScene;
pub use rain::RainScene;
pub use solar::SolarScene;
pub use tomjerry::TomJerryScene;
