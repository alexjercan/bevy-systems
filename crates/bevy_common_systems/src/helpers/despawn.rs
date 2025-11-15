//! A Bevy plugin to immediately despawn entities when a marker component is added.
//!
//! ## Overview
//!
//! This plugin provides a simple mechanism to despawn entities as soon as they
//! are marked with the `DespawnEntity` component. You can attach this component
//! to any entity and it will be removed from the world automatically in the same frame.
//!
//! This can be useful for one-time effects, temporary entities, or cleanup
//! without needing a separate system each time.
//!
//! ## Usage
//!
//! ```rust
//! commands.spawn((
//!     DespawnEntity, // entity will be despawned immediately
//! ));
//! ```

use bevy::prelude::*;

pub mod prelude {
    pub use super::{DespawnEntity, DespawnEntityPlugin};
}

/// Marker component that indicates an entity should be despawned immediately.
///
/// Adding this component to an entity triggers the plugin to remove it
/// from the world in the same frame.
#[derive(Component, Clone, Debug, Reflect)]
pub struct DespawnEntity;

/// Plugin that handles immediate despawning of entities marked with `DespawnEntity`.
pub struct DespawnEntityPlugin;

impl Plugin for DespawnEntityPlugin {
    fn build(&self, app: &mut App) {
        debug!("DespawnEntityPlugin: build");

        app.add_observer(on_insert_despawn_entity);
    }
}

/// Observer system that runs when a `DespawnEntity` component is inserted.
///
/// This system immediately despawns the entity.
fn on_insert_despawn_entity(insert: On<Insert, DespawnEntity>, mut commands: Commands) {
    let entity = insert.entity;
    trace!("on_insert_despawn_entity: entity {:?}", entity);

    commands.entity(entity).despawn();
}
