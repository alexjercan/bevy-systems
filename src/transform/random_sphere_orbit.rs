use bevy::prelude::*;
use rand::prelude::*;

pub mod prelude {
    pub use super::{RandomSphereOrbit, SphereRandomOrbitPlugin};
}

#[derive(Component, Clone, Debug, Reflect)]
#[require(RandomSphereOrbitState, Transform)]
/// Component to define a spherical orbit around a center point.
pub struct RandomSphereOrbit {
    /// Radius of the sphere (distance from origin or from a center)
    pub radius: f32,
    /// Speed (in radians per second) of movement along the sphere surface
    pub angular_speed: f32,
    /// (Optional) center of the sphere (in world space)
    pub center: Vec3,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[require(RandomSphereOrbitNext)]
struct RandomSphereOrbitState {
    theta: f32,
    phi: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
struct RandomSphereOrbitNext {
    theta: f32,
    phi: f32,
}

pub struct SphereRandomOrbitPlugin;

impl Plugin for SphereRandomOrbitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RandomSphereOrbit>()
            .register_type::<RandomSphereOrbitState>()
            .register_type::<RandomSphereOrbitNext>();

        app.add_systems(
            Update,
            (
                sphere_random_orbit_next_system,
                sphere_random_orbit_follow_system,
            ),
        );
    }
}

/// System: pick a new random “next” angles for orbits that have reached (or nearly reached) their current next
fn sphere_random_orbit_next_system(mut query: Query<(&RandomSphereOrbitState, &mut RandomSphereOrbitNext)>) {
    let mut rng = rand::rng();

    for (state, mut next) in query.iter_mut() {
        let dtheta = (next.theta - state.theta).abs();
        let dphi = (next.phi - state.phi).abs();

        let threshold = 0.01;
        if dtheta < threshold && dphi < threshold {
            let new_theta = rng.random_range(0.0..(std::f32::consts::TAU));

            let new_phi =
                rng.random_range(-std::f32::consts::FRAC_PI_2..std::f32::consts::FRAC_PI_2);
            next.theta = new_theta;
            next.phi = new_phi;
        }
    }
}

/// System: move the state toward `next` gradually, and update the Transform
fn sphere_random_orbit_follow_system(
    time: Res<Time>,
    mut query: Query<(
        &RandomSphereOrbit,
        &mut RandomSphereOrbitState,
        &RandomSphereOrbitNext,
        &mut Transform,
    )>,
) {
    let dt = time.delta_secs();

    for (orbit, mut state, next, mut tf) in query.iter_mut() {
        // Interpolate angles toward next
        let delta_theta = next.theta - state.theta;
        let delta_phi = next.phi - state.phi;

        // We can move with angular_speed; i.e. maximum angular change per second
        let max_delta = orbit.angular_speed * dt;

        // Move theta
        let new_theta = if delta_theta.abs() <= max_delta {
            next.theta
        } else {
            state.theta + delta_theta.signum() * max_delta
        };

        // Move phi
        let new_phi = if delta_phi.abs() <= max_delta {
            next.phi
        } else {
            state.phi + delta_phi.signum() * max_delta
        };

        state.theta = new_theta;
        state.phi = new_phi;

        // Convert spherical to Cartesian
        // theta: azimuth around Y axis; phi: elevation from equator
        let cos_phi = state.phi.cos();
        let x = orbit.radius * cos_phi * state.theta.cos();
        let y = orbit.radius * state.phi;
        let z = orbit.radius * cos_phi * state.theta.sin();

        let new_pos = orbit.center + Vec3::new(x, y, z);
        tf.translation = new_pos;
    }
}
