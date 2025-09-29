use bevy::prelude::*;

pub mod prelude {
    pub use super::{OrbitCamera, OrbitCameraInput, OrbitCameraPlugin};
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct OrbitCamera {
    /// The initial focus point of the camera
    pub focus: Vec3,
    /// The orbit sensitivity of the camera (for yaw/pitch)
    pub orbit_sensitivity: f32,
    /// The zoom sensitivity of the camera (scroll wheel)
    pub zoom_sensitivity: f32,
    /// Minimum zoom distance (radius)
    pub min_zoom: f32,
    /// Maximum zoom distance
    pub max_zoom: f32,
    /// The smoothing factor when interpolating to target (0..1)
    pub orbit_smooth: f32,
    /// The smoothing factor for zoom interpolation
    pub zoom_smooth: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            orbit_sensitivity: 0.05,
            zoom_sensitivity: 2.5,
            min_zoom: 10.0,
            max_zoom: 50.0,
            orbit_smooth: 0.1,
            zoom_smooth: 0.1,
        }
    }
}

#[derive(Component, Default, Debug, Reflect)]
pub struct OrbitCameraInput {
    pub orbit: Vec2, // (delta_yaw, delta_pitch) from input
    pub zoom: f32,
}

impl OrbitCameraInput {
    fn has_input(&self) -> bool {
        self.orbit.length_squared() > 0.0 || self.zoom.abs() > 0.0
    }
}

/// The target (desired) yaw / pitch / radius that camera will lerp toward
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
struct OrbitCameraTarget {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

/// The current transform state (yaw / pitch / radius) that is being smoothed toward target
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
struct OrbitCameraState {
    yaw: f32,
    pitch: f32,
    radius: f32,
}

/// Plugin
pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OrbitCamera>()
            .register_type::<OrbitCameraInput>()
            .register_type::<OrbitCameraTarget>()
            .register_type::<OrbitCameraState>();

        app.add_observer(initialize_orbit_camera);
        app.add_systems(
            Update,
            (orbit_camera_update_target_system, orbit_camera_apply_system).chain(),
        );
    }
}

/// Initialize camera state and target from config
fn initialize_orbit_camera(
    trigger: Trigger<OnAdd, OrbitCamera>,
    q_cameras: Query<(&OrbitCamera, &Transform), Without<OrbitCameraState>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((camera, transform)) = q_cameras.get(entity) else {
        warn!("OrbitCamera component added, but the entity is not setup correctly");
        return;
    };

    let comp_vec = transform.translation - camera.focus;
    let yaw = comp_vec.z.atan2(comp_vec.x);
    let radius = comp_vec.length().max(camera.min_zoom).min(camera.max_zoom);
    let pitch = (comp_vec.y / radius).asin();

    commands.entity(entity).insert((
        OrbitCameraState { yaw, pitch, radius },
        OrbitCameraTarget { yaw, pitch, radius },
        OrbitCameraInput::default(),
    ));
}

/// Update the *target* yaw, pitch, radius from the input and camera config
fn orbit_camera_update_target_system(
    mut cams: Query<(&OrbitCamera, &mut OrbitCameraInput, &mut OrbitCameraTarget)>,
) {
    for (cam, mut input, mut target) in cams.iter_mut() {
        if input.has_input() {
            // yaw/pitch: invert or adjust signs as you prefer
            target.yaw -= input.orbit.x * cam.orbit_sensitivity;
            target.pitch -= input.orbit.y * cam.orbit_sensitivity;

            // clamp pitch so camera doesn’t flip over (for example between -π/2..+π/2)
            let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01;
            target.pitch = target.pitch.clamp(-pitch_limit, pitch_limit);

            // zoom: scale multiplicatively or additively
            let zoom_change = -input.zoom * cam.zoom_sensitivity;
            target.radius = (target.radius + zoom_change).clamp(cam.min_zoom, cam.max_zoom);
        }

        // reset input accumulation for next frame
        input.orbit = Vec2::ZERO;
        input.zoom = 0.0;
    }
}

/// Smoothly interpolate (or “lerp”) from current state to target, and set camera transform
fn orbit_camera_apply_system(
    mut cams: Query<(
        &OrbitCamera,
        &mut OrbitCameraState,
        &OrbitCameraTarget,
        &mut Transform,
        &GlobalTransform,
    )>,
) {
    for (cam, mut state, target, mut tf, _global) in cams.iter_mut() {
        // Smooth (interpolate) yaw, pitch, radius
        state.yaw = state.yaw + (target.yaw - state.yaw) * cam.orbit_smooth;
        state.pitch = state.pitch + (target.pitch - state.pitch) * cam.orbit_smooth;
        state.radius = state.radius + (target.radius - state.radius) * cam.zoom_smooth;

        // Compute camera position in spherical coordinates relative to focus
        // yaw = around Y axis, pitch = vertical angle
        let x = state.radius * state.pitch.cos() * state.yaw.cos();
        let y = state.radius * state.pitch.sin();
        let z = state.radius * state.pitch.cos() * state.yaw.sin();

        let focus = cam.focus;
        let cam_pos = focus + Vec3::new(x, y, z);
        tf.translation = cam_pos;

        // Look at the focus point
        tf.look_at(focus, Vec3::Y);
    }
}
