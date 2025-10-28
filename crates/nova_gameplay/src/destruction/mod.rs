//! A Bevy plugin that handles entity destruction when health reaches zero.

pub mod despawn;
pub mod explode;

pub mod prelude {
    pub use super::despawn::prelude::*;
    pub use super::explode::prelude::*;

    pub use super::DestructionHealthPlugin;
}

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

/// A plugin that health reaching zero results in entity destruction.
pub struct DestructionHealthPlugin;

impl Plugin for DestructionHealthPlugin {
    fn build(&self, app: &mut App) {
        debug!("DestructionHealthPlugin: build");

        app.add_plugins(despawn::DespawnOnDestroyPlugin);
        app.add_plugins(explode::ExplodeOnDestroyPlugin);

        app.add_observer(on_health_spawn);
    }
}

fn on_health_spawn(
    add: On<Add, Health>,
    mut commands: Commands,
    q_despawn: Query<&despawn::DespawnOnDestroy, With<Health>>,
) {
    let entity = add.entity;
    trace!("on_health_spawn: entity {:?}", entity);

    let Ok(_) = q_despawn.get(entity) else {
        // NOTE: When a Health component is added, ensure the entity has a DespawnOnDestroy marker.
        debug!(
            "on_health_spawn: adding DespawnOnDestroy to entity {:?}",
            entity
        );
        commands
            .entity(entity)
            .insert(despawn::DespawnOnDestroy::default());
        return;
    };
}
