//! TODO: Document this module

use bevy::prelude::*;

use crate::meth::LerpSnap;

/// The RTS camera component, which allows for orbiting, panning, and zooming around a focus point.
#[derive(Component, Clone, Copy, Debug)]
pub struct RTSCamera {
    /// The initial focus point of the camera
    pub focus: Vec3,
    /// The orbit sensitivity of the camera
    pub orbit_sensitivity: f32,
    /// The pan sensitivity of the camera
    pub pan_sensitivity: f32,
    /// The zoom sensitivity of the camera
    pub zoom_sensitivity: f32,
    /// Minimum zoom distance
    pub min_zoom: f32,
    /// Maximum zoom distance
    pub max_zoom: f32,
    /// The orbit smoothing factor
    pub orbit_smooth: f32,
    /// The zoom smoothing factor
    pub zoom_smooth: f32,
}

impl Default for RTSCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            orbit_sensitivity: 0.01,
            pan_sensitivity: 0.1,
            zoom_sensitivity: 2.5,
            min_zoom: 0.1,
            max_zoom: 100.0,
            orbit_smooth: 0.1,
            zoom_smooth: 0.1,
        }
    }
}

/// The input component for the RTS camera, which stores the current input state.
/// This component should be updated by user input systems to control the camera.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct RTSCameraInput {
    pub pan: Vec2,
    pub orbit: Vec2,
    pub zoom: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct RTSCameraTarget {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    radius: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct RTSCameraState {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    radius: f32,
}

/// The system set for the RTS camera plugin
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RTSCameraSet;

pub struct RTSCameraPlugin;

impl Plugin for RTSCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (initialize, update_target, update_state, sync_transform)
                .in_set(RTSCameraSet)
                .chain(),
        );
    }
}

fn initialize(
    mut commands: Commands,
    q_camera: Query<
        (Entity, &Transform, &RTSCamera),
        (Without<RTSCameraTarget>, Without<RTSCameraState>),
    >,
) {
    for (entity, transform, camera) in q_camera.iter() {
        let comp_vec = transform.translation - camera.focus;
        let yaw = comp_vec.z.atan2(comp_vec.x);
        let radius = comp_vec.length().max(camera.min_zoom).min(camera.max_zoom);
        let pitch = (comp_vec.y / radius).asin();
        let focus = camera.focus;

        commands
            .entity(entity)
            .insert(RTSCameraTarget { focus, yaw, pitch, radius })
            .insert(RTSCameraState { focus, yaw, pitch, radius });
    }
}

fn update_target(mut q_camera: Query<(&RTSCamera, &RTSCameraInput, &mut RTSCameraTarget)>) {
    for (camera, input, mut target) in q_camera.iter_mut() {
        target.yaw += input.orbit.x * camera.orbit_sensitivity;
        target.pitch += input.orbit.y * camera.orbit_sensitivity;

        target.radius = (target.radius - input.zoom * camera.zoom_sensitivity)
            .clamp(camera.min_zoom, camera.max_zoom);

        let zoom_factor = (target.radius - camera.min_zoom) / (camera.max_zoom - camera.min_zoom);
        let rotation = Quat::from_rotation_y(target.yaw);
        target.focus -= rotation * input.pan.extend(0.0).xzy() * camera.pan_sensitivity * zoom_factor;
    }
}

fn update_state(
    time: Res<Time>,
    mut q_camera: Query<(&mut RTSCameraState, &RTSCameraTarget, &RTSCamera)>,
) {
    for (mut state, target, camera) in q_camera.iter_mut() {
        if state.focus != target.focus {
            state.focus = target.focus;
        }

        if state.yaw != target.yaw {
            state.yaw = state
                .yaw
                .lerp_and_snap(target.yaw, camera.orbit_smooth, time.delta_secs());
        }

        if state.pitch != target.pitch {
            state.pitch = state
                .pitch
                .lerp_and_snap(target.pitch, camera.orbit_smooth, time.delta_secs());
        }

        if state.radius != target.radius {
            state.radius =
                state
                    .radius
                    .lerp_and_snap(target.radius, camera.zoom_smooth, time.delta_secs());
        }
    }
}

fn sync_transform(
    mut q_camera: Query<(&mut Transform, &RTSCameraState, &RTSCamera), Changed<RTSCameraState>>,
) {
    for (mut transform, state, camera) in q_camera.iter_mut() {
        let rotation = Quat::from_rotation_y(state.yaw) * Quat::from_rotation_x(-state.pitch);
        let translation = camera.focus + state.focus + rotation * Vec3::new(0.0, 0.0, state.radius);

        *transform = Transform::IDENTITY
            .with_translation(translation)
            .with_rotation(rotation);
    }
}
