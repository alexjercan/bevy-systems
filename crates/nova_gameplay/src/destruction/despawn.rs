use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::DespawnOnDestroy;
    pub use super::DespawnOnDestroyPlugin;
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct DespawnOnDestroy;

pub struct DespawnOnDestroyPlugin;

impl Plugin for DespawnOnDestroyPlugin {
    fn build(&self, app: &mut App) {
        debug!("DespawnOnDestroyPlugin: build");

        app.add_observer(handle_despawn_on_destroy);
    }
}

fn handle_despawn_on_destroy(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_despawn: Query<Entity, With<DespawnOnDestroy>>,
) {
    let entity = add.entity;
    trace!("handle_despawn_on_destroy: entity {:?}", entity);

    let Ok(_) = q_despawn.get(entity) else {
        warn!(
            "handle_despawn_on_destroy: entity {:?} not found in q_despawn",
            entity
        );
        return;
    };

    commands.entity(entity).despawn();
}
