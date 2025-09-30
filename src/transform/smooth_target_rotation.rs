use bevy::prelude::*;

pub mod prelude {
    pub use super::{SmoothTargetRotation, SmoothTargetRotationPlugin, SmoothTargetRotationTarget};
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[require(Transform, GlobalTransform, SmoothTargetRotationTarget)]
pub struct SmoothTargetRotation {
    /// The rotation speed in radians per second
    pub turn_speed: f32,
}

impl Default for SmoothTargetRotation {
    fn default() -> Self {
        Self {
            turn_speed: std::f32::consts::PI, // 180 degrees per second
        }
    }
}

/// The target (desired) yaw / pitch / radius that Transform will lerp toward
#[derive(Component, Default, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct SmoothTargetRotationTarget(pub Quat);

pub struct SmoothTargetRotationPlugin;

impl Plugin for SmoothTargetRotationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmoothTargetRotation>()
            .register_type::<SmoothTargetRotationTarget>();

        app.add_systems(Update, smooth_target_rotation_update_system);
    }
}

fn smooth_target_rotation_update_system(
    time: Res<Time>,
    mut q_rotation: Query<(&SmoothTargetRotation, &SmoothTargetRotationTarget, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (rotation, target, mut transform) in &mut q_rotation {
        let current = transform.rotation;
        let target = **target;

        let t = (rotation.turn_speed * dt).min(1.0);
        transform.rotation = current.slerp(target, t);
    }
}
