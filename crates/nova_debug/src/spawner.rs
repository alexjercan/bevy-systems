use bevy::{color::palettes::tailwind, prelude::*};
use nova_gameplay::{
    bevy_common_systems::projectiles::spawner::ProjectileSpawnerFireState, prelude::*,
};

pub struct SpawnerDebugPlugin;

impl Plugin for SpawnerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (draw_debug_gizmos_spawner, draw_debug_gizmos_projectile,).in_set(super::DebugSystems));
    }
}

fn draw_debug_gizmos_spawner(
    mut gizmos: Gizmos,
    q_spawner: Query<
        (&GlobalTransform, &ProjectileSpawnerFireState),
        With<ProjectileSpawnerMarker>,
    >,
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

fn draw_debug_gizmos_projectile(
    mut gizmos: Gizmos,
    q_spawner: Query<
        (&GlobalTransform, &ProjectileVelocity),
        With<ProjectileMarker>,
    >,
) {
    for (transform, velocity) in &q_spawner {
        let origin = transform.translation();
        let dir = velocity.normalize_or_zero();
        let color = tailwind::BLUE_500;

        gizmos.sphere(transform.to_isometry(), 0.2, color);
        gizmos.line(origin, origin + dir, color);
    }
}
