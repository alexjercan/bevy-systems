//! TODO: Add description in this crate

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod sections;
pub mod projectiles;
pub mod attachments;

pub mod prelude {
    pub use super::sections::prelude::*;
    pub use super::projectiles::prelude::*;
    pub use super::attachments::prelude::*;

    pub use super::spaceship_root;
    pub use super::SpaceshipConfig;
    pub use super::SpaceshipRootMarker;
    pub use super::SpaceshipPlugin;
    pub use super::SpaceshipPluginSet;
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
        app.add_plugins(sections::SectionPlugin { render: self.render });
        app.add_plugins(projectiles::ProjectilePlugin { render: self.render });
        app.add_plugins(attachments::AttachmentPlugin);

        app.configure_sets(
            Update,
            sections::SectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            sections::SectionPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            projectiles::ProjectilePluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            projectiles::ProjectilePluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            Update,
            attachments::AttachmentPluginSet.in_set(SpaceshipPluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            attachments::AttachmentPluginSet.in_set(SpaceshipPluginSet),
        );
    }
}
