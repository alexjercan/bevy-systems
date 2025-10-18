//! Bullet projectile implementation.
//!
//! The bullet projectile is a kinematic projectile that moves in a straight line
//! at a constant speed. It uses raycasting to detect collisions.

use crate::helpers::prelude::*;
use avian3d::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::bullet_projectile;
    pub use super::BulletProjectileConfig;
    pub use super::BulletProjectileMarker;
    pub use super::BulletProjectilePlugin;
    pub use super::BulletProjectilePluginSet;
    pub use super::BulletProjectileRenderMesh;
    pub use super::BulletProjectileHit;
}

/// Configuration for a bullet projectile.
#[derive(Clone, Debug, Reflect)]
pub struct BulletProjectileConfig {
    /// Muzzle speed of the projectile in units per second.
    pub muzzle_speed: f32,
    /// Lifetime of the projectile in seconds.
    pub lifetime: f32,
    /// The mass of the bullet projectile.
    pub mass: f32,
    /// Optional render mesh for the projectile.
    pub render_mesh: Option<Handle<Scene>>,
}

impl Default for BulletProjectileConfig {
    fn default() -> Self {
        Self {
            muzzle_speed: 50.0,
            lifetime: 5.0,
            mass: 0.1,
            render_mesh: None,
        }
    }
}

/// Marker component for bullet projectiles.
#[derive(Component, Clone, Debug, Reflect)]
pub struct BulletProjectileMarker;

/// Muzzle speed of the bullet projectile.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileMuzzleSpeed(pub f32);

/// Mass of the bullet projectile.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileMass(pub f32);

/// Render mesh for the bullet projectile.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileRenderMesh(pub Option<Handle<Scene>>);

/// Previous position of the bullet projectile for sweep collision detection.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct BulletProjectilePrev(Option<Vec3>);

/// Helper function to create a bullet projectile entity bundle.
pub fn bullet_projectile(config: BulletProjectileConfig) -> impl Bundle {
    (
        BulletProjectileMarker,
        TempEntity(config.lifetime),
        BulletProjectileMuzzleSpeed(config.muzzle_speed),
        BulletProjectileMass(config.mass),
        BulletProjectileRenderMesh(config.render_mesh),
        BulletProjectilePrev(None),
    )
}

impl super::ProjectileBundle for BulletProjectileConfig {
    fn projectile_bundle(&self) -> impl Bundle {
        bullet_projectile(self.clone())
    }
}

/// Message sent when a bullet projectile hits an entity.
#[derive(Message, Clone, Debug)]
pub struct BulletProjectileHit {
    /// The projectile entity that hit.
    pub projectile: Entity,
    /// The entity that was hit.
    pub hit_entity: Entity,
    /// The point of impact in world space.
    pub hit_point: Vec3,
    /// The normal of the surface hit.
    pub hit_normal: Vec3,
    /// The impact energy of the hit.
    pub impact_energy: f32,
}

/// System set for the bullet projectile plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BulletProjectilePluginSet;

/// A plugin that enables the BulletProjectile component and its related systems.
#[derive(Default)]
pub struct BulletProjectilePlugin {
    pub render: bool,
}

impl Plugin for BulletProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BulletProjectileHit>();

        app.add_systems(
            Update,
            update_ray_projectiles.in_set(BulletProjectilePluginSet),
        );

        app.add_systems(
            FixedUpdate,
            update_sweep_collisions.in_set(BulletProjectilePluginSet),
        );

        if self.render {
            app.add_observer(insert_projectile_render);
        }
    }
}

fn insert_projectile_render(
    add: On<Add, BulletProjectileMarker>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_render_mesh: Query<&BulletProjectileRenderMesh>,
) {
    let entity = add.entity;
    debug!("Inserting BulletProjectile render for entity: {:?}", entity);
    let Ok(render_mesh) = q_render_mesh.get(entity) else {
        warn!(
            "BulletProjectile entity {:?} missing BulletProjectileRenderMesh component",
            entity
        );
        return;
    };

    match &**render_mesh {
        Some(scene_handle) => {
            commands.entity(entity).insert((children![(
                Name::new("Bullet Projectile Render"),
                SceneRoot(scene_handle.clone()),
            ),],));
        }
        None => {
            commands.entity(entity).insert((children![(
                Name::new("Bullet Projectile Render"),
                Mesh3d(meshes.add(Cuboid::new(0.05, 0.05, 0.3))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
            ),],));
        }
    }
}

fn update_ray_projectiles(
    mut q_projectiles: Query<
        (&BulletProjectileMuzzleSpeed, &mut Transform),
        With<super::ProjectileMarker>,
    >,
    time: Res<Time>,
) {
    for (speed, mut transform) in &mut q_projectiles {
        let distance = **speed * time.delta_secs();
        let forward = transform.forward();
        transform.translation += distance * *forward;
    }
}

fn update_sweep_collisions(
    mut commands: Commands,
    query: SpatialQuery,
    mut q_projectiles: Query<
        (
            Entity,
            &mut Transform,
            &mut BulletProjectilePrev,
            &BulletProjectileMuzzleSpeed,
            &BulletProjectileMass,
        ),
        With<super::ProjectileMarker>,
    >,
    mut writer: MessageWriter<BulletProjectileHit>,
) {
    let filter = SpatialQueryFilter::default();

    for (entity, transform, mut prev, speed, mass) in &mut q_projectiles {
        if prev.is_none() {
            **prev = Some(transform.translation);
            continue;
        }

        let origin = prev.unwrap();
        let direction = transform.translation - origin;
        **prev = Some(transform.translation);

        let Ok((direction, distance)) = Dir3::new_and_length(direction) else {
            continue;
        };

        if let Some(ray_hit_data) = query.cast_ray(origin, direction, distance, true, &filter) {
            commands.entity(entity).despawn();

            let distance = ray_hit_data.distance;
            let hit_point = origin + direction * distance;
            let impact_energy = 0.5 * **mass * (**speed * **speed);

            writer.write(BulletProjectileHit {
                projectile: entity,
                hit_entity: ray_hit_data.entity,
                hit_point,
                hit_normal: ray_hit_data.normal,
                impact_energy,
            });
        }
    }
}
