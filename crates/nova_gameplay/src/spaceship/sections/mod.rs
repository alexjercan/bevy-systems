//! This module contains all the sections of a spaceship.

use bevy::prelude::*;

pub mod base_section;
pub mod controller_section;
pub mod hull_section;
pub mod thruster_section;
pub mod turret_section;

pub mod prelude {
    pub use super::{
        base_section::prelude::*, controller_section::prelude::*, hull_section::prelude::*,
        thruster_section::prelude::*, turret_section::prelude::*, SpaceshipSectionPlugin,
    };
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
