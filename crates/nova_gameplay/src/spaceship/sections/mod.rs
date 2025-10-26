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

    pub use super::SectionPlugin;
    pub use super::SectionPluginSet;
}

/// A system set that will contain all the systems related to the spaceship plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionPluginSet;

/// A plugin that adds all the spaceship sections and their related systems.
#[derive(Default, Clone, Debug)]
pub struct SectionPlugin {
    pub render: bool,
}

impl Plugin for SectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            hull_section::HullSectionPlugin {
                render: self.render,
                ..default()
            },
            thruster_section::ThrusterSectionPlugin {
                render: self.render,
                ..default()
            },
            turret_section::TurretSectionPlugin {
                render: self.render,
                ..default()
            },
            controller_section::ControllerSectionPlugin {
                render: self.render,
                ..default()
            },
        ));

        app.configure_sets(
            Update,
            thruster_section::ThrusterSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            thruster_section::ThrusterSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            Update,
            controller_section::ControllerSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            controller_section::ControllerSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            Update,
            turret_section::TurretSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            turret_section::TurretSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            Update,
            hull_section::HullSectionPluginSet.in_set(SectionPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            hull_section::HullSectionPluginSet.in_set(SectionPluginSet),
        );
    }
}
