//! A Bevy plugin to manage temporary entities that despawn after a set duration.
//!
//! This plugin allows you to mark any entity as temporary by adding the
//! `TempEntity` component with a duration in seconds. The entity will
//! automatically despawn after the duration has elapsed.
//!
//! ## Components
//!
//! - `TempEntity` - The duration in seconds before the entity despawns.
//! - `TempEntityState` - Internal timer used to track elapsed time. Not
//!   intended to be modified manually.
//!
//! ## Usage
//!
//! ```rust
//! commands.spawn((
//!     TempEntity(5.0), // despawns after 5 seconds
//! ));
//! ```
//!
//! The plugin handles initialization and updating of timers automatically.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{TempEntity, TempEntityPlugin, TempEntitySystems};
}

/// Component indicating that the entity is temporary.
///
/// The inner value is the lifetime of the entity in seconds.
/// When the timer runs out, the entity will be automatically despawned.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct TempEntity(pub f32);

/// Internal state for temporary entities.
///
/// This component stores the timer used to track when the entity
/// should be despawned. It is automatically inserted and updated
/// by the plugin.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TempEntityState(Timer);

/// System set for the TempEntityPlugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TempEntitySystems {
    Sync,
}

/// Plugin that manages temporary entities.
///
/// Automatically inserts the timer state on entities with `TempEntity` and
/// updates timers each frame to despawn entities when the duration expires.
pub struct TempEntityPlugin;

impl Plugin for TempEntityPlugin {
    fn build(&self, app: &mut App) {
        debug!("TempEntityPlugin: build");

        app.add_observer(on_insert_temp_entity);

        // Update stage ensures timers advance correctly with the frame delta.
        app.add_systems(
            Update,
            (update_temp_entities,)
                .chain()
                .in_set(TempEntitySystems::Sync),
        );
    }
}

/// Initialize the internal timer when a TempEntity is added.
fn on_insert_temp_entity(
    insert: On<Insert, TempEntity>,
    mut commands: Commands,
    q_temp: Query<&TempEntity>,
) {
    let entity = insert.entity;
    trace!("on_insert_temp_entity: entity {:?}", entity);

    let Ok(temp_entity) = q_temp.get(entity) else {
        error!(
            "on_insert_temp_entity: entity {:?} not found in q_temp",
            entity
        );
        return;
    };

    commands
        .entity(entity)
        .insert(TempEntityState(Timer::from_seconds(
            **temp_entity,
            TimerMode::Once,
        )));
}

/// Update timers for temporary entities and despawn them when finished.
fn update_temp_entities(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TempEntityState)>,
) {
    for (entity, mut temp_state) in query.iter_mut() {
        temp_state.tick(time.delta());

        if temp_state.is_finished() {
            commands.entity(entity).despawn();
            trace!("update_temp_entities: despawn entity {:?}", entity);
        }
    }
}
