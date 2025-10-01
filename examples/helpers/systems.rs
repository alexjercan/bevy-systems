use bevy::prelude::*;

#[derive(Component)]
pub struct DebugAxisMarker;

pub fn draw_debug_gizmos_axis(
    mut gizmos: Gizmos,
    q_transform: Query<&GlobalTransform, With<DebugAxisMarker>>,
) {
    // Draw the xyz axis of all entities with a GlobalTransform
    for transform in &q_transform {
        let origin = transform.translation();
        let x_axis = transform.rotation() * Vec3::X * 2.0;
        let y_axis = transform.rotation() * Vec3::Y * 2.0;
        let z_axis = transform.rotation() * Vec3::NEG_Z * 2.0;

        gizmos.line(origin, origin + x_axis, Color::srgb(0.9, 0.1, 0.1));
        gizmos.line(origin, origin + y_axis, Color::srgb(0.1, 0.9, 0.1));
        gizmos.line(origin, origin + z_axis, Color::srgb(0.1, 0.1, 0.9));
    }
}

