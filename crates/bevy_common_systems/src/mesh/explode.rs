use std::collections::VecDeque;

/// A Bevy plugin that makes entities explode into pieces when they are destroyed.
use bevy::prelude::*;
use rand::Rng;

use super::slicer::mesh_slice;

pub mod prelude {
    pub use super::{
        ExplodableEntityMarker, ExplodableFragments, ExplodableMesh, ExplodeMesh, ExplodeMeshPlugin,
    };
}

const MAX_ITERATIONS: usize = 10;

/// Marker component for entities that can explode. This should be added to the root entity of the
/// object. The ExplodeMesh plugin will search for all Mesh3d components in the entity and its
/// children, and it will build an ExplodableMesh component containing those meshes.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct ExplodableEntityMarker;

/// Component that holds the list of mesh entities that can be exploded.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ExplodableMesh(pub Vec<Entity>);

/// A fragment of an explodable mesh.
#[derive(Clone, Debug, Reflect)]
pub struct ExplodableFragment {
    pub origin: Entity,
    pub mesh: Handle<Mesh>,
    pub direction: Dir3,
}

/// The collection of generated fragments from an exploded mesh. This will be added to the entity
/// after the ExplodeMesh event is processed.
#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct ExplodableFragments(pub Vec<ExplodableFragment>);

/// Event that triggers the explosion of a mesh into fragments.
#[derive(Event, Clone, Debug, Reflect)]
pub struct ExplodeMesh {
    /// The entity to explode.
    pub entity: Entity,
    /// The number of fragments to create.
    pub fragment_count: usize,
}

impl Default for ExplodeMesh {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            fragment_count: 4,
        }
    }
}

pub struct ExplodeMeshPlugin;

impl Plugin for ExplodeMeshPlugin {
    fn build(&self, app: &mut App) {
        debug!("ExplodeMeshPlugin: build");

        // TODO: How can I implement this using observers only?
        app.add_systems(Update, setup_explode_mesh_children);
        app.add_observer(handle_explosion);
    }
}

fn handle_explosion(
    explode: On<ExplodeMesh>,
    mut commands: Commands,
    q_explode: Query<&ExplodableMesh, With<ExplodableEntityMarker>>,
    q_mesh: Query<&Mesh3d>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let entity = explode.entity;
    trace!("handle_explosion: entity {:?}", entity);

    let fragment_count = explode.fragment_count;

    let Ok(mesh_entities) = q_explode.get(entity) else {
        warn!(
            "handle_explosion: entity {:?} not found in q_explode.",
            entity,
        );
        return;
    };

    let mut fragment_meshes = Vec::new();
    for mesh_entity in &**mesh_entities {
        let Ok(mesh3d) = q_mesh.get(*mesh_entity) else {
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
            fragment_meshes.push(ExplodableFragment {
                origin: *mesh_entity,
                mesh: meshes.add(mesh.clone()),
                direction: Dir3::new_unchecked(normal.normalize()),
            });
        }
    }

    commands
        .entity(entity)
        .insert(ExplodableFragments(fragment_meshes));
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
    // NOTE: We only handle StandardMaterial for now
    q_mesh: Query<Entity, (With<Mesh3d>, With<MeshMaterial3d<StandardMaterial>>)>,
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
