use bevy::prelude::*;

pub mod player;

pub mod prelude {
    pub use super::SpaceshipInputPlugin;

    pub use super::player::prelude::*;
}

pub struct SpaceshipInputPlugin;

impl Plugin for SpaceshipInputPlugin {
    fn build(&self, app: &mut App) {
        debug!("SpaceshipInputPlugin: build");

        app.add_plugins(player::SpaceshipPlayerInputPlugin);
    }
}
