use bevy::{color::palettes::tailwind, prelude::*};
use nova_gameplay::{bevy_common_systems::projectiles::spawner::ProjectileSpawnerFireState, prelude::*};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_debug_gizmos,).in_set(super::DebugSystems));
    }
}

fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    q_spawner: Query<(&GlobalTransform, &ProjectileSpawnerFireState), With<ProjectileSpawnerMarker>>,
) {
    for (transform, fire_state) in &q_spawner {
        let origin = transform.translation();
        let dir = transform.forward() * 2.0;

        let color = if fire_state.is_finished() {
            tailwind::GREEN_500
        } else {
            tailwind::YELLOW_500
        };

        gizmos.sphere(transform.to_isometry(), 0.2, color);
        gizmos.line(origin, origin + dir, color);
    }
}
