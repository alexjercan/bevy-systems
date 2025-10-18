//! This module contains all the sections of a spaceship.

use bevy::prelude::*;

pub mod spawner;
pub mod bullet_projectile;

pub mod prelude {
    pub use super::spawner::prelude::*;
    pub use super::bullet_projectile::prelude::*;
}

#[derive(Component, Clone, Debug, Reflect)]
pub struct ProjectileMarker;

pub trait ProjectileBundle {
    fn projectile_bundle(&self) -> impl Bundle;
}
