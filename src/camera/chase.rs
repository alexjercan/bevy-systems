use bevy::prelude::*;

pub mod prelude {
    pub use super::{ChaseCamera, ChaseCameraTargetMarker, ChaseCameraAnchor, ChaseCameraPlugin};
}

/// The Case Camera Component is used to add some attributes to the camera
///
/// The Case Camera will follow a target and lag behind it a bit to give a smooth transition and
/// give a perspective of movement. We also want the camera to look in front of the target, so we
/// add a focus offset point. The Chase Camera will also have a rotation orbit style to look around
/// the focus point at the target.
#[derive(Component, Debug, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct ChaseCamera {
    /// Distance behind the target
    pub radius: f32,
    /// Vertical offset above the target
    pub height: f32,
    /// How far ahead of the target to look
    pub focus_distance: f32,
}

impl Default for ChaseCamera {
    fn default() -> Self {
        Self {
            radius: 20.0,
            height: 5.0,
            focus_distance: 20.0,
        }
    }
}

/// The target marker component for the chase camera to follow
#[derive(Component, Default, Debug, Reflect)]
pub struct ChaseCameraTargetMarker;

/// The anchor point for the chase camera to use as a frame of reference
#[derive(Component, Default, Debug, Reflect)]
pub struct ChaseCameraAnchor {
    pub achor_rot: Quat,
}

pub struct ChaseCameraPlugin;

impl Plugin for ChaseCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChaseCamera>()
            .register_type::<ChaseCameraTargetMarker>()
            .register_type::<ChaseCameraAnchor>();

        app.add_observer(initialize_chase_camera);
        app.add_systems(Update, chase_camera_update_system);
    }
}

/// Initialize camera state and target from config
fn initialize_chase_camera(trigger: Trigger<OnAdd, ChaseCamera>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert((ChaseCameraAnchor::default(),));
}

fn chase_camera_update_system(
    mut q_camera: Query<(&ChaseCamera, &ChaseCameraAnchor, &mut Transform), With<ChaseCamera>>,
    target: Single<&GlobalTransform, With<ChaseCameraTargetMarker>>,
) {
    let target_transform = target.into_inner();
    let target_pos = target_transform.translation();

    for (chase, input, mut transform) in q_camera.iter_mut() {
        let desired_pos = target_pos
            + input.achor_rot * Vec3::Z * chase.radius
            + input.achor_rot * Vec3::Y * chase.height;

        transform.translation = desired_pos;

        let focus = target_pos - input.achor_rot * Vec3::Z * chase.focus_distance;
        transform.look_at(focus, input.achor_rot * Vec3::Y);
    }
}
