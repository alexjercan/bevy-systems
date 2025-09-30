use bevy::prelude::*;

pub mod prelude {
    pub use super::{PointRotation, PointRotationInput, PointRotationPlugin};
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct PointRotation {}

/// The delta by how much to rotate the point
#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct PointRotationInput(pub Vec2);

pub struct PointRotationPlugin;

impl Plugin for PointRotationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PointRotation>()
            .register_type::<PointRotationInput>();

        app.add_observer(initialize_point_rotation_system);
        app.add_systems(Update, (point_rotation_sync_system,));
    }
}

fn initialize_point_rotation_system(
    trigger: Trigger<OnAdd, PointRotation>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert(PointRotationInput(Vec2::ZERO));
}

fn point_rotation_sync_system(
    mut q_point: Query<(&mut PointRotationInput, &mut Transform), With<PointRotation>>,
) {
    for (mut target, mut transform) in &mut q_point {
        let delta_x = target.x;
        let delta_y = target.y;

        // Rotate around the Y axis (yaw)
        let yaw = Quat::from_rotation_y(delta_x);
        // Rotate around the X axis (pitch)
        let pitch = Quat::from_rotation_x(delta_y);

        // Apply the rotations
        transform.rotation = yaw * transform.rotation; // Yaw first
        transform.rotation = transform.rotation * pitch; // Then pitch

        **target = Vec2::ZERO;
    }
}
