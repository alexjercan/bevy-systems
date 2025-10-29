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
    pub use super::BulletProjectileSystems;
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

/// Helper function to create a bullet projectile entity bundle.
pub fn bullet_projectile(config: BulletProjectileConfig) -> impl Bundle {
    (
        BulletProjectileMarker,
        RigidBody::Dynamic,
        Collider::sphere(0.05),
        Mass(config.mass),
        TempEntity(config.lifetime),
        LinearVelocity(config.velocity),
    )
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
    fn build(&self, _app: &mut App) {
        debug!("BulletProjectilePlugin: build");
    }
}
