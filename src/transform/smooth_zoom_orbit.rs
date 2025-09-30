use bevy::prelude::*;

use crate::prelude::LerpSnap;

pub mod prelude {
    pub use super::{SmoothZoomOrbit, SmoothZoomOrbitPlugin, SmoothZoomOrbitTarget};
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[require(Transform, GlobalTransform)]
pub struct SmoothZoomOrbit {
    /// The initial focus point of the Transform
    pub focus: Vec3,
    /// Minimum zoom distance (radius)
    pub min_zoom: f32,
    /// Maximum zoom distance
    pub max_zoom: f32,
    /// The smoothing factor when interpolating to target (0..1)
    pub orbit_smooth: f32,
    /// The smoothing factor for zoom interpolation
    pub zoom_smooth: f32,
}

impl Default for SmoothZoomOrbit {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            min_zoom: 10.0,
            max_zoom: 50.0,
            orbit_smooth: 0.1,
            zoom_smooth: 0.1,
        }
    }
}

/// The target (desired) yaw / pitch / radius that Transform will lerp toward
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
pub struct SmoothZoomOrbitTarget {
    pub radius: f32,
    pub theta: f32,
    pub phi: f32,
}

/// The current transform state (yaw / pitch / radius) that is being smoothed toward target
#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
struct SmoothZoomOrbitState {
    radius: f32,
    theta: f32,
    phi: f32,
}

pub struct SmoothZoomOrbitPlugin;

impl Plugin for SmoothZoomOrbitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SmoothZoomOrbit>()
            .register_type::<SmoothZoomOrbitTarget>()
            .register_type::<SmoothZoomOrbitState>();

        app.add_observer(initialize_smooth_zoom_orbit_system);
        app.add_systems(
            Update,
            (
                smooth_zoom_orbit_update_system,
                smooth_zoom_orbit_sync_system,
            ),
        );
    }
}

fn initialize_smooth_zoom_orbit_system(
    trigger: Trigger<OnAdd, SmoothZoomOrbit>,
    q_orbit: Query<(&SmoothZoomOrbit, &Transform), Without<SmoothZoomOrbitState>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((orbit, transform)) = q_orbit.get(entity) else {
        warn!(
            "initialize_smooth_zoom_orbit_system: entity {:?} is not setup correctly",
            entity
        );
        return;
    };

    let offset = transform.translation - orbit.focus;
    let radius = offset.length().clamp(orbit.min_zoom, orbit.max_zoom);
    let theta = offset.z.atan2(offset.x);
    let phi = (offset.y / radius).asin();

    commands.entity(entity).insert((
        SmoothZoomOrbitTarget { radius, theta, phi },
        SmoothZoomOrbitState { radius, theta, phi },
    ));
}

fn smooth_zoom_orbit_update_system(
    time: Res<Time>,
    mut q_orbit: Query<(
        &SmoothZoomOrbit,
        &SmoothZoomOrbitTarget,
        &mut SmoothZoomOrbitState,
    )>,
) {
    let dt = time.delta_secs();

    for (orbit, target, mut state) in &mut q_orbit {
        state.radius = state
            .radius
            .lerp_and_snap(target.radius, orbit.zoom_smooth, dt);
        state.theta = state
            .theta
            .lerp_and_snap(target.theta, orbit.orbit_smooth, dt);
        state.phi = state.phi.lerp_and_snap(target.phi, orbit.orbit_smooth, dt);
    }
}

fn smooth_zoom_orbit_sync_system(
    mut q_orbit: Query<(&SmoothZoomOrbit, &SmoothZoomOrbitState, &mut Transform)>,
) {
    for (orbit, state, mut transform) in &mut q_orbit {
        let radius = state.radius.clamp(orbit.min_zoom, orbit.max_zoom);
        let phi = state.phi.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
        let theta = state.theta;

        let rotation = Quat::from_rotation_y(theta) * Quat::from_rotation_x(-phi);
        let offset = rotation * Vec3::new(0.0, 0.0, radius);
        *transform = Transform::from_translation(orbit.focus + offset).with_rotation(rotation);
    }
}
