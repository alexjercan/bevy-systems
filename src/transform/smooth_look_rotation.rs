use bevy::prelude::*;

pub mod prelude {
    pub use super::{SmoothLookRotation, SmoothLookRotationPlugin, SmoothLookRotationTarget};
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct SmoothLookRotation {
    pub yaw_speed: f32,   // rad/s
    pub pitch_speed: f32, // rad/s
    pub min_pitch: Option<f32>,   // rad
    pub max_pitch: Option<f32>,   // rad
}

impl Default for SmoothLookRotation {
    fn default() -> Self {
        Self {
            yaw_speed: std::f32::consts::PI,   // 180 degrees per second
            pitch_speed: std::f32::consts::PI, // 180 degrees per second
            min_pitch: None,
            max_pitch: None,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct SmoothLookRotationTarget {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct SmoothLookRotationState {
    yaw: f32,
    pitch: f32,
}

pub struct SmoothLookRotationPlugin;

impl Plugin for SmoothLookRotationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmoothLookRotation>()
            .register_type::<SmoothLookRotationTarget>()
            .register_type::<SmoothLookRotationState>();

        app.add_observer(initialize_smooth_look_system);
        app.add_systems(Update, smooth_look_rotation_update_system);
    }
}

fn initialize_smooth_look_system(
    trigger: Trigger<OnInsert, SmoothLookRotation>,
    q_look: Query<&Transform, With<SmoothLookRotation>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok(transform) = q_look.get(entity) else {
        warn!(
            "initialize_smooth_look_system: entity {:?} is not setup correctly",
            entity
        );
        return;
    };

    let yaw = transform.rotation.to_euler(EulerRot::YXZ).0;
    let pitch = transform.rotation.to_euler(EulerRot::YXZ).1;

    commands.entity(entity).insert((
        SmoothLookRotationTarget { yaw, pitch },
        SmoothLookRotationState { yaw, pitch },
    ));
}

fn smooth_look_rotation_update_system(
    time: Res<Time>,
    mut q_look: Query<(
        &SmoothLookRotation,
        &SmoothLookRotationTarget,
        &mut SmoothLookRotationState,
        &mut Transform,
    )>,
) {
    let dt = time.delta_secs();
    for (look, target, mut state, mut transform) in &mut q_look {
        // Update yaw
        let yaw_diff = (target.yaw - state.yaw + std::f32::consts::PI)
            .rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;
        let max_yaw_change = look.yaw_speed * dt;
        if yaw_diff.abs() <= max_yaw_change {
            state.yaw = target.yaw;
        } else {
            state.yaw += yaw_diff.signum() * max_yaw_change;
        }

        // Update pitch
        let pitch_diff = target.pitch - state.pitch;
        let max_pitch_change = look.pitch_speed * dt;
        if pitch_diff.abs() <= max_pitch_change {
            state.pitch = target.pitch;
        } else {
            state.pitch += pitch_diff.signum() * max_pitch_change;
        }

        // Clamp pitch
        if let Some(min_pitch) = look.min_pitch {
            state.pitch = state.pitch.max(min_pitch);
        }
        if let Some(max_pitch) = look.max_pitch {
            state.pitch = state.pitch.min(max_pitch);
        }

        // Apply new rotation
        transform.rotation =
            Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
    }
}
