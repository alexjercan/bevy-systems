//! A plugin that integrates collision damage functionality into a Bevy app.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_common_systems::prelude::*;

pub mod prelude {
    pub use super::CollisionDamageGluePlugin;
}

const DAMAGE_MODIFIER: f32 = 1.00;

/// A plugin that glues collision damage functionality into the app.
pub struct CollisionDamageGluePlugin;

impl Plugin for CollisionDamageGluePlugin {
    fn build(&self, app: &mut App) {
        debug!("CollisionDamageGluePlugin: build");

        app.add_observer(on_collision_hit_to_damage);
    }
}

fn on_collision_hit_to_damage(
    hit: On<CollisionDamageEvent>,
    mut commands: Commands,
    q_mass: Query<&ComputedMass>,
) {
    let amount = hit.relative_velocity.length() * DAMAGE_MODIFIER;
    let mass = q_mass.get(hit.other).map(|m| m.value()).unwrap_or(1.0);
    let amount = amount * mass;

    println!(
        "on_collision_hit_to_damage: entity {:?} took damage {:.2}",
        hit.entity, amount
    );

    commands.trigger(HealthApplyDamage {
        target: hit.entity,
        source: Some(hit.other),
        amount,
    });
}
