//! A Bevy plugin that handles entity destruction when health reaches zero.

pub mod despawn;
pub mod explode;

pub mod prelude {
    pub use super::despawn::prelude::*;
    pub use super::explode::prelude::*;

    pub use super::DestructionHealthPlugin;
    pub use super::DestructionHealthPluginSet;
}

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

/// A system set that will contain all the systems related to the destruction health plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DestructionHealthPluginSet;

/// A plugin that health reaching zero results in entity destruction.
pub struct DestructionHealthPlugin;

impl Plugin for DestructionHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(despawn::DespawnOnDestroyPlugin);
        app.add_plugins(explode::ExplodeOnDestroyPlugin);

        app.add_observer(on_health_spawn);

        app.configure_sets(
            Update,
            despawn::DespawnOnDestroyPluginSet.in_set(DestructionHealthPluginSet),
        );
        app.configure_sets(
            Update,
            explode::ExplodeOnDestroyPluginSet.in_set(DestructionHealthPluginSet),
        );
    }
}

/// When a Health component is added, ensure the entity has a DespawnOnDestroy marker.
fn on_health_spawn(
    add: On<Add, Health>,
    mut commands: Commands,
    q_despawn: Query<&despawn::DespawnOnDestroy, With<Health>>,
) {
    let entity = add.entity;
    debug!("Health component added to entity {:?}", entity);

    let Ok(_) = q_despawn.get(entity) else {
        debug!(
            "Entity {:?} does not have DespawnOnDestroy marker, spawning with default behavior.",
            entity
        );
        commands
            .entity(entity)
            .insert(despawn::DespawnOnDestroy::default());
        return;
    };
}
