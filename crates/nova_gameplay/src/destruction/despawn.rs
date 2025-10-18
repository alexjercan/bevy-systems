use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::DespawnOnDestroyPlugin;
    pub use super::DespawnOnDestroyPluginSet;
    pub use super::DespawnOnDestroy;
}

#[derive(Component, Default, Clone, Debug, Reflect)]
pub struct DespawnOnDestroy;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DespawnOnDestroyPluginSet;

pub struct DespawnOnDestroyPlugin;

impl Plugin for DespawnOnDestroyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_despawn_on_destroy);
    }
}

fn handle_despawn_on_destroy(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_despawn: Query<Entity, With<DespawnOnDestroy>>,
) {
    let entity = add.entity;
    debug!("Handling destruction for entity {:?}", entity);

    let Ok(_) = q_despawn.get(entity) else {
        warn!("Destroyed entity {:?} missing DespawnOnDestroy component, skipping despawn.", entity);
        return;
    };

    commands.entity(entity).despawn();
}
