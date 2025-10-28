use crate::meth::prelude::*;
use bevy::prelude::*;

pub mod prelude {
    pub use super::SphereOrbit;
    pub use super::SphereOrbitInput;
    pub use super::SphereOrbitOutput;
    pub use super::SphereOrbitPlugin;
    pub use super::SphereOrbitSystems;
}

/// Component to define a spherical orbit around a center point.
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct SphereOrbit {
    /// Radius of the sphere (distance from origin or from a center)
    pub radius: f32,
    /// (Optional) center of the sphere (in world space)
    pub center: Vec3,
    /// Initial theta angle (azimuth around Y axis)
    pub initial_theta: f32,
    /// Initial phi angle (elevation from equator)
    pub initial_phi: f32,
    /// Smoothing factor (between 0 and 1) for the orbit movement
    /// 0 = no smoothing, 1 = full smoothing
    pub smoothing: f32,
}

#[derive(Component, Clone, Debug, Default, Deref, DerefMut, Reflect)]
pub struct SphereOrbitOutput(pub Vec3);

#[derive(Component, Default, Clone, Copy, Debug, Reflect)]
pub struct SphereOrbitInput {
    pub theta: f32,
    pub phi: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
struct SphereOrbitState {
    theta: f32,
    phi: f32,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SphereOrbitSystems {
    Sync,
}

/// Plugin to manage entities with `SphereOrbit` component.
///
/// SphereOrbit allows an entity to orbit around a point on the surface of a sphere.
pub struct SphereOrbitPlugin;

impl Plugin for SphereOrbitPlugin {
    fn build(&self, app: &mut App) {
        debug!("SphereOrbitPlugin: build");

        app.add_observer(initialize_sphere_orbit_system);

        app.add_systems(
            PostUpdate,
            (sphere_update_state, sphere_update_output)
                .chain()
                .in_set(SphereOrbitSystems::Sync),
        );
    }
}

/// Initialize orbit state and next target angles
fn initialize_sphere_orbit_system(
    insert: On<Insert, SphereOrbit>,
    mut commands: Commands,
    q_orbit: Query<&SphereOrbit, With<SphereOrbit>>,
) {
    let entity = insert.entity;
    trace!("initialize_sphere_orbit_system: entity {:?}", entity);

    let Ok(orbit) = q_orbit.get(entity) else {
        warn!(
            "initialize_sphere_orbit_system: entity {:?} not found in q_orbit",
            entity
        );
        return;
    };

    commands.entity(entity).insert((
        SphereOrbitState {
            theta: orbit.initial_theta,
            phi: orbit.initial_phi,
        },
        SphereOrbitInput {
            theta: orbit.initial_theta,
            phi: orbit.initial_phi,
        },
        SphereOrbitOutput(
            spherical_to_cartesian(orbit.radius, orbit.initial_theta, orbit.initial_phi)
                + orbit.center,
        ),
    ));
}

fn sphere_update_state(
    time: Res<Time>,
    mut query: Query<(&SphereOrbit, &mut SphereOrbitState, &SphereOrbitInput)>,
) {
    let dt = time.delta_secs();

    for (orbit, mut state, next) in query.iter_mut() {
        let smoothing = orbit.smoothing.clamp(0.0, 1.0);
        let new_theta = state.theta.lerp_and_snap(next.theta, smoothing, dt);
        let new_phi = state.phi.lerp_and_snap(next.phi, smoothing, dt);

        state.theta = new_theta;
        state.phi = new_phi;
    }
}

fn sphere_update_output(
    mut query: Query<(&SphereOrbit, &SphereOrbitState, &mut SphereOrbitOutput)>,
) {
    for (orbit, state, mut output) in query.iter_mut() {
        let pos = spherical_to_cartesian(orbit.radius, state.theta, state.phi) + orbit.center;
        **output = pos;
    }
}
