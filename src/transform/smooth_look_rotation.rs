//! A Bevy plugin that provides smooth look rotation functionality for entities. To rotate around
//! the local axes, use `Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0)`. This provides a
//! component to configure the rotation speeds and limits, a target component to set desired
//! yaw and pitch, and an output component that holds the current yaw and pitch.

use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        SmoothLookRotation, SmoothLookRotationOutput, SmoothLookRotationPlugin,
        SmoothLookRotationTarget,
    };
}

/// Component to configure smooth look rotation parameters. Add this to an entity to enable smooth
/// look rotation.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct SmoothLookRotation {
    /// Initial yaw in radians.
    pub initial_yaw: f32,
    /// Initial pitch in radians.
    pub initial_pitch: f32,
    /// Yaw rotation speed in radians per second.
    pub yaw_speed: f32,
    /// Pitch rotation speed in radians per second.
    pub pitch_speed: f32,
    /// Optional minimum pitch angle in radians.
    pub min_pitch: Option<f32>,
    /// Optional maximum pitch angle in radians.
    pub max_pitch: Option<f32>,
}

impl Default for SmoothLookRotation {
    fn default() -> Self {
        Self {
            initial_yaw: 0.0,
            initial_pitch: 0.0,
            yaw_speed: std::f32::consts::PI, // 180 degrees per second
            pitch_speed: std::f32::consts::PI, // 180 degrees per second
            min_pitch: None,
            max_pitch: None,
        }
    }
}

/// Component to set the target yaw and pitch angles in radians. Update this component to change
/// the desired look direction.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct SmoothLookRotationTarget {
    pub yaw: f32,
    pub pitch: f32,
}

/// Component that holds the current yaw and pitch angles in radians. This is updated smoothly
/// over time based on the target and speed settings.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct SmoothLookRotationOutput {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct SmoothLookRotationPlugin;

impl Plugin for SmoothLookRotationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmoothLookRotation>()
            .register_type::<SmoothLookRotationTarget>()
            .register_type::<SmoothLookRotationOutput>();

        app.add_observer(initialize_smooth_look_system);
        app.add_systems(Update, smooth_look_rotation_update_system);
    }
}

fn initialize_smooth_look_system(
    trigger: Trigger<OnInsert, SmoothLookRotation>,
    q_look: Query<&SmoothLookRotation>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok(look) = q_look.get(entity) else {
        warn!(
            "initialize_smooth_look_system: entity {:?} is not setup correctly",
            entity
        );
        return;
    };

    let yaw = look.initial_yaw;
    let pitch = look.initial_pitch;

    commands.entity(entity).insert((
        SmoothLookRotationTarget { yaw, pitch },
        SmoothLookRotationOutput { yaw, pitch },
    ));
}

fn smooth_look_rotation_update_system(
    time: Res<Time>,
    mut q_look: Query<(
        &SmoothLookRotation,
        &SmoothLookRotationTarget,
        &mut SmoothLookRotationOutput,
    )>,
) {
    let dt = time.delta_secs();
    for (look, target, mut state) in &mut q_look {
        let yaw_diff = (target.yaw - state.yaw + std::f32::consts::PI)
            .rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;
        let max_yaw_change = look.yaw_speed * dt;
        if yaw_diff.abs() <= max_yaw_change {
            state.yaw = target.yaw;
        } else {
            state.yaw += yaw_diff.signum() * max_yaw_change;
        }

        let pitch_diff = target.pitch - state.pitch;
        let max_pitch_change = look.pitch_speed * dt;
        if pitch_diff.abs() <= max_pitch_change {
            state.pitch = target.pitch;
        } else {
            state.pitch += pitch_diff.signum() * max_pitch_change;
        }

        if let Some(min_pitch) = look.min_pitch {
            state.pitch = state.pitch.max(min_pitch);
        }
        if let Some(max_pitch) = look.max_pitch {
            state.pitch = state.pitch.min(max_pitch);
        }
    }
}
