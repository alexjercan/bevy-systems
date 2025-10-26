//! A Bevy plugin that applies damage to entities upon collision.
//! TODO: Rename this from collision_damage to collision_impact or something like that

use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::CollisionDamageEvent;
    pub use super::CollisionDamageMarker;
    pub use super::CollisionDamagePlugin;
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct CollisionDamageMarker;

/// Event to signal that an entity has taken collision damage.
#[derive(Event, Clone, Debug)]
pub struct CollisionDamageEvent {
    /// The entity that took damage.
    pub entity: Entity,
    /// The entity that was hit.
    pub other: Entity,
    // /// The point of impact in world space.
    // pub hit_point: Vec3,
    // /// The normal of the surface hit.
    // pub hit_normal: Vec3,
    /// The relative velocity at the point of impact.
    pub relative_velocity: Vec3,
}

pub struct CollisionDamagePlugin;

impl Plugin for CollisionDamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(insert_collision_events);
        app.add_observer(on_collision_event);
    }
}

fn insert_collision_events(add: On<Add, CollisionDamageMarker>, mut commands: Commands) {
    let entity = add.entity;
    debug!(
        "Inserting collision events for CollisionDamageMarker: {:?}",
        entity
    );

    commands.entity(entity).insert(CollisionEventsEnabled);
}

fn on_collision_event(
    collision: On<CollisionStart>,
    mut commands: Commands,
    q_velocity: Query<&LinearVelocity, With<RigidBody>>,
) {
    let Some(body) = collision.body1 else {
        debug!("No body1 found for collision event");
        return;
    };

    let Some(other) = collision.body2 else {
        debug!("No body2 found for collision event");
        return;
    };

    let velocity1 = q_velocity.get(body).map(|v| v.0).unwrap_or_default();
    let velocity2 = q_velocity.get(other).map(|v| v.0).unwrap_or_default();

    let relative_velocity = velocity1 - velocity2;

    commands.trigger(CollisionDamageEvent {
        entity: body,
        other,
        // hit_point: collision.contact_point,
        // hit_normal: collision.contact_normal,
        relative_velocity,
    });
}
