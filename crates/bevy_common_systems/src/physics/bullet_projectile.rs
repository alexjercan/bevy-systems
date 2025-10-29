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
    pub use super::BulletProjectileHit;
    pub use super::BulletProjectileMarker;
    pub use super::BulletProjectileMass;
    pub use super::BulletProjectilePlugin;
    pub use super::BulletProjectileSystems;
    pub use super::BulletProjectileVelocity;
}

/// Configuration for a bullet projectile.
#[derive(Clone, Debug, Reflect)]
pub struct BulletProjectileConfig {
    /// Lifetime of the projectile in seconds.
    pub lifetime: f32,
    /// The mass of the bullet projectile.
    pub mass: f32,
    /// The linear velocity of the bullet projectile.
    pub velocity: Vec3,
}

impl Default for BulletProjectileConfig {
    fn default() -> Self {
        Self {
            lifetime: 5.0,
            mass: 0.1,
            velocity: Vec3::ZERO,
        }
    }
}

/// Marker component for bullet projectiles.
#[derive(Component, Clone, Debug, Reflect)]
pub struct BulletProjectileMarker;

/// Mass of the bullet projectile.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileMass(pub f32);

/// Velocity of the bullet projectile.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct BulletProjectileVelocity(pub Vec3);

/// Previous position of the bullet projectile for sweep collision detection.
#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
struct BulletProjectilePrev(Option<Vec3>);

/// Helper function to create a bullet projectile entity bundle.
pub fn bullet_projectile(config: BulletProjectileConfig) -> impl Bundle {
    (
        BulletProjectileMarker,
        TempEntity(config.lifetime),
        BulletProjectileMass(config.mass),
        BulletProjectileVelocity(config.velocity),
        BulletProjectilePrev(None),
    )
}

/// Message sent when a bullet projectile hits an entity.
#[derive(Event, Clone, Debug)]
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
pub enum BulletProjectileSystems {
    Sync,
}

/// A plugin that enables the BulletProjectile component and its related systems.
#[derive(Default)]
pub struct BulletProjectilePlugin;

impl Plugin for BulletProjectilePlugin {
    fn build(&self, app: &mut App) {
        debug!("BulletProjectilePlugin: build");

        app.add_systems(
            FixedUpdate,
            (update_ray_projectiles, update_sweep_collisions).in_set(BulletProjectileSystems::Sync),
        );
    }
}

fn update_ray_projectiles(
    mut q_projectiles: Query<
        (&BulletProjectileVelocity, &mut Transform),
        With<BulletProjectileMarker>,
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (velocity, mut transform) in &mut q_projectiles {
        transform.translation += **velocity * dt;
    }
}

fn update_sweep_collisions(
    mut commands: Commands,
    query: SpatialQuery,
    mut q_projectiles: Query<
        (
            Entity,
            &Transform,
            &BulletProjectileVelocity,
            &BulletProjectileMass,
            &mut BulletProjectilePrev,
        ),
        With<BulletProjectileMarker>,
    >,
) {
    let filter = SpatialQueryFilter::default();

    for (entity, transform, velocity, mass, mut prev) in &mut q_projectiles {
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
            // NOTE: Maybe in the future I don't want to despawn this right away, but maybe allow
            // having penetration. Also I would rather send the speed instead of the energy and
            // compute the energy where it is needed.

            trace!(
                "update_sweep_collisions: projectile {:?} hit entity {:?} at distance {}",
                entity,
                ray_hit_data.entity,
                ray_hit_data.distance
            );

            commands.entity(entity).despawn();

            let distance = ray_hit_data.distance;
            let hit_point = origin + direction * distance;
            let speed = velocity.length();
            let impact_energy = 0.5 * **mass * speed * speed;

            commands.trigger(BulletProjectileHit {
                projectile: entity,
                hit_entity: ray_hit_data.entity,
                hit_point,
                hit_normal: ray_hit_data.normal,
                impact_energy,
            });
        }
    }
}
