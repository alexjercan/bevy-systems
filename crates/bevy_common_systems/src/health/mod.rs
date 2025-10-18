//! Health component and related systems for Bevy games.

use bevy::prelude::*;

pub mod prelude {
    pub use super::DamageApply;
    pub use super::DestroyedMarker;
    pub use super::Health;
    pub use super::HealthPlugin;
    pub use super::HealthPluginSet;
}

/// Component representing the health of an entity.
#[derive(Component, Clone, Debug, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

/// Marker component indicating that an entity has been destroyed. When the Health
/// component's current value reaches zero, this marker can be added to signify destruction.
#[derive(Component, Clone, Debug, Reflect)]
pub struct DestroyedMarker;

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

/// Event to apply damage to an entity.
#[derive(Message, Clone, Debug)]
pub struct DamageApply {
    pub target: Entity,
    pub source: Option<Entity>,
    pub amount: f32,
}

/// System set for the bullet projectile plugin.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HealthPluginSet;

/// A plugin that enables the Health component and its related systems.
#[derive(Default)]
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageApply>();

        app.add_systems(Update, update_damage_systems.in_set(HealthPluginSet));
    }
}

fn update_damage_systems(
    mut commands: Commands,
    mut reader: MessageReader<DamageApply>,
    mut q_health: Query<(Entity, &mut Health), Without<DestroyedMarker>>,
) {
    for event in reader.read() {
        if let Ok((entity, mut health)) = q_health.get_mut(event.target) {
            health.current -= event.amount;
            if health.current <= 0.0 {
                health.current = 0.0;
                commands.entity(entity).insert(DestroyedMarker);
            }
        }
    }
}
