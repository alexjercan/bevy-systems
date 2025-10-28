//! This module contains all the sections of a spaceship.
//!
//! TODO: I don't really like the use of generic stuff... rethink the spawner and projectile
//! architecture.

use bevy::prelude::*;

pub mod bullet_projectile;
pub mod spawner;

pub mod prelude {
    pub use super::bullet_projectile::prelude::*;
    pub use super::spawner::prelude::*;

    pub use super::ProjectileBundle;
    pub use super::ProjectileMarker;
    pub use super::ProjectilePlugin;
    pub use super::ProjectileVelocity;
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct ProjectileMarker;

#[derive(Component, Clone, Debug, Deref, DerefMut, Reflect)]
pub struct ProjectileVelocity(pub Vec3);

pub trait ProjectileBundle {
    fn projectile_bundle(&self) -> impl Bundle;
}

pub struct ProjectilePlugin {
    pub render: bool,
}

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        debug!("ProjectilePlugin: build");

        app.add_plugins(spawner::ProjectileSpawnerPlugin::<
            bullet_projectile::BulletProjectileConfig,
        >::default());

        app.add_plugins(bullet_projectile::BulletProjectilePlugin {
            render: self.render,
        });
    }
}
