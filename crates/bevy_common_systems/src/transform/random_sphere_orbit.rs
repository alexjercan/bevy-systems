//! Module implementing random spherical orbit behavior for entities in Bevy.
//!
//! The `RandomSphereOrbit` component allows an entity to orbit around a point on the surface of a
//! sphere, randomly picking new target angles to move toward over time.

use crate::meth::prelude::*;
use bevy::prelude::*;
use rand::prelude::*;

pub mod prelude {
    pub use super::RandomSphereOrbit;
    pub use super::RandomSphereOrbitOutput;
    pub use super::SphereRandomOrbitPlugin;
    pub use super::SphereRandomOrbitSystems;
}

/// Component to define a spherical orbit around a center point.
#[derive(Component, Clone, Debug, Default, Reflect)]
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

#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
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
pub enum SphereRandomOrbitSystems {
    Sync,
}

/// Plugin to manage entities with `RandomSphereOrbit` component.
///
/// RandomSphereOrbit allows an entity to orbit around a point on the surface of a sphere,
/// randomly picking new target angles to move toward over time.
pub struct SphereRandomOrbitPlugin;

impl Plugin for SphereRandomOrbitPlugin {
    fn build(&self, app: &mut App) {
        debug!("SphereRandomOrbitPlugin: build");

        app.add_observer(initialize_random_sphere_orbit_system);

        app.add_systems(
            PostUpdate,
            (
                random_sphere_choose_next,
                random_sphere_update_state,
                random_sphere_update_output,
            )
                .chain()
                .in_set(SphereRandomOrbitSystems::Sync),
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
    trace!("initialize_random_sphere_orbit_system: entity {:?}", entity);

    let Ok(orbit) = q_orbit.get(entity) else {
        warn!(
            "initialize_random_sphere_orbit_system: entity {:?} not found in q_orbit",
            entity
        );
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
        **output = pos;
    }
}
