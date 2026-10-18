//! trix-effects — 26 animated ASCII/Unicode effects for the terminal.

pub mod app;
pub mod canvas;
pub mod cli;
pub mod color;
pub mod help;
pub mod hud;
pub mod meta;
pub mod rng;
pub mod scene;
pub mod scenes;
pub mod term;
pub mod theme;
pub mod transition;
pub mod world;

pub use app::run;
