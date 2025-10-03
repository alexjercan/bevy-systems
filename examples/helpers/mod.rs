#![allow(dead_code)]

mod setup_systems;
mod systems;
mod plugins;

pub mod prelude {
    pub use super::setup_systems::*;
    pub use super::plugins::*;
}
