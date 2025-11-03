use std::collections::VecDeque;

/// A Bevy plugin that makes entities explode into pieces when they are destroyed.
use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use super::slicer::mesh_slice;

pub mod prelude {
    pub use super::{
        ExplodableEntityMarker, ExplodableMesh, ExplodeMesh, ExplodeMeshPlugin, FragmentMeshMarker,
    };
}

const MAX_ITERATIONS: usize = 10;

#[derive(Component, Clone, Debug, Reflect)]
pub struct FragmentMeshMarker;

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct ExplodableEntityMarker;

#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ExplodableMesh(pub Vec<Entity>);

#[derive(Component, Clone, Debug, Reflect)]
#[require(ExplodableMesh)]
pub struct ExplodeMesh {
    pub fragment_count: usize,
    pub force_multiplier_range: (f32, f32),
}

impl Default for ExplodeMesh {
    fn default() -> Self {
        Self {
            fragment_count: 4,
            force_multiplier_range: (2.0, 5.0),
        }
    }
}

pub struct ExplodeMeshPlugin;

impl Plugin for ExplodeMeshPlugin {
    fn build(&self, app: &mut App) {
        debug!("ExplodeOnDestroyPlugin: build");

        // TODO: How can I implement this using observers only?
        app.add_systems(Update, setup_explode_mesh_children);
        app.add_observer(handle_explosion);
    }
}

fn handle_explosion(
    add: On<Add, ExplodeMesh>,
    mut commands: Commands,
    q_explode: Query<(&ExplodeMesh, &ExplodableMesh)>,
    mut q_mesh: Query<(
        &GlobalTransform,
        &Mesh3d,
        &MeshMaterial3d<StandardMaterial>,
        &mut Visibility,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let entity = add.entity;
    trace!("handle_explosion: entity {:?}", entity);

    let Ok((
        &ExplodeMesh {
            fragment_count,
            force_multiplier_range,
        },
        explode,
    )) = q_explode.get(entity)
    else {
        warn!(
            "handle_explosion: entity {:?} not found in q_explode.",
            entity,
        );
        return;
    };

    for mesh_entity in &**explode {
        let Ok((transform, mesh3d, material3d, mut visibility)) = q_mesh.get_mut(*mesh_entity)
        else {
            warn!(
                "explode_mesh: mesh_entity {:?} not found in q_mesh.",
                entity,
            );
            return;
        };

        let Some(mesh) = meshes.get(&**mesh3d) else {
            warn!("explode_mesh: mesh_entity {:?} has no mesh data.", entity,);
            return;
        };

        trace!(
            "handle_explosion: mesh_entity {:?} fragment_count {}",
            mesh_entity,
            fragment_count
        );

        *visibility = Visibility::Hidden;

        let Some(fragments) =
            slice_mesh_into_fragments(&mesh.clone(), fragment_count, MAX_ITERATIONS)
        else {
            warn!(
                "explode_mesh: entity {:?} failed to slice mesh into fragments.",
                entity
            );
            return;
        };

        for (mesh, normal) in fragments {
            let transform = transform.compute_transform();
            let offset = normal * 0.5;
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
                        * rand::rng()
                            .random_range(force_multiplier_range.0..force_multiplier_range.1),
                ),
            ));
        }
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
                    "slice_mesh_into_fragments: could not slice mesh with plane normal {:?} at point {:?}.",
                    plane_normal, plane_point
                );
                continue;
            };

            fragments.push((pos, plane_normal));
            fragments.push((neg, -plane_normal));
        }

        if fragments.len() >= fragment_count {
            return Some(fragments);
        } else if fragments.is_empty() {
            warn!("slice_mesh_into_fragments: no fragments generated after slicing.");
            return None;
        } else {
            queue = VecDeque::from(fragments);
        }
    }

    None
}

fn setup_explode_mesh_children(
    mut commands: Commands,
    q_mesh: Query<Entity, With<Mesh3d>>,
    q_children: Query<&Children>,
    q_explode: Query<
        (Entity, Option<&Children>),
        (
            With<ExplodableEntityMarker>,
            Or<(Changed<Children>, Added<ExplodableEntityMarker>)>,
        ),
    >,
) {
    for (entity, children) in &q_explode {
        trace!("setup_explode_mesh: entity {:?}", entity);

        let mut meshes = Vec::new();
        if let Ok(mesh_entity) = q_mesh.get(entity) {
            meshes.push(mesh_entity);
        }

        if let Some(children) = children {
            for child in children.iter() {
                let mut queue: VecDeque<Entity> = VecDeque::from([child]);
                while let Some(child) = queue.pop_front() {
                    if let Ok(mesh_entity) = q_mesh.get(child) {
                        meshes.push(mesh_entity);
                    }

                    if let Ok(child_children) = q_children.get(child) {
                        for grandchild in child_children {
                            queue.push_back(*grandchild);
                        }
                    }
                }
            }
        }

        commands.entity(entity).insert(ExplodableMesh(meshes));
    }
}
