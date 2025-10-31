//! This module contains all the sections of a spaceship.

use bevy::prelude::*;

pub mod base_section;
pub mod controller_section;
pub mod hull_section;
pub mod thruster_section;
pub mod turret_section;

pub mod prelude {
    pub use super::base_section::prelude::*;
    pub use super::controller_section::prelude::*;
    pub use super::hull_section::prelude::*;
    pub use super::thruster_section::prelude::*;
    pub use super::turret_section::prelude::*;

    pub use super::SpaceshipSectionPlugin;
}

/// A plugin that adds all the spaceship sections and their related systems.
#[derive(Default, Clone, Debug)]
pub struct SpaceshipSectionPlugin {
    pub render: bool,
}

impl Plugin for SpaceshipSectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            hull_section::HullSectionPlugin {
                render: self.render,
            },
            thruster_section::ThrusterSectionPlugin {
                render: self.render,
            },
            turret_section::TurretSectionPlugin {
                render: self.render,
            },
            controller_section::ControllerSectionPlugin {
                render: self.render,
            },
        ));
    }
}
