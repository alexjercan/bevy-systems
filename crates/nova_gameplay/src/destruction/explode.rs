/// A Bevy plugin that makes entities explode into pieces when they are destroyed.
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

pub mod prelude {
    pub use super::ExplodeOnDestroy;
    pub use super::ExplodeOnDestroyPlugin;
    pub use super::ExplodeOnDestroyPluginSet;
    pub use super::FragmentMeshMarker;
}

const MAX_ITERATIONS: usize = 10;

#[derive(Component, Clone, Debug, Reflect)]
pub struct FragmentMeshMarker;

/// A component that makes an entity explode into pieces when it is destroyed.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ExplodeOnDestroy {
    /// The entity that contains the mesh we will use for the "explosion"
    pub mesh_entity: Option<Entity>,
    /// The number of fragments to create when the entity is destroyed.
    pub fragment_count: usize,
    /// The force multiplier range applied to each fragment when exploded.
    pub force_multiplier_range: (f32, f32),
}

impl Default for ExplodeOnDestroy {
    fn default() -> Self {
        Self {
            mesh_entity: None,
            fragment_count: 10,
            force_multiplier_range: (2.0, 5.0),
        }
    }
}

/// A system set that will contain all the systems related to the explode on destroy plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplodeOnDestroyPluginSet;

/// A plugin that makes entities explode into pieces when they are destroyed.
pub struct ExplodeOnDestroyPlugin;

impl Plugin for ExplodeOnDestroyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_explosion);
    }
}

fn handle_explosion(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_explode: Query<&ExplodeOnDestroy>,
    mut q_mesh: Query<(
        &GlobalTransform,
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let entity = add.entity;
    debug!("Handling explosion for entity {:?}", entity);

    let Ok(explode) = q_explode.get(entity) else {
        debug!(
            "Destroyed entity {:?} missing ExplodeOnDestroy component, skipping explosion.",
            entity
        );
        return;
    };

    let Some(mesh_entity) = explode.mesh_entity else {
        debug!(
            "ExplodeOnDestroy component on entity {:?} has no mesh_entity set, skipping explosion.",
            entity
        );
        return;
    };

    let Ok((transform, mesh3d, material3d, mut visibility)) = q_mesh.get_mut(mesh_entity) else {
        warn!("Mesh entity {:?} for explosion on entity {:?} does not have a Mesh3d component, skipping explosion.", mesh_entity, entity);
        return;
    };

    let Some(mesh) = meshes.get(&**mesh3d) else {
        warn!(
            "Mesh asset for entity {:?} not found, skipping explosion.",
            mesh_entity
        );
        return;
    };

    debug!(
        "Exploding entity {:?} with mesh entity {:?} into {} fragments.",
        entity, mesh_entity, explode.fragment_count
    );

    *visibility = Visibility::Hidden;

    let Some(fragments) =
        slice_mesh_into_fragments(&mesh.clone(), explode.fragment_count, MAX_ITERATIONS)
    else {
        warn!(
            "Failed to slice mesh for entity {:?} into fragments.",
            entity
        );
        return;
    };

    for (mesh, normal) in fragments {
        let transform = transform.compute_transform();
        let offset = normal * 0.1;
        let transform = transform.with_translation(transform.translation + offset);

        commands.spawn((
            Name::new("Explosion Fragment"),
            FragmentMeshMarker,
            Mesh3d(meshes.add(mesh.clone())),
            material3d.clone(),
            transform,
            Visibility::Visible,
            RigidBody::Dynamic,
            Collider::convex_hull_from_mesh(&mesh).unwrap_or(Collider::sphere(0.5)),
            LinearVelocity(
                normal
                    * rand::rng().random_range(
                        explode.force_multiplier_range.0..explode.force_multiplier_range.1,
                    ),
            ),
        ));
    }
}

fn slice_mesh_into_fragments(
    original: &Mesh,
    fragment_count: usize,
    max_iterations: usize,
) -> Option<Vec<(Mesh, Vec3)>> {
    let mut queue = VecDeque::from([(original.clone(), Vec3::ZERO)]);
    let mut rng = rand::rng();

    for _ in 0..max_iterations {
        let mut fragments = vec![];
        while let Some((mesh, _)) = queue.pop_front() {
            let plane_point = Vec3::ZERO;
            let plane_normal = {
                let u: f32 = rng.random_range(-1.0..1.0);
                let theta: f32 = rng.random_range(0.0..std::f32::consts::TAU);
                let r = (1.0 - u * u).sqrt();
                Vec3::new(r * theta.cos(), r * theta.sin(), u).normalize()
            };

            let Some((pos, neg)) = mesh_slice(&mesh, plane_normal, plane_point) else {
                warn!(
                    "Failed to slice mesh into fragments... implement better code next time, bruh."
                );
                continue;
            };

            fragments.push((pos, plane_normal));
            fragments.push((neg, -plane_normal));
        }

        if fragments.len() >= fragment_count {
            return Some(fragments);
        } else if fragments.is_empty() {
            warn!("Could not generate more fragments, returning what we have.");
            return None;
        } else {
            queue = VecDeque::from(fragments);
        }
    }

    None
}
