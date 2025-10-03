// TODO: Refactor to not apply the Transform directly, but rather have an output component

use bevy::prelude::*;
use rand::prelude::*;

pub mod prelude {
    pub use super::{
        RandomSphereOrbit, RandomSphereOrbitOutput, SphereRandomOrbitPlugin,
        SphereRandomOrbitPluginSet,
    };
}

/// Component to define a spherical orbit around a center point.
#[derive(Component, Clone, Debug, Reflect)]
pub struct RandomSphereOrbit {
    /// Radius of the sphere (distance from origin or from a center)
    pub radius: f32,
    /// Speed (in radians per second) of movement along the sphere surface
    pub angular_speed: f32,
    /// (Optional) center of the sphere (in world space)
    pub center: Vec3,
    /// Initial theta angle (azimuth around Y axis)
    pub initial_theta: f32,
    /// Initial phi angle (elevation from equator)
    pub initial_phi: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct RandomSphereOrbitOutput(pub Vec3);

#[derive(Component, Clone, Debug, Default, Reflect)]
struct RandomSphereOrbitState {
    theta: f32,
    phi: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
struct RandomSphereOrbitNext {
    theta: f32,
    phi: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SphereRandomOrbitPluginSet;

/// Plugin to manage entities with `RandomSphereOrbit` component.
///
/// RandomSphereOrbit allows an entity to orbit around a point on the surface of a sphere,
/// randomly picking new target angles to move toward over time.
pub struct SphereRandomOrbitPlugin;

impl Plugin for SphereRandomOrbitPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_random_sphere_orbit_system);

        app.add_systems(
            Update,
            (
                random_sphere_choose_next,
                random_sphere_update_state,
                random_sphere_update_output,
            )
                .chain()
                .in_set(SphereRandomOrbitPluginSet),
        );
    }
}

/// Initialize orbit state and next target angles
fn initialize_random_sphere_orbit_system(
    insert: On<Insert, RandomSphereOrbit>,
    mut commands: Commands,
    q_orbit: Query<&RandomSphereOrbit, With<RandomSphereOrbit>>,
) {
    let entity = insert.entity;
    let Ok(orbit) = q_orbit.get(entity) else {
        warn!("initialize_random_sphere_orbit_system: entity does not have RandomSphereOrbit component");
        return;
    };

    commands.entity(entity).insert((
        RandomSphereOrbitState {
            theta: orbit.initial_theta,
            phi: orbit.initial_phi,
        },
        RandomSphereOrbitNext {
            theta: orbit.initial_theta,
            phi: orbit.initial_phi,
        },
        RandomSphereOrbitOutput(
            spherical_to_cartesian(orbit.radius, orbit.initial_theta, orbit.initial_phi)
                + orbit.center,
        ),
    ));
}

fn random_sphere_choose_next(
    mut query: Query<(&RandomSphereOrbitState, &mut RandomSphereOrbitNext)>,
) {
    let mut rng = rand::rng();

    for (state, mut next) in query.iter_mut() {
        let dtheta = (next.theta - state.theta).abs();
        let dphi = (next.phi - state.phi).abs();

        let threshold = std::f32::EPSILON;
        if dtheta < threshold && dphi < threshold {
            let new_theta = rng.random_range(0.0..(std::f32::consts::TAU));

            let new_phi =
                rng.random_range(-std::f32::consts::FRAC_PI_2..std::f32::consts::FRAC_PI_2);
            next.theta = new_theta;
            next.phi = new_phi;
        }
    }
}

fn random_sphere_update_state(
    time: Res<Time>,
    mut query: Query<(
        &RandomSphereOrbit,
        &mut RandomSphereOrbitState,
        &RandomSphereOrbitNext,
    )>,
) {
    let dt = time.delta_secs();

    for (orbit, mut state, next) in query.iter_mut() {
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
    }
}

fn random_sphere_update_output(
    mut query: Query<(
        &RandomSphereOrbit,
        &RandomSphereOrbitState,
        &mut RandomSphereOrbitOutput,
    )>,
) {
    for (orbit, state, mut output) in query.iter_mut() {
        let pos = spherical_to_cartesian(orbit.radius, state.theta, state.phi) + orbit.center;
        output.0 = pos;
    }
}

fn spherical_to_cartesian(radius: f32, theta: f32, phi: f32) -> Vec3 {
    let cos_phi = phi.cos();
    let x = radius * cos_phi * theta.cos();
    let y = radius * phi;
    let z = radius * cos_phi * theta.sin();
    Vec3::new(x, y, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_to_cartesian() {
        let radius = 1.0;
        let theta = 0.0;
        let phi = 0.0;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(1.0, 0.0, 0.0), 1e-6));

        let theta = std::f32::consts::FRAC_PI_2;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(0.0, 0.0, 1.0), 1e-6));

        let phi = std::f32::consts::FRAC_PI_2;
        let pos = spherical_to_cartesian(radius, theta, phi);
        assert!(pos.abs_diff_eq(Vec3::new(0.0, std::f32::consts::FRAC_PI_2, 0.0), 1e-6));
    }
}
