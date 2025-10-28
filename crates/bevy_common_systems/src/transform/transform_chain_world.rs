use bevy::prelude::*;

pub mod prelude {
    pub use super::TransformChainWorld;
    pub use super::TransformChainWorldMarker;
    pub use super::TransformChainWorldPlugin;
    pub use super::TransformChainWorldSystems;
}

/// Marker component for entities that should have their world transforms computed via going up the
/// hierarchy chain.
#[derive(Component, Clone, Copy, Debug)]
pub struct TransformChainWorldMarker;

#[derive(Component, Clone, Copy, Debug)]
pub struct TransformChainWorld {
    pub scale: Vec3,
    pub rotation: Quat,
    pub translation: Vec3,
    pub matrix: Mat4,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransformChainWorldSystems {
    Sync,
}

/// Plugin to compute world transforms for entities with TransformChainWorldMarker.
pub struct TransformChainWorldPlugin;

impl Plugin for TransformChainWorldPlugin {
    fn build(&self, app: &mut App) {
        debug!("TransformChainWorldPlugin: build");

        app.add_observer(initialize_cache_spawner_world);

        app.add_systems(
            Update,
            cache_spawner_world.in_set(TransformChainWorldSystems::Sync),
        );
    }
}

fn initialize_cache_spawner_world(
    insert: On<Insert, TransformChainWorldMarker>,
    mut commands: Commands,
) {
    let entity = insert.entity;
    trace!("initialize_cache_spawner_world: entity {:?}", entity);

    commands.entity(entity).insert(TransformChainWorld {
        scale: Vec3::ONE,
        rotation: Quat::IDENTITY,
        translation: Vec3::ZERO,
        matrix: Mat4::IDENTITY,
    });
}

fn cache_spawner_world(
    q_transform: Query<(&Transform, Option<&ChildOf>)>,
    mut q_chain: Query<(Entity, &mut TransformChainWorld), With<TransformChainWorldMarker>>,
) {
    for (entity, mut chain) in &mut q_chain {
        let mut mats = Vec::<Mat4>::new();

        let mut current_entity = entity;
        loop {
            let Ok((transform, parent)) = q_transform.get(current_entity) else {
                warn!(
                    "cache_spawner_world: entity {:?} not found in q_transform",
                    current_entity
                );
                break;
            };

            mats.push(transform.to_matrix());

            if let Some(ChildOf(parent)) = parent {
                current_entity = *parent;
            } else {
                break;
            }
        }

        let world = mats
            .iter()
            .rev()
            .fold(Mat4::IDENTITY, |acc, mat| acc * *mat);
        let (scale, rotation, translation) = world.to_scale_rotation_translation();

        chain.scale = scale;
        chain.rotation = rotation;
        chain.translation = translation;
        chain.matrix = world;
    }
}
