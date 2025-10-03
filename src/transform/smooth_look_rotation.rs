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
    pub axis: Vec3,
    pub initial: f32,
    pub speed: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
}

impl Default for SmoothLookRotation {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            initial: 0.0,
            speed: std::f32::consts::PI, // 180 degrees per second
            min: None,
            max: None,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct SmoothLookRotationTarget(pub f32);

#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct SmoothLookRotationOutput(pub f32);

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

    commands.entity(entity).insert((
        SmoothLookRotationTarget(look.initial),
        SmoothLookRotationOutput(look.initial),
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
        let angle_diff = **target - **state;
        let max_angle_change = look.speed * dt;
        if angle_diff.abs() <= max_angle_change {
            **state = **target;
        } else {
            **state += angle_diff.signum() * max_angle_change;
        }

        if let Some(min) = look.min {
            **state = state.max(min);
        }
        if let Some(max) = look.max {
            **state = state.min(max);
        }
    }
}
