//! This module contains all the sections of a spaceship.

use bevy::prelude::*;

pub mod bullet_projectile;
pub mod spawner;

pub mod prelude {
    pub use super::bullet_projectile::prelude::*;
    pub use super::spawner::prelude::*;
    pub use super::ProjectileBundle;
    pub use super::ProjectileMarker;
    pub use super::ProjectilePlugin;
    pub use super::ProjectilePluginSet;
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct ProjectileMarker;

pub trait ProjectileBundle {
    fn projectile_bundle(&self) -> impl Bundle;
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectilePluginSet;

pub struct ProjectilePlugin {
    pub render: bool,
}

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(spawner::ProjectileSpawnerPlugin::<
            bullet_projectile::BulletProjectileConfig,
        >::default());

        app.add_plugins(bullet_projectile::BulletProjectilePlugin {
            render: self.render,
        });

        app.configure_sets(
            Update,
            spawner::ProjectileSpawnerPluginSet.in_set(ProjectilePluginSet),
        );
        app.configure_sets(
            FixedUpdate,
            spawner::ProjectileSpawnerPluginSet.in_set(ProjectilePluginSet),
        );
    }
}
