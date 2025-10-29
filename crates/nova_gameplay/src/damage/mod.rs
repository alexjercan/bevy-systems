//! A Bevy plugin that handles damage.

pub mod collision;

pub mod prelude {
    pub use super::collision::prelude::*;

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

        app.add_plugins(collision::CollisionDamageGluePlugin);

        app.add_observer(on_rigidbody_spawn);
    }
}

fn on_rigidbody_spawn(
    add: On<Add, ColliderOf>,
    mut commands: Commands,
    q_collider: Query<&ColliderOf>,
    q_health: Query<(), (With<Health>, With<RigidBody>)>,
) {
    let entity = add.entity;
    trace!("on_rigidbody_spawn: entity {:?}", entity);

    let Ok(collider) = q_collider.get(entity) else {
        warn!(
            "on_rigidbody_spawn: entity {:?} not found in q_collider",
            entity
        );
        return;
    };

    let Ok(_) = q_health.get(collider.body) else {
        // NOTE: RigidBody does not have Health component
        return;
    };

    // NOTE: Add collision damage for all rigid bodies with Health component
    debug!(
        "on_rigidbody_spawn: adding CollisionDamageMarker to entity {:?}",
        entity
    );
    commands.entity(entity).insert(CollisionDamageMarker);
}
