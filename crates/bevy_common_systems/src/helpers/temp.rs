//! A Bevy plugin to handle temporary entities that despawn after a set duration.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{TempEntity, TempEntityPlugin, TempEntitySystems};
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct TempEntity(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TempEntityState(Timer);

/// System sets for TempEntityPlugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TempEntitySystems {
    Sync,
}

/// Plugin to handle temporary entities that despawn after a set duration.
pub struct TempEntityPlugin;

impl Plugin for TempEntityPlugin {
    fn build(&self, app: &mut App) {
        debug!("TempEntityPlugin: build");

        app.add_observer(on_insert_temp_entity);

        // NOTE: Using Update stage to ensure timers are updated correctly.
        // TODO: Check what happens if this system is called before other systems that use the
        // TempEntity. Will the .depsawn() call despawn the entity imediatelly or at the end of the
        // frame?
        app.add_systems(
            Update,
            (update_temp_entities,)
                .chain()
                .in_set(TempEntitySystems::Sync),
        );
    }
}

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
