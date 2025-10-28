use bevy::prelude::*;

pub mod prelude {
    pub use super::{
        SmoothLookRotation, SmoothLookRotationOutput, SmoothLookRotationPlugin,
        SmoothLookRotationTarget, SmoothLookRotationPluginSet,
    };
}

/// Component that makes an entity smoothly rotate to look along a specified axis.
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct SmoothLookRotation {
    /// The axis to rotate around (e.g., Vec3::Y for yaw).
    pub axis: Vec3,
    /// The initial angle in radians.
    pub initial: f32,
    /// The speed of rotation in radians per second.
    pub speed: f32,
    /// Optional minimum angle limit in radians.
    pub min: Option<f32>,
    /// Optional maximum angle limit in radians.
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

/// The target angle that the entity should smoothly rotate towards.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct SmoothLookRotationTarget(pub f32);

/// The current output angle of the smooth look rotation.
#[derive(Component, Clone, Copy, Debug, Deref, DerefMut, Reflect)]
pub struct SmoothLookRotationOutput(pub f32);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmoothLookRotationPluginSet;

/// A plugin that will enable the SmoothLookRotation system.
///
/// SmoothLookRotation allows an entity to smoothly rotate to face a target angle around a
/// specified axis. This is useful for creating smooth camera or object rotations
/// based on user input or other dynamic targets.
pub struct SmoothLookRotationPlugin;

impl Plugin for SmoothLookRotationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_smooth_look_system);
        app.add_systems(
            Update,
            smooth_look_rotation_update_system.in_set(SmoothLookRotationPluginSet),
        );
    }
}

fn initialize_smooth_look_system(
    insert: On<Insert, SmoothLookRotation>,
    q_look: Query<&SmoothLookRotation>,
    mut commands: Commands,
) {
    let entity = insert.entity;
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
        let angle_diff = normalize_angle(angle_diff);

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

fn normalize_angle(angle: f32) -> f32 {
    let mut a = (angle + std::f32::consts::PI) % std::f32::consts::TAU;
    if a < 0.0 {
        a += std::f32::consts::TAU;
    }
    a - std::f32::consts::PI
}
