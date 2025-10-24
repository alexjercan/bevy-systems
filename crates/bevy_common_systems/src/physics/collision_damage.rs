//! A Bevy plugin that applies damage to entities upon collision.

use bevy::prelude::*;
use avian3d::prelude::*;

pub mod prelude {
    pub use super::CollisionDamageMarker;
    pub use super::CollisionDamageEvent;
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
    /// The point of impact in world space.
    pub hit_point: Vec3,
    /// The normal of the surface hit.
    pub hit_normal: Vec3,
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

fn insert_collision_events(
    add: On<Add, CollisionDamageMarker>,
    mut commands: Commands,
) {
    let entity = add.entity;
    debug!("Inserting collision events for CollisionDamageMarker: {:?}", entity);

    commands.entity(entity).insert(CollisionEventsEnabled);
}

fn on_collision_event(
    collision: On<CollisionStart>,
    mut commands: Commands,
) {
    println!("Collision detected between {:?} and {:?}", collision.collider1, collision.collider2);
    println!("Collision detected between {:?} and {:?}", collision.body1, collision.body2);
}
