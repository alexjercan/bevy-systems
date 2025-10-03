//! This module contains all the sections of a spaceship.

use avian3d::prelude::*;
use bevy::prelude::*;

mod controller_section;
mod engine_section;
mod hull_section;
mod turret_section;

pub mod prelude {
    pub use super::controller_section::prelude::*;
    pub use super::engine_section::prelude::*;
    pub use super::hull_section::prelude::*;
    pub use super::turret_section::prelude::*;

    pub use super::spaceship_root;
    pub use super::SpaceshipConfig;
    pub use super::SpaceshipPlugin;
    pub use super::SpaceshipPluginSet;
    pub use super::SpaceshipRootMarker;
}

#[derive(Default, Clone, Debug)]
pub struct SpaceshipConfig {}

pub fn spaceship_root(_config: SpaceshipConfig) -> impl Bundle {
    (
        Name::new("Spaceship Root"),
        SpaceshipRootMarker,
        RigidBody::Dynamic,
        Transform::default(),
        Visibility::Visible,
    )
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct SpaceshipRootMarker;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceshipPluginSet;

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SpaceshipRootMarker>();

        app.add_plugins((
            hull_section::HullSectionPlugin,
            engine_section::EngineSectionPlugin,
            turret_section::TurretSectionPlugin,
            controller_section::ControllerSectionPlugin,
        ));

        app.configure_sets(
            Update,
            engine_section::EngineSectionPluginSet.after(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            controller_section::ControllerSectionPluginSet.after(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            turret_section::TurretSectionPluginSet.after(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            hull_section::HullSectionPluginSet.after(SpaceshipPluginSet),
        );
    }
}
