pub mod asteroid;
pub mod spaceship;

pub mod prelude {
    pub use super::{asteroid::prelude::*, spaceship::prelude::*, ScenarioObjectsPlugin};
}

use bevy::prelude::*;

pub struct ScenarioObjectsPlugin {
    pub render: bool,
}

impl Plugin for ScenarioObjectsPlugin {
    fn build(&self, app: &mut App) {
        debug!("ScenarioObjectsPlugin: build");

        app.add_plugins(asteroid::AsteroidPlugin {
            render: self.render,
        });
        app.add_plugins(spaceship::SpaceshipPlugin);
    }
}
