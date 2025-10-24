//! A plugin that integrates projectile damage functionality into a Bevy app.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::ProjectileDamageGluePlugin;
    pub use super::ProjectileDamageGluePluginSet;
}

const DAMAGE_MODIFIER: f32 = 0.01;

/// A system set that will contain all the systems related to the projectile damage glue plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectileDamageGluePluginSet;

/// A plugin that glues projectile damage functionality into the app.
pub struct ProjectileDamageGluePlugin;

impl Plugin for ProjectileDamageGluePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_projectile_hit_to_damage);
    }
}

fn on_projectile_hit_to_damage(
    hit: On<BulletProjectileHit>,
    mut commands: Commands,
) {
    commands.trigger(DamageApply {
        target: hit.hit_entity,
        source: Some(hit.projectile),
        amount: hit.impact_energy * DAMAGE_MODIFIER,
    });
}
