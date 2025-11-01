//! A Bevy plugin that handles damage.

pub mod prelude {
    pub use super::DamagePlugin;
}

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

use crate::events::{OnDestroyedEvent, OnDestroyedEventInfo};

const DAMAGE_MODIFIER: f32 = 1.00;

/// A plugin that handles damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        debug!("DamagePlugin: build");

        app.add_observer(on_rigidbody_spawn);
        app.add_observer(on_collision_hit_to_damage);
        app.add_observer(on_destroyed_entity);
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

fn on_collision_hit_to_damage(
    hit: On<CollisionDamageEvent>,
    mut commands: Commands,
    q_mass: Query<&ComputedMass>,
) {
    let amount = hit.relative_velocity.length() * DAMAGE_MODIFIER;
    let mass = q_mass.get(hit.other).map(|m| m.value()).unwrap_or(1.0);
    let amount = amount * mass;

    commands.trigger(HealthApplyDamage {
        target: hit.entity,
        source: Some(hit.other),
        amount,
    });
}

fn on_destroyed_entity(add: On<Add, DestroyedMarker>, mut commands: Commands) {
    let entity = add.entity;
    trace!("on_destroyed_entity: entity {:?}", entity);

    commands.fire::<OnDestroyedEvent>(OnDestroyedEventInfo { entity });
}
