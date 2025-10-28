use bevy::{color::palettes::tailwind, prelude::*};
use nova_gameplay::prelude::*;

pub struct TurretSectionDebugPlugin;

impl Plugin for TurretSectionDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (debug_draw_barrel_direction).in_set(super::DebugSystems),
        );
    }
}

const DEBUG_LINE_LENGTH: f32 = 100.0;

fn debug_draw_barrel_direction(
    q_muzzle: Query<&GlobalTransform, With<TurretSectionBarrelMuzzleMarker>>,
    mut gizmos: Gizmos,
) {
    for muzzle_transform in &q_muzzle {
        let barrel_pos = muzzle_transform.translation();
        let barrel_dir = muzzle_transform.forward();

        let line_length = DEBUG_LINE_LENGTH;
        let line_end = barrel_pos + barrel_dir * line_length;

        let color = tailwind::RED_500;
        gizmos.line(barrel_pos, line_end, color);
    }
}
