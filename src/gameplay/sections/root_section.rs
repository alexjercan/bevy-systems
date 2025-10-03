//! This module will contain the root section of a spaceship. This is the section that all other
//! sections are attached to and it will spawn the spaceship in the world. It is responsible for
//! spaceship rotation using PD Torque Control.
//!
//! This component will be the rigidbody of the spaceship. All other sections will be children of
//! this section. This section will have a stable torque PD controller to maintain spaceship
//! orientation.

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::physics::prelude::*;

pub mod prelude {
    pub use super::root_section;
    pub use super::RootSectionConfig;
    pub use super::RootSectionMarker;
    pub use super::RootSectionPlugin;
}

#[derive(Default, Clone, Debug)]
pub struct RootSectionConfig {}

pub fn root_section(_config: RootSectionConfig) -> impl Bundle {
    (
        Name::new("Root Section"),
        RootSectionMarker,
        RigidBody::Dynamic,
        StableTorquePdController {
            frequency: 4.0,
            damping_ratio: 4.0,
            max_torque: 100.0,
        },
        Transform::default(),
        Visibility::Visible,
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct RootSectionMarker;

pub struct RootSectionPlugin;

impl Plugin for RootSectionPlugin {
    fn build(&self, app: &mut App) {
        // NOTE: How can we check that the TorquePdControllerPlugin is added?
        app.register_type::<RootSectionMarker>();
    }
}
