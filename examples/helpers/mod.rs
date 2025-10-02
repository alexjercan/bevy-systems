#![allow(dead_code)]

mod setup_systems;
mod systems;
mod plugins;
mod render;

pub mod prelude {
    pub use super::setup_systems::*;
    pub use super::systems::*;
    pub use super::plugins::*;
    pub use super::render::*;
}
