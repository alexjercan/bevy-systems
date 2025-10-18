use bevy::prelude::*;

pub mod prelude {
    pub use super::TempEntity;
    pub use super::TempEntityPlugin;
}

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct TempEntity(pub f32);

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct TempEntityState(Timer);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TempEntityPluginSet;

pub struct TempEntityPlugin;

impl Plugin for TempEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_insert_temp_entity);
        app.add_systems(Update, update_temp_entities.in_set(TempEntityPluginSet));
    }
}

fn on_insert_temp_entity(
    insert: On<Insert, TempEntity>,
    mut commands: Commands,
    q_temp: Query<&TempEntity>,
) {
    let entity = insert.entity;
    debug!("Inserting TempEntity: {:?}", entity);
    let Ok(temp_entity) = q_temp.get(entity) else {
        warn!(
            "TempEntity entity {:?} missing TempEntity component",
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
            debug!("Despawning TempEntity: {:?}", entity);
        }
    }
}
