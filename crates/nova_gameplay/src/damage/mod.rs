//! A Bevy plugin that handles damage.

pub mod collision;
pub mod projectile;

pub mod prelude {
    pub use super::collision::prelude::*;
    pub use super::projectile::prelude::*;

    pub use super::DamagePlugin;
    pub use super::DamagePluginSet;
}

use bevy::prelude::*;

/// A system set that will contain all the systems related to the damage plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamagePluginSet;

/// A plugin that handles damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(projectile::ProjectileDamageGluePlugin);
        app.add_plugins(collision::CollisionDamageGluePlugin);

        app.configure_sets(
            Update,
            projectile::ProjectileDamageGluePluginSet.in_set(DamagePluginSet),
        );
        app.configure_sets(
            Update,
            collision::CollisionDamageGluePluginSet.in_set(DamagePluginSet),
        );
    }
}
