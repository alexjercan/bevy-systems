//! A Bevy plugin to handle despawning entities immediately.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{DespawnEntity, DespawnEntityPlugin};
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct DespawnEntity;

pub struct DespawnEntityPlugin;

impl Plugin for DespawnEntityPlugin {
    fn build(&self, app: &mut App) {
        debug!("DespawnEntityPlugin: build");

        app.add_observer(on_insert_despawn_entity);
    }
}

fn on_insert_despawn_entity(insert: On<Insert, DespawnEntity>, mut commands: Commands) {
    let entity = insert.entity;
    trace!("on_insert_despawn_entity: entity {:?}", entity);

    commands.entity(entity).despawn();
}
