use bevy::prelude::*;

pub mod prelude {
    pub use super::{ChaseCamera, ChaseCameraInput, ChaseCameraPlugin, ChaseCameraPluginSet};
}

/// The Case Camera Component is used to add some attributes to the camera
///
/// The Case Camera will follow a target and lag behind it a bit to give a smooth transition and
/// give a perspective of movement. We also want the camera to look in front of the target, so we
/// add a focus offset point. The Chase Camera will also have a rotation orbit style to look around
/// the focus point at the target.
#[derive(Component, Debug, Reflect)]
pub struct ChaseCamera {
    /// Offset distance behind the target
    pub offset: Vec3,
    /// How far ahead of the target to look
    pub focus_offset: Vec3,
}

impl Default for ChaseCamera {
    fn default() -> Self {
        Self {
            offset: Vec3::new(0.0, 5.0, -20.0),
            focus_offset: Vec3::new(0.0, 0.0, 20.0),
        }
    }
}

/// The anchor point for the chase camera to use as a frame of reference
#[derive(Component, Default, Debug, Reflect)]
pub struct ChaseCameraInput {
    /// The rotation that we want to match
    pub achor_rot: Quat,
    /// The position that we want to match (and we will add the offset to this)
    pub anchor_pos: Vec3,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChaseCameraPluginSet;

pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_chase_camera);
        app.add_systems(
            Update,
            chase_camera_update_system.in_set(ChaseCameraPluginSet),
        );
    }
}

/// Initialize camera state and target from config
fn initialize_chase_camera(insert: On<Insert, ChaseCamera>, mut commands: Commands) {
    let entity = insert.entity;
    commands
        .entity(entity)
        .insert((ChaseCameraInput::default(),));
}

fn chase_camera_update_system(
    mut q_camera: Query<(&ChaseCamera, &ChaseCameraInput, &mut Transform), With<ChaseCamera>>,
) {
    for (chase, input, mut transform) in q_camera.iter_mut() {
        let target_pos = input.anchor_pos;
        let desired_pos = target_pos
            + input.achor_rot * Vec3::NEG_Z * chase.offset.z
            + input.achor_rot * Vec3::Y * chase.offset.y
            + input.achor_rot * Vec3::X * chase.offset.x;

        transform.translation = desired_pos;

        let focus = target_pos
            + input.achor_rot * Vec3::NEG_Z * chase.focus_offset.z
            + input.achor_rot * Vec3::Y * chase.focus_offset.y
            + input.achor_rot * Vec3::X * chase.focus_offset.x;

        transform.look_at(focus, input.achor_rot * Vec3::Y);
    }
}
