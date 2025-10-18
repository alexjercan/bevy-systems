//! A plugin that integrates projectile damage functionality into a Bevy app.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::ProjectileDamageGluePlugin;
    pub use super::ProjectileDamageGluePluginSet;
}

/// A system set that will contain all the systems related to the projectile damage glue plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProjectileDamageGluePluginSet;

/// A plugin that glues projectile damage functionality into the app.
pub struct ProjectileDamageGluePlugin;

impl Plugin for ProjectileDamageGluePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, projectile_hit_to_damage.in_set(ProjectileDamageGluePluginSet));
    }
}

fn projectile_hit_to_damage(
    mut hit_reader: MessageReader<BulletProjectileHit>,
    mut damage_writer: MessageWriter<DamageApply>,
) {
    for hit in hit_reader.read() {
        damage_writer.write(DamageApply {
            target: hit.hit_entity,
            source: Some(hit.projectile),
            amount: hit.impact_energy,
        });
    }
}
