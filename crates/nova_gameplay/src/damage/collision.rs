//! A plugin that integrates collision damage functionality into a Bevy app.

use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::CollisionDamageGluePlugin;
    pub use super::CollisionDamageGluePluginSet;
}

const DAMAGE_MODIFIER: f32 = 0.01;

/// A system set that will contain all the systems related to the collision damage glue plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionDamageGluePluginSet;

/// A plugin that glues collision damage functionality into the app.
pub struct CollisionDamageGluePlugin;

impl Plugin for CollisionDamageGluePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_collision_hit_to_damage);
    }
}

fn on_collision_hit_to_damage(
    hit: On<CollisionDamageEvent>,
    mut commands: Commands,
) {
    // TODO: Calculate damage based on relative velocity and DAMAGE_MODIFIER

    commands.trigger(DamageApply {
        target: hit.entity,
        source: Some(hit.other),
        amount: 0.0,
    });
}
