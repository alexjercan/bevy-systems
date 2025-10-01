mod setup_systems;
mod systems;
mod plugins;

pub mod prelude {
    pub use super::systems::*;
    pub use super::plugins::*;
}
