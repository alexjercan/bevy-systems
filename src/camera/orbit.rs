use bevy::prelude::*;

use crate::prelude::{SmoothZoomOrbit, SmoothZoomOrbitTarget};

pub mod prelude {
    pub use super::{OrbitCameraInput, OrbitCameraPlugin};
}

#[derive(Component, Default, Debug, Reflect)]
pub struct OrbitCameraInput {
    pub orbit: Vec2,
    pub zoom: f32,
}

impl OrbitCameraInput {
    fn has_input(&self) -> bool {
        self.orbit.length_squared() > 0.0 || self.zoom.abs() > 0.0
    }
}

/// Plugin
pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OrbitCameraInput>();

        app.add_observer(initialize_orbit_camera);
        app.add_systems(Update, orbit_camera_update_target_system);
    }
}

/// Initialize camera state and target from config
fn initialize_orbit_camera(trigger: Trigger<OnAdd, SmoothZoomOrbit>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert((OrbitCameraInput::default(),));
}

/// Update the *target* yaw, pitch, radius from the input and camera config
fn orbit_camera_update_target_system(
    mut q_camera: Query<(
        &SmoothZoomOrbit,
        &mut OrbitCameraInput,
        &mut SmoothZoomOrbitTarget,
    )>,
) {
    for (camera, mut input, mut target) in q_camera.iter_mut() {
        if input.has_input() {
            target.theta -= input.orbit.x;
            target.phi -= input.orbit.y;

            let phi_limit = std::f32::consts::FRAC_PI_2 - 0.01;
            target.phi = target.phi.clamp(-phi_limit, phi_limit);

            let zoom_change = -input.zoom;
            target.radius = (target.radius + zoom_change).clamp(camera.min_zoom, camera.max_zoom);
        }

        input.orbit = Vec2::ZERO;
        input.zoom = 0.0;
    }
}
