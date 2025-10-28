//! Module for computing and storing world transforms in a transform chain.
//!
//! NOTE: This is a temporary fix until Bevy's GlobalTransform works as expected with avian3d.
//! TODO: Figure out how to make GlobalTransform work properly and deprecate this module.

use bevy::prelude::*;

pub mod prelude {
    pub use super::TransformChainWorld;
    pub use super::TransformChainWorldPlugin;
    pub use super::TransformChainWorldSystems;
}

#[derive(Component, Clone, Copy, Debug)]
pub struct TransformChainWorld {
    scale: Vec3,
    rotation: Quat,
    translation: Vec3,
    matrix: Mat4,
}

impl Default for TransformChainWorld {
    fn default() -> Self {
        Self {
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            translation: Vec3::ZERO,
            matrix: Mat4::IDENTITY,
        }
    }
}

/// NOTE: Added some methods that are also on GlobalTransform to make it easier to replace once the
/// GlobalTransform works as expected with avian3d...
impl TransformChainWorld {
    #[inline]
    pub fn translation(&self) -> Vec3 {
        self.translation
    }

    #[inline]
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    #[inline]
    pub fn to_matrix(&self) -> Mat4 {
        self.matrix
    }

    #[inline]
    pub fn local_z(&self) -> Dir3 {
        // Quat * unit vector is length 1
        Dir3::new_unchecked(self.rotation * Vec3::Z)
    }

    #[inline]
    pub fn forward(&self) -> Dir3 {
        -self.local_z()
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransformChainWorldSystems {
    Sync,
}

/// Plugin to compute world transforms for entities with TransformChainWorld.
pub struct TransformChainWorldPlugin;

impl Plugin for TransformChainWorldPlugin {
    fn build(&self, app: &mut App) {
        debug!("TransformChainWorldPlugin: build");

        // TODO: I am using Update here because transforms are broken if I don't use Update. Need
        // to investigate why stuff doesn't work prorperly with avian3d physics...
        app.add_systems(
            Update,
            cache_spawner_world.in_set(TransformChainWorldSystems::Sync),
        );
    }
}

fn cache_spawner_world(
    q_transform: Query<(&Transform, Option<&ChildOf>)>,
    mut q_chain: Query<(Entity, &mut TransformChainWorld)>,
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
