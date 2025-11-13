//! A Bevy plugin that handles damage.

pub mod prelude {
    pub use super::{DamagePlugin, MeshFragmentMarker};
}

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;
use bevy_rand::prelude::*;
use nova_events::prelude::*;
use rand::Rng;

const DAMAGE_MODIFIER: f32 = 1.00;

#[derive(Component, Debug, Clone, Reflect)]
pub struct MeshFragmentMarker;

/// A plugin that handles damage.
pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        debug!("DamagePlugin: build");

        app.add_observer(on_rigidbody_spawn);
        app.add_observer(on_collision_hit_to_damage);
        app.add_observer(on_destroyed_entity);
        app.add_observer(on_explode_entity);
        app.add_observer(handle_entity_explosion);
    }
}

fn on_rigidbody_spawn(
    add: On<Add, ColliderOf>,
    mut commands: Commands,
    q_collider: Query<&ColliderOf>,
    q_health: Query<(), (With<Health>, With<RigidBody>)>,
) {
    let entity = add.entity;
    trace!("on_rigidbody_spawn: entity {:?}", entity);

    let Ok(collider) = q_collider.get(entity) else {
        warn!(
            "on_rigidbody_spawn: entity {:?} not found in q_collider",
            entity
        );
        return;
    };

    let Ok(_) = q_health.get(collider.body) else {
        // NOTE: RigidBody does not have Health component
        return;
    };

    // NOTE: Add collision damage for all rigid bodies with Health component
    debug!(
        "on_rigidbody_spawn: adding CollisionImpactMarker to entity {:?}",
        entity
    );
    commands.entity(entity).insert(CollisionImpactMarker);
}

fn on_collision_hit_to_damage(
    hit: On<CollisionImpactEvent>,
    mut commands: Commands,
    q_mass: Query<&ComputedMass>,
) {
    let amount = hit.relative_velocity.length() * DAMAGE_MODIFIER;
    let mass = q_mass.get(hit.other).map(|m| m.value()).unwrap_or(1.0);
    let amount = amount * mass;

    commands.trigger(HealthApplyDamage {
        target: hit.entity,
        source: Some(hit.other),
        amount,
    });
}

fn on_destroyed_entity(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_info: Query<(&EntityId, &EntityTypeName), With<DestroyedMarker>>,
) {
    let entity = add.entity;
    trace!("on_destroyed_entity: entity {:?}", entity);

    let Ok((id, type_name)) = q_info.get(entity) else {
        warn!(
            "on_destroyed_entity: entity {:?} not found in q_info",
            entity
        );
        return;
    };

    commands.fire::<OnDestroyedEvent>(OnDestroyedEventInfo {
        id: id.to_string(),
        type_name: type_name.to_string(),
    });
}

fn on_explode_entity(
    add: On<Add, DestroyedMarker>,
    mut commands: Commands,
    q_explode: Query<(), (With<ExplodableEntity>, With<DestroyedMarker>)>,
) {
    let entity = add.entity;
    trace!("on_explode_entity: entity {:?}", entity);

    let Ok(_) = q_explode.get(entity) else {
        // NOTE: Not an explodable entity
        return;
    };

    debug!("on_explode_entity: entity {:?} will explode", entity);
    commands
        .entity(entity)
        .insert(ExplodeMesh { fragment_count: 4 });
}

fn handle_entity_explosion(
    add: On<Add, ExplodeFragments>,
    mut commands: Commands,
    q_explode: Query<&ExplodeFragments, With<ExplodableEntity>>,
    q_mesh: Query<(&GlobalTransform, &MeshMaterial3d<StandardMaterial>), With<Mesh3d>>,
    meshes: ResMut<Assets<Mesh>>,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
) {
    let entity = add.entity;
    trace!("handle_entity_explosion: entity {:?}", entity);

    let Ok(fragments) = q_explode.get(entity) else {
        warn!(
            "handle_entity_explosion: entity {:?} not found in q_explode.",
            entity,
        );
        return;
    };

    for fragment in fragments.iter() {
        let Ok((transform, mesh_material)) = q_mesh.get(fragment.origin) else {
            warn!(
                "handle_entity_explosion: mesh_entity {:?} not found in q_mesh.",
                fragment.origin,
            );
            continue;
        };

        let transform = transform.compute_transform();
        let offset = fragment.direction * 0.5;
        let velocity = fragment.direction * rng.random_range(2.0..5.0);
        let transform = transform.with_translation(transform.translation + offset);
        let Some(mesh) = meshes.get(&fragment.mesh) else {
            warn!(
                "handle_entity_explosion: mesh_entity {:?} has no mesh data.",
                fragment.origin,
            );
            continue;
        };

        commands.spawn((
            MeshFragmentMarker,
            Name::new(format!("Explosion Fragment of {:?}", entity)),
            Mesh3d(fragment.mesh.clone()),
            mesh_material.clone(),
            transform,
            RigidBody::Dynamic,
            Collider::convex_hull_from_mesh(mesh).unwrap_or(Collider::sphere(0.5)),
            LinearVelocity(velocity),
        ));
    }

    // TODO: How can I just disable the object in case I still need it somehow?
    commands.entity(entity).despawn();
}
