//! A plugin that integrates projectile damage functionality into a Bevy app.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::ProjectileDamageGluePlugin;
}

const DAMAGE_MODIFIER: f32 = 0.01;

/// A plugin that glues projectile damage functionality into the app.
pub struct ProjectileDamageGluePlugin;

impl Plugin for ProjectileDamageGluePlugin {
    fn build(&self, app: &mut App) {
        debug!("ProjectileDamageGluePlugin: build");

        app.add_observer(on_projectile_hit_to_damage);
    }
}

fn on_projectile_hit_to_damage(hit: On<BulletProjectileHit>, mut commands: Commands) {
    commands.trigger(HealthApplyDamage {
        target: hit.hit_entity,
        source: Some(hit.projectile),
        amount: hit.impact_energy * DAMAGE_MODIFIER,
    });
}
