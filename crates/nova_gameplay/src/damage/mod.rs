//! A Bevy plugin that handles damage.

pub mod collision;
pub mod projectile;

pub mod prelude {
    pub use super::collision::prelude::*;
    pub use super::projectile::prelude::*;

    pub use super::DamagePlugin;
}

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

/// A plugin that handles damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        debug!("DamagePlugin: build");

        app.add_plugins(projectile::ProjectileDamageGluePlugin);
        app.add_plugins(collision::CollisionDamageGluePlugin);

        app.add_observer(on_rigidbody_spawn);
    }
}

fn on_rigidbody_spawn(add: On<Add, ColliderOf>, mut commands: Commands) {
    let entity = add.entity;
    trace!("on_rigidbody_spawn: entity {:?}", entity);

    // NOTE: Add collision damage for all rigid bodies.
    debug!(
        "on_rigidbody_spawn: adding CollisionDamageMarker to entity {:?}",
        entity
    );
    commands.entity(entity).insert(CollisionDamageMarker);
}
