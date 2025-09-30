use bevy::prelude::*;

pub mod prelude {
    pub use super::{PointRotation, PointRotationInput, PointRotationPlugin};
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct PointRotation {}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
struct PointRotationState {
    forward: Vec3,
    right: Vec3,
}

/// The delta by how much to rotate the point
#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct PointRotationInput(pub Vec2);

pub struct PointRotationPlugin;

impl Plugin for PointRotationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PointRotation>()
            .register_type::<PointRotationInput>();

        app.add_observer(initialize_point_rotation_system);
        app.add_systems(
            Update,
            (point_rotation_update_system, point_rotation_sync_system),
        );
    }
}

fn initialize_point_rotation_system(
    trigger: Trigger<OnAdd, PointRotation>,
    q_point: Query<&GlobalTransform, With<PointRotation>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok(transform) = q_point.get(entity) else {
        warn!("PointRotation is not setup correctly");
        return;
    };

    let forward = transform.forward();
    let forward = forward.normalize();

    let right = transform.right();
    let right = right.normalize();

    commands
        .entity(entity)
        .insert(PointRotationInput(Vec2::ZERO))
        .insert(PointRotationState { forward, right });
}

fn point_rotation_update_system(
    mut q_point: Query<(&mut PointRotationInput, &mut PointRotationState), With<PointRotation>>,
) {
    for (mut input, mut state) in &mut q_point {
        let delta_x = input.x;
        let delta_y = input.y;

        if delta_x != 0.0 {
            let up = state.forward.cross(state.right).normalize();
            let yaw = Quat::from_axis_angle(up, delta_x);
            state.forward = (yaw * state.forward).normalize();
            state.right   = (yaw * state.right).normalize();
        }

        if delta_y != 0.0 {
            let pitch = Quat::from_axis_angle(state.right, -delta_y);
            state.forward = (pitch * state.forward).normalize();
            let up = state.forward.cross(state.right).normalize();
            state.right = up.cross(state.forward).normalize();
        }

        **input = Vec2::ZERO;
    }
}

fn point_rotation_sync_system(
    mut q_point: Query<(&PointRotationState, &mut Transform), With<PointRotation>>,
) {
    for (state, mut transform) in &mut q_point {
        let forward = state.forward.normalize();
        let right   = state.right.normalize();
        let up      = forward.cross(right).normalize();

        // Local basis: right, up, forward
        let mat3 = Mat3::from_cols(right, -up, -forward);
        transform.rotation = Quat::from_mat3(&mat3);
    }
}
