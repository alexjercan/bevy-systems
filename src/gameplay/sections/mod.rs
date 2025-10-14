//! This module contains all the sections of a spaceship.

use avian3d::prelude::*;
use bevy::prelude::*;

mod controller_section;
mod hull_section;
mod thruster_section;
mod turret_section;

pub mod prelude {
    pub use super::controller_section::prelude::*;
    pub use super::hull_section::prelude::*;
    pub use super::thruster_section::prelude::*;
    pub use super::turret_section::prelude::*;

    pub use super::spaceship_root;
    pub use super::SpaceshipConfig;
    pub use super::SpaceshipPlugin;
    pub use super::SpaceshipPluginSet;
    pub use super::SpaceshipRootMarker;
}

/// Configuration for the spaceship root entity.
#[derive(Default, Clone, Debug)]
pub struct SpaceshipConfig {
    /// The transform of the spaceship root entity.
    pub transform: Transform,
}

/// Helper function to create a spaceship root entity bundle.
pub fn spaceship_root(config: SpaceshipConfig) -> impl Bundle {
    (
        Name::new("Spaceship Root"),
        SpaceshipRootMarker,
        RigidBody::Dynamic,
        config.transform,
        Visibility::Visible,
    )
}

/// This will be the root component for the entire spaceship. All other sections will be children
/// of this entity.
#[derive(Component, Clone, Debug, Reflect)]
pub struct SpaceshipRootMarker;

/// A system set that will contain all the systems related to the spaceship plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceshipPluginSet;

/// A plugin that adds all the spaceship sections and their related systems.
#[derive(Default, Clone, Debug)]
pub struct SpaceshipPlugin {
    pub render: bool,
}

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpaceshipRootMarker>();

        app.add_plugins((
            hull_section::HullSectionPlugin {
                render: self.render,
                ..default()
            },
            thruster_section::ThrusterSectionPlugin {
                render: self.render,
                ..default()
            },
            turret_section::TurretSectionPlugin,
            controller_section::ControllerSectionPlugin,
        ));

        app.configure_sets(
            Update,
            thruster_section::ThrusterSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            thruster_section::ThrusterSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            controller_section::ControllerSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            controller_section::ControllerSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            turret_section::TurretSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            turret_section::TurretSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            hull_section::HullSectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            hull_section::HullSectionPluginSet.in_set(SpaceshipPluginSet),
        );
    }
}
