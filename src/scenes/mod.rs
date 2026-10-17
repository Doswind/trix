//! The five effects.  Each scene owns its state and draws into a Canvas.

pub mod dragon;
pub mod fire;
pub mod map;
pub mod matrix;
pub mod rain;

pub use dragon::DragonScene;
pub use fire::FireScene;
pub use map::MapScene;
pub use matrix::MatrixScene;
pub use rain::RainScene;
