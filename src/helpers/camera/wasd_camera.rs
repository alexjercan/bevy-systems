//! TODO: Document this module

use bevy::prelude::*;

/// The WASD camera component, which allows for wasd movement and mouse look.
#[derive(Component, Clone, Copy, Debug)]
pub struct WASDCamera {
    /// The look sensitivity of the camera
    pub look_sensitivity: f32,
    /// The wasd sensitivity of the camera
    pub wasd_sensitivity: f32,
}

impl Default for WASDCamera {
    fn default() -> Self {
        Self {
            look_sensitivity: 0.0025,
            wasd_sensitivity: 0.5,
        }
    }
}

/// The input component for the WASD camera, which stores the current input state.
/// This component should be updated by user input systems to control the camera.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WASDCameraInput {
    pub pan: Vec2,
    pub wasd: Vec2,
    pub vertical: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct WASDCameraTarget {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct WASDCameraState {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WASDCameraSet;

pub struct WASDCameraPlugin;

impl Plugin for WASDCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (initialize, update_target, update_state, sync_transform)
                .in_set(WASDCameraSet)
                .chain(),
        );
    }
}

fn initialize(
    mut commands: Commands,
    q_camera: Query<
        (Entity, &Transform),
        (
            With<WASDCamera>,
            Without<WASDCameraTarget>,
            Without<WASDCameraState>,
        ),
    >,
) {
    for (entity, transform) in q_camera.iter() {
        let translation = transform.translation;
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        commands.entity(entity).insert((
            WASDCameraTarget {
                position: translation,
                yaw,
                pitch,
            },
            WASDCameraState {
                position: translation,
                yaw,
                pitch,
            },
        ));
    }
}

fn update_target(mut q_camera: Query<(&WASDCamera, &WASDCameraInput, &mut WASDCameraTarget)>) {
    for (camera, input, mut target) in q_camera.iter_mut() {
        target.yaw -= input.pan.x * camera.look_sensitivity;
        target.pitch -= input.pan.y * camera.look_sensitivity;

        let rotation = Quat::from_euler(EulerRot::YXZ, target.yaw, target.pitch, 0.0);

        let forward = rotation * Vec3::NEG_Z;
        let right = Quat::from_rotation_y(target.yaw) * Vec3::X;

        target.position += forward * input.wasd.y * camera.wasd_sensitivity
            + right * input.wasd.x * camera.wasd_sensitivity;

        target.position += Vec3::Y * input.vertical * camera.wasd_sensitivity;
    }
}

fn update_state(mut q_camera: Query<(&mut WASDCameraState, &WASDCameraTarget)>) {
    for (mut state, target) in q_camera.iter_mut() {
        state.position = target.position;
        state.yaw = target.yaw;
        state.pitch = target.pitch;
    }
}

fn sync_transform(
    mut q_camera: Query<(&mut Transform, &WASDCameraState), Changed<WASDCameraState>>,
) {
    for (mut transform, state) in q_camera.iter_mut() {
        let rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        *transform = Transform {
            translation: state.position,
            rotation,
            ..Default::default()
        };
    }
}
