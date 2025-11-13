use std::collections::VecDeque;

/// A Bevy plugin that makes entities explode into pieces when they are destroyed.
use bevy::prelude::*;
use rand::Rng;

use super::builder::TriangleMeshBuilder;

pub mod prelude {
    pub use super::{ExplodableEntity, ExplodeFragments, ExplodeMesh, ExplodeMeshPlugin};
}

const MAX_ITERATIONS: usize = 10;

/// A fragment of an explodable mesh.
#[derive(Clone, Debug, Reflect)]
pub struct ExplodeFragment {
    pub origin: Entity,
    pub mesh: Handle<Mesh>,
    pub direction: Dir3,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct ExplodableEntity;

/// The collection of generated fragments from an exploded mesh. This will be added to the entity
/// after the ExplodeMesh event is processed.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ExplodeFragments(pub Vec<ExplodeFragment>);

/// Component that triggers the explosion of a mesh into fragments.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ExplodeMesh {
    /// The number of fragments to create.
    pub fragment_count: usize,
}

impl Default for ExplodeMesh {
    fn default() -> Self {
        Self { fragment_count: 4 }
    }
}

pub struct ExplodeMeshPlugin;

impl Plugin for ExplodeMeshPlugin {
    fn build(&self, app: &mut App) {
        debug!("ExplodeMeshPlugin: build");

        app.add_observer(handle_explosion);
    }
}

fn handle_explosion(
    add: On<Add, ExplodeMesh>,
    mut commands: Commands,
    q_explode: Query<(&ExplodeMesh, Option<&Children>)>,
    q_mesh: Query<(Entity, &Mesh3d), (With<Mesh3d>, With<MeshMaterial3d<StandardMaterial>>)>,
    q_children: Query<&Children>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let entity = add.entity;
    trace!("handle_explosion: entity {:?}", entity);

    let Ok((explode, children)) = q_explode.get(entity) else {
        warn!(
            "handle_explosion: entity {:?} not found in q_explode.",
            entity,
        );
        return;
    };

    let fragment_count = explode.fragment_count;

    let mut mesh_entities = Vec::new();
    if let Ok(mesh_entity) = q_mesh.get(entity) {
        mesh_entities.push(mesh_entity);
    }

    if let Some(children) = children {
        for child in children.iter() {
            let mut queue: VecDeque<Entity> = VecDeque::from([child]);
            while let Some(child) = queue.pop_front() {
                if let Ok(mesh_entity) = q_mesh.get(child) {
                    mesh_entities.push(mesh_entity);
                }

                if let Ok(child_children) = q_children.get(child) {
                    for grandchild in child_children {
                        queue.push_back(*grandchild);
                    }
                }
            }
        }
    }

    let mut fragment_meshes = Vec::new();
    for (mesh_entity, mesh3d) in mesh_entities.into_iter() {
        let Some(mesh) = meshes.get(&**mesh3d) else {
            warn!(
                "handle_explosion: mesh_entity {:?} has no mesh data.",
                mesh_entity
            );
            return;
        };

        trace!(
            "handle_explosion: mesh_entity {:?} fragment_count {}",
            mesh_entity,
            fragment_count
        );

        let Some(fragments) = explode_mesh(&mesh.clone(), fragment_count, MAX_ITERATIONS) else {
            warn!(
                "explode_mesh: entity {:?} failed to slice mesh into fragments.",
                entity
            );
            return;
        };

        for (mesh, normal) in fragments {
            fragment_meshes.push(ExplodeFragment {
                origin: mesh_entity,
                mesh: meshes.add(mesh.clone()),
                direction: Dir3::new_unchecked(normal.normalize()),
            });
        }
    }

    commands
        .entity(entity)
        .insert(ExplodeFragments(fragment_meshes));
}

fn explode_mesh(
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

            let Some((pos, neg)) = TriangleMeshBuilder::from(mesh).slice(plane_normal, plane_point)
            else {
                warn!(
                    "slice_mesh_into_fragments: could not slice mesh with plane normal {:?} at point {:?}.",
                    plane_normal, plane_point
                );
                continue;
            };

            fragments.push((pos.build(), plane_normal));
            fragments.push((neg.build(), -plane_normal));
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
