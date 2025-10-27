use bevy::prelude::*;

pub mod player;

pub mod prelude {
    pub use super::SpaceshipInputPlugin;
    pub use super::SpaceshipInputPluginSet;

    pub use super::player::prelude::*;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceshipInputPluginSet;

pub struct SpaceshipInputPlugin;

impl Plugin for SpaceshipInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(player::SpaceshipPlayerInputPlugin);

        app.configure_sets(
            Update,
            player::SpaceshipPlayerInputPluginSet.in_set(SpaceshipInputPluginSet),
        );
    }
}
